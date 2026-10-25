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
        "rebuild",
        "bus_reply",
        "render_job",
        "render_done",
        "render_failed",
    ];
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(commands)),
    )
    .expect("tauri build");
}
