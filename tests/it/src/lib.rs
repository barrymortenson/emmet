// SPDX-License-Identifier: AGPL-3.0-or-later
//! The harness the integration tests in `tests/` share. They run the built binaries, so `cargo build --workspace`
//! comes first (`just test` does it).
//!
//! Nothing here touches the real app data or home: every `emmet` gets `--data-dir`, and HOME (USERPROFILE and
//! APPDATA on Windows) points at the test home. CI's runner guard sets `EMMET_DATA_DIR` and `EMMET_TEST_HOME` inside
//! the runner's temp dir; locally they default to `target/it-data` and `target/it-home`.

pub mod agents;
pub mod chatgpt;
pub mod corpus;
pub mod schema;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use serde_json::Value;

pub fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

pub fn fixture(rel: &str) -> PathBuf {
    repo().join("tests/fixtures").join(rel)
}

/// target/<profile>/, from a test (target/<profile>/deps/) or a binary (target/<profile>/).
pub fn target_dir() -> PathBuf {
    let mut dir = std::env::current_exe().expect("test exe path");
    dir.pop();
    if dir.ends_with("deps") {
        dir.pop();
    }
    dir
}

/// The `emmet` binary next to this test's own build output (target/<profile>/), or `EMMET_BIN`.
pub fn emmet_bin() -> PathBuf {
    let bin = std::env::var_os("EMMET_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| target_dir().join(format!("emmet{}", std::env::consts::EXE_SUFFIX)));
    assert!(
        bin.exists(),
        "{} is missing; run `cargo build --workspace` first",
        bin.display()
    );
    bin
}

fn env_dir(var: &str, default: &str) -> PathBuf {
    let dir = std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_else(|| target_dir().parent().expect("target/").join(default));
    std::fs::create_dir_all(&dir).expect("create test dir");
    dir
}

/// The home directory every spawned process sees.
pub fn test_home() -> PathBuf {
    env_dir("EMMET_TEST_HOME", "it-home")
}

/// `emmet` with the test home, no inherited data dir, and no agent or API-key variables.
pub fn emmet() -> Command {
    let mut cmd = Command::new(emmet_bin());
    isolate(&mut cmd);
    cmd
}

/// Points HOME at the test home and drops variables a test must not inherit.
pub fn isolate(cmd: &mut Command) {
    let home = test_home();
    cmd.env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("APPDATA", home.join("AppData/Roaming"));
    cmd.env("LOCALAPPDATA", home.join("AppData/Local"));
    for (key, _) in std::env::vars_os() {
        let k = key.to_string_lossy().to_uppercase();
        if k.starts_with("EMMET_") || k.contains("API_KEY") || k.contains("TOKEN") || k.starts_with("UV_") {
            cmd.env_remove(&key);
        }
    }
}

/// A fresh, empty directory for one test under `target/it-tmp/`.
pub fn temp_dir(name: &str) -> PathBuf {
    static N: AtomicU32 = AtomicU32::new(0);
    let dir = target_dir().parent().expect("target/").join("it-tmp").join(format!(
        "{name}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

/// A zip of ldraw-mini laid out like the official `complete.zip` (everything under `ldraw/`), plus extra entries.
pub fn library_zip(extra: &[(&str, &[u8])]) -> Vec<u8> {
    use std::io::Write;
    let mut buf = std::io::Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut buf);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let root = fixture("ldraw-mini");
    let mut files = Vec::new();
    collect(&root, &root, &mut files);
    files.sort();
    for rel in files {
        zip.start_file(format!("ldraw/{rel}"), opts).expect("zip entry");
        zip.write_all(&std::fs::read(root.join(&rel)).expect("read fixture"))
            .expect("zip write");
    }
    for (name, bytes) in extra {
        zip.start_file(*name, opts).expect("zip entry");
        zip.write_all(bytes).expect("zip write");
    }
    zip.finish().expect("zip finish");
    buf.into_inner()
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("read fixture dir") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            collect(root, &path, out);
        } else {
            out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
}

pub fn sha256(bytes: &[u8]) -> String {
    use sha2::Digest;
    sha2::Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// The data dir most tests share, with ldraw-mini installed from a local zip and the parts index built, once per
/// test binary (cargo runs the binaries one after another).
pub fn shared_library() -> &'static Path {
    static DATA: OnceLock<PathBuf> = OnceLock::new();
    DATA.get_or_init(|| {
        let data = env_dir("EMMET_DATA_DIR", "it-data");
        let zip = library_zip(&[]);
        let installed = std::fs::read_to_string(data.join("library.json")).unwrap_or_default();
        if !installed.contains(&sha256(&zip)) || !data.join("parts.sqlite").exists() {
            let zip_path = data.join("ldraw-mini.zip");
            std::fs::write(&zip_path, &zip).expect("write library zip");
            let out = emmet()
                .arg("--data-dir")
                .arg(&data)
                .args([
                    "library",
                    "install",
                    "--release",
                    "ldraw-mini",
                    "--sha256",
                    &sha256(&zip),
                    "--zip",
                ])
                .arg(&zip_path)
                .output()
                .expect("run emmet library install");
            assert!(out.status.success(), "library install failed: {}", text(&out));
        }
        data
    })
}

/// [`shared_library`] with Python bootstrapped too (`emmet doctor --fix`).
pub fn shared_data() -> &'static Path {
    static DATA: OnceLock<PathBuf> = OnceLock::new();
    DATA.get_or_init(|| {
        let data = shared_library().to_path_buf();
        let out = emmet()
            .arg("--data-dir")
            .arg(&data)
            .args(["doctor", "--fix"])
            .output()
            .expect("run emmet doctor");
        assert!(out.status.success(), "doctor --fix failed: {}", text(&out));
        data
    })
}

pub fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Runs `emmet --json --data-dir <shared> <args>` and returns the envelope and the exit code.
pub fn emmet_json(args: &[&str]) -> (Value, i32) {
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(args)
        .output()
        .expect("run emmet");
    envelope(&out)
}

pub fn envelope(out: &Output) -> (Value, i32) {
    let v: Value =
        serde_json::from_slice(&out.stdout).unwrap_or_else(|e| panic!("stdout isn't JSON ({e}): {}", text(out)));
    (v, out.status.code().unwrap_or(-1))
}

/// Copies a project fixture (`tests/fixtures/projects/<name>`) into a fresh temp dir and trusts it.
pub fn project(name: &str) -> PathBuf {
    let dir = temp_dir(name).join(name);
    copy_tree(&fixture("projects").join(name), &dir);
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["trust", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "trust failed: {}", text(&out));
    dir
}

pub fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create dir");
    for entry in std::fs::read_dir(from).expect("read dir") {
        let path = entry.expect("entry").path();
        let dest = to.join(path.file_name().unwrap());
        if path.is_dir() {
            copy_tree(&path, &dest);
        } else {
            std::fs::copy(&path, &dest).expect("copy file");
        }
    }
}

/// Replaces values that change from run to run or machine to machine, for snapshots: each path in `paths` (in
/// either separator style) by its label, and the envelope's timing and version.
pub fn redact(mut v: Value, paths: &[(&Path, &str)]) -> Value {
    fn walk(v: &mut Value, paths: &[(String, &str)]) {
        match v {
            Value::String(s) => {
                for (p, label) in paths {
                    *s = s.replace(p.as_str(), label).replace(&p.replace('\\', "/"), label);
                }
                *s = s.replace('\\', "/");
            }
            Value::Array(a) => a.iter_mut().for_each(|x| walk(x, paths)),
            Value::Object(o) => o.values_mut().for_each(|x| walk(x, paths)),
            _ => {}
        }
    }
    // Longest first, so a path inside another is replaced before its parent; canonical forms too (/private/var).
    let mut all: Vec<(String, &str)> = Vec::new();
    for (p, label) in paths {
        all.push((p.display().to_string(), label));
        if let Ok(c) = p.canonicalize() {
            all.push((c.display().to_string(), label));
        }
    }
    all.sort_by_key(|(p, _)| std::cmp::Reverse(p.len()));
    walk(&mut v, &all);
    if let Some(meta) = v.get_mut("meta").and_then(Value::as_object_mut) {
        for key in ["ms", "emmet", "inputs_hash"] {
            if meta.contains_key(key) {
                meta.insert(key.into(), Value::String(format!("[{key}]")));
            }
        }
    }
    v
}

/// Runs `emmet --json --data-dir <shared> <args> --project <dir>`.
pub fn emmet_in(project: &Path, args: &[&str]) -> (Value, i32) {
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(args)
        .arg("--project")
        .arg(project)
        .output()
        .expect("run emmet");
    envelope(&out)
}

/// Checks a render: a PNG of `size` × `size` that isn't one flat color.
pub fn check_png(png: &[u8], size: u32) {
    assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "not a PNG");
    let width = u32::from_be_bytes(png[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(png[20..24].try_into().unwrap());
    assert_eq!((width, height), (size, size), "the PNG's size");
    // One flat color compresses to a few hundred bytes at any of these sizes; a model with edges and shading to
    // more than a kilobyte (a 128 px front view of scene-colors is 1.7 KB). The E2E specs decode and compare pixels.
    assert!(png.len() > 1_000, "the PNG looks blank ({} bytes)", png.len());
}
