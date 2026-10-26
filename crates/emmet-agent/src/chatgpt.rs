// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sign in with ChatGPT for open-source apps (1E, D-35, plan section 10, ADR 0011): Codex in the chat pane on the
//! user's ChatGPT plan, exactly as OpenAI documents it (docs/terms/).
//!
//! - The host: one stable, opaque `ext_agent_host_id` (`urn:uuid:` + a UUIDv4), made before the first sign-in and kept
//!   in app data (`chatgpt/host.json`). It's an identifier, not a credential.
//! - Registrations: the first sign-in registers Emmet with `client_id=dynamic_agent_client` and
//!   `agent_name_hint="Emmet"`; the callback returns the issued client id, which every later sign-in to that account
//!   reuses. Each registration's non-secret facts (label, email, `sub`, whether plan use is on) are in
//!   `chatgpt/accounts.json`, so the picker and detection never read a secret.
//! - Credentials: one record per issued client id in the OS keychain (`chatgpt:<client id>`), in the shape of OpenAI's
//!   example record, never in a file, a log, or a transcript.
//! - The flow: a loopback listener on 127.0.0.1, then the system browser, with a fresh state, nonce, and PKCE verifier
//!   per attempt; the code exchange; the ID token's signature and claims; the granted scopes.
//! - Renewal: refreshes are serialized (a lock in this process and a file lock across processes; every write to the
//!   accounts or a record takes it too) and replace each token the response carries, keeping the refresh token, ID
//!   token, and scopes only when it leaves them out. Sign-out revokes the refresh token, then clears the record and
//!   keeps the registration.
//! - A file or record Emmet can't read is an error that names it, never an empty list: an unreadable `accounts.json`
//!   must not move Codex to the API key, nor be saved over.
//!
//! Tests point the issuer at `fake-openai` with `EMMET_FAKE_OPENAI=<base URL>`, and replace the system browser with
//! `EMMET_FAKE_BROWSER=<dir>`, in a debug build only, like `EMMET_FAKE_KEYCHAIN`.

mod jwt;

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{AgentError, AgentErrorCode, Result, env, keychain, time};

/// Emmet's name at OpenAI: the registration's `agent_name_hint` and the app-server's `clientInfo.name`.
pub const AGENT_NAME: &str = "Emmet";
/// The scope that lets an app use the user's ChatGPT plan. A valid ID token alone doesn't.
pub const PLAN_SCOPE: &str = "chatgpt.tokens.use.direct";
/// ChatGPT Settings → Usage: where Manage usage goes.
pub const USAGE_URL: &str = "https://chatgpt.com/settings/usage";

const ISSUER: &str = "https://auth.openai.com";
const RESOURCE: &str = "https://api.openai.com/v1";
const SCOPES: &str = "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
const DYNAMIC: &str = "dynamic_agent_client";
/// The loopback port tried first (OpenAI's example); any free port works after it.
const PORT: u16 = 1455;
/// How long a sign-in waits for the browser.
const WAIT: Duration = Duration::from_secs(600);
/// A refresh token that can't be used again: sign in again with the saved client id.
const UNUSABLE: &[&str] = &[
    "invalid_grant",
    "invalid_refresh_token",
    "token_expired",
    "refresh_token_expired",
    "refresh_token_invalidated",
    "refresh_token_reused",
];

/// Refreshes in this process, one at a time; `refresh.lock` does the same across processes.
static REFRESH: Mutex<()> = Mutex::new(());

/// Which way Codex pays, as the user chose it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    #[serde(rename = "chatgpt")]
    ChatGpt,
    ApiKey,
}

