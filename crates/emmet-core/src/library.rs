// SPDX-License-Identifier: AGPL-3.0-or-later
//! The LDraw library: download the pinned `complete.zip`, check its sha256, unzip it into app data with a zip-slip
//! guard, and verify it later (plan sections 12 and 15). Downloads resume from a `.part` file.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::data::DataDir;
use crate::envelope::{Error, ErrorCode, Result};
use crate::fsx::{self, write_atomic};

/// The library release this Emmet version pins, and where it comes from.
pub const RELEASE: &str = "2026-10";
pub const URL: &str = "https://library.ldraw.org/library/updates/complete.zip";
pub const SHA256: &str = "023083918da2d3697c794fb4267975509584f3611b993095874206a803fa610f";

/// Refuse downloads and unzipped contents past these sizes (the 2026-10 zip is 147 MB, about 550 MB unzipped).
const MAX_DOWNLOAD: u64 = 1 << 30;
const MAX_UNZIPPED: u64 = 4 << 30;

/// What `library.json` records about the installed library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Installed {
    pub release: String,
    /// sha256 of the zip it came from.
    pub sha256: String,
    pub files: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Status {
    pub installed: Option<Installed>,
    /// The release this Emmet version pins.
    pub pinned: String,
    pub path: String,
    /// Parts in the index, or None when it isn't built.
    pub indexed_parts: Option<usize>,
}

pub enum Source<'a> {
    /// Download over HTTPS (plain HTTP only to this machine, for tests).
    Url(&'a str),
    /// A `complete.zip` already on disk.
    Zip(&'a Path),
}

pub fn status(data: &DataDir) -> Status {
    Status {
        installed: installed(data),
        pinned: RELEASE.into(),
        path: data.ldraw().display().to_string(),
        indexed_parts: crate::index::Index::open(data).ok().and_then(|i| i.part_count().ok()),
    }
}

/// The installed library, or None when there's none or it's damaged (`require` tells the two apart).
pub fn installed(data: &DataDir) -> Option<Installed> {
    require(data).ok()
}

/// The installed library: `LIBRARY_MISSING` when there's none, `LIBRARY_CORRUPT` when library.json is there but
/// unreadable, so a damaged install isn't mistaken for a missing one.
pub fn require(data: &DataDir) -> Result<Installed> {
    let missing = || {
        Error::new(
            ErrorCode::LibraryMissing,
            format!("The LDraw library isn't installed in {}.", data.root().display()),
            "Run emmet library install.",
        )
    };
    let path = data.library_json();
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Err(missing()),
        Err(e) => return Err(Error::io(&path, e)),
    };
    let info: Installed = serde_json::from_str(&text).map_err(|e| {
        Error::new(
            ErrorCode::LibraryCorrupt,
            format!("{} is damaged: {e}", path.display()),
            "Run emmet library install to put the library back.",
        )
        .at_file(path.display().to_string())
    })?;
    if !data.ldraw().is_dir() {
        return Err(missing());
    }
    Ok(info)
}

/// Downloads (or takes) the zip, checks it against `sha256`, unzips it, and builds the parts index.
pub fn install(
    data: &DataDir,
    release: &str,
    sha256: &str,
    source: Source,
    progress: &mut dyn FnMut(u64, Option<u64>),
) -> Result<Status> {
    let (zip_path, downloaded) = match source {
        Source::Zip(path) => (path.to_path_buf(), false),
        Source::Url(url) => (download(data, release, url, progress)?, true),
    };
    let got = fsx::sha256_file(&zip_path)?;
    if got != sha256.to_lowercase() {
        if downloaded {
            let _ = std::fs::remove_file(&zip_path);
        }
        return Err(Error::new(
            ErrorCode::LibraryCorrupt,
            format!("The library zip's sha256 is {got}, but this Emmet expects {sha256} (release {release})."),
            "The download is damaged or the file at the URL changed. Run emmet library install again; if it fails \
             the same way, the pinned release moved and Emmet needs an update.",
        )
        .at_file(zip_path.display().to_string()));
    }
    let files = unzip(data, &zip_path)?;
    let info = Installed {
        release: release.into(),
        sha256: got,
        files,
    };
    write_atomic(&data.library_json(), &serde_json::to_vec_pretty(&info).expect("json"))?;
    crate::index::build(data)?;
    if downloaded {
        let _ = std::fs::remove_file(&zip_path);
    }
    Ok(status(data))
}

fn agent() -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder()
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_per_read(Some(Duration::from_secs(60)))
        .build()
        .new_agent()
}

fn download_err(message: String) -> Error {
    Error::new(
        ErrorCode::LibraryMissing,
        message,
        "Run emmet library install again; it picks up where it stopped.",
    )
}

