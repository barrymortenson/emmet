// SPDX-License-Identifier: AGPL-3.0-or-later
//! The GUI-subsystem build, like the Emmet app's own exe on Windows (D-08).
#![windows_subsystem = "windows"]

fn main() {
    emmet_spike_binaries::serve("gui");
}
