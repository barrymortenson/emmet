// SPDX-License-Identifier: AGPL-3.0-or-later
//! Keeps the E2E build's webview drawing while it's covered on macOS (D-50; only with the `webdriver` feature, which
//! main.rs refuses in a release build). CI's Mac E2E runs on the Mac Barry works on, so his windows cover the app's.
//! lib.rs does the rest of D-50: no focus taken at launch, and no Dock icon, Cmd-Tab entry, or menu bar.

use objc2::runtime::AnyObject;
use objc2::{msg_send, sel};

/// Keeps the window's webview drawing while it's covered. WebKit otherwise marks a covered page hidden and stops its
/// frames, and every E2E wait on the next frame hangs until the window shows. WKWebView's private
/// `_setWindowOcclusionDetectionEnabled:` turns that off (a covered page kept 60 fps in a test, 2026-10-29). If a
/// WebKit ever drops it, the app warns on stderr (which the E2E run doesn't show), and wdio.conf.mjs's visibility check
/// fails within seconds whenever the window is covered, with a message that names this file.
#[allow(unsafe_code)]
pub fn keep_drawing(window: &tauri::WebviewWindow) -> tauri::Result<()> {
    window.with_webview(|webview| {
        // SAFETY: `inner()` is the window's live WKWebView, and `with_webview` runs this on the main thread, where
        // AppKit and WebKit calls belong. Both messages take and return plain values, and the setter is sent only
        // when the webview answers to it.
        unsafe {
            let wk = &*webview.inner().cast::<AnyObject>();
            let setter = sel!(_setWindowOcclusionDetectionEnabled:);
            let known: bool = msg_send![wk, respondsToSelector: setter];
            if known {
                let _: () = msg_send![wk, _setWindowOcclusionDetectionEnabled: false];
            } else {
                eprintln!("e2e: WebKit has no _setWindowOcclusionDetectionEnabled:, so a covered window stops drawing");
            }
        }
    })
}