/// Streams `url` into `downloads/complete-<release>.zip.part`, resuming from what's there with a Range request.
fn download(data: &DataDir, release: &str, url: &str, progress: &mut dyn FnMut(u64, Option<u64>)) -> Result<PathBuf> {
    // Plain HTTP only to this machine, for tests. The host is the whole authority, so "http://localhost:@elsewhere"
    // (a user name before the real host) doesn't count as local.
    let local = url.strip_prefix("http://").is_some_and(|rest| {
        let authority = rest.split('/').next().unwrap_or("");
        let host = authority.rsplit_once(':').map_or(authority, |(h, _)| h);
        !authority.contains('@') && (host == "127.0.0.1" || host == "localhost")
    });
    if !url.starts_with("https://") && !local {
        return Err(Error::new(
            ErrorCode::LibraryMissing,
            format!("Refusing to download the library over plain HTTP: {url}"),
            "Use an https:// URL.",
        ));
    }
    let dir = data.downloads();
    std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
    let part = dir.join(format!("complete-{release}.zip.part"));
    let have = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
    let mut req = agent().get(url);
    if have > 0 {
        req = req.header("Range", format!("bytes={have}-"));
    }
    let mut resp = match req.call() {
        Ok(r) => r,
        // 416: the partial file is already as long as (or longer than) the file on the server. It can't resume, so
        // start over next time rather than fail the same way forever.
        Err(ureq::Error::StatusCode(416)) => {
            let _ = std::fs::remove_file(&part);
            return Err(download_err(format!(
                "The partial download of {url} doesn't match the server's file."
            )));
        }
        Err(e) => return Err(download_err(format!("The library download from {url} failed: {e}"))),
    };
    let resumed = resp.status() == 206;
    let length = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok());
    let start = if resumed { have } else { 0 };
    let total = length.map(|l| l + start);
    if total.is_some_and(|t| t > MAX_DOWNLOAD) {
        let _ = std::fs::remove_file(&part);
        return Err(download_err(format!(
            "The library at {url} is larger than {MAX_DOWNLOAD} bytes; refusing it."
        )));
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(resumed)
        .truncate(!resumed)
        .open(&part)
        .map_err(|e| Error::io(&part, e))?;
    let mut reader = resp.body_mut().as_reader();
    let mut done = start;
    let mut buf = vec![0u8; 1 << 16];
    progress(done, total);
    loop {
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                let _ = file.sync_all();
                return Err(download_err(format!(
                    "The library download stopped after {} of {} MB: {e}",
                    mb(done),
                    total.map_or("?".into(), mb)
                )));
            }
        };
        file.write_all(&buf[..n]).map_err(|e| Error::io(&part, e))?;
        done += n as u64;
        if done > MAX_DOWNLOAD {
            drop(file);
            let _ = std::fs::remove_file(&part);
            return Err(download_err(format!(
                "The library at {url} is larger than {MAX_DOWNLOAD} bytes; refusing it."
            )));
        }
        progress(done, total);
    }
    file.sync_all().map_err(|e| Error::io(&part, e))?;
    if total.is_some_and(|t| done < t) {
        return Err(download_err(format!(
            "The library download stopped after {} of {} MB.",
            mb(done),
            mb(total.unwrap())
        )));
    }
    let zip = dir.join(format!("complete-{release}.zip"));
    std::fs::rename(&part, &zip).map_err(|e| Error::io(&zip, e))?;
    Ok(zip)
}

fn mb(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / 1e6)
}

fn corrupt(zip: &Path, message: String) -> Error {
    Error::new(
        ErrorCode::LibraryCorrupt,
        message,
        "The zip may have been tampered with. Nothing was installed; get a fresh copy with emmet library install.",
    )
    .at_file(zip.display().to_string())
}

