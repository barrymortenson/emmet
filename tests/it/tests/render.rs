// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet render` with no app open (plan sections 09 and 13, ADR 0005): it starts the app's render mode, a hidden
//! window that renders through the viewer and exits, and writes the PNG. 1B Done when 3; the app-open path is
//! tests/e2e/specs/render.e2e.mjs.

use emmet_it::{check_png, emmet_in, project};

#[test]
fn render_without_the_app_uses_render_mode() {
    let dir = project("scene-colors");
    let (env, code) = emmet_in(
        &dir,
        &["render", "--view", "iso", "--size", "256", "--highlight", "p1,p2"],
    );
    assert_eq!(code, 0, "{env}");
    let data = &env["data"];
    assert_eq!(data["via"], "render-mode");
    assert_eq!(data["size"], 256);
    assert_eq!(data["highlighted"], serde_json::json!(["p1", "p2"]));
    assert_eq!(data["camera"]["view"], "iso");
    let path = dir.join("build").join("renders").join("iso.png");
    assert_eq!(data["path"], path.display().to_string());
    check_png(&std::fs::read(&path).unwrap(), 256);
    // Render mode cleans up after itself.
    let left: Vec<_> = std::fs::read_dir(dir.join("build/renders"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(left.len(), 1, "{left:?}");
}

#[test]
fn render_runs_a_model_that_is_out_of_date_first() {
    let dir = project("scene-grid");
    std::fs::write(dir.join("params.json"), r#"{"count": 12}"#).unwrap();
    let out = dir.join("out.png");
    let (env, code) = emmet_in(&dir, &["render", "--size", "64", "--out", out.to_str().unwrap()]);
    assert_eq!(code, 0, "{env}");
    let placements: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("build/placements.json")).unwrap()).unwrap();
    assert_eq!(placements["parts"].as_array().unwrap().len(), 12);
    check_png(&std::fs::read(&out).unwrap(), 64);
}

#[test]
fn a_model_that_fails_is_reported_not_rendered() {
    let dir = project("name-error");
    let (env, code) = emmet_in(&dir, &["render"]);
    assert_eq!(code, 3, "{env}");
    assert_eq!(env["error"]["code"], "MODEL_EXEC_FAILED");
}

/// A bus.json left by an app that crashed names a socket nobody answers: render mode takes over, and an error says
/// why the open app wasn't used.
#[test]
fn a_stale_bus_file_falls_back_to_render_mode() {
    let dir = project("scene-colors");
    let data = emmet_it::shared_data();
    let info = data.join("bus.json");
    let left = std::fs::read(&info).ok();
    std::fs::write(
        &info,
        r#"{"v":1,"name":"emmet-0000000000000000.sock","token":"00","pid":1}"#,
    )
    .unwrap();
    let (env, code) = emmet_in(&dir, &["render", "--size", "64"]);
    let (env2, _) = {
        let out = emmet_it::emmet()
            .env("EMMET_APP", dir.join("no-such-app"))
            .arg("--json")
            .arg("--data-dir")
            .arg(data)
            .args(["render", "--size", "64", "--project"])
            .arg(&dir)
            .output()
            .unwrap();
        emmet_it::envelope(&out)
    };
    match left {
        Some(bytes) => std::fs::write(&info, bytes).unwrap(),
        None => std::fs::remove_file(&info).unwrap(),
    }
    assert_eq!(code, 0, "{env}");
    assert_eq!(env["data"]["via"], "render-mode");
    assert_eq!(env2["error"]["code"], "RENDER_UNAVAILABLE");
    let message = env2["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("The open app wasn't used: Emmet isn't running"),
        "{message}"
    );
}
