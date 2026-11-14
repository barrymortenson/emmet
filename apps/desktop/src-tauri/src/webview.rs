// SPDX-License-Identifier: AGPL-3.0-or-later
//! The WebView2 check (Windows; plan session 5B). Emmet's window is drawn by Microsoft Edge WebView2, which Windows 11
//! carries and keeps current, and which the installer adds to a Windows 10 that lacks it. A machine can still have
//! none, or one frozen at an old version by policy, and the window would come up blank; so the app says what's wrong
//! and how to fix it instead.

/// The oldest WebView2 that draws Emmet's window: the styles use `color-mix()`, which arrived in 111 (March 2023).
/// The installer asks for the same version (tauri.release.conf.json).
pub const MINIMUM: &str = "111.0.0.0";

/// What to tell the user when the installed WebView2 (`found`: its version, or None when there is none) can't draw
/// the window; None when it can.
pub fn problem(found: Option<&str>) -> Option<String> {
    let fix = "Install the Evergreen Runtime from https://developer.microsoft.com/microsoft-edge/webview2 and start \
               Emmet again.";
    match found {
        None => Some(format!(
            "Emmet needs Microsoft Edge WebView2 to draw its window, and this PC doesn't have it. {fix}"
        )),
        Some(v) if older(v, MINIMUM) => Some(format!(
            "Emmet needs Microsoft Edge WebView2 {} or newer to draw its window, and this PC has {v}. {fix}",
            MINIMUM.split('.').next().unwrap_or(MINIMUM)
        )),
        Some(_) => None,
    }
}

/// True when dotted version `a` is older than `b`, comparing number by number. A part that isn't a number counts as
/// 0, so a version Emmet can't read is never taken for a new one.
fn older(a: &str, b: &str) -> bool {
    let parts = |v: &str| -> Vec<u64> { v.split('.').map(|p| p.trim().parse().unwrap_or(0)).collect() };
    let (a, b) = (parts(a), parts(b));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x < y;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_old_or_missing_webview2_gets_a_message_that_says_what_to_install() {
        for ok in ["111.0.0.0", "111.0.1661.41", "142.0.3595.53", "200.1"] {
            assert_eq!(problem(Some(ok)), None, "{ok}");
        }
        for old in ["110.0.1587.69", "99.9.9.9", "9.200.0.0", "", "unknown"] {
            let said = problem(Some(old)).unwrap_or_else(|| panic!("{old} is too old"));
            assert!(
                said.contains("WebView2 111 or newer") && said.contains("developer.microsoft.com"),
                "{said}"
            );
        }
        assert!(problem(None).unwrap().contains("doesn't have it"));
        // The installer asks for the same version.
        let release = include_str!("../tauri.release.conf.json");
        assert!(
            release.contains(&format!("\"minimumWebview2Version\": \"{MINIMUM}\"")),
            "{release}"
        );
    }
}
