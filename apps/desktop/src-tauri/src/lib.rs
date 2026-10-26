// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Emmet desktop host (plan section 04): the window, the typed commands, the project watcher, the run queue that
//! calls the emmet sidecar, the bus server, the `emmet-lib://` protocol, and render mode.
//!
//! `Emmet [project] [--data-dir dir]` opens the app, with `project` open if given.
//! `Emmet --render <project> --view iso --size 1024 --out file.png [--highlight a,b] [--data-dir dir]` renders the
//! project's last build in a hidden window, writes the PNG (and the camera next to it as .json), and exits (ADR 0005).

pub mod app;
pub mod chat;
pub mod library;
pub mod paths;
pub mod recent;
pub mod runs;
pub mod setup;
pub mod watcher;

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use emmet_core::data::DataDir;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Render mode gives up after this long, so a stuck webview can't hang `emmet render`.
const RENDER_TIMEOUT: Duration = Duration::from_secs(100);

#[derive(Debug, Clone, PartialEq)]
pub struct RenderArgs {
    pub project: PathBuf,
    pub view: String,
    pub size: u32,
    pub out: PathBuf,
    pub highlight: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub data_dir: Option<PathBuf>,
    pub open: Option<PathBuf>,
    pub render: Option<RenderArgs>,
}

/// Parses the app's arguments (without the program name).
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, String> {
    let mut it = args.into_iter();
    let (mut data_dir, mut open, mut render) = (None, None, None::<PathBuf>);
    let (mut view, mut size, mut out, mut highlight) = ("iso".to_string(), 1024u32, None, Vec::new());
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().ok_or_else(|| format!("{name} needs a value"));
        match a.as_str() {
            "--data-dir" => data_dir = Some(PathBuf::from(value("--data-dir")?)),
            "--render" => render = Some(PathBuf::from(value("--render")?)),
            "--view" => view = value("--view")?,
            "--size" => {
                size = value("--size")?
                    .parse()
                    .map_err(|_| "--size needs a number of pixels")?
            }
            "--out" => out = Some(PathBuf::from(value("--out")?)),
            "--highlight" => {
                highlight = value("--highlight")?
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(String::from)
                    .collect()
            }
            // macOS adds -psn_… when Finder launches an app.
            s if s.starts_with("-psn_") => {}
            s if s.starts_with('-') => return Err(format!("unknown option {s}")),
            s => open = Some(PathBuf::from(s)),
        }
    }
    if !["iso", "front", "back", "left", "right", "top"].contains(&view.as_str()) {
        return Err(format!(
            "there's no view '{view}': use iso, front, back, left, right, or top"
        ));
    }
    let render = match render {
        Some(project) => Some(RenderArgs {
            project,
            view,
            size: size.clamp(16, 4096),
            out: out.ok_or("--render needs --out <file.png>")?,
            highlight,
        }),
        None => None,
    };
    Ok(Args { data_dir, open, render })
}

/// The tauri-specta builder: every command and event the webview can use.
pub fn specta() -> tauri_specta::Builder<tauri::Wry> {
    use app::*;
    tauri_specta::Builder::<tauri::Wry>::new()
        .commands(tauri_specta::collect_commands![
            app_info,
            ui_state,
            open_dialog,
            new_project_dialog,
            open_recent,
            get_params,
            set_params,
            list_colors,
            rebuild,
            bus_reply,
            render_job,
            render_done,
            render_failed,
            crate::setup::setup,
            crate::chat::agents,
            crate::chat::chat_start,
            crate::chat::chat_send,
            crate::chat::chat_steer,
            crate::chat::chat_note,
            crate::chat::chat_stop,
            crate::chat::chat_respond,
            crate::chat::chat_answer,
            crate::chat::chat_state,
            crate::chat::chat_end,
            crate::chat::usage,
            crate::chat::openai_key_saved,
            crate::chat::openai_key_set,
        ])
        .events(tauri_specta::collect_events![
            ProjectOpened,
            RunStarted,
            RunFinished,
            BusRender,
            BusHighlight,
            crate::chat::ChatEvent
        ])
        .dangerously_cast_bigints_to_number()
        .disable_serde_phases()
}

/// Writes the TypeScript bindings (apps/desktop/src/bindings.ts). A test fails when the committed file is stale.
pub fn export_bindings(path: &std::path::Path) -> Result<(), String> {
    let ts = specta_typescript::Typescript::default().header(
        "// SPDX-License-Identifier: AGPL-3.0-or-later\n// Generated by tauri-specta from apps/desktop/src-tauri; don't edit. \
         Regenerate: EMMET_UPDATE_BINDINGS=1 cargo test -p emmet-desktop --test bindings\n",
    );
    specta().export(ts, path).map_err(|e| e.to_string())
}

