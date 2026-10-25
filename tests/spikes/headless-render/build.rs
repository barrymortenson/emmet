// SPDX-License-Identifier: AGPL-3.0-or-later
// Copies three.js and LDrawLoader from node_modules into web/vendor/ (gitignored) before Tauri embeds web/.

use std::path::Path;

fn main() {
    let three = Path::new("node_modules/three");
    assert!(
        three.exists(),
        "node_modules/three is missing: run `just deps` (pnpm install) first"
    );
    let vendor = Path::new("web/vendor");
    std::fs::create_dir_all(vendor).unwrap();
    for (from, to) in [
        ("build/three.module.js", "three.module.js"),
        ("build/three.core.js", "three.core.js"),
        ("examples/jsm/loaders/LDrawLoader.js", "LDrawLoader.js"),
        (
            "examples/jsm/materials/LDrawConditionalLineMaterial.js",
            "LDrawConditionalLineMaterial.js",
        ),
    ] {
        std::fs::copy(three.join(from), vendor.join(to)).unwrap();
        println!("cargo:rerun-if-changed=node_modules/three/{from}");
    }
    tauri_build::build();
}
