// SPDX-License-Identifier: AGPL-3.0-or-later
//! Each library crate links and reports the workspace version.

#[test]
fn every_crate_reports_the_workspace_version() {
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(emmet_core::version(), version);
    assert_eq!(emmet_agent::version(), version);
    assert_eq!(emmet_bus::version(), version);
}
