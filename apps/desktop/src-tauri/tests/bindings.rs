// SPDX-License-Identifier: AGPL-3.0-or-later
//! The TypeScript bindings the UI imports (apps/desktop/src/bindings.ts) match the host's commands and events
//! (plan section 06: CI fails if the bindings are stale). EMMET_UPDATE_BINDINGS=1 rewrites them.

use std::path::Path;

#[test]
fn bindings_are_fresh() {
    let committed = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/bindings.ts");
    let fresh = emmet_it::temp_dir("bindings").join("bindings.ts");
    emmet_desktop::export_bindings(&fresh).expect("export bindings");
    let fresh = std::fs::read_to_string(&fresh).expect("read fresh bindings");
    if std::env::var_os("EMMET_UPDATE_BINDINGS").is_some() {
        std::fs::write(&committed, &fresh).expect("write bindings");
        return;
    }
    let old = std::fs::read_to_string(&committed).unwrap_or_default();
    assert!(
        old.replace("\r\n", "\n") == fresh,
        "apps/desktop/src/bindings.ts is stale; run EMMET_UPDATE_BINDINGS=1 cargo test -p emmet-desktop --test bindings"
    );
}
