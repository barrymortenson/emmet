// SPDX-License-Identifier: AGPL-3.0-or-later
//! Reference photos (4D's Done when 2): a geotagged photo saved the way `chat_attach` saves it keeps none of its
//! metadata, and the paths the page sends back are held to the project's own `refs/`.

use emmet_core::refs;
use emmet_it::{repo, temp_dir};

pub fn geotagged() -> Vec<u8> {
    std::fs::read(repo().join("tests/fixtures/photos/geotagged.jpg")).unwrap()
}

/// What a photo's metadata looks like in its bytes: EXIF and its GPS tags, XMP, ICC, the comment.
pub fn has_metadata(jpeg: &[u8]) -> Option<&'static str> {
    let has = |needle: &[u8]| jpeg.windows(needle.len()).any(|w| w == needle);
    [
        ("EXIF", &b"Exif\0\0"[..]),
        ("TIFF", b"II*\0"),
        ("GPS", b"GPS"),
        ("XMP", b"xmpmeta"),
        ("ICC", b"ICC_PROFILE"),
        ("the comment", b"Broadway"),
    ]
    .into_iter()
    .find(|(_, needle)| has(needle))
    .map(|(name, _)| name)
}

/// The image data: from the first scan to the end.
fn scan(jpeg: &[u8]) -> &[u8] {
    let at = jpeg.windows(2).position(|w| w == [0xFF, 0xDA]).unwrap();
    &jpeg[at..]
}

#[test]
fn a_geotagged_photo_is_saved_without_its_metadata() {
    let src = geotagged();
    // The fixture is what a phone writes (tests/fixtures/photos/make.py; macOS ImageIO reads its GPS block).
    assert_eq!(has_metadata(&src), Some("EXIF"));
    let root = temp_dir("refs-geotagged");
    let saved = refs::save(&root, "IMG_2424.jpg", &src).unwrap();
    assert!(
        saved.path.starts_with("refs/img-2424-") && saved.path.ends_with(".jpg"),
        "{saved:?}"
    );
    assert_eq!((saved.width, saved.height), (64, 48));
    let bytes = std::fs::read(root.join(&saved.path)).unwrap();
    assert_eq!(has_metadata(&bytes), None);
    // Every segment left is a table, the frame, or a bare JFIF header: no APP1 to APP15, no comment.
    let mut i = 2;
    while bytes[i + 1] != 0xDA {
        let code = bytes[i + 1];
        assert!(
            code == 0xE0 || !(0xE1..=0xEF).contains(&code) && code != 0xFE,
            "segment {code:02X} kept"
        );
        i += 2 + usize::from(u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]));
    }
    // The pixels are untouched.
    assert_eq!(scan(&bytes), scan(&src));
    // The same photo again is the same file.
    assert_eq!(refs::save(&root, "IMG_2424.jpg", &src).unwrap(), saved);
    assert_eq!(std::fs::read_dir(root.join("refs")).unwrap().count(), 1);
    // One opened and saved in another app since (which can put metadata back) fails the send-time check, naming
    // the file, and attaching the photo again writes it clean.
    let file = root.join(&saved.path);
    std::fs::write(&file, &src).unwrap();
    let e = refs::check(&file).unwrap_err();
    assert!(
        e.message.contains(saved.path.trim_start_matches("refs/")) && e.message.contains("metadata"),
        "{}",
        e.message
    );
    refs::save(&root, "IMG_2424.jpg", &src).unwrap();
    assert_eq!(refs::check(&file).unwrap(), std::fs::read(&file).unwrap());
    assert_eq!(has_metadata(&std::fs::read(&file).unwrap()), None);
}

#[test]
fn a_photo_over_the_limits_or_not_a_jpeg_is_refused() {
    let root = temp_dir("refs-limits");
    let png = std::fs::read(repo().join("apps/desktop/src-tauri/icons/32x32.png")).unwrap();
    let e = refs::save(&root, "icon.png", &png).unwrap_err();
    assert_eq!(e.message, "That isn't a JPEG.");
    // A frame wider than the long edge allows: the agents would scale it, so the app does it first.
    let mut wide = geotagged();
    let sof = wide.windows(2).position(|w| w == [0xFF, 0xC0]).unwrap();
    wide[sof + 7..sof + 9].copy_from_slice(&3000u16.to_be_bytes());
    let e = refs::save(&root, "wide.jpg", &wide).unwrap_err();
    assert!(e.message.contains("3000 × 48"), "{}", e.message);
    // The long edge exactly at the limit is fine (the page scales to it); one more pixel isn't.
    wide[sof + 7..sof + 9].copy_from_slice(&2049u16.to_be_bytes());
    assert!(refs::prepare(&wide).is_err());
    wide[sof + 7..sof + 9].copy_from_slice(&2048u16.to_be_bytes());
    assert_eq!(refs::prepare(&wide).unwrap().width, 2048);
    assert!(!root.join("refs").exists() || std::fs::read_dir(root.join("refs")).unwrap().count() == 0);
}

#[test]
fn only_a_photo_in_the_projects_own_refs_resolves() {
    let root = temp_dir("refs-resolve");
    let saved = refs::save(&root, "a.jpg", &geotagged()).unwrap();
    assert_eq!(
        refs::resolve(&root, &saved.path).unwrap(),
        emmet_core::fsx::canonical(&root.join(&saved.path)).unwrap()
    );
    std::fs::write(root.join("model.py"), "").unwrap();
    std::fs::write(root.join("refs/notes.txt"), "").unwrap();
    for rel in [
        "model.py",
        "refs/../model.py",
        "refs/notes.txt",
        "refs/missing.jpg",
        "refs/sub/a.jpg",
        "refs/.hidden.jpg",
        "/etc/hosts",
        "refs\\..\\model.jpg",
    ] {
        assert!(refs::resolve(&root, rel).is_err(), "{rel} resolved");
    }
    // A refs folder that's a link out of the project resolves nothing, and nothing is written through it.
    #[cfg(unix)]
    {
        let other = temp_dir("refs-elsewhere");
        let linked = temp_dir("refs-linked");
        std::fs::create_dir_all(other.join("x")).unwrap();
        std::fs::copy(
            root.join(&saved.path),
            other.join(saved.path.trim_start_matches("refs/")),
        )
        .unwrap();
        std::os::unix::fs::symlink(&other, linked.join("refs")).unwrap();
        assert!(refs::resolve(&linked, &saved.path).is_err());
        assert!(refs::save(&linked, "b.jpg", &geotagged()).is_err());
        // A link inside refs/ to a file elsewhere resolves nothing either (chat_photo would read what it points at).
        let secret = other.join("id_ed25519");
        std::fs::write(&secret, "a key").unwrap();
        std::os::unix::fs::symlink(&secret, root.join("refs/key.jpg")).unwrap();
        assert!(refs::resolve(&root, "refs/key.jpg").is_err());
    }
    // Nothing bigger than a photo Emmet saves is read.
    std::fs::write(root.join("refs/huge.jpg"), vec![0u8; refs::MAX_BYTES + 1]).unwrap();
    assert!(refs::resolve(&root, "refs/huge.jpg").is_err());
}
