// SPDX-License-Identifier: AGPL-3.0-or-later
// No console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// The WebDriver server lets anything on localhost drive the app, so it never ships (ADR 0002).
#[cfg(all(feature = "webdriver", not(debug_assertions)))]
compile_error!("the webdriver feature is for debug E2E builds only");

fn main() {
    let builder = tauri::Builder::default();
    #[cfg(feature = "webdriver")]
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());
    builder
        .run(tauri::generate_context!())
        .expect("error while running Emmet");
}
