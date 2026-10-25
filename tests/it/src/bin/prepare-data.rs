// SPDX-License-Identifier: AGPL-3.0-or-later
//! Prints the shared test data dir (ldraw-mini installed, the parts index built, Python bootstrapped), setting it up
//! first if needed. The E2E suite (tests/e2e/wdio.conf.mjs) runs this so the app and the integration tests share one.

fn main() {
    println!("{}", emmet_it::shared_data().display());
}