/// The emmet sidecar: `EMMET_BIN`, else next to this executable.
fn sidecar() -> PathBuf {
    if let Some(p) = std::env::var_os("EMMET_BIN") {
        return PathBuf::from(p);
    }
    let exe = std::env::current_exe().unwrap_or_default();
    exe.with_file_name(format!("emmet{}", std::env::consts::EXE_SUFFIX))
}

static RENDER_OUT: OnceLock<PathBuf> = OnceLock::new();

/// Ends render mode: writes the PNG and camera, or reports the failure, and exits.
fn finish_render(result: Result<(String, serde_json::Value), String>) {
    let Some(out) = RENDER_OUT.get() else { return };
    let code = match result.and_then(|(png, camera)| {
        let bytes = emmet_core::base64::decode(&png).ok_or("the window sent a PNG that isn't base64")?;
        emmet_core::fsx::write_atomic(out, &bytes).map_err(|e| e.message)?;
        let camera = serde_json::to_string_pretty(&camera).unwrap_or_default();
        emmet_core::fsx::write_atomic(&out.with_extension("json"), camera.as_bytes()).map_err(|e| e.message)?;
        Ok(())
    }) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("render failed: {e}");
            1
        }
    };
    std::process::exit(code);
}

pub fn run() {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Emmet: {e}");
            std::process::exit(2);
        }
    };
    let data = match DataDir::resolve(args.data_dir.as_deref()) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Emmet: {}", e.message);
            std::process::exit(2);
        }
    };
    let render = match &args.render {
        Some(r) => {
            let _ = RENDER_OUT.set(std::path::absolute(&r.out).unwrap_or_else(|_| r.out.clone()));
            match app::render_job_for(&r.project, &r.view, r.size, r.highlight.clone()) {
                Ok(job) => Some(job),
                Err(e) => {
                    eprintln!("render failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        None => None,
    };
    let rendering = render.is_some();
    let specta = specta();
    let library = Arc::new(library::Library::new(data.ldraw()));
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol("emmet-lib", move |_ctx, request, responder| {
            let library = library.clone();
            std::thread::spawn(move || {
                let uri = request.uri();
                let reply = library.respond(uri.path(), uri.query());
                let response = tauri::http::Response::builder()
                    .status(reply.status)
                    .header("Content-Type", reply.mime)
                    // The page's origin is the app's, so these fetches are cross-origin (ADR 0005).
                    .header("Access-Control-Allow-Origin", "*")
                    .body(reply.body)
                    .expect("response");
                responder.respond(response);
            });
        })
        .manage(app::AppState::new(data, sidecar(), render))
        .invoke_handler(specta.invoke_handler());
    #[cfg(feature = "webdriver")]
    let builder = builder.plugin(tauri_plugin_wdio_webdriver::init());
    builder
        .setup(move |tauri_app| {
            specta.mount_events(tauri_app);
            let handle = tauri_app.handle().clone();
            if rendering {
                let job = tauri_app.state::<app::AppState>().render.clone().expect("render job");
                let size = f64::from(job.size.max(256));
                WebviewWindowBuilder::new(tauri_app, "render", WebviewUrl::default())
                    .title("Emmet render")
                    .inner_size(size, size)
                    .visible(false)
                    .build()?;
                std::thread::spawn(|| {
                    std::thread::sleep(RENDER_TIMEOUT);
                    finish_render(Err(format!("no render after {} s", RENDER_TIMEOUT.as_secs())));
                });
                return Ok(());
            }
            let dev = tauri_app.config().identifier.ends_with(".dev");
            WebviewWindowBuilder::new(tauri_app, "main", WebviewUrl::default())
                .title(if dev { "Emmet DEV" } else { "Emmet" })
                .inner_size(1200.0, 800.0)
                .min_inner_size(800.0, 560.0)
                .build()?;
            let state = tauri_app.state::<app::AppState>();
            state.start_runs(&handle);
            // The login shell's environment and the agents' detection, ready before the user picks an agent.
            let warm = handle.clone();
            std::thread::spawn(move || {
                let _ = crate::chat::agents_now(&warm);
            });
            let bus_handle = handle.clone();
            match emmet_bus::Server::start(&state.data, move |msg| app::on_bus(&bus_handle, msg)) {
                // The server lives as long as the app.
                Ok(server) => {
                    tauri_app.manage(server);
                }
                Err(e) => state.notify(format!(
                    "emmet render and highlight can't reach this window: the bus didn't start ({}). They use render \
                     mode instead.",
                    e.message
                )),
            }
            if let Some(dir) = &args.open
                && let Err(e) = app::open(&handle, dir)
            {
                state.notify(format!("Couldn't open {}: {} {}", dir.display(), e.message, e.hint));
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Emmet")
        .run(|handle, event| {
            // Tauri exits without dropping managed state, so the bus's token file is removed, and the agent's CLI
            // ended, here.
            if let tauri::RunEvent::Exit = event {
                if let Some(server) = handle.try_state::<emmet_bus::Server>() {
                    server.close();
                }
                handle.state::<app::AppState>().chat.close();
            }
        });
}
