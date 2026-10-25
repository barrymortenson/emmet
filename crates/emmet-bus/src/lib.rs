// SPDX-License-Identifier: AGPL-3.0-or-later
//! emmet-bus v0 (plan sections 04 and 09, docs/spec/bus.md): how `emmet` and `emmet mcp` reach the running app for
//! ephemeral UI events. A local socket (a Unix socket, a named pipe on Windows) named from the data dir, a per-launch
//! token in `<data>/bus.json`, and one NDJSON request and reply per line. Messages are UI-only: they can't run code or
//! write files. When no app is listening, or the app doesn't have the project open, the reply says `delivered: false`.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use emmet_core::data::DataDir;
use emmet_core::envelope::{Error, Result};
use emmet_core::fsx;
use interprocess::local_socket::prelude::*;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, Stream};
use serde::{Deserialize, Serialize};

/// This crate's version, which tracks the workspace.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The protocol version in every request; a reply to another version is `delivered: false`.
pub const VERSION: u32 = 1;

/// What the app writes to `<data>/bus.json` while it listens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub v: u32,
    pub name: String,
    pub token: String,
    pub pid: u32,
}

/// A message for the app. `project` is the project's absolute path; the app answers only for the project it has open.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Render the project through the open viewer and return the PNG.
    Render {
        project: String,
        view: String,
        size: u32,
        #[serde(default)]
        highlight: Vec<String>,
    },
    /// Highlight parts by Emmet id in the open viewer.
    Highlight { project: String, ids: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub v: u32,
    pub token: String,
    pub id: u64,
    #[serde(flatten)]
    pub message: Message,
}

/// The app's answer. `delivered: false` carries `reason`; a delivered message that failed carries `error`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Reply {
    pub id: u64,
    pub delivered: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The rendered PNG, base64.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub png: Option<String>,
    /// Where the render's camera was: `{view, position, target}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highlighted: Vec<String>,
}

impl Reply {
    pub fn undelivered(reason: impl Into<String>) -> Self {
        Self {
            reason: Some(reason.into()),
            ..Default::default()
        }
    }
}

pub fn info_path(data: &DataDir) -> PathBuf {
    data.root().join("bus.json")
}

/// The socket name for a data dir: one app per data dir, and the dev, test, and installed apps never meet.
fn socket_name(data: &DataDir) -> String {
    let hash = fsx::sha256_hex(data.root().to_string_lossy().as_bytes());
    format!("emmet-{}.sock", &hash[..16])
}

/// The app's listener. Dropping it stops answering and removes `bus.json` if it is still this server's.
pub struct Server {
    info: PathBuf,
    token: String,
    stop: Arc<AtomicBool>,
}

impl Server {
    /// Listens for the app, calling `handler` for each message (on a thread per connection, so a slow render doesn't
    /// hold up a highlight).
    pub fn start(data: &DataDir, handler: impl Fn(Message) -> Reply + Send + Sync + 'static) -> Result<Self> {
        let name = socket_name(data);
        let ns = name
            .clone()
            .to_ns_name::<GenericNamespaced>()
            .map_err(|e| Error::internal(format!("the bus socket name {name}: {e}")))?;
        let listener = ListenerOptions::new()
            .name(ns)
            .try_overwrite(true)
            .create_sync()
            .map_err(|e| Error::internal(format!("starting the bus on {name}: {e}")))?;
        #[cfg(unix)]
        restrict(&std::path::Path::new("/tmp").join(&name));
        // 64 hex digits from OS-seeded keys (fsx::random_hex): about 128 bits a client must guess.
        let token = fsx::random_hex(4);
        let info = Info {
            v: VERSION,
            name,
            token: token.clone(),
            pid: std::process::id(),
        };
        let path = info_path(data);
        fsx::write_private(&path, serde_json::to_string_pretty(&info).expect("json").as_bytes())?;
        let stop = Arc::new(AtomicBool::new(false));
        let (handler, want, halt) = (Arc::new(handler), token.clone(), stop.clone());
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if halt.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(conn) = conn else { continue };
                let (handler, want) = (handler.clone(), want.clone());
                std::thread::spawn(move || serve(conn, &want, &*handler));
            }
        });
        Ok(Self {
            info: path,
            token,
            stop,
        })
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let ours = std::fs::read_to_string(&self.info)
            .ok()
            .and_then(|t| serde_json::from_str::<Info>(&t).ok())
            .is_some_and(|i| i.token == self.token);
        if ours {
            let _ = std::fs::remove_file(&self.info);
        }
    }
}

/// Only this user may open the socket (section 15). The token file is created private (fsx::write_private); a client
/// that connects before this chmod still needs the token. 1C adds the Windows pipe ACL and the socket's tests.
#[cfg(unix)]
fn restrict(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

/// Compares tokens in time that doesn't depend on where they differ.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn serve(conn: Stream, token: &str, handler: &(dyn Fn(Message) -> Reply + Send + Sync)) {
    let mut reader = BufReader::new(conn);
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok_and(|n| n > 0) {
        let reply = match serde_json::from_str::<Request>(line.trim()) {
            Ok(req) if req.v != VERSION => Reply {
                id: req.id,
                ..Reply::undelivered(format!("the app speaks bus v{VERSION}, not v{}", req.v))
            },
            Ok(req) if !same(&req.token, token) => Reply {
                id: req.id,
                ..Reply::undelivered("wrong token")
            },
            Ok(req) => Reply {
                id: req.id,
                ..handler(req.message)
            },
            Err(e) => Reply::undelivered(format!("not a bus request: {e}")),
        };
        let mut text = serde_json::to_string(&reply).expect("reply serializes");
        text.push('\n');
        if reader.get_mut().write_all(text.as_bytes()).is_err() {
            return;
        }
        // One wrong token ends the connection, so a client can't try tokens over one stream.
        if reply.reason.as_deref() == Some("wrong token") {
            return;
        }
        line.clear();
    }
}

/// Sends one message to the app for this data dir and waits up to `timeout` for the reply. Never fails: no app, a
/// stale token file, or no answer in time is `delivered: false` with the reason.
pub fn send(data: &DataDir, message: Message, timeout: Duration) -> Reply {
    let Some(info) = std::fs::read_to_string(info_path(data))
        .ok()
        .and_then(|t| serde_json::from_str::<Info>(&t).ok())
    else {
        return Reply::undelivered("Emmet isn't running");
    };
    let Ok(name) = info.name.clone().to_ns_name::<GenericNamespaced>() else {
        return Reply::undelivered("bus.json names no socket");
    };
    let Ok(mut conn) = Stream::connect(name) else {
        return Reply::undelivered("Emmet isn't running");
    };
    let req = Request {
        v: VERSION,
        token: info.token,
        id: 1,
        message,
    };
    let mut text = serde_json::to_string(&req).expect("request serializes");
    text.push('\n');
    if conn.write_all(text.as_bytes()).is_err() {
        return Reply::undelivered("Emmet closed the connection");
    }
    // The sync stream has no read timeout on every platform, so the read runs on its own thread.
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(conn).read_line(&mut line);
        let _ = tx.send(line);
    });
    match rx.recv_timeout(timeout) {
        Ok(line) if !line.trim().is_empty() => serde_json::from_str(line.trim())
            .unwrap_or_else(|e| Reply::undelivered(format!("Emmet's reply isn't a bus reply: {e}"))),
        Ok(_) => Reply::undelivered("Emmet closed the connection"),
        Err(_) => Reply::undelivered(format!("Emmet didn't answer in {} s", timeout.as_secs())),
    }
}
