// SPDX-License-Identifier: AGPL-3.0-or-later
//! The TypeScript bindings the UI imports (apps/desktop/src/bindings.ts) match the host's commands and events
//! (plan section 06: CI fails if the bindings are stale). EMMET_UPDATE_BINDINGS=1 rewrites them.

use std::path::Path;

#[test]
fn bindings_are_fresh() {
    let committed = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");
    let fresh = emmet_it::temp_dir("bindings").join("bindings.ts");
    emmet_desktop::export_bindings(&fresh).expect("export bindings");
    let fresh = std::fs::read_to_string(&fresh).expect("read fresh bindings");
    if std::env::var_os("EMMET_UPDATE_BINDINGS").is_some() {
        std::fs::write(&committed, &fresh).expect("write bindings");
        return;
    }
    let old = std::fs::read_to_string(&committed).unwrap_or_default();
    assert!(
        old.replace("\r\n", "\n") == fresh,
        "apps/desktop/src/bindings.ts is stale; run EMMET_UPDATE_BINDINGS=1 cargo test -p emmet-desktop --test bindings"
    );
}

/// Every command in the bindings is in build.rs's list and allowed by a capability; Tauri rejects any other call at
/// run time with a bare string (1D's Set up button shipped that way).
#[test]
fn every_command_is_allowed() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fresh = emmet_it::temp_dir("bindings-acl").join("bindings.ts");
    emmet_desktop::export_bindings(&fresh).expect("export bindings");
    let bindings = std::fs::read_to_string(&fresh).expect("read bindings");
    let commands: Vec<&str> = bindings
        .split("__TAURI_INVOKE")
        .skip(2) // the import, then one per command
        .filter_map(|s| {
            s.split_once("(\"")
                .and_then(|(_, rest)| rest.split_once('"'))
                .map(|(name, _)| name)
        })
        .collect();
    assert!(commands.len() > 20, "found {commands:?}");
    let build = std::fs::read_to_string(dir.join("build.rs")).expect("read build.rs");
    let mut allowed = Vec::new();
    for entry in std::fs::read_dir(dir.join("capabilities")).expect("capabilities") {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        let cap: serde_json::Value = serde_json::from_str(&text).unwrap();
        for p in cap["permissions"].as_array().unwrap() {
            allowed.extend(p.as_str().map(String::from));
        }
    }
    for c in commands {
        assert!(
            build.contains(&format!("\"{c}\"")),
            "{c} isn't in build.rs's command list"
        );
        let permission = format!("allow-{}", c.replace('_', "-"));
        assert!(allowed.contains(&permission), "no capability allows {c} ({permission})");
    }
}
