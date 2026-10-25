# ADR 0007: The engine's new dependencies

Status: accepted, 2026-10-24 (session 1A)

## Context

Session 1A builds the engine walking skeleton. Plan section 06 names most of what it needs (serde, thiserror,
rusqlite with bundled SQLite and FTS5, glam, zip, sha2, clap, rmcp, schemars, tokio, insta). Two needs had no named
crate: an HTTP client for the library download (`complete.zip`, 146 MB from library.ldraw.org, resumable with a Range
request), and a TOML parser for `emmet.toml` and `data/synonyms.toml`.

## Decision

Barry approved two crates (docs/decisions.md, 2026-10-24):

| Crate | Version | License | For | Features |
|---|---|---|---|---|
| ureq | 3.5.0 | MIT OR Apache-2.0 | `emmet library install`: a blocking HTTPS GET with a Range header | `rustls-no-provider`, `_ring`, `platform-verifier` (rustls with the OS trust store; no gzip, no bundled root list) |
| toml | 1.1.8 | MIT OR Apache-2.0 | reading `emmet.toml` and `data/synonyms.toml` | default |

Options weighed, smallest first:

- **Spawn the system curl** (macOS and Windows 10+ ship one) with `--proto =https -C -`. No crate, but a dependency
  on the OS's curl and parsing its exit codes. Not chosen.
- **ureq** (chosen): small and synchronous, which fits the CLI; Range requests are a header.
- **reqwest**: heavier, but tauri-plugin-updater brings it in 6A anyway. Not chosen for the CLI sidecar.

`toml` was already in `Cargo.lock` as a build-time dependency of tauri-build.

The MCP contract test checks structured output against each tool's JSON Schema with a small validator in
`tests/it/src/schema.rs` (the subset schemars writes) instead of a JSON Schema crate.

## Consequences

- `cargo deny check` passes: every crate in the new tree is on the license allowlist.
- The library pin is a URL plus a sha256 (`crates/emmet-core/src/library.rs`). library.ldraw.org serves the newest
  release at `complete.zip`, so once LDraw publishes the next release, a fresh install of this Emmet fails with
  `LIBRARY_CORRUPT` (checksum mismatch) until Emmet pins the new release. Hosting the pinned zip with Emmet's own
  releases (6A) would remove that window.

## Rerun

```sh
cargo tree -p emmet-core -e normal --depth 1
cargo deny check
```