/// What Codex will run on now.
#[derive(Debug, Clone, PartialEq)]
pub enum CodexUse {
    /// The active ChatGPT account, signed in with plan use on.
    Plan {
        id: String,
        label: String,
    },
    /// The user chose ChatGPT, but the active account can't be used: `fix` says what to do.
    PlanOff {
        id: Option<String>,
        fix: String,
    },
    ApiKey,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Registration {
    client_id: String,
    /// The validated ID token's `sub`; None while a first registration's exchange hasn't succeeded.
    #[serde(default)]
    subject: Option<String>,
    #[serde(default)]
    email: Option<String>,
    label: String,
    #[serde(default)]
    plan: bool,
    #[serde(default)]
    signed_in: bool,
    /// The one-time "You're using your ChatGPT plan" notice was shown.
    #[serde(default)]
    notice_seen: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Accounts {
    #[serde(default)]
    active: Option<String>,
    #[serde(default)]
    codex: Option<Choice>,
    #[serde(default)]
    registrations: Vec<Registration>,
}

/// One registration's record in the keychain, in the shape of OpenAI's example.
#[derive(Clone, Serialize, Deserialize)]
struct Credentials {
    email: Option<String>,
    issuer: String,
    subject: String,
    client_id: String,
    ext_agent_host_id: String,
    id_token: String,
    access_token: Option<String>,
    refresh_token: Option<String>,
    token_type: String,
    expires_in: i64,
    /// Unix seconds.
    expires_at: i64,
    #[serde(default)]
    earliest_refresh_at: Option<i64>,
    scopes: Vec<String>,
    saved_at: String,
}

/// A saved ChatGPT account, for the picker.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct AccountView {
    /// The issued client id: the registration's stable identity in Emmet.
    pub id: String,
    pub label: String,
    pub email: Option<String>,
    pub active: bool,
    pub signed_in: bool,
    /// ChatGPT plan use was granted.
    pub plan: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct ChatGptState {
    pub accounts: Vec<AccountView>,
    /// What the user chose for Codex, if anything.
    pub codex: Option<Choice>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SignInResult {
    /// `notice`: show the one-time "You're using your ChatGPT plan" now.
    SignedIn {
        account: AccountView,
        notice: bool,
    },
    /// The user declined in the browser (`access_denied`).
    Declined,
    Canceled,
    /// No answer from the browser in 10 minutes.
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SignOut {
    /// OpenAI confirmed it revoked the session. False when it didn't, or when there was no refresh token left to
    /// revoke (the record was gone or unreadable); the tokens are cleared anyway, and the user can disconnect Emmet in
    /// ChatGPT Settings.
    pub confirmed: bool,
}

fn err(code: AgentErrorCode, message: impl Into<String>) -> AgentError {
    AgentError::new(code, message)
}

fn io(e: impl std::fmt::Display) -> AgentError {
    err(AgentErrorCode::Io, e.to_string())
}

fn now_s() -> i64 {
    (time::now_ms() / 1000) as i64
}

fn random(n: usize) -> Vec<u8> {
    let mut b = vec![0; n];
    SystemRandom::new().fill(&mut b).expect("the system random source");
    b
}

/// 32 random bytes, base64url without padding: a state, a nonce, or a PKCE verifier (43 characters).
fn random_token() -> String {
    emmet_core::base64::encode_url(&random(32))
}

/// PKCE S256: base64url(SHA-256(verifier)), without padding.
pub fn pkce_challenge(verifier: &str) -> String {
    emmet_core::base64::encode_url(ring::digest::digest(&ring::digest::SHA256, verifier.as_bytes()).as_ref())
}

/// `urn:uuid:` and a random UUIDv4.
fn new_host_id() -> String {
    let mut b = random(16);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!(
        "urn:uuid:{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}

/// Percent-encodes everything but RFC 3986's unreserved characters.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            b => format!("%{b:02X}"),
        })
        .collect()
}

fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let hex = |j: usize| b.get(j).and_then(|c| (*c as char).to_digit(16));
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' => match (hex(i + 1), hex(i + 2)) {
                (Some(h), Some(l)) => {
                    out.push((h * 16 + l) as u8);
                    i += 2;
                }
                _ => out.push(b'%'),
            },
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A query string's pairs, decoded.
pub fn parse_query(q: &str) -> Vec<(String, String)> {
    q.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (decode(k), decode(v))
        })
        .collect()
}

fn query(pairs: &[(&str, &str)]) -> String {
    pairs
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

fn http() -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder()
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .tls_config(tls)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .new_agent()
}

/// A form POST: the status and the JSON body (Null if it isn't JSON). Err when there was no whole answer: a body cut
/// off mid-download is a network failure, not OpenAI's.
fn post_form(url: &str, form: &[(&str, &str)]) -> std::result::Result<(u16, Value), String> {
    let mut r = http()
        .post(url)
        .send_form(form.iter().copied())
        .map_err(|e| e.to_string())?;
    let status = r.status().as_u16();
    let body = r.body_mut().read_to_string().map_err(|e| e.to_string())?;
    Ok((status, serde_json::from_str(&body).unwrap_or(Value::Null)))
}

fn get_json(url: &str) -> Result<Value> {
    let mut r = http().get(url).call().map_err(|e| {
        err(
            AgentErrorCode::Agent,
            format!("couldn't reach OpenAI's sign-in service: {e}"),
        )
    })?;
    if r.status().as_u16() != 200 {
        return Err(err(
            AgentErrorCode::Agent,
            format!("OpenAI's sign-in service answered {} for {url}", r.status()),
        ));
    }
    let body = r.body_mut().read_to_string().unwrap_or_default();
    serde_json::from_str(&body).map_err(|e| err(AgentErrorCode::Agent, format!("{url}: {e}")))
}

/// OpenAI's error code from a token or revocation response.
fn oauth_error(body: &Value) -> String {
    body["error"]
        .as_str()
        .or(body["error"]["code"].as_str())
        .unwrap_or("")
        .to_string()
}

