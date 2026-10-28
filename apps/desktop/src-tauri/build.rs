// SPDX-License-Identifier: AGPL-3.0-or-later
// The app's own commands are listed here, so each window's capability (capabilities/*.json) must allow each one by
// name; a command no capability names can't be called (plan section 15).
fn main() {
    let commands = &[
        "app_info",
        "ui_state",
        "open_dialog",
        "new_project_dialog",
        "open_recent",
        "get_params",
        "set_params",
        "list_colors",
        "data_settings",
        "set_color_names",
        "rebuild",
        "bus_reply",
        "render_job",
        "render_done",
        "render_failed",
        "setup",
        "agent_install",
        "claude_sign_in",
        "agent_cancel",
        "provider_notice",
        "provider_notice_seen",
        "open_terminal",
        "open_plans",
        "agents",
        "chat_start",
        "chat_send",
        "chat_steer",
        "chat_note",
        "chat_stop",
        "chat_respond",
        "chat_answer",
        "chat_state",
        "chat_end",
        "usage",
        "openai_key_saved",
        "openai_key_set",
        "chatgpt_state",
        "chatgpt_sign_in",
        "chatgpt_cancel",
        "chatgpt_sign_out",
        "chatgpt_choose",
        "chatgpt_notice_seen",
        "open_chatgpt_usage",
    ];
    // The tests in tests/ link Tauri too, and on Windows they need the Common Controls v6 manifest that tauri-build
    // embeds only in the app.
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let manifest =
            std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("windows-test-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg-tests=/MANIFESTINPUT:{}", manifest.display());
    }
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(commands)),
    )
    .expect("tauri build");
}
