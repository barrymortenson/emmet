// SPDX-License-Identifier: AGPL-3.0-or-later
//! Filesystem helpers: atomic writes (plan section 07) and hashing.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use crate::envelope::{Error, Result};

/// `root/<dirs…>/<name>` for Emmet to write, with each folder made if it's missing; None when a folder on the way or the
/// file is a link (or not what it should be). A project may be someone else's, so Emmet never writes through a link
/// it carries.
pub fn own_file(root: &Path, dirs: &[&str], name: &str) -> Option<PathBuf> {
    let mut dir = root.to_path_buf();
    for part in dirs {
        dir.push(part);
        match std::fs::symlink_metadata(&dir) {
            Ok(m) if m.is_dir() && !m.file_type().is_symlink() => {}
            Ok(_) => return None,
            Err(_) => std::fs::create_dir(&dir).ok()?,
        }
    }
    let path = dir.join(name);
    let linked = std::fs::symlink_metadata(&path).is_ok_and(|m| !m.is_file());
    (!linked).then_some(path)
}

/// Writes `bytes` to `path` so readers never see half a file: a temp file in the same directory, fsync, rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    write_atomic_with(path, bytes, false)
}

/// [`write_atomic`] for a secret: on Unix the file is readable by this user only (0600) from the moment it exists.
pub fn write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    write_atomic_with(path, bytes, true)
}

fn write_atomic_with(path: &Path, bytes: &[u8], private: bool) -> Result<()> {
    let dir = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    // A fresh name each time, created with create_new: it fails on anything already there, a planted symlink included.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nonce = RandomState::new().build_hasher().finish() ^ COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{name}.{}-{nonce:016x}.tmp", std::process::id()));
    let written = (|| {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        if private {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        #[cfg(not(unix))]
        let _ = private;
        let mut f = options.open(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        rename(&tmp, path)
    })();
    written.map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        Error::io(path, e)
    })
}

/// `std::fs::rename`, tried again for up to two seconds while Windows says another process has the file: OneDrive
/// and other sync clients, virus scanners, and the search indexer open a file for a moment just after it changes, and
/// a rename onto a file they hold fails with a sharing violation, a lock violation, or access denied.
pub fn rename(from: &Path, to: &Path) -> std::io::Result<()> {
    let mut wait = std::time::Duration::from_millis(10);
    let start = std::time::Instant::now();
    loop {
        match std::fs::rename(from, to) {
            Err(e) if held(&e) && start.elapsed() < std::time::Duration::from_secs(2) => {
                std::thread::sleep(wait);
                wait = (wait * 2).min(std::time::Duration::from_millis(200));
            }
            done => return done,
        }
    }
}

/// True for the errors Windows gives while another process holds a file: ERROR_ACCESS_DENIED (5),
/// ERROR_SHARING_VIOLATION (32), and ERROR_LOCK_VIOLATION (33).
fn held(e: &std::io::Error) -> bool {
    cfg!(windows) && matches!(e.raw_os_error(), Some(5 | 32 | 33))
}

/// `words` × 64 random bits as hex, from the OS-seeded keys std hashes with, mixed with the time and process id.
/// For secrets that only need to be unguessable to other local processes: the machine id and the bus token.
pub fn random_hex(words: usize) -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut out = String::new();
    for _ in 0..words {
        let mut h = RandomState::new().build_hasher();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        h.write_u128(now);
        h.write_u32(std::process::id());
        h.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
        out.push_str(&format!("{:016x}", h.finish()));
    }
    out
}

/// Lowercase hex sha256.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

/// Lowercase hex sha256 of a file, read in chunks.
pub fn sha256_file(path: &Path) -> Result<String> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).map_err(|e| Error::io(path, e))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        match f.read(&mut buf).map_err(|e| Error::io(path, e))? {
            0 => break,
            n => h.update(&buf[..n]),
        }
    }
    Ok(hex(&h.finalize()))
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `path` relative to `root` with `/` separators, or the full path when it isn't under `root`.
pub fn display_rel(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(rel) => rel.to_string_lossy().replace('\\', "/"),
        Err(_) => path.display().to_string(),
    }
}

/// `canonicalize`, without the `\\?\` prefix Windows adds, so the path reads and joins like the one the user typed
/// (Python, error messages, and path comparisons all see `C:\…` or `\\server\share\…`).
pub fn canonical(path: &Path) -> std::io::Result<std::path::PathBuf> {
    let c = path.canonicalize()?;
    if cfg!(windows)
        && let Some(plain) = c.to_str().and_then(plain_windows)
    {
        return Ok(std::path::PathBuf::from(plain));
    }
    Ok(c)
}

/// A verbatim Windows path as the plain one that names the same file: `\\?\C:\a` is `C:\a` and `\\?\UNC\pc\share\a`
/// is `\\pc\share\a`. None for any other path, and for a verbatim path with no plain spelling (a volume GUID).
fn plain_windows(path: &str) -> Option<String> {
    let rest = path.strip_prefix(r"\\?\")?;
    if let Some(share) = rest.strip_prefix(r"UNC\") {
        return Some(format!(r"\\{share}"));
    }
    let b = rest.as_bytes();
    (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && b[2] == b'\\').then(|| rest.to_string())
}

/// A name as one plain file or folder name: no separators, no characters Windows refuses, no control characters, and
/// no leading dots (so never "." or ".."). A project's name comes from its emmet.toml, which a shared project writes.
pub fn file_name(name: &str) -> String {
    let clean: String = name
        .chars()
        .map(|c| {
            if c.is_control() || "/\\:*?\"<>|".contains(c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let words = clean.split_whitespace().collect::<Vec<_>>().join(" ");
    let clean = words.trim_start_matches(['.', ' ']);
    if clean.is_empty() {
        "Model".into()
    } else {
        clean.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::plain_windows;

    #[test]
    fn verbatim_windows_paths_read_as_plain_ones() {
        assert_eq!(
            plain_windows(r"\\?\C:\Users\a b\Emmet").as_deref(),
            Some(r"C:\Users\a b\Emmet")
        );
        assert_eq!(
            plain_windows(r"\\?\UNC\Mac\Home\Emmet").as_deref(),
            Some(r"\\Mac\Home\Emmet")
        );
        for other in [
            r"C:\Users",
            r"\\Mac\Home",
            r"\\?\Volume{4c1b02c1-d990-11dc-99ae-806e6f6e6963}\a",
            r"\\?\C:",
            "/tmp/a",
        ] {
            assert_eq!(plain_windows(other), None, "{other}");
        }
    }
}