/// Opens `url` in the system browser. `EMMET_FAKE_BROWSER=<dir>` (a debug build only) instead notes it in
/// `<dir>/opened.txt` (an authorize URL without its query, since a returning one carries `id_token_hint`) and, for the
/// fake issuer, fetches it and follows the redirect to the loopback callback, as a browser would, noting the callback's
/// HTTP status in `<dir>/callbacks.txt`.
pub fn open_url(env: &[(String, String)], url: &str) -> Result<()> {
    if cfg!(debug_assertions)
        && let Some(dir) = env::get(env, "EMMET_FAKE_BROWSER")
    {
        let noted = if url.contains("/authorize?") {
            url.split('?').next().unwrap_or(url)
        } else {
            url
        };
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(Path::new(dir).join("opened.txt"))
            .map_err(io)?;
        let _ = writeln!(f, "{noted}");
        if fake_issuer(env).is_some_and(|base| url.starts_with(&base)) {
            let (url, dir) = (url.to_string(), PathBuf::from(dir));
            std::thread::spawn(move || {
                let status = match ureq::get(&url).call() {
                    Ok(r) => r.status().as_u16().to_string(),
                    Err(ureq::Error::StatusCode(c)) => c.to_string(),
                    Err(e) => e.to_string(),
                };
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(dir.join("callbacks.txt"))
                {
                    let _ = writeln!(f, "{status}");
                }
            });
        }
        return Ok(());
    }
    let mut cmd = if cfg!(windows) {
        let mut c = std::process::Command::new("rundll32.exe");
        c.arg("url.dll,FileProtocolHandler");
        c
    } else {
        std::process::Command::new("open")
    };
    cmd.arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    crate::session::no_window(&mut cmd);
    // Both hand the URL to the system and exit at once, so their status says whether a browser got it.
    let status = cmd
        .status()
        .map_err(|e| err(AgentErrorCode::Io, format!("couldn't open the browser: {e}")))?;
    if !status.success() {
        return Err(err(
            AgentErrorCode::Io,
            format!("couldn't open the browser ({status}). Is a default web browser set?"),
        ));
    }
    Ok(())
}

fn fake_issuer(env: &[(String, String)]) -> Option<String> {
    if !cfg!(debug_assertions) {
        return None;
    }
    env::get(env, "EMMET_FAKE_OPENAI").map(|s| s.trim_end_matches('/').to_string())
}

const PAGE: &str = "<!doctype html><meta charset=utf-8><title>Emmet</title>\
<body style=\"font:16px system-ui,sans-serif;margin:4em auto;max-width:28em;text-align:center\">";

fn respond(mut s: &TcpStream, status: &str, body: &str) {
    let html = format!("{PAGE}<p>{body}</p></body>");
    let _ = write!(
        s,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nConnection: close\r\n\
         Content-Length: {}\r\n\r\n{html}",
        html.len()
    );
    let _ = s.flush();
}

enum Waited {
    Callback(Vec<(String, String)>),
    Canceled,
    TimedOut,
}

/// Waits for the browser's callback with this attempt's `state`. A request with another state (a stale tab, or a
/// forgery) is answered and ignored, and the wait goes on, as it does after a failed accept.
fn wait_callback(listener: &TcpListener, state: &str, cancel: &AtomicBool, until: Instant) -> Result<Waited> {
    listener.set_nonblocking(true).map_err(io)?;
    loop {
        if cancel.load(Ordering::SeqCst) {
            return Ok(Waited::Canceled);
        }
        if Instant::now() >= until {
            return Ok(Waited::TimedOut);
        }
        let stream = match listener.accept() {
            Ok((s, _)) => s,
            Err(_) => {
                std::thread::sleep(Duration::from_millis(20));
                continue;
            }
        };
        // An accepted socket can inherit the listener's non-blocking mode; read it blocking, or not at all.
        if stream.set_nonblocking(false).is_err() {
            continue;
        }
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut line = String::new();
        if BufReader::new((&stream).take(8192)).read_line(&mut line).is_err() {
            continue;
        }
        let target = line.split_whitespace().nth(1).unwrap_or("");
        let (path, q) = target.split_once('?').unwrap_or((target, ""));
        if path != "/auth/callback" {
            respond(&stream, "404 Not Found", "Not found.");
            continue;
        }
        let pairs = parse_query(q);
        if !pairs.iter().any(|(k, v)| k == "state" && v == state) {
            respond(
                &stream,
                "400 Bad Request",
                "This sign-in link isn't the one Emmet is waiting for. Go back to Emmet and press Continue with ChatGPT.",
            );
            continue;
        }
        // Emmet still has to exchange the code and check the ID token, so the page doesn't say it worked.
        let declined = pairs.iter().any(|(k, _)| k == "error");
        respond(
            &stream,
            "200 OK",
            if declined {
                "Nothing was changed. You can close this tab and go back to Emmet."
            } else {
                "You can close this tab. Emmet finishes signing you in and shows the result."
            },
        );
        return Ok(Waited::Callback(pairs));
    }
}

fn get<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

/// The token endpoint's answer.
struct Tokens {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
    token_type: String,
    expires_in: i64,
    scopes: Option<Vec<String>>,
    earliest_refresh_at: Option<i64>,
}

impl Tokens {
    fn from(body: &Value) -> Result<Self> {
        let access_token = body["access_token"].as_str().filter(|s| !s.is_empty()).ok_or_else(|| {
            err(
                AgentErrorCode::AuthFailed,
                "OpenAI's token response has no access token.",
            )
        })?;
        let earliest = &body["earliest_refresh_at"];
        Ok(Self {
            access_token: access_token.into(),
            refresh_token: body["refresh_token"].as_str().map(String::from),
            id_token: body["id_token"].as_str().map(String::from),
            token_type: body["token_type"].as_str().unwrap_or("Bearer").into(),
            expires_in: body["expires_in"].as_i64().unwrap_or(3600),
            scopes: body["scope"]
                .as_str()
                .map(|s| s.split_whitespace().map(String::from).collect()),
            earliest_refresh_at: earliest
                .as_i64()
                .or_else(|| earliest.as_str().and_then(time::parse_rfc3339)),
        })
    }
}

