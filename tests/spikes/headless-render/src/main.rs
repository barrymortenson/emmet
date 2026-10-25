// SPDX-License-Identifier: AGPL-3.0-or-later
//! ADR 0005 spike: does three.js render in a hidden Tauri window, and how fast? Renders a cube, or an LDraw model
//! through LDrawLoader, then writes a PNG and exits.
//!
//!   emmet-spike-render --out <png> [--mode hidden|offscreen] [--size 512] [--model <file> --library <ldraw dir>]
//!
//! Prints one `mark <name> <ms since start>` line per step. `offscreen` is the fallback: a visible window placed off
//! the screen, for webviews that don't render while hidden.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tauri::http::{Response, StatusCode};
use tauri::ipc::{InvokeBody, Request};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

static START: OnceLock<Instant> = OnceLock::new();

struct Args {
    out: PathBuf,
    hidden: bool,
    size: u32,
    model: Option<PathBuf>,
    library: Option<PathBuf>,
}

fn args() -> Args {
    let mut a = Args {
        out: "render.png".into(),
        hidden: true,
        size: 512,
        model: None,
        library: None,
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let value = it.next().unwrap_or_else(|| panic!("{flag} needs a value"));
        match flag.as_str() {
            "--out" => a.out = value.into(),
            "--mode" => a.hidden = value != "offscreen",
            "--size" => a.size = value.parse().expect("--size takes a number"),
            "--model" => a.model = Some(value.into()),
            "--library" => a.library = Some(value.into()),
            _ => panic!("unknown flag {flag}"),
        }
    }
    a
}

fn elapsed() -> u128 {
    START.get().unwrap().elapsed().as_millis()
}

#[tauri::command]
fn mark(name: String) {
    println!("mark {name} {}", elapsed());
}

#[tauri::command]
fn fail(app: AppHandle, message: String) {
    eprintln!("render failed: {message}");
    app.exit(3);
}

#[tauri::command]
fn save_png(app: AppHandle, request: Request<'_>) -> Result<(), String> {
    let InvokeBody::Raw(bytes) = request.body() else {
        return Err("expected raw PNG bytes".into());
    };
    let out = &app.state::<Args>().out;
    std::fs::write(out, bytes).map_err(|e| e.to_string())?;
    println!("mark saved {} ({} bytes to {})", elapsed(), bytes.len(), out.display());
    app.exit(0);
    Ok(())
}

/// A file under `root` for a request path, matched case-insensitively (LDraw references mix cases), or None if the
/// path escapes `root`.
fn resolve(root: &Path, rel: &str) -> Option<PathBuf> {
    let mut path = root.to_path_buf();
    for part in rel.split('/').filter(|p| !p.is_empty()) {
        if part == ".." || part.contains('\\') {
            return None;
        }
        let exact = path.join(part);
        path = if exact.exists() {
            exact
        } else {
            std::fs::read_dir(&path).ok()?.flatten().map(|e| e.path()).find(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().eq_ignore_ascii_case(part))
            })?
        };
    }
    let (path, root) = (path.canonicalize().ok()?, root.canonicalize().ok()?);
    (path.starts_with(&root) && path.is_file()).then_some(path)
}

fn main() {
    START.set(Instant::now()).unwrap();
    let args = args();
    let (size, hidden) = (args.size, args.hidden);
    let base = if cfg!(windows) {
        "http://ldraw.localhost/"
    } else {
        "ldraw://localhost/"
    };
    let model_name = args
        .model
        .as_ref()
        .map(|m| m.file_name().unwrap().to_string_lossy().into_owned());
    let config = format!(
        "window.__SPIKE__ = {{ size: {size}, base: {base:?}, model: {} }};",
        model_name.as_ref().map_or("null".into(), |n| format!("{n:?}"))
    );
    tauri::Builder::default()
        .register_uri_scheme_protocol("ldraw", |ctx, request| {
            let args = ctx.app_handle().state::<Args>();
            let path = percent_decode(request.uri().path());
            let file = match (path.strip_prefix("/model/"), &args.model, &args.library) {
                (Some(_), Some(model), _) => {
                    resolve(model.parent().unwrap(), model.file_name().unwrap().to_str().unwrap())
                }
                (None, _, Some(library)) => resolve(library, &path),
                _ => None,
            };
            // The page's origin is the app's, so these fetches are cross-origin and need CORS.
            let response = Response::builder().header("access-control-allow-origin", "*");
            match file.and_then(|f| std::fs::read(f).ok()) {
                Some(body) => response.header("content-type", "text/plain").body(body).unwrap(),
                None => response.status(StatusCode::NOT_FOUND).body(Vec::new()).unwrap(),
            }
        })
        .manage(args)
        .invoke_handler(tauri::generate_handler![mark, fail, save_png])
        .setup(move |app| {
            let mut window = WebviewWindowBuilder::new(app, "render", WebviewUrl::App("index.html".into()))
                .title("render spike")
                .inner_size(f64::from(size), f64::from(size))
                .initialization_script(&config)
                .visible(!hidden);
            if !hidden {
                window = window.position(-10_000.0, -10_000.0);
            }
            window.build()?;
            println!("mark window {}", elapsed());
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_secs(60));
                eprintln!("render timed out after 60 s");
                handle.exit(2);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running the render spike");
}

/// Decodes %XX escapes in a URL path (model names can contain spaces).
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(v) = s.get(i + 1..i + 3).and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
