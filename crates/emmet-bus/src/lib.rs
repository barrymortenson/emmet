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
use interprocess::local_socket::{ListenerOptions, Name, Stream};
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
    /// `emmet_host approve`, Claude Code's permission prompt (plan section 10): the app shows an approval card for the
    /// chat session and replies with the decision as `answer`. Held open until the user decides.
    Approve {
        session: String,
        request: serde_json::Value,
    },
    /// `emmet_host ask_user` (D-32): the app shows a question card and replies with the answer as `answer`. Held open
    /// until the user answers, Stop cancels it, or the sender's timeout nears.
    Ask {
        session: String,
        question: serde_json::Value,
    },
    /// `emmet mcp --session` (D-33): takes the session's pending redirect notes, as `answer` (a list).
    Notes { session: String },
}

/// One request. The token itself never crosses the socket: `auth` proves the client read it from bus.json, and the
/// reply's `auth` proves the server did, so a process squatting on the socket's name can neither drive the app nor
/// answer for it (plan section 15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub v: u32,
    /// Random, new for each request.
    pub nonce: String,
    /// [`proof`]`("client", token, nonce, id:message)`.
    pub auth: String,
    pub id: u64,
    #[serde(flatten)]
    pub message: Message,
}

/// sha256 of `side:token:nonce:body`, in hex: `body` is the request's id and message, or the reply without its
/// `auth`, so a proof can't be moved to another message.
pub fn proof(side: &str, token: &str, nonce: &str, body: &str) -> String {
    fsx::sha256_hex(format!("{side}:{token}:{nonce}:{body}").as_bytes())
}

fn request_body(id: u64, message: &Message) -> String {
    format!("{id}:{}", serde_json::to_string(message).expect("message serializes"))
}

fn reply_body(reply: &Reply) -> String {
    serde_json::to_string(&Reply {
        auth: None,
        ..reply.clone()
    })
    .expect("reply serializes")
}

/// Nonces the server has seen, so a request can't be sent twice (the newest few thousand).
#[derive(Default)]
struct Seen {
    order: std::collections::VecDeque<String>,
    set: std::collections::HashSet<String>,
}

impl Seen {
    /// True the first time `nonce` comes.
    fn first(&mut self, nonce: &str) -> bool {
        if !self.set.insert(nonce.to_string()) {
            return false;
        }
        self.order.push_back(nonce.to_string());
        if self.order.len() > 4096
            && let Some(old) = self.order.pop_front()
        {
            self.set.remove(&old);
        }
        true
    }
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
    /// Where the render's camera was: `{view, position, target, fov}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub camera: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub highlighted: Vec<String>,
    /// The answer to `approve`, `ask`, or `notes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<serde_json::Value>,
    /// [`proof`]`("server", token, nonce, the reply without auth)`, on the answer to an authenticated request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
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

/// The socket for `name`: on macOS the file `/tmp/<name>`, named by its path so its mode can be set before it's bound;
/// on Windows the named pipe `\\.\pipe\<name>`.
fn socket(name: &str) -> std::io::Result<Name<'static>> {
    #[cfg(unix)]
    {
        use interprocess::local_socket::GenericFilePath;
        socket_path(name)
            .to_fs_name::<GenericFilePath>()
            .map(|n| n.into_owned())
    }
    #[cfg(windows)]
    {
        use interprocess::local_socket::GenericNamespaced;
        name.to_string().to_ns_name::<GenericNamespaced>()
    }
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
        let ns = socket(&name).map_err(|e| Error::internal(format!("the bus socket name {name}: {e}")))?;
        let listener = user_only(ListenerOptions::new().name(ns).try_overwrite(true))?
            .create_sync()
            .map_err(|e| Error::internal(format!("starting the bus on {name}: {e}")))?;
        #[cfg(unix)]
        restrict(&name);
        // 64 hex digits, but every RandomState in a thread grows from one OS-seeded 128-bit key, so about 128 bits a
        // client must guess (fsx::random_hex).
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
        let seen = Arc::new(std::sync::Mutex::new(Seen::default()));
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if halt.load(Ordering::SeqCst) {
                    break;
                }
                let Ok(conn) = conn else { continue };
                let (handler, want, seen) = (handler.clone(), want.clone(), seen.clone());
                std::thread::spawn(move || serve(conn, &want, &seen, &*handler));
            }
        });
        Ok(Self {
            info: path,
            token,
            stop,
        })
    }
}

