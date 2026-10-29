// SPDX-License-Identifier: AGPL-3.0-or-later
//! The E2E build's window on macOS (D-50; only with the `webdriver` feature, which debug builds alone may have). CI's
//! Mac E2E runs on the Mac Barry works on, so the app stays out of his way: it doesn't take focus at launch, has no Dock
//! icon or Cmd-Tab entry, and keeps drawing while his windows cover it.

use objc2::runtime::AnyObject;
use objc2::{msg_send, sel};

/// Keeps the window's webview drawing while it's covered. WebKit otherwise marks a covered page hidden and stops its
/// frames, and every E2E wait on the next frame hangs until the window shows. WKWebView's private
/// `_setWindowOcclusionDetectionEnabled:` turns that off (a covered page kept 60 fps in a test, 2026-10-29). If a
/// WebKit ever drops it, the app says so and wdio.conf.mjs reports a covered window as before.
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
