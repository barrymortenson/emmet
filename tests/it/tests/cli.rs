// SPDX-License-Identifier: AGPL-3.0-or-later
//! Runs the built `emmet` binary and checks its output.

use std::path::PathBuf;
use std::process::Command;

/// The `emmet` binary next to this test's own build output (target/<profile>/), or `EMMET_BIN`.
fn emmet() -> Command {
    let bin = std::env::var_os("EMMET_BIN").map(PathBuf::from).unwrap_or_else(|| {
        let mut dir = std::env::current_exe().expect("test exe path");
        dir.pop(); // deps/
        dir.pop(); // <profile>/
        dir.join(format!("emmet{}", std::env::consts::EXE_SUFFIX))
    });
    assert!(
        bin.exists(),
        "{} is missing; run `cargo build --workspace` first",
        bin.display()
    );
    Command::new(bin)
}

#[test]
fn version_json_is_the_success_envelope() {
    let out = emmet().args(["--version", "--json"]).output().expect("run emmet");
    assert!(out.status.success(), "exit {:?}", out.status);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("stdout is JSON");
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        v,
        serde_json::json!({ "ok": true, "data": { "version": version }, "warnings": [], "meta": { "emmet": version } })
    );
}

#[test]
fn no_command_is_a_usage_error() {
    let out = emmet().output().expect("run emmet");
    assert_eq!(out.status.code(), Some(2));
}
