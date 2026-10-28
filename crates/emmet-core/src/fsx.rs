// SPDX-License-Identifier: AGPL-3.0-or-later
//! Filesystem helpers: atomic writes (plan section 07) and hashing.

use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};

use crate::envelope::{Error, Result};

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
        std::fs::rename(&tmp, path)
    })();
    written.map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        Error::io(path, e)
    })
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

/// `canonicalize`, without the `\\?\` prefix Windows adds to drive paths, so the path reads and joins like the one
/// the user typed (Python, error messages, and path comparisons all see `C:\…`).
pub fn canonical(path: &Path) -> std::io::Result<std::path::PathBuf> {
    let c = path.canonicalize()?;
    #[cfg(windows)]
    {
        let s = c.to_string_lossy();
        if let Some(rest) = s.strip_prefix(r"\\?\")
            && rest.as_bytes().get(1) == Some(&b':')
        {
            return Ok(std::path::PathBuf::from(rest));
        }
    }
    Ok(c)
}
