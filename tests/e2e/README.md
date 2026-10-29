# End-to-end tests

WebdriverIO specs with `@wdio/tauri-service` drive the real app (plan section 16, ADR 0002). `just e2e` builds the
debug app with the embedded WebDriver server (the `webdriver` feature of `emmet-desktop`, which never compiles into a
release build) and runs every spec in `specs/`; `just e2e hello` runs one.

The service starts the app and talks to the server inside it, so no browser driver is installed. `wdio.conf.mjs`
selects the app's one window up front, which turns off the service's per-command focus check (that check needs the
service's companion plugin, which Emmet doesn't use).

On macOS the E2E build stays out of the way (D-50, `apps/desktop/src-tauri/src/e2e.rs`): it doesn't take focus at
launch, has no Dock icon or Cmd-Tab entry, and keeps drawing while other windows cover it. Don't minimize it: WebKit
still stops drawing a minimized window, and the specs then fail with "the app's window is minimized or hidden".