/// Unzips everything under `ldraw/` into the data dir's `ldraw/`, replacing the old library only once the new one is
/// complete. Every entry is checked before anything is written: one that would land outside the library (an absolute
/// path, `..`, a drive letter) fails the whole install.
fn unzip(data: &DataDir, zip_path: &Path) -> Result<usize> {
    let file = std::fs::File::open(zip_path).map_err(|e| Error::io(zip_path, e))?;
    let mut zip =
        zip::ZipArchive::new(file).map_err(|e| corrupt(zip_path, format!("Can't read the library zip: {e}")))?;
    let mut entries = Vec::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let entry = zip
            .by_index_raw(i)
            .map_err(|e| corrupt(zip_path, format!("Can't read the library zip: {e}")))?;
        let name = String::from_utf8_lossy(entry.name_raw()).into_owned();
        // The raw name is checked as well as zip's own guard, which quietly turns "/x" into "x".
        let raw_ok =
            !name.starts_with(['/', '\\']) && !name.contains(['\\', ':']) && !name.split('/').any(|c| c == "..");
        let rel = entry
            .enclosed_name()
            .filter(|p| raw_ok && p.components().all(|c| matches!(c, std::path::Component::Normal(_))))
            .ok_or_else(|| {
                corrupt(
                    zip_path,
                    format!("The library zip has an entry that escapes its folder: {name}"),
                )
            })?;
        total += entry.size();
        if total > MAX_UNZIPPED {
            return Err(corrupt(
                zip_path,
                format!("The library zip unpacks to more than {MAX_UNZIPPED} bytes."),
            ));
        }
        if let Ok(inner) = rel.strip_prefix("ldraw")
            && !entry.is_dir()
            && !inner.as_os_str().is_empty()
        {
            entries.push((i, inner.to_path_buf()));
        }
    }
    if entries.is_empty() {
        return Err(corrupt(zip_path, "The library zip has no ldraw/ folder.".into()));
    }
    let target = data.ldraw();
    let staging = data.root().join(format!("ldraw.new-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&staging);
    let mut manifest = String::new();
    let unpacked = (|| -> Result<()> {
        for (i, rel) in &entries {
            let mut entry = zip
                .by_index(*i)
                .map_err(|e| corrupt(zip_path, format!("Can't read the library zip: {e}")))?;
            let dest = staging.join(rel);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent).map_err(|e| Error::io(parent, e))?;
            }
            let mut bytes = Vec::with_capacity(entry.size() as usize);
            entry
                .read_to_end(&mut bytes)
                .map_err(|e| corrupt(zip_path, format!("Can't unzip {}: {e}", rel.display())))?;
            std::fs::write(&dest, &bytes).map_err(|e| Error::io(&dest, e))?;
            manifest.push_str(&format!(
                "{}  {}\n",
                fsx::sha256_hex(&bytes),
                rel.to_string_lossy().replace('\\', "/")
            ));
        }
        Ok(())
    })();
    if let Err(e) = unpacked {
        let _ = std::fs::remove_dir_all(&staging);
        return Err(e);
    }
    // The manifest goes in first under a temp name, so the swap can't leave a library without one.
    let manifest_tmp = data.root().join("ldraw.sha256.new");
    write_atomic(&manifest_tmp, manifest.as_bytes())?;
    let old = data.root().join("ldraw.old");
    if old.exists() {
        std::fs::remove_dir_all(&old).map_err(|e| Error::io(&old, e))?;
    }
    let had_old = target.exists();
    if had_old {
        std::fs::rename(&target, &old).map_err(|e| Error::io(&target, e))?;
    }
    if let Err(e) = std::fs::rename(&staging, &target) {
        if had_old {
            let _ = std::fs::rename(&old, &target); // put the working library back
        }
        let _ = std::fs::remove_dir_all(&staging);
        return Err(Error::io(&target, e));
    }
    std::fs::rename(&manifest_tmp, data.library_manifest()).map_err(|e| Error::io(&data.library_manifest(), e))?;
    if had_old && let Err(e) = std::fs::remove_dir_all(&old) {
        eprintln!("emmet: couldn't remove the previous library at {}: {e}", old.display());
    }
    Ok(entries.len())
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Verified {
    pub release: String,
    pub files: usize,
    /// Parts in the index after `--reindex`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reindexed_parts: Option<usize>,
}

/// Re-hashes every installed file against the manifest written at install time.
pub fn verify(data: &DataDir, reindex: bool) -> Result<Verified> {
    let info = require(data)?;
    let manifest_path = data.library_manifest();
    let manifest = std::fs::read_to_string(&manifest_path).map_err(|e| Error::io(&manifest_path, e))?;
    let root = data.ldraw();
    let mut bad = Vec::new();
    let mut files = 0;
    for line in manifest.lines() {
        let Some((hash, rel)) = line.split_once("  ") else {
            return Err(Error::new(
                ErrorCode::LibraryCorrupt,
                format!(
                    "{} has a line that isn't \"sha256  path\": {line}",
                    manifest_path.display()
                ),
                "Run emmet library install to put the library back.",
            )
            .at_file(manifest_path.display().to_string()));
        };
        files += 1;
        let ok = std::fs::read(root.join(rel))
            .map(|b| fsx::sha256_hex(&b) == hash)
            .unwrap_or(false);
        if !ok {
            bad.push(rel.to_string());
        }
    }
    if files != info.files {
        return Err(Error::new(
            ErrorCode::LibraryCorrupt,
            format!(
                "{} lists {files} files, but {} were installed.",
                manifest_path.display(),
                info.files
            ),
            "Run emmet library install to put the library back.",
        )
        .at_file(manifest_path.display().to_string()));
    }
    if !bad.is_empty() {
        let shown = bad.iter().take(5).cloned().collect::<Vec<_>>().join(", ");
        return Err(Error::new(
            ErrorCode::LibraryCorrupt,
            format!("{} of {files} library files are missing or changed: {shown}", bad.len()),
            "Run emmet library install to put the library back.",
        )
        .at_file(root.join(&bad[0]).display().to_string()));
    }
    let reindexed_parts = if reindex {
        Some(crate::index::build(data)?)
    } else {
        None
    };
    Ok(Verified {
        release: info.release,
        files,
        reindexed_parts,
    })
}
