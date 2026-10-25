// SPDX-License-Identifier: AGPL-3.0-or-later
//! Setting up Python recovers from a damaged runtime in app data. CI found this: a cache restored part of an earlier
//! run's Python install, and uv couldn't create the venv on top of it.

use emmet_it::{emmet, envelope, temp_dir};

#[test]
fn doctor_fix_recovers_from_a_broken_python_folder() {
    let data = temp_dir("broken-python");
    std::fs::create_dir_all(data.join("python")).unwrap();
    // A file where uv expects its install folder makes `uv venv` fail the way the half-restored cache did.
    std::fs::write(data.join("python/installs"), "not a folder").unwrap();
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(&data)
        .args(["doctor", "--fix"])
        .output()
        .unwrap();
    let (v, _) = envelope(&out);
    let python = v["data"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "python")
        .unwrap()
        .clone();
    assert_eq!(python["ok"], true, "{v}");
}
