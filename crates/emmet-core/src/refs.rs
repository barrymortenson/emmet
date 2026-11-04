// SPDX-License-Identifier: AGPL-3.0-or-later
//! Reference photos (4D): what the user attaches in the chat, saved in the project's `refs/` folder so the project
//! stays reproducible. The app converts every photo to JPEG first (PNG, JPEG, and HEIC in; the webview re-encodes
//! it); this module is the guarantee that no metadata goes further: it walks the JPEG's segments and drops every
//! APPn and COM segment (EXIF with its GPS block, XMP, ICC, IPTC, maker notes, comments) before the file is saved or
//! sent, keeping only a bare JFIF header and Adobe's color-transform flag. It also holds every image to the limits
//! the providers document (docs/spec/sessions/4D.md): a long edge of at most 2,048 px and at most 3.75 MB.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::envelope::{Error, ErrorCode, Result};
use crate::fsx;

/// The folder in a project that holds reference photos.
pub const DIR: &str = "refs";
/// The longest edge an attachment may have, in pixels: OpenAI's `high` detail fits 2048 × 2048, and Claude takes up to
/// 2,576 (3,000 when a request has more than 20 images).
pub const MAX_EDGE: u32 = 2048;
/// The largest attachment, in bytes: 5 MB once base64-encoded, Claude's smallest documented per-image limit (Bedrock).
pub const MAX_BYTES: usize = 3_932_160;

/// A photo saved in `refs/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefImage {
    /// The project-relative path, with `/`: `refs/<name>-<hash>.jpg`.
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub bytes: u32,
}

/// A JPEG with its metadata dropped, and its size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clean {
    pub jpeg: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

fn bad(message: impl Into<String>) -> Error {
    Error::new(
        ErrorCode::ParamInvalid,
        message,
        "Try the photo again, or save it as a JPEG first.",
    )
}

/// `bytes` as a JPEG with every APPn and COM segment dropped (a JFIF header without its thumbnail, and Adobe's
/// color-transform segment, stay). Fails on anything that isn't a whole baseline or progressive JPEG.
pub fn clean_jpeg(bytes: &[u8]) -> Result<Clean> {
    if bytes.len() < 4 || bytes[0] != 0xFF || bytes[1] != 0xD8 {
        return Err(bad("That isn't a JPEG."));
    }
    let mut out = Vec::with_capacity(bytes.len());
    out.extend_from_slice(&[0xFF, 0xD8]);
    let mut size: Option<(u32, u32)> = None;
    let mut scanned = false;
    let mut i = 2;
    loop {
        // A marker: 0xFF, any fill bytes (more 0xFF), then its code.
        if i >= bytes.len() || bytes[i] != 0xFF {
            return Err(bad("The JPEG is damaged or cut short."));
        }
        while i < bytes.len() && bytes[i] == 0xFF {
            i += 1;
        }
        let Some(&code) = bytes.get(i) else {
            return Err(bad("The JPEG is cut short."));
        };
        i += 1;
        match code {
            0xD9 => {
                if !scanned {
                    return Err(bad("The JPEG has no image data."));
                }
                out.extend_from_slice(&[0xFF, 0xD9]);
                break;
            }
            // Markers without a length.
            0x01 | 0xD0..=0xD7 => {
                out.extend_from_slice(&[0xFF, code]);
                continue;
            }
            0x00 | 0xD8 => return Err(bad("The JPEG is damaged.")),
            _ => {}
        }
        let len = match bytes.get(i..i + 2) {
            Some(b) => usize::from(u16::from_be_bytes([b[0], b[1]])),
            None => return Err(bad("The JPEG is cut short.")),
        };
        if len < 2 || i + len > bytes.len() {
            return Err(bad("The JPEG is damaged or cut short."));
        }
        let body = &bytes[i + 2..i + len];
        let segment = &bytes[i - 2..i + len];
        i += len;
        match code {
            // JFIF: kept, without its thumbnail (version, units, and density only).
            0xE0 if body.starts_with(b"JFIF\0") && body.len() >= 14 => {
                out.extend_from_slice(&[0xFF, 0xE0, 0x00, 0x10]);
                out.extend_from_slice(&body[..12]);
                out.extend_from_slice(&[0, 0]);
            }
            // Adobe's color transform: how the decoder reads the channels, nothing about the photo.
            0xEE if body.starts_with(b"Adobe") && body.len() == 12 => out.extend_from_slice(segment),
            // Every other application segment and every comment: EXIF, XMP, ICC, IPTC, maker notes.
            0xE0..=0xEF | 0xFE => {}
            // Start of frame (not DHT, JPG, or DAC): the image's size.
            0xC0..=0xCF if !matches!(code, 0xC4 | 0xC8 | 0xCC) => {
                if body.len() < 5 {
                    return Err(bad("The JPEG is damaged."));
                }
                let h = u32::from(u16::from_be_bytes([body[1], body[2]]));
                let w = u32::from(u16::from_be_bytes([body[3], body[4]]));
                size.get_or_insert((w, h));
                out.extend_from_slice(segment);
            }
            0xDA => {
                out.extend_from_slice(segment);
                scanned = true;
                // The entropy-coded data runs to the next marker that isn't a stuffed 0xFF00 or a restart.
                let start = i;
                while i + 1 < bytes.len() {
                    if bytes[i] == 0xFF && bytes[i + 1] != 0x00 && !(0xD0..=0xD7).contains(&bytes[i + 1]) {
                        break;
                    }
                    i += 1;
                }
                if i + 1 >= bytes.len() {
                    return Err(bad("The JPEG is cut short."));
                }
                out.extend_from_slice(&bytes[start..i]);
            }
            _ => out.extend_from_slice(segment),
        }
    }
    let Some((width, height)) = size.filter(|(w, h)| *w > 0 && *h > 0) else {
        return Err(bad("The JPEG has no image size."));
    };
    Ok(Clean {
        jpeg: out,
        width,
        height,
    })
}

