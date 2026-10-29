# ADR 0000: Pinned versions

Status: accepted, 2026-10-24 (session 0A)

## Decision

These are the exact versions 0A installed and tested. The lockfiles (`Cargo.lock`, `pnpm-lock.yaml`,
`python/emmet_dsl/uv.lock`) pin every transitive dependency. Dependabot proposes updates weekly, grouped; a minor
update merges when CI is green, and a major update gets its own PR with a note (plan section 06).

### Toolchain

| Tool | Version | Pinned in |
|------|---------|-----------|
| Rust | 1.99.0 (rustfmt, clippy) | `rust-toolchain.toml` |
| Node | 22.21.1 | `.node-version` |
| pnpm | 11.22.0 | `package.json` `packageManager` |
| uv | 0.12.22 | CI (`setup-uv`); `pyproject.toml` build backend `uv_build >=0.12.22,<0.13` |
| Python | 3.13 (3.13.16 installed, uv-managed only) | `python/emmet_dsl/.python-version`, `python-preference = "only-managed"` |
| just | 1.58.0 | CI (`install-action`) |
| gitleaks | 8.30.1 | CI (download with a pinned sha256) |
| cargo-deny | 0.20.2 | CI (`install-action`) |

### Libraries

| Package | Version |
|---------|---------|
| tauri | 2.12.5 |
| tauri-build | 2.7.4 |
| @tauri-apps/cli | 2.12.4 |
| clap | 4.6.7 |
| serde_json | 1.0.154 |
| react, react-dom | 19.3.1 |
| @types/react, @types/react-dom | 19.3.0 |
| vite | 8.3.5 |
| @vitejs/plugin-react | 6.1.2 |
| typescript | 6.0.3 |
| eslint | 10.13.0 |
| @eslint/js | 10.0.1 |
| typescript-eslint | 8.72.0 |
| globals | 17.13.0 |
| ruff | 0.17.2 |
| three (0B render spike; the viewer, 1B) | 0.186.1 |
| @types/three (dev only; approved 2026-10-25, docs/decisions.md) | 0.186.0 |
| @tauri-apps/api (Tauri's frontend half; 1B) | 2.12.4 |
| tauri-plugin-dialog (1B: open and new project, Rust side only) | 2.8.3 |
| tauri-specta, specta (pinned with `=`, as tauri-specta asks while in rc) | 2.0.0-rc.26 |
| specta-typescript | 0.0.13 |
| notify (the project watcher, 1B) | 8.2.0 |
| interprocess (emmet-bus, 1B) | 2.4.6 |
| pixelmatch (visual baselines, 1B) | 8.0.0 |
| radix-ui (the Radix primitives in one package, D-29 and section 06; 4A) | 1.8.1 |
| tauri-plugin-wdio-webdriver (E2E only, behind the `webdriver` feature; ADR 0002) | 1.5.1 |
| @wdio/tauri-service | 1.5.1 |
| @wdio/cli, @wdio/local-runner, @wdio/mocha-framework, @wdio/spec-reporter | 9.32.0 |
| @wdio/globals | 9.31.3 |
| keyring-core, apple-native-keyring-store, windows-native-keyring-store (the Codex key in the keychain; approved 2026-10-25, ADR 0009) | 1.0.1, 1.0.2, 1.1.0 |
| widestring (Windows only: the bus pipe's ACL; approved 2026-10-25, ADR 0009) | 1.2.1 |

### GitHub Actions (pinned by SHA)

| Action | Version | SHA |
|--------|---------|-----|
| actions/checkout | v7.0.1 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| actions/setup-node | v7.1.0 | `949feb2413d6458794dcd2491c4babbbce0c15c1` |
| pnpm/action-setup | v6.1.0 | `ea17c68df8912ef543352723c149a84f56e3d413` |
| astral-sh/setup-uv | v10.3.2 | `07a047e608db84b3d8c06cd9a7c7dcc9c355c18e` |
| Swatinem/rust-cache | v2.9.3 | `d1527c8f67ef9af357290475d4af4b0fab8f12e0` |
| taiki-e/install-action | v2.87.31 | `1e29709c0ec63babbe8f266397a465f29a825f91` |
| actions/upload-artifact (1B: E2E screenshots, renders, bench) | v7.0.2 | `cf430e030ddbb5b0abf93d22962f4752f3646cd9` |

## Why these

- **Tauri 2.12**, the latest 2.x. Tauri 3 is in alpha; D-02 says Tauri 2.
- **TypeScript 6.0.3**, not 7.0.2: typescript-eslint 8.72 supports TypeScript below 6.1.
- **Node 22.21.1 and pnpm 11.22.0** are the versions active on the development Mac. Node 24 and pnpm 12 are also
  installed there; moving to them is a major update with its own PR.
- **1B's versions** are one step behind the newest where the newest was days old (notify 8.2.1, interprocess
  2.4.7), the same caution pnpm's minimum release age applies to npm packages.
- **WebdriverIO 9, not 10:** `@wdio/tauri-service` 1.5.1 declares `webdriverio ^9.0.0` (ADR 0002). Added in 0B.
- **`globals`** is the one package outside the plan's named list: the eslint config needs it for Node globals in
  scripts and tests. It is data only, with no dependencies.
- **The macOS cross targets** (`x86_64-apple-darwin` for the universal build, D-01) aren't in `rust-toolchain.toml` yet,
  so CI doesn't download them; 6A adds them with the release workflow.