impl Server {
    /// Stops answering and removes `bus.json` if it is still this server's. The app calls it as it exits, since
    /// Tauri ends the process without dropping managed state; `Drop` calls it too.
    pub fn close(&self) {
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

impl Drop for Server {
    fn drop(&mut self) {
        self.close();
    }
}

/// Only this user may open the socket (section 15): mode 0600 on macOS, set as soon as the socket is bound; on Windows
/// a pipe ACL that grants only the pipe's owner (the user running the app) and SYSTEM. The token file is private too
/// (fsx::write_private), so a client that connects in the moment before the chmod still can't send a message.
#[cfg(unix)]
fn user_only(opts: ListenerOptions<'_>) -> Result<ListenerOptions<'_>> {
    // macOS refuses fchmod on a socket before bind (EINVAL), so the mode is set just after it (`restrict`).
    Ok(opts)
}

#[cfg(unix)]
fn restrict(name: &str) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(socket_path(name), std::fs::Permissions::from_mode(0o600));
}

/// Where the socket file is (macOS).
#[cfg(unix)]
pub fn socket_path(name: &str) -> PathBuf {
    std::path::Path::new("/tmp").join(name)
}

#[cfg(windows)]
fn user_only(opts: ListenerOptions<'_>) -> Result<ListenerOptions<'_>> {
    use interprocess::os::windows::local_socket::ListenerOptionsExt;
    use interprocess::os::windows::security_descriptor::SecurityDescriptor;
    // Protected DACL: generic all for OWNER RIGHTS and LocalSystem, nothing for anyone else.
    let sddl = widestring::U16CString::from_str("D:P(A;;GA;;;OW)(A;;GA;;;SY)").expect("no nul in the SDDL");
    let sd = SecurityDescriptor::deserialize(&sddl)
        .map_err(|e| Error::internal(format!("the bus pipe's access list: {e}")))?;
    Ok(opts.security_descriptor(sd))
}

/// Compares tokens in time that doesn't depend on where they differ.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len() && a.bytes().zip(b.bytes()).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn serve(conn: Stream, token: &str, seen: &std::sync::Mutex<Seen>, handler: &(dyn Fn(Message) -> Reply + Send + Sync)) {
    let mut reader = BufReader::new(conn);
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok_and(|n| n > 0) {
        let mut refused = false;
        let reply = match serde_json::from_str::<Request>(line.trim()) {
            Ok(req) if req.v != VERSION => Reply {
                id: req.id,
                ..Reply::undelivered(format!("the app speaks bus v{VERSION}, not v{}", req.v))
            },
            Ok(req)
                if !same(
                    &req.auth,
                    &proof("client", token, &req.nonce, &request_body(req.id, &req.message)),
                ) =>
            {
                refused = true;
                Reply {
                    id: req.id,
                    ..Reply::undelivered("wrong token")
                }
            }
            Ok(req) if !seen.lock().expect("seen").first(&req.nonce) => {
                refused = true;
                Reply {
                    id: req.id,
                    ..Reply::undelivered("that request was already sent")
                }
            }
            Ok(req) => {
                let mut reply = Reply {
                    id: req.id,
                    ..handler(req.message)
                };
                reply.auth = Some(proof("server", token, &req.nonce, &reply_body(&reply)));
                reply
            }
            Err(e) => Reply::undelivered(format!("not a bus request: {e}")),
        };
        let mut text = serde_json::to_string(&reply).expect("reply serializes");
        text.push('\n');
        if reader.get_mut().write_all(text.as_bytes()).is_err() {
            return;
        }
        // One wrong token ends the connection, so a client can't try tokens over one stream.
        if refused {
            return;
        }
        line.clear();
    }
}

/// Sends one message to the app for this data dir and waits up to `timeout` for the reply. Never fails: no app, a
/// stale token file, or no answer in time is `delivered: false` with the reason.
/// True when something answers on the socket `bus.json` names: an app (or `emmet eval`) is serving this data dir.
/// [`Server::start`] takes the socket over, so a second server checks this first.
pub fn listening(data: &DataDir) -> bool {
    std::fs::read_to_string(info_path(data))
        .ok()
        .and_then(|t| serde_json::from_str::<Info>(&t).ok())
        .and_then(|info| socket(&info.name).ok())
        .is_some_and(|name| Stream::connect(name).is_ok())
}

pub fn send(data: &DataDir, message: Message, timeout: Duration) -> Reply {
    let Some(info) = std::fs::read_to_string(info_path(data))
        .ok()
        .and_then(|t| serde_json::from_str::<Info>(&t).ok())
    else {
        return Reply::undelivered("Emmet isn't running");
    };
    let Ok(name) = socket(&info.name) else {
        return Reply::undelivered("bus.json names no socket");
    };
    let Ok(mut conn) = Stream::connect(name) else {
        return Reply::undelivered("Emmet isn't running");
    };
    let nonce = fsx::random_hex(2);
    let req = Request {
        v: VERSION,
        auth: proof("client", &info.token, &nonce, &request_body(1, &message)),
        nonce: nonce.clone(),
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
        Ok(line) if !line.trim().is_empty() => match serde_json::from_str::<Reply>(line.trim()) {
            // A delivered answer must prove it came from the app that wrote bus.json.
            Ok(r)
                if r.delivered && r.auth.as_deref() != Some(&proof("server", &info.token, &nonce, &reply_body(&r))) =>
            {
                Reply::undelivered("the answer on the bus didn't come from Emmet")
            }
            Ok(r) => r,
            Err(e) => Reply::undelivered(format!("Emmet's reply isn't a bus reply: {e}")),
        },
        Ok(_) => Reply::undelivered("Emmet closed the connection"),
        Err(_) => Reply::undelivered(format!("Emmet didn't answer in {} s", timeout.as_secs())),
    }
}
