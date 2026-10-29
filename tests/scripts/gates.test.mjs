// SPDX-License-Identifier: AGPL-3.0-or-later
// The supply-chain and secret gates fail when broken: cargo deny with the repo's deny.toml, gitleaks, and the git hooks.
import assert from "node:assert/strict";
import { randomBytes } from "node:crypto";
import { join } from "node:path";
import { test } from "node:test";
import { ROOT, run, tempDir, tempRepo, writeFiles } from "./helpers.mjs";

const crate = (name, license, deps = "") =>
  `[package]\nname = "${name}"\nversion = "0.1.0"\nedition = "2024"\nlicense = "${license}"\n\n[dependencies]\n${deps}`;

function deny(depLicense) {
  const dir = tempDir();
  writeFiles(dir, {
    // publish = false marks the app as private, like Emmet's own crates, which deny.toml skips.
    "Cargo.toml": crate("app", "AGPL-3.0-or-later", 'dep = { path = "dep" }\n').replace("[dependencies]", "publish = false\n\n[dependencies]"),
    "src/lib.rs": "",
    "dep/Cargo.toml": crate("dep", depLicense),
    "dep/src/lib.rs": "",
  });
  return run("cargo", ["deny", "--manifest-path", join(dir, "Cargo.toml"), "--config", join(ROOT, "deny.toml"), "check", "licenses"]);
}

test("cargo deny accepts a dependency on the allowlist", () => {
  const r = deny("MIT OR Apache-2.0");
  assert.equal(r.code, 0, r.out);
});

test("cargo deny rejects a dependency outside the license allowlist", () => {
  const r = deny("GPL-3.0-only");
  assert.notEqual(r.code, 0, r.out);
  assert.match(r.out, /GPL-3.0-only/);
  assert.match(r.out, /licenses FAILED/);
});

// Built at run time so no secret-shaped string is ever committed.
const fakeToken = () => "ghp_" + randomBytes(27).toString("base64url").replace(/[-_]/g, "a").slice(0, 36);

test("gitleaks fails on a committed token", () => {
  const { dir, git } = tempRepo();
  writeFiles(dir, { "config.txt": `token = "${fakeToken()}"\n` });
  git("add", "-A");
  git("commit", "-q", "-m", "add config");
  const r = run("gitleaks", ["git", "--redact", "--no-banner", dir]);
  assert.equal(r.code, 1, r.out);
  assert.match(r.out, /leaks found: 1/);
});

test("gitleaks passes on a clean history", () => {
  const { dir, git } = tempRepo();
  writeFiles(dir, { "README.md": "nothing secret\n" });
  git("add", "-A");
  git("commit", "-q", "-m", "clean");
  assert.equal(run("gitleaks", ["git", "--redact", "--no-banner", dir]).code, 0);
});

test("the pre-commit hook blocks a staged token", () => {
  const { dir, git } = tempRepo();
  writeFiles(dir, { "config.txt": `token = "${fakeToken()}"\n` });
  git("add", "-A");
  const r = run("bash", [join(ROOT, ".githooks/pre-commit")], { cwd: dir });
  assert.equal(r.code, 1, r.out);
  assert.match(r.out, /leaks found: 1/);
});

test("the pre-commit hook blocks a badly formatted Rust file", () => {
  const { dir, git } = tempRepo();
  writeFiles(dir, { "src/lib.rs": "pub fn f( ) {  }\n" });
  git("add", "-A");
  const r = run("bash", [join(ROOT, ".githooks/pre-commit")], { cwd: dir });
  assert.equal(r.code, 1, r.out);
});

test("the pre-push hook refuses main and allows a branch", () => {
  const hook = join(ROOT, ".githooks/pre-push");
  const sha = "1".repeat(40);
  const toMain = run("bash", [hook, "origin", "url"], { input: `refs/heads/main ${sha} refs/heads/main ${sha}\n` });
  assert.equal(toMain.code, 1);
  assert.match(toMain.out, /refusing to push to main/);
  const toBranch = run("bash", [hook, "origin", "url"], { input: `refs/heads/x ${sha} refs/heads/bmortenson/x ${sha}\n` });
  assert.equal(toBranch.code, 0, toBranch.out);
});

// scripts/unsafe-check.mjs, which `just check` runs: the workspace denies unsafe code rather than forbidding it, so the
// E2E build's one call into WebKit can allow it (D-50), and the check keeps every other Rust file free of it.
function unsafeCheck(files) {
  const { dir, git } = tempRepo();
  writeFiles(dir, { "apps/desktop/src-tauri/src/e2e.rs": "#[allow(unsafe_code)]\nfn f() { unsafe {} }\n", ...files });
  git("add", "apps");
  return run("node", [join(ROOT, "scripts/unsafe-check.mjs"), dir]);
}

test("unsafe code passes in the E2E build's WebKit call, and in this repo", () => {
  const r = unsafeCheck({ "crates/a/src/lib.rs": "// no unsafety here\nfn g() {}\n" });
  assert.equal(r.code, 0, r.out);
  const repo = run("node", [join(ROOT, "scripts/unsafe-check.mjs")]);
  assert.equal(repo.code, 0, repo.out);
});

test("unsafe code anywhere else fails, committed or not", () => {
  for (const [file, text] of [
    ["crates/a/src/lib.rs", "fn g() { unsafe {} }\n"],
    ["crates/a/build.rs", "#![allow(unsafe_code)]\n"],
  ]) {
    const r = unsafeCheck({ [file]: text });
    assert.equal(r.code, 1, r.out);
    assert.match(r.out, new RegExp(`unsafe code outside .*\\n  ${file}`));
  }
});
