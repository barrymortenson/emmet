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

/// The host's commands, from freshly exported bindings.
fn commands() -> Vec<String> {
    let fresh = emmet_it::temp_dir("bindings-acl").join("bindings.ts");
    emmet_desktop::export_bindings(&fresh).expect("export bindings");
    let bindings = std::fs::read_to_string(&fresh).expect("read bindings");
    let commands: Vec<String> = bindings
        .split("__TAURI_INVOKE")
        .skip(2) // the import, then one per command
        .filter_map(|s| {
            s.split_once("(\"")
                .and_then(|(_, rest)| rest.split_once('"'))
                .map(|(name, _)| name.to_string())
        })
        .collect();
    assert!(commands.len() > 20, "found {commands:?}");
    commands
}

/// Each capability file's window and permissions.
fn capabilities() -> Vec<(String, Vec<String>)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).expect("capabilities") {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        let cap: serde_json::Value = serde_json::from_str(&text).unwrap();
        let windows: Vec<&str> = cap["windows"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|w| w.as_str())
            .collect();
        assert_eq!(windows.len(), 1, "one window per capability file: {windows:?}");
        let permissions = cap["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().expect("a permission's name").to_string());
        out.push((windows[0].to_string(), permissions.collect()));
    }
    out
}

/// Every command in the bindings is in build.rs's list and allowed by a capability; Tauri rejects any other call at
/// run time with a bare string (1D's Set up button shipped that way).
#[test]
fn every_command_is_allowed() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let build = std::fs::read_to_string(dir.join("build.rs")).expect("read build.rs");
    let allowed: Vec<String> = capabilities().into_iter().flat_map(|(_, p)| p).collect();
    for c in commands() {
        assert!(
            build.contains(&format!("\"{c}\"")),
            "{c} isn't in build.rs's command list"
        );
        let permission = format!("allow-{}", c.replace('_', "-"));
        assert!(allowed.contains(&permission), "no capability allows {c} ({permission})");
    }
}

/// What the main window may ask of Tauri itself, beyond Emmet's own commands: listen for the host's events, and work
/// its own title row. A new entry here is a decision for the security review (docs/security/review-1.md), not a line
/// to add in passing.
const CORE: [&str; 10] = [
    "core:event:allow-listen",
    "core:event:allow-unlisten",
    "core:window:allow-start-dragging",
    "core:window:allow-close",
    "core:window:allow-toggle-maximize",
    "core:window:allow-internal-toggle-maximize",
    "core:window:allow-minimize",
    "core:window:allow-is-maximized",
    "core:window:allow-is-fullscreen",
    "core:window:allow-set-fullscreen",
];

/// And nothing else is (plan section 15): each permission is one of Emmet's commands by name or one of the few core
/// ones above, the render window has only its five commands, no plugin's permissions (fs, shell, dialog, opener) are
/// granted to any window, and no permission file is left over from a command that's gone.
#[test]
fn the_capabilities_allow_nothing_else() {
    let commands = commands();
    let command_of = |permission: &str| permission.strip_prefix("allow-").map(|c| c.replace('-', "_"));
    for (window, permissions) in capabilities() {
        for p in &permissions {
            if p.contains(':') {
                assert!(
                    window == "main" && CORE.contains(&p.as_str()),
                    "{window} is granted {p}"
                );
            } else {
                let c = command_of(p).unwrap_or_else(|| panic!("{window} is granted {p}"));
                assert!(
                    commands.contains(&c),
                    "{window} is granted {p}, and there is no command {c}"
                );
            }
        }
        if window == "render" {
            assert!(
                permissions.iter().all(|p| p.starts_with("allow-render-")),
                "{permissions:?}"
            );
        }
    }
    let generated = Path::new(env!("CARGO_MANIFEST_DIR")).join("permissions/autogenerated");
    for entry in std::fs::read_dir(generated).expect("permissions") {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        assert!(
            commands.contains(&name),
            "{} is a permission for no command",
            path.display()
        );
    }
}
