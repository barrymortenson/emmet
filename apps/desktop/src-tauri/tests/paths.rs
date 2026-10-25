// SPDX-License-Identifier: AGPL-3.0-or-later
//! Traversal tests for every IPC path argument (plan section 15, 1B Done when 5): the `emmet-lib://` protocol, which
//! takes a path in its URL, and `open_recent`, which takes a path from the webview. Nothing outside the library or
//! the recents list is reachable, however the path is spelled.

use std::path::Path;

use emmet_core::data::DataDir;
use emmet_desktop::library::Library;
use emmet_desktop::recent;

/// A small library with a part, a subpart, and a primitive, next to a secret it must never serve.
fn library() -> (std::path::PathBuf, Library) {
    let dir = emmet_it::temp_dir("lib-paths");
    let lib = dir.join("ldraw");
    for (rel, text) in [
        (
            "parts/3001.dat",
            "0 Brick  2 x  4\n0 !LDRAW_ORG Part\n1 16 0 0 0 1 0 0 0 1 0 0 0 1 s\\3001s01.dat\n",
        ),
        (
            "parts/s/3001s01.dat",
            "0 ~Brick  2 x  4 without Front Face\n1 16 0 0 0 1 0 0 0 1 0 0 0 1 stud.dat\n",
        ),
        ("p/stud.dat", "0 Stud\n4 16 0 0 0 1 0 0 1 1 0 0 1 0\n"),
        ("LDConfig.ldr", "0 !COLOUR Black CODE 0 VALUE #1B2A34 EDGE #808080\n"),
    ] {
        let path = lib.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    std::fs::write(dir.join("secret.txt"), "not a part").unwrap();
    (dir.clone(), Library::new(lib))
}

fn status(lib: &Library, path: &str) -> u16 {
    lib.respond(path, None).status
}

#[test]
fn the_library_protocol_serves_library_files_in_any_case() {
    let (_, lib) = library();
    assert_eq!(status(&lib, "/parts/3001.dat"), 200);
    assert_eq!(status(&lib, "/PARTS/S/3001S01.DAT"), 200);
    assert_eq!(status(&lib, "/LDConfig.ldr"), 200);
    assert_eq!(status(&lib, "/parts/9999.dat"), 404);
}

#[test]
fn the_library_protocol_refuses_every_way_out() {
    let (_, lib) = library();
    for path in [
        "/../secret.txt",
        "/parts/../../secret.txt",
        "/parts/%2e%2e/%2e%2e/secret.txt",
        "/parts%2F..%2F..%2Fsecret.txt",
        "/parts%5C..%5C..%5Csecret.txt",
        "/parts\\..\\..\\secret.txt",
        "/%2Fetc%2Fhosts",
        "//etc/hosts",
        "/C:%5CWindows%5Cwin.ini",
        "/./parts/3001.dat",
        "/parts//3001.dat",
        "/%ZZ",
    ] {
        assert_eq!(status(&lib, path), 404, "{path} should be refused");
    }
}

#[test]
fn the_library_protocol_refuses_a_link_out_of_the_library() {
    let (dir, lib) = library();
    let link = dir.join("ldraw/parts/leak.dat");
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(dir.join("secret.txt"), &link).is_ok();
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_file(dir.join("secret.txt"), &link).is_ok();
    if !made {
        // Windows without Developer Mode can't make links; nothing to escape through then.
        eprintln!("skipped: this machine can't create symlinks");
        return;
    }
    assert_eq!(status(&lib, "/parts/leak.dat"), 404);
}

#[test]
fn a_pack_holds_every_file_a_part_uses_and_reports_what_is_missing() {
    let (_, lib) = library();
    let pack = lib.pack(&["3001", "../secret", "9999"]);
    assert_eq!(pack.parts["3001"], ["3001.dat", "parts/s/3001s01.dat", "stud.dat"]);
    assert!(pack.files["stud.dat"].starts_with("0 Stud"));
    assert_eq!(pack.missing["9999"], ["9999.dat"]);
    assert_eq!(pack.missing["../secret"], ["../secret.dat"]);
    assert!(!pack.files.values().any(|t| t.contains("not a part")));
    let reply = lib.respond("/pack", Some("parts=3001%2C..%2Fsecret"));
    assert_eq!(reply.status, 200);
    assert!(!String::from_utf8(reply.body).unwrap().contains("not a part"));
}

#[test]
fn open_recent_opens_only_a_project_on_the_list() {
    let dir = emmet_it::temp_dir("recent");
    let data = DataDir::at(dir.join("data"));
    let listed = dir.join("listed");
    let other = dir.join("other");
    std::fs::create_dir_all(listed.join("models")).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    let listed = emmet_core::fsx::canonical(&listed).unwrap();
    recent::touch(&data, &listed, "Listed");

    assert_eq!(recent::find(&data, &listed.display().to_string()), Some(listed.clone()));
    // The same folder, spelled another way, is still that project.
    let roundabout = listed.join("models").join("..");
    assert_eq!(
        recent::find(&data, &roundabout.display().to_string()),
        Some(listed.clone())
    );
    for path in [
        other.display().to_string(),
        listed.join("..").display().to_string(),
        listed.join("models").display().to_string(),
        listed.join("../other").display().to_string(),
        "/".into(),
        "".into(),
        "relative/path".into(),
    ] {
        assert_eq!(recent::find(&data, &path), None, "{path} isn't on the list");
    }
}

#[test]
fn open_recent_refuses_a_link_to_an_unlisted_folder() {
    let dir = emmet_it::temp_dir("recent-link");
    let data = DataDir::at(dir.join("data"));
    let listed = dir.join("listed");
    let other = dir.join("other");
    std::fs::create_dir_all(&listed).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    recent::touch(&data, &emmet_core::fsx::canonical(&listed).unwrap(), "Listed");
    let link = dir.join("looks-listed");
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(&other, &link).is_ok();
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_dir(&other, &link).is_ok();
    if !made {
        eprintln!("skipped: this machine can't create symlinks");
        return;
    }
    assert_eq!(recent::find(&data, &link.display().to_string()), None);
    assert!(Path::new(&link).exists());
}
