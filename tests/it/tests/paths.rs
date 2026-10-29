// SPDX-License-Identifier: AGPL-3.0-or-later
//! Path cases that differ on Windows (plan section 12): spaces and non-ASCII in the project path, a relative
//! --project, finding the project from a subfolder, and part ids written with a different case or a .dat suffix.
//! The LDraw reference cases (backslashes, mixed case) are in ldraw.rs. Both run on macOS and Windows.

use emmet_it::{emmet, envelope, shared_data, temp_dir};

#[test]
fn a_project_path_with_spaces_and_accents() {
    let parent = temp_dir("paths");
    let dir = parent.join("Mes maquettes é/My Cottage");
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .arg("new")
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(envelope(&out).1, 0, "{}", emmet_it::text(&out));

    // A relative --project, from the parent folder.
    let out = emmet()
        .current_dir(&parent)
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(["run", "--project", "Mes maquettes é/My Cottage"])
        .output()
        .unwrap();
    let (v, code) = envelope(&out);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["parts"], 147);
    assert!(v["meta"]["project"].as_str().unwrap().ends_with("My Cottage"), "{v}");

    // No --project at all, from a folder inside the project: emmet looks upward for emmet.toml.
    std::fs::create_dir_all(dir.join("models/roofs")).unwrap();
    let out = emmet()
        .current_dir(dir.join("models/roofs"))
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .arg("run")
        .output()
        .unwrap();
    assert_eq!(envelope(&out).1, 0, "{}", emmet_it::text(&out));
}

#[test]
fn part_ids_and_errors_use_forward_slashes_and_lowercase() {
    let dir = temp_dir("ids").join("ids");
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success());
    let model = "# SPDX-License-Identifier: AGPL-3.0-or-later\nfrom emmet_dsl import Model, model\n\n\n@model\ndef ids():\n    \"\"\"Part ids in other spellings.\"\"\"\n    m = Model()\n    m.add(\"3001.DAT\", \"red\", at=(0, 0, 0))\n    m.add(\" 3062B \", \"LIGHT_BLUISH_GREY\", at=(4, 0, 0))\n    return m\n";
    std::fs::write(dir.join("model.py"), model).unwrap();
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(["run", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    let (v, code) = envelope(&out);
    assert_eq!(code, 0, "{v}");
    let mpd = std::fs::read_to_string(dir.join("build/model.mpd")).unwrap();
    assert!(mpd.contains(" 3001.dat\n") && mpd.contains(" 3062b.dat\n"), "{mpd}");
    assert!(mpd.contains("\n1 71 "), "LIGHT_BLUISH_GREY is code 71: {mpd}");
    assert!(!mpd.contains('\r'), "LF line ends on every OS");
}
