# Connector data

Where parts connect, for the validator (plan section 11, docs/spec/validator.md). Three sources, in order:

1. Emmet's primitive table, in `crates/emmet-core/src/check/connectors.rs`: top studs and underside tubes the LDCad
   data has no entry for.
2. `ldcad-shadow.ldr`: the LDCad Shadow Library's snap data, converted to Emmet's connector table by
   `scripts/make-shadow-data` from a pinned commit. Each shadow file's `!LDCAD SNAP_*` lines, unchanged, under a
   `0 FILE` line naming the library file they extend.
3. The overrides, `*.toml`: parts the other sources get wrong or miss. Each `[[override]]` names the part (or subpart),
   why, the LDCad snap lines in its frame, whether they `replace` everything else, and a test model in
   `tests/fixtures/validator/connections/` that validates with no errors only with it
   (`tests/it/tests/validator_corpus.rs`, `each_override_holds_in_its_test_model`). A new override file goes in
   `OVERRIDES` in `connectors.rs` too.

## License and credit

`ldcad-shadow.ldr` is CC BY-SA 4.0, like its source: the LDCad Shadow Library by Roland Melkert and its
contributors, https://github.com/RolandMelkert/LDCadShadowLibrary, licensed under CC BY-SA 4.0 (`LICENSE.md` here is
its legal code). Changed: converted to Emmet's connector table (its comments, history, and other meta lines left out).
Emmet compiles the file's bytes in unchanged, as data (Barry, 2026-10-27, docs/decisions.md and ADR 0003); NOTICE and
the app's About (from 4A) credit it. The overrides (`*.toml`) are Emmet's own, AGPL-3.0-or-later with the rest of the repo.