/// Sign in with ChatGPT for one machine: its host id, its registrations, and their credentials.
pub struct ChatGpt {
    dir: PathBuf,
    env: Vec<(String, String)>,
    issuer: String,
}

impl ChatGpt {
    /// `data` is the app data dir; `env` carries the test settings (the fake keychain, issuer, and browser).
    pub fn new(data: &Path, env: &[(String, String)]) -> Self {
        Self {
            dir: data.join("chatgpt"),
            env: env.to_vec(),
            issuer: fake_issuer(env).unwrap_or_else(|| ISSUER.into()),
        }
    }

    fn authorize_url(&self) -> String {
        format!("{}/api/accounts/authorize", self.issuer)
    }

    fn token_url(&self) -> String {
        format!("{}/api/accounts/oauth/token", self.issuer)
    }

    fn discovery(&self) -> Result<Value> {
        get_json(&format!("{}/.well-known/openid-configuration", self.issuer))
    }

    /// An endpoint from discovery, accepted only under the issuer's base URL (`https://auth.openai.com/`, or the
    /// fake's in tests).
    fn endpoint(&self, d: &Value, key: &str) -> Result<String> {
        let url = d[key].as_str().unwrap_or("");
        let ours = url.starts_with(&format!("{}/", self.issuer));
        if !ours {
            return Err(err(
                AgentErrorCode::Agent,
                format!("OpenAI's sign-in discovery gave no usable {key}."),
            ));
        }
        Ok(url.to_string())
    }