/// Cleans `bytes` and checks it against the limits; what the app saves and sends.
pub fn prepare(bytes: &[u8]) -> Result<Clean> {
    let clean = clean_jpeg(bytes)?;
    if clean.width.max(clean.height) > MAX_EDGE {
        return Err(Error::new(
            ErrorCode::ParamInvalid,
            format!(
                "The photo is {} × {} pixels; the agents take at most {MAX_EDGE} on its longest side.",
                clean.width, clean.height
            ),
            "Attach it again: Emmet scales it down when it attaches it.",
        ));
    }
    if clean.jpeg.len() > MAX_BYTES {
        return Err(Error::new(
            ErrorCode::ParamInvalid,
            format!(
                "The photo is {:.1} MB; the agents take at most 3.75 MB.",
                clean.jpeg.len() as f64 / 1e6
            ),
            "Attach it again: Emmet compresses it when it attaches it.",
        ));
    }
    Ok(clean)
}

/// A file name's stem as a short slug: lowercase letters, digits, and dashes; "photo" when nothing is left.
fn slug(name: &str) -> String {
    let stem = Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut out = String::new();
    for c in stem.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
        if out.len() >= 40 {
            break;
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() { "photo".into() } else { out }
}

/// Saves a photo in `<root>/refs/`, cleaned and checked: `<name>-<hash>.jpg`, so the same photo attached twice is one
/// file. `name` is the file's own name, for the slug only.
pub fn save(root: &Path, name: &str, bytes: &[u8]) -> Result<RefImage> {
    let clean = prepare(bytes)?;
    let hash = fsx::sha256_hex(&clean.jpeg);
    let file = format!("{}-{}.jpg", slug(name), &hash[..12]);
    let path = fsx::own_file(root, &[DIR], &file).ok_or_else(|| {
        Error::new(
            ErrorCode::IoError,
            "The project's refs folder is a link, or isn't a folder.",
            "Emmet never writes through a link in a project. Remove or rename refs, then attach the photo again.",
        )
    })?;
    if !path.is_file() {
        fsx::write_atomic(&path, &clean.jpeg)?;
    }
    Ok(RefImage {
        path: format!("{DIR}/{file}"),
        width: clean.width,
        height: clean.height,
        bytes: clean.jpeg.len() as u32,
    })
}

/// The file a ref names, checked: `refs/<name>.jpg` exactly (one level, no `..`), a regular file that resolves inside
/// the project's own `refs/`. The app checks every path the page sends with this before reading or sending it.
pub fn resolve(root: &Path, rel: &str) -> Result<PathBuf> {
    let not_ref = || {
        Error::new(
            ErrorCode::ParamInvalid,
            format!("{rel} isn't a photo in this project."),
            "Attach the photo again.",
        )
    };
    let name = rel.strip_prefix("refs/").ok_or_else(not_ref)?;
    if name.is_empty() || name.contains(['/', '\\', ':']) || name.starts_with('.') || !name.ends_with(".jpg") {
        return Err(not_ref());
    }
    let dir = fsx::canonical(&root.join(DIR)).map_err(|_| not_ref())?;
    let root = fsx::canonical(root).map_err(|_| not_ref())?;
    if dir != root.join(DIR) {
        return Err(not_ref());
    }
    let path = dir.join(name);
    match std::fs::symlink_metadata(&path) {
        Ok(m) if m.is_file() => Ok(path),
        _ => Err(not_ref()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A JPEG's skeleton: SOI, the segments, an SOS with `scan` as its data, EOI.
    fn jpeg(segments: &[(u8, &[u8])], scan: &[u8]) -> Vec<u8> {
        let mut v = vec![0xFF, 0xD8];
        for (code, body) in segments {
            v.extend_from_slice(&[0xFF, *code]);
            v.extend_from_slice(&((body.len() + 2) as u16).to_be_bytes());
            v.extend_from_slice(body);
        }
        v.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x08, 1, 1, 0, 0, 63, 0]);
        v.extend_from_slice(scan);
        v.extend_from_slice(&[0xFF, 0xD9]);
        v
    }
    const SOF: &[u8] = &[8, 0x01, 0x2C, 0x01, 0x90, 1, 1, 0x11, 0]; // 400 wide, 300 tall

    #[test]
    fn drops_every_app_and_comment_segment_and_keeps_the_scan() {
        let mut jfif = b"JFIF\0\x01\x02\x00\x00\x48\x00\x48\x02\x01".to_vec();
        jfif.extend_from_slice(&[9; 6]); // a 2 × 1 thumbnail
        // Stuffed 0xFF00 bytes and a restart marker inside the scan, which must survive as they are.
        let scan = [0x12, 0xFF, 0x00, 0x34, 0xFF, 0xD3, 0x56];
        let src = jpeg(
            &[
                (0xE0, &jfif),
                (0xE1, b"Exif\0\0GPS 34.05N 118.24W"),
                (0xE1, b"http://ns.adobe.com/xap/1.0/\0<x:xmpmeta>"),
                (0xE2, b"ICC_PROFILE\0\x01\x01"),
                (0xED, b"Photoshop 3.0\0"),
                (0xEE, b"Adobe\0\x64\0\0\0\0\x01"),
                (0xFE, b"taken at home"),
                (0xDB, &[0; 65]),
                (0xC0, SOF),
            ],
            &scan,
        );
        let c = clean_jpeg(&src).unwrap();
        assert_eq!((c.width, c.height), (400, 300));
        let want = jpeg(
            &[
                (0xE0, b"JFIF\0\x01\x02\x00\x00\x48\x00\x48\x00\x00"),
                (0xEE, b"Adobe\0\x64\0\0\0\0\x01"),
                (0xDB, &[0; 65]),
                (0xC0, SOF),
            ],
            &scan,
        );
        assert_eq!(c.jpeg, want);
        // Cleaning is idempotent.
        assert_eq!(clean_jpeg(&c.jpeg).unwrap().jpeg, want);
    }

    #[test]
    fn a_progressive_jpeg_keeps_its_scans_and_tables_between_them() {
        let mut src = jpeg(&[(0xC2, SOF), (0xC4, &[0; 20])], &[1, 2, 3]);
        src.truncate(src.len() - 2);
        src.extend_from_slice(&[0xFF, 0xC4, 0x00, 0x04, 7, 7]);
        src.extend_from_slice(&[0xFF, 0xE1, 0x00, 0x05, b'G', b'P', b'S']);
        src.extend_from_slice(&[0xFF, 0xDA, 0x00, 0x08, 1, 1, 0, 0, 63, 0, 4, 5, 0xFF, 0xD9]);
        let c = clean_jpeg(&src).unwrap();
        assert!(!c.jpeg.windows(3).any(|w| w == b"GPS"));
        assert!(c.jpeg.ends_with(&[
            0xFF, 0xC4, 0x00, 0x04, 7, 7, 0xFF, 0xDA, 0x00, 0x08, 1, 1, 0, 0, 63, 0, 4, 5, 0xFF, 0xD9
        ]));
    }

    #[test]
    fn refuses_what_isnt_a_whole_jpeg() {
        let good = jpeg(&[(0xC0, SOF)], &[1, 2]);
        for bytes in [
            &b"\x89PNG\r\n\x1a\n"[..],
            &good[..good.len() - 2],                         // no EOI
            &good[..10],                                     // cut in a segment
            &jpeg(&[(0xDB, &[0; 4])], &[1])[..],             // no frame, so no size
            &[0xFF, 0xD8, 0xFF, 0xE1, 0x00, 0x40, 1, 2][..], // a length past the end
        ] {
            assert!(clean_jpeg(bytes).is_err(), "{bytes:?}");
        }
    }

    #[test]
    fn slugs() {
        assert_eq!(slug("IMG_2424.HEIC"), "img-2424");
        assert_eq!(
            slug("CleanShot 2026-11-04 at 12.42.18 PM@2x.png"),
            "cleanshot-2026-11-04-at-12-42-18-pm-2x"
        );
        assert_eq!(slug("..."), "photo");
        assert_eq!(slug(""), "photo");
        assert_eq!(slug("Ünïcode éé.jpg"), "n-code");
    }
}