    /// A file in `chatgpt/`, or None if it isn't there. Any other failure to read it is an error that names it.
    fn read(&self, name: &str) -> Result<Option<String>> {
        let file = self.dir.join(name);
        match std::fs::read_to_string(&file) {
            Ok(s) => Ok(Some(s)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(unreadable(&file, &e.to_string())),
        }
    }

    /// This machine's `ext_agent_host_id`, made and saved the first time it's asked for. A `host.json` that can't be
    /// read is an error, never a new id: the id must stay the same for the host's life.
    pub fn host_id(&self) -> Result<String> {
        if let Some(text) = self.read("host.json")? {
            return serde_json::from_str::<Value>(&text)
                .ok()
                .and_then(|v| v["ext_agent_host_id"].as_str().map(String::from))
                .ok_or_else(|| unreadable(&self.dir.join("host.json"), "it has no ext_agent_host_id"));
        }
        let id = new_host_id();
        std::fs::create_dir_all(&self.dir).map_err(io)?;
        emmet_core::fsx::write_atomic(
            &self.dir.join("host.json"),
            json!({ "ext_agent_host_id": id }).to_string().as_bytes(),
        )
        .map_err(io)?;
        Ok(id)
    }

    /// The accounts: empty if there's no `accounts.json` yet; an error if there is one Emmet can't read.
    fn load(&self) -> Result<Accounts> {
        match self.read("accounts.json")? {
            None => Ok(Accounts::default()),
            Some(text) => {
                serde_json::from_str(&text).map_err(|e| unreadable(&self.dir.join("accounts.json"), &e.to_string()))
            }
        }
    }

    fn save(&self, a: &Accounts) -> Result<()> {
        std::fs::create_dir_all(&self.dir).map_err(io)?;
        emmet_core::fsx::write_atomic(
            &self.dir.join("accounts.json"),
            serde_json::to_string_pretty(a).expect("json").as_bytes(),
        )
        .map_err(io)
    }

    /// Changes the accounts under the lock, so a refresh or another window can't save a stale copy over it.
    fn update<T>(&self, f: impl FnOnce(&mut Accounts) -> Result<T>) -> Result<T> {
        let _locks = self.lock()?;
        self.update_locked(f)
    }

    /// [`Self::update`] for a caller that already holds the lock.
    fn update_locked<T>(&self, f: impl FnOnce(&mut Accounts) -> Result<T>) -> Result<T> {
        let mut a = self.load()?;
        let out = f(&mut a)?;
        self.save(&a)?;
        Ok(out)
    }

    fn entry(id: &str) -> String {
        format!("chatgpt:{id}")
    }

    /// The registration's keychain record, or None. A record that doesn't parse is an error, never "signed out".
    fn credentials(&self, id: &str) -> Result<Option<Credentials>> {
        match keychain::get_entry(&self.env, &Self::entry(id))? {
            None => Ok(None),
            Some(s) => serde_json::from_str(&s).map(Some).map_err(|e| {
                err(
                    AgentErrorCode::Io,
                    format!("Emmet can't read its keychain entry {}: {e}", Self::entry(id)),
                )
            }),
        }
    }

    fn store(&self, c: &Credentials) -> Result<()> {
        keychain::set_entry(
            &self.env,
            &Self::entry(&c.client_id),
            &serde_json::to_string(c).expect("json"),
        )
    }

    /// The saved accounts and the user's choice, without reading a secret.
    pub fn state(&self) -> Result<ChatGptState> {
        let a = self.load()?;
        Ok(ChatGptState {
            accounts: a
                .registrations
                .iter()
                .filter(|r| r.subject.is_some())
                .map(|r| view(r, a.active.as_deref()))
                .collect(),
            codex: a.codex,
        })
    }

    /// What Codex runs on now. No secret is read. Accounts Emmet can't read stop Codex with the reason, rather than
    /// move it to the API key.
    pub fn codex_use(&self) -> CodexUse {
        let a = match self.load() {
            Ok(a) => a,
            Err(e) => {
                return CodexUse::PlanOff {
                    id: None,
                    fix: e.message,
                };
            }
        };
        if a.codex != Some(Choice::ChatGpt) {
            return CodexUse::ApiKey;
        }
        let Some(r) = a
            .active
            .as_deref()
            .and_then(|id| a.registrations.iter().find(|r| r.client_id == id))
        else {
            return CodexUse::PlanOff {
                id: None,
                fix: "Continue with ChatGPT in Settings → Agents.".into(),
            };
        };
        if !r.signed_in {
            CodexUse::PlanOff {
                id: Some(r.client_id.clone()),
                fix: format!(
                    "Sign in to {} again: Continue with ChatGPT in Settings → Agents.",
                    r.label
                ),
            }
        } else if !r.plan {
            CodexUse::PlanOff {
                id: Some(r.client_id.clone()),
                fix: format!(
                    "ChatGPT plan use isn't enabled for {}. Enable it in Settings → Agents, or use an OpenAI API key.",
                    r.label
                ),
            }
        } else {
            CodexUse::Plan {
                id: r.client_id.clone(),
                label: r.label.clone(),
            }
        }
    }

    /// Codex runs on the ChatGPT account `id`, or on the API key (None). Signing in to an account also chooses it.
    pub fn choose(&self, id: Option<&str>) -> Result<()> {
        self.update(|a| {
            match id {
                Some(id) => {
                    if !a.registrations.iter().any(|r| r.client_id == id) {
                        return Err(err(
                            AgentErrorCode::NotSignedIn,
                            "That ChatGPT account isn't saved in Emmet.",
                        ));
                    }
                    a.active = Some(id.into());
                    a.codex = Some(Choice::ChatGpt);
                }
                None => a.codex = Some(Choice::ApiKey),
            }
            Ok(())
        })
    }

    /// The one-time notice was shown for `id`.
    pub fn notice_seen(&self, id: &str) -> Result<()> {
        self.update(|a| {
            if let Some(r) = a.registrations.iter_mut().find(|r| r.client_id == id) {
                r.notice_seen = true;
            }
            Ok(())
        })
    }

    /// Signs in in the system browser: a new account (`account` None), or again to the saved one. `consent` asks
    /// OpenAI to show the permission screen again (`prompt=consent`), to enable plan use after a decline. The account
    /// becomes active only once its ID token and identity are validated.
    pub fn sign_in(&self, account: Option<&str>, consent: bool, cancel: &AtomicBool) -> Result<SignInResult> {
        let host = self.host_id()?;
        let accounts = self.load()?;
        // A saved account, or a first registration whose exchange failed: its issued client id is reused.
        let reg = match account {
            Some(id) => Some(
                accounts
                    .registrations
                    .iter()
                    .find(|r| r.client_id == id)
                    .cloned()
                    .ok_or_else(|| {
                        err(
                            AgentErrorCode::NotSignedIn,
                            "That ChatGPT account isn't saved in Emmet.",
                        )
                    })?,
            ),
            None => accounts.registrations.iter().find(|r| r.subject.is_none()).cloned(),
        };
        let hint = match &reg {
            Some(r) => self.credentials(&r.client_id)?.map(|c| c.id_token),
            None => None,
        };
        let listener = TcpListener::bind(("127.0.0.1", PORT))
            .or_else(|_| TcpListener::bind(("127.0.0.1", 0)))
            .map_err(io)?;
        let redirect = format!(
            "http://127.0.0.1:{}/auth/callback",
            listener.local_addr().map_err(io)?.port()
        );
        let (state, nonce, verifier) = (random_token(), random_token(), random_token());
        let challenge = pkce_challenge(&verifier);
        let mut q: Vec<(&str, &str)> = match &reg {
            None => vec![("client_id", DYNAMIC), ("agent_name_hint", AGENT_NAME)],
            Some(r) => {
                let mut q = vec![("client_id", r.client_id.as_str())];
                if let Some(h) = &hint {
                    q.push(("id_token_hint", h));
                }
                if let Some(e) = &r.email {
                    q.push(("login_hint", e));
                }
                q
            }
        };
        q.extend([
            ("ext_agent_host_id", host.as_str()),
            ("response_type", "code"),
            ("redirect_uri", redirect.as_str()),
            ("scope", SCOPES),
            ("resource", RESOURCE),
            ("state", state.as_str()),
            ("nonce", nonce.as_str()),
            ("code_challenge_method", "S256"),
            ("code_challenge", challenge.as_str()),
        ]);
        if consent {
            q.push(("prompt", "consent"));
        }
        open_url(&self.env, &format!("{}?{}", self.authorize_url(), query(&q)))?;
        let cb = match wait_callback(&listener, &state, cancel, Instant::now() + WAIT)? {
            Waited::Callback(cb) => cb,
            Waited::Canceled => return Ok(SignInResult::Canceled),
            Waited::TimedOut => return Ok(SignInResult::TimedOut),
        };
        drop(listener);
        if let Some(e) = get(&cb, "error") {
            if e == "access_denied" {
                return Ok(SignInResult::Declined);
            }
            let why = get(&cb, "error_description").unwrap_or(e);
            return Err(err(
                AgentErrorCode::AuthFailed,
                format!("ChatGPT sign-in didn't finish: {why}"),
            ));
        }
        let code = get(&cb, "code")
            .filter(|c| !c.is_empty())
            .ok_or_else(|| err(AgentErrorCode::AuthFailed, "ChatGPT sign-in returned no code."))?;
        let client_id = match &reg {
            None => get(&cb, "client_id")
                .filter(|c| !c.is_empty() && *c != DYNAMIC)
                .ok_or_else(|| {
                    err(
                        AgentErrorCode::AuthFailed,
                        "ChatGPT didn't finish registering Emmet. Press Continue with ChatGPT to try again.",
                    )
                })?
                .to_string(),
            Some(r) => {
                if get(&cb, "client_id").is_some_and(|c| c != r.client_id) {
                    return Err(err(
                        AgentErrorCode::AuthFailed,
                        format!(
                            "ChatGPT answered for a different registration than {}, so nothing was changed.",
                            r.label
                        ),
                    ));
                }
                r.client_id.clone()
            }
        };
        if reg.is_none() {
            // Saved before anything else can fail, so a retry reuses the issued client id instead of registering
            // Emmet again; it stays out of the picker until a sign-in validates it.
            self.update(|a| {
                a.registrations.retain(|x| x.client_id != client_id);
                a.registrations.push(Registration {
                    client_id: client_id.clone(),
                    label: "ChatGPT account".into(),
                    ..Default::default()
                });
                Ok(())
            })?;
        }

        let (status, body) = post_form(
            &self.token_url(),
            &[
                ("grant_type", "authorization_code"),
                ("client_id", &client_id),
                ("code", code),
                ("code_verifier", &verifier),
                ("redirect_uri", &redirect),
                ("resource", RESOURCE),
            ],
        )
        .map_err(|e| {
            err(
                AgentErrorCode::Agent,
                format!("couldn't reach OpenAI's sign-in service: {e}"),
            )
        })?;
        if status != 200 {
            // The code is spent; the next attempt reuses the issued client id (saved above).
            let code = oauth_error(&body);
            return Err(err(
                AgentErrorCode::AuthFailed,
                if code == "invalid_grant" {
                    "ChatGPT sign-in didn't finish. Press Continue with ChatGPT to try again.".to_string()
                } else {
                    format!("OpenAI refused the sign-in ({status} {code}).")
                },
            ));
        }
        let tokens = Tokens::from(&body)?;
        let id_token = tokens
            .id_token
            .clone()
            .ok_or_else(|| err(AgentErrorCode::AuthFailed, "OpenAI's token response has no ID token."))?;
        let jwks_url = self.endpoint(&self.discovery()?, "jwks_uri")?;
        let claims = jwt::verify(
            &id_token,
            &get_json(&jwks_url)?,
            &jwt::Expect {
                issuer: &self.issuer,
                audience: &client_id,
                nonce: &nonce,
                now: now_s(),
            },
        )
        .map_err(|why| {
            err(
                AgentErrorCode::AuthFailed,
                format!("Emmet couldn't verify the sign-in: {why}."),
            )
        })?;
        if let Some(sub) = reg.as_ref().and_then(|r| r.subject.as_ref())
            && *sub != claims.sub
        {
            return Err(err(
                AgentErrorCode::AuthFailed,
                format!(
                    "That's a different ChatGPT account from {}, so nothing was changed. To add it, choose Add \
                     another account.",
                    reg.as_ref().map_or("", |r| r.label.as_str())
                ),
            ));
        }
        // The token response's scopes decide; OAuth lets it leave them out when they're the ones the callback gave.
        let scopes = tokens
            .scopes
            .clone()
            .or_else(|| get(&cb, "scope").map(|s| s.split_whitespace().map(String::from).collect()))
            .unwrap_or_default();
        let plan = scopes.iter().any(|s| s == PLAN_SCOPE);
        // Under the lock from here: a refresh of this registration in a running chat can't write its older record
        // over this one, or save stale accounts.
        let _locks = self.lock()?;
        self.store(&Credentials {
            email: claims.email.clone(),
            issuer: self.issuer.clone(),
            subject: claims.sub.clone(),
            client_id: client_id.clone(),
            ext_agent_host_id: host,
            id_token,
            access_token: Some(tokens.access_token),
            refresh_token: tokens.refresh_token,
            token_type: tokens.token_type,
            expires_in: tokens.expires_in,
            expires_at: now_s() + tokens.expires_in,
            earliest_refresh_at: tokens.earliest_refresh_at,
            scopes,
            saved_at: time::rfc3339(time::now_ms()),
        })?;
        let (r, active) = self.update_locked(|a| {
            let label = match a
                .registrations
                .iter()
                .find(|r| r.client_id == client_id && r.subject.is_some())
            {
                Some(r) => r.label.clone(),
                None => unique_label(a, claims.email.as_deref()),
            };
            let seen = a
                .registrations
                .iter()
                .find(|r| r.client_id == client_id)
                .is_some_and(|r| r.notice_seen);
            let r = Registration {
                client_id: client_id.clone(),
                subject: Some(claims.sub),
                email: claims.email,
                label,
                plan,
                signed_in: true,
                notice_seen: seen,
            };
            a.registrations.retain(|x| x.client_id != client_id);
            a.registrations.push(r.clone());
            a.active = Some(client_id.clone());
            a.codex = Some(Choice::ChatGpt);
            Ok((r, a.active.clone()))
        })?;
        Ok(SignInResult::SignedIn {
            account: view(&r, active.as_deref()),
            notice: r.plan && !r.notice_seen,
        })
    }

    /// Changes registration `id`'s flags. The caller holds the lock.
    fn set_flags(&self, id: &str, f: impl FnOnce(&mut Registration)) -> Result<()> {
        self.update_locked(|a| {
            if let Some(r) = a.registrations.iter_mut().find(|r| r.client_id == id) {
                f(r);
            }
            Ok(())
        })
    }

    /// Holds the refresh locks: this process's, then `refresh.lock` for other processes.
    fn lock(&self) -> Result<(std::sync::MutexGuard<'static, ()>, std::fs::File)> {
        let g = REFRESH.lock().unwrap_or_else(|e| e.into_inner());
        std::fs::create_dir_all(&self.dir).map_err(io)?;
        let f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(self.dir.join("refresh.lock"))
            .map_err(io)?;
        f.lock().map_err(io)?;
        Ok((g, f))
    }

    /// An access token for `id`, refreshed if it has less than `min_valid` left. When OpenAI's `earliest_refresh_at`
    /// hasn't passed, or the refresh can't reach OpenAI (a network failure or a 5xx), the current token is returned
    /// if it has more than a minute left. `refused` is a token the API refused: it's renewed even if it hasn't
    /// expired, unless another caller already replaced it.
    pub fn access_token(&self, id: &str, refused: Option<&str>, min_valid: Duration) -> Result<String> {
        let _locks = self.lock()?;
        let signed_out = || {
            err(
                AgentErrorCode::NotSignedIn,
                "Your ChatGPT sign-in has ended. Continue with ChatGPT in Settings → Agents to sign in again.",
            )
        };
        // No record (the keychain was cleared), or one without tokens: signed out, as the picker will now say.
        let Some(mut c) = self.credentials(id)? else {
            self.set_flags(id, |r| r.signed_in = false)?;
            return Err(signed_out());
        };
        let (Some(current), Some(refresh)) = (c.access_token.clone(), c.refresh_token.clone()) else {
            self.set_flags(id, |r| r.signed_in = false)?;
            return Err(signed_out());
        };
        let now = now_s();
        let left = c.expires_at - now;
        let renewed_elsewhere = refused.is_some_and(|r| r != current);
        let want = refused == Some(current.as_str()) || left <= min_valid.as_secs() as i64;
        let too_early = c.earliest_refresh_at.is_some_and(|t| t > now) && refused.is_none() && left > 60;
        if renewed_elsewhere || !want || too_early {
            return Ok(current);
        }
        let answer = post_form(
            &self.token_url(),
            &[
                ("grant_type", "refresh_token"),
                ("client_id", id),
                ("refresh_token", &refresh),
                ("resource", RESOURCE),
            ],
        );
        let (status, body) = match answer {
            Ok((s, b)) if s < 500 => (s, b),
            // A network failure or a 5xx never erases credentials; a token that's still good is used.
            other => {
                if left > 60 && refused.is_none() {
                    return Ok(current);
                }
                let why = match other {
                    Ok((s, _)) => format!("it answered {s}"),
                    Err(e) => e,
                };
                return Err(err(
                    AgentErrorCode::Agent,
                    format!("Couldn't renew your ChatGPT sign-in ({why}). Try again in a few minutes."),
                ));
            }
        };
        if status != 200 {
            let code = oauth_error(&body);
            if UNUSABLE.contains(&code.as_str()) {
                c.access_token = None;
                c.refresh_token = None;
                self.store(&c)?;
                self.set_flags(id, |r| r.signed_in = false)?;
                return Err(signed_out());
            }
            if code == "invalid_client" {
                // A registration OpenAI no longer knows can't be signed in to again: forget it, so Continue with
                // ChatGPT registers afresh.
                keychain::clear_entry(&self.env, &Self::entry(id))?;
                self.update_locked(|a| {
                    a.registrations.retain(|r| r.client_id != id);
                    if a.active.as_deref() == Some(id) {
                        a.active = None;
                    }
                    Ok(())
                })?;
                return Err(err(
                    AgentErrorCode::NotSignedIn,
                    "OpenAI no longer accepts Emmet's registration for this ChatGPT account (invalid_client), so \
                     Emmet removed it. Continue with ChatGPT in Settings → Agents to sign in again.",
                ));
            }
            return Err(err(
                AgentErrorCode::AuthFailed,
                format!(
                    "OpenAI refused to renew your ChatGPT sign-in ({status} {code}). If it keeps happening, sign out \
                     in Settings → Agents and sign in again."
                ),
            ));
        }
        let t = Tokens::from(&body)?;
        c.access_token = Some(t.access_token.clone());
        c.refresh_token = t.refresh_token.or(c.refresh_token);
        if let Some(idt) = t.id_token {
            c.id_token = idt;
        }
        c.token_type = t.token_type;
        c.expires_in = t.expires_in;
        c.expires_at = now_s() + t.expires_in;
        c.earliest_refresh_at = t.earliest_refresh_at;
        if let Some(s) = t.scopes {
            c.scopes = s;
        }
        c.saved_at = time::rfc3339(time::now_ms());
        self.store(&c)?;
        let plan = c.scopes.iter().any(|s| s == PLAN_SCOPE);
        self.set_flags(id, |r| r.plan = plan)?;
        Ok(t.access_token)
    }

    /// Signs `id` out: revokes its refresh token (retrying a network failure or 5xx), then clears its tokens. The
    /// registration and the host id stay, for a later sign-in.
    pub fn sign_out(&self, id: &str) -> Result<SignOut> {
        let _locks = self.lock()?;
        // Confirmed only when OpenAI says it revoked a refresh token. A record that's gone or unreadable has none to
        // revoke, so the session may still be live at OpenAI.
        let confirmed = match self.credentials(id).ok().flatten().and_then(|c| c.refresh_token) {
            Some(rt) => self.revoke(id, &rt),
            None => false,
        };
        self.set_flags(id, |r| r.signed_in = false)?;
        keychain::clear_entry(&self.env, &Self::entry(id))?;
        Ok(SignOut { confirmed })
    }

    fn revoke(&self, id: &str, refresh: &str) -> bool {
        let mut wait = Duration::from_millis(250);
        for _ in 0..4 {
            let answer = self
                .discovery()
                .and_then(|d| self.endpoint(&d, "revocation_endpoint"))
                .ok()
                .map(|url| {
                    post_form(
                        &url,
                        &[
                            ("token", refresh),
                            ("token_type_hint", "refresh_token"),
                            ("client_id", id),
                        ],
                    )
                });
            match answer {
                Some(Ok((200, _))) => return true,
                Some(Ok((s, _))) if s < 500 => return false,
                _ => {}
            }
            std::thread::sleep(wait);
            wait *= 2;
        }
        false
    }
}

/// A file in `chatgpt/` Emmet can't read, as an error that says which and what to do.
fn unreadable(file: &Path, why: &str) -> AgentError {
    err(
        AgentErrorCode::Io,
        format!(
            "Emmet can't read {} ({why}). Fix or remove that file, then try again; removing it forgets the saved \
             ChatGPT accounts.",
            file.display()
        ),
    )
}

fn view(r: &Registration, active: Option<&str>) -> AccountView {
    AccountView {
        id: r.client_id.clone(),
        label: r.label.clone(),
        email: r.email.clone(),
        active: active == Some(r.client_id.as_str()),
        signed_in: r.signed_in,
        plan: r.signed_in && r.plan,
    }
}

/// The email, or "ChatGPT account", made distinct from the other registrations' labels ("… (2)"): two registrations
/// can share an email (two workspaces).
fn unique_label(a: &Accounts, email: Option<&str>) -> String {
    let base = email.unwrap_or("ChatGPT account");
    let taken = |l: &str| a.registrations.iter().any(|r| r.subject.is_some() && r.label == l);
    if !taken(base) {
        return base.into();
    }
    (2..)
        .map(|n| format!("{base} ({n})"))
        .find(|l| !taken(l))
        .expect("a free label")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_matches_rfc_7636() {
        assert_eq!(
            pkce_challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn query_strings_round_trip() {
        let pairs = [
            ("redirect_uri", "http://127.0.0.1:1455/auth/callback"),
            ("scope", "openid profile"),
            ("x", "a+b%c&d=é"),
        ];
        let q = query(&pairs);
        assert!(!q.contains(' ') && !q.contains(':'), "{q}");
        let back = parse_query(&q);
        assert_eq!(back, pairs.map(|(k, v)| (k.to_string(), v.to_string())));
        assert_eq!(
            parse_query("scope=a+b&bad=%zz&end=%4"),
            [
                ("scope".to_string(), "a b".to_string()),
                ("bad".into(), "%zz".into()),
                ("end".into(), "%4".into())
            ]
        );
    }

    #[test]
    fn host_ids_are_uuid_v4_urns() {
        let id = new_host_id();
        let u = id.strip_prefix("urn:uuid:").expect("a urn:uuid");
        let groups: Vec<&str> = u.split('-').collect();
        assert_eq!(groups.iter().map(|g| g.len()).collect::<Vec<_>>(), [8, 4, 4, 4, 12]);
        assert!(groups[2].starts_with('4'), "version 4: {id}");
        assert!("89ab".contains(&groups[3][..1]), "the RFC 4122 variant: {id}");
        assert_ne!(new_host_id(), id);
    }
}
