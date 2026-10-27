// SPDX-License-Identifier: AGPL-3.0-or-later
//! Agent setup without a terminal (D-41, plan section 10, ADR 0012): the providers' own installers and Claude Code's
//! own sign-in, run from Rust only after the user clicks, each unmodified, plus Antigravity's terminal mode (D-42).
//!
//! - **Install** runs the command the provider documents, word for word, in a shell Emmet starts: Anthropic's native
//!   installer for Claude Code, OpenAI's standalone installer for Codex (with its documented `CODEX_NON_INTERACTIVE`,
//!   since nobody can answer its prompts). Both check their downloads' SHA-256 themselves and install for this user
//!   only. Anthropic also publishes who signs Claude Code, so Emmet checks that signature with the OS's own tool
//!   before it reports success. Cancel ends the installer and everything it started.
//! - **Sign-in** runs `claude auth login`, which opens the browser itself and finishes through its `localhost`
//!   callback. Its stdin is closed and its output is read only to tell how it ended: Emmet never sees, keeps, or
//!   passes on a code or a token. If the browser shows a code to paste, the user signs in from Terminal instead
//!   (Barry 2026-10-26).
//! - **Antigravity** has no command that adds an MCP server, so Emmet writes the project's `.agents/mcp_config.json`
//!   on a click (Barry 2026-10-26). It never writes `~/.gemini`.
//!
//! Whatever the installers and the CLIs write is theirs (D-41); Emmet's own code here writes only the Antigravity
//! project file and `setup.json` in its own app data.
//!
//! Tests: in a debug build, `EMMET_FAKE_INSTALL=<bin>` runs `<bin> claude|codex` in place of the installer, and
//! `EMMET_FAKE_SIGNATURE=ok` accepts an unsigned Claude Code (the fakes aren't signed).

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{AgentKind, Detection, detect, env};

/// The longest an installer may run before Emmet stops it.
pub const INSTALL_LIMIT: Duration = Duration::from_secs(15 * 60);
/// The longest Emmet waits for a sign-in in the browser.
pub const SIGN_IN_LIMIT: Duration = Duration::from_secs(10 * 60);

/// Each provider's page that says which plans include its CLI (docs/terms/quotes.json, checked 2026-10-26).
pub const CLAUDE_PLANS: &str = "https://code.claude.com/docs/en/setup#authenticate";
pub const CODEX_PLANS: &str = "https://learn.chatgpt.com/docs/pricing";

/// Anthropic's Developer ID team (code.claude.com/docs/en/setup: macOS builds are "signed by \"Anthropic PBC\" and
/// notarized by Apple").
const ANTHROPIC_TEAM: &str = "Q6L2SF6YDW";
/// The Windows signer the same page names: "signed by \"Anthropic, PBC\"".
const ANTHROPIC_SIGNER: &str = "CN=\"Anthropic, PBC\"";

/// What went wrong, for the UI's one retry and its words.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum Failure {
    /// The provider's servers couldn't be reached.
    Offline,
    /// The installer's own checksum check failed.
    Checksum,
    /// The installed binary isn't signed by its publisher.
    Signature,
    Canceled,
    TimedOut,
    /// An install or sign-in for this agent is already running.
    Busy,
    Failed,
}

/// A failed install or sign-in, in plain words; `detail` is the tool's own last line, when it said something useful.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct SetupError {
    pub failure: Failure,
    pub message: String,
    pub detail: Option<String>,
}

impl SetupError {
    fn new(failure: Failure, message: impl Into<String>, detail: Option<String>) -> Self {
        Self {
            failure,
            message: message.into(),
            detail,
        }
    }
}

impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// Stops a running install or sign-in, with everything it started. Cloned into the UI's cancel command.
#[derive(Clone, Default)]
pub struct Cancel(Arc<CancelInner>);

#[derive(Default)]
struct CancelInner {
    canceled: AtomicBool,
    pid: Mutex<Option<u32>>,
}

impl Cancel {
    pub fn cancel(&self) {
        self.0.canceled.store(true, Ordering::SeqCst);
        if let Some(pid) = *self.0.pid.lock().unwrap_or_else(|e| e.into_inner()) {
            kill_tree(pid);
        }
    }

    pub fn canceled(&self) -> bool {
        self.0.canceled.load(Ordering::SeqCst)
    }

    /// Whether `other` is this same handle, not just one like it.
    pub fn same(&self, other: &Cancel) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    fn running(&self, pid: Option<u32>) {
        *self.0.pid.lock().unwrap_or_else(|e| e.into_inner()) = pid;
        if pid.is_some() && self.canceled() {
            self.cancel();
        }
    }
}

/// Ends `pid` and its descendants. The child leads its own process group on Unix; on Windows, taskkill follows the
/// tree.
fn kill_tree(pid: u32) {
    if cfg!(windows) {
        let mut cmd = Command::new(system32("taskkill.exe"));
        cmd.args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        crate::session::no_window(&mut cmd);
        let _ = cmd.status();
    } else {
        let group = format!("-{pid}");
        let _ = Command::new("/bin/kill").args(["-TERM", "--", &group]).status();
        // Whatever ignores TERM gets KILL a moment later.
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(2));
            let _ = Command::new("/bin/kill")
                .args(["-KILL", "--", &group])
                .stderr(Stdio::null())
                .status();
        });
    }
}

/// A Windows system tool by full path, so PATH can't put another one first.
fn system32(exe: &str) -> PathBuf {
    let root = std::env::var_os("SystemRoot").unwrap_or_else(|| "C:\\Windows".into());
    Path::new(&root).join("System32").join(exe)
}

fn powershell() -> PathBuf {
    system32("WindowsPowerShell\\v1.0\\powershell.exe")
}

/// How a job ended: its exit code (None when a signal ended it, on Unix) and every line it printed, stdout and stderr
/// together.
struct Ended {
    code: Option<i32>,
    lines: Vec<String>,
    timed_out: bool,
}

/// How long to wait for the last output once the program has exited (or was ended): a helper it started may hold its
/// output open, and that must not keep the card waiting.
const DRAIN: Duration = Duration::from_secs(2);

/// Runs `cmd` to the end, in its own process group, passing each line it prints to `on_line`; past `limit`, or on
/// cancel, it's ended with everything it started.
fn run(mut cmd: Command, cancel: &Cancel, limit: Duration, on_line: &mut dyn FnMut(&str)) -> std::io::Result<Ended> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    crate::session::no_window(&mut cmd);
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    let mut child = cmd.spawn()?;
    cancel.running(Some(child.id()));
    let (tx, rx) = channel::<String>();
    for stream in [
        child.stdout.take().map(|s| Box::new(s) as Box<dyn Read + Send>),
        child.stderr.take().map(|s| Box::new(s) as Box<dyn Read + Send>),
    ]
    .into_iter()
    .flatten()
    {
        let tx = tx.clone();
        // Not joined: a reader whose pipe a stray helper holds open ends when that helper does.
        std::thread::spawn(move || {
            for line in BufReader::new(stream).split(b'\n').map_while(|l| l.ok()) {
                let line = String::from_utf8_lossy(&line).trim_end_matches('\r').to_string();
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
    }
    drop(tx);
    let deadline = Instant::now() + limit;
    let mut lines = Vec::new();
    let (mut timed_out, mut ended_at, mut status) = (false, None::<Instant>, None);
    loop {
        match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                on_line(&line);
                lines.push(line);
                continue;
            }
            Err(RecvTimeoutError::Disconnected) if status.is_some() => break,
            // Its output is closed but it's still running: check on it now and then, without spinning.
            Err(RecvTimeoutError::Disconnected) => std::thread::sleep(Duration::from_millis(100)),
            Err(RecvTimeoutError::Timeout) => {}
        }
        if status.is_none()
            && let Some(s) = child.try_wait()?
        {
            status = Some(s);
            ended_at.get_or_insert_with(Instant::now);
        }
        if ended_at.is_none() && (cancel.canceled() || Instant::now() >= deadline) {
            timed_out = !cancel.canceled();
            kill_tree(child.id());
            ended_at = Some(Instant::now());
        }
        if ended_at.is_some_and(|t| t.elapsed() >= DRAIN) {
            break;
        }
    }
    let status = match status {
        Some(s) => s,
        None => {
            // Still running after the kill and the wait: end it directly.
            let _ = child.kill();
            child.wait()?
        }
    };
    cancel.running(None);
    Ok(Ended {
        code: status.code(),
        lines,
        timed_out,
    })
}

/// The provider's documented install command (code.claude.com/docs/en/setup, github.com/openai/codex): in bash with
/// `pipefail` on macOS, so a failed download is a failed install, or in PowerShell by full path on Windows.
fn installer(kind: AgentKind, env: &[(String, String)]) -> Command {
    if cfg!(debug_assertions)
        && let Some(fake) = env::get(env, "EMMET_FAKE_INSTALL")
    {
        let mut cmd = Command::new(fake);
        cmd.arg(match kind {
            AgentKind::Claude => "claude",
            AgentKind::Codex => "codex",
        });
        crate::session::set_env(&mut cmd, env);
        return cmd;
    }
    let documented = match (kind, cfg!(windows)) {
        (AgentKind::Claude, false) => "curl -fsSL https://claude.ai/install.sh | bash",
        (AgentKind::Codex, false) => "curl -fsSL https://chatgpt.com/codex/install.sh | sh",
        (AgentKind::Claude, true) => "irm https://claude.ai/install.ps1 | iex",
        (AgentKind::Codex, true) => "irm https://chatgpt.com/codex/install.ps1 | iex",
    };
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new(powershell());
        cmd.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", documented]);
        cmd
    } else {
        let mut cmd = Command::new("/bin/bash");
        cmd.args(["-c", &format!("set -o pipefail; {documented}")]);
        cmd
    };
    crate::session::set_env(&mut cmd, env);
    if kind == AgentKind::Codex {
        // "Set to 1, true, or yes to skip prompts" (its --help): Emmet's window can't answer them.
        cmd.env("CODEX_NON_INTERACTIVE", "1");
    }
    cmd
}

fn provider(kind: AgentKind) -> &'static str {
    match kind {
        AgentKind::Claude => "Anthropic",
        AgentKind::Codex => "OpenAI",
    }
}

/// What the installer's own words say went wrong, from curl's, PowerShell's, and both installers' messages.
fn classify(lines: &[String]) -> Option<Failure> {
    let text = lines.join("\n").to_lowercase();
    let any = |words: &[&str]| words.iter().any(|w| text.contains(w));
    if any(&[
        "checksum verification failed",
        "checksum did not match",
        "did not match expected digest",
    ]) {
        Some(Failure::Checksum)
    } else if any(&[
        "could not resolve host",
        "couldn't resolve host",
        "failed to connect",
        "could not connect",
        "connection refused",
        "connection timed out",
        "network is unreachable",
        "no such host is known",
        "remote name could not be resolved",
        "unable to connect to the remote server",
        "failed to get latest version",
        "failed to get a valid version",
        "download failed",
    ]) {
        Some(Failure::Offline)
    } else {
        None
    }
}

/// The installer's last line worth showing: not blank, not a progress bar.
fn last_line(lines: &[String]) -> Option<String> {
    lines
        .iter()
        .rev()
        .map(|l| l.trim())
        .find(|l| !l.is_empty() && l.chars().any(char::is_alphabetic))
        .map(|l| l.chars().take(300).collect())
}

/// Installs `kind` with its provider's official installer, unless it's already there. `env` is the user's own
/// environment ([`env::login_env`]); `on_line` sees each line the installer prints. Returns the new detection.
pub fn install(
    kind: AgentKind,
    env: &[(String, String)],
    has_key: bool,
    cancel: &Cancel,
    on_line: &mut dyn FnMut(&str),
) -> Result<Detection, SetupError> {
    let before = detect::detect(kind, env, has_key);
    if let Some(bin) = &before.path {
        // Try again after a signature failure lands here, with the refused copy still in place.
        if kind == AgentKind::Claude {
            verify_signature(bin, env)?;
        }
        return Ok(before);
    }
    let who = provider(kind);
    let name = kind.label();
    let ended = run(installer(kind, env), cancel, INSTALL_LIMIT, on_line).map_err(|e| {
        SetupError::new(
            Failure::Failed,
            format!("Emmet couldn't start {who}'s installer: {e}."),
            None,
        )
    })?;
    let detail = last_line(&ended.lines);
    // A cancel that came after the installer finished doesn't undo it.
    if cancel.canceled() && ended.code != Some(0) {
        return Err(SetupError::new(
            Failure::Canceled,
            format!("Install canceled. {name} isn't set up; install it again whenever you like."),
            None,
        ));
    }
    if ended.timed_out {
        return Err(SetupError::new(
            Failure::TimedOut,
            "The installer was still running after 15 minutes, so Emmet stopped it. Check that you're online, then \
             try again.",
            detail,
        ));
    }
    // Only a failed installer counts its own words: OpenAI's falls back to GitHub after a bad download, and succeeds.
    match (ended.code, classify(&ended.lines)) {
        (Some(c), Some(Failure::Checksum)) if c != 0 => {
            return Err(SetupError::new(
                Failure::Checksum,
                format!(
                    "The download didn't match the checksum {who} publishes, so the installer stopped and installed \
                     nothing. Try again; if it happens again, something on your network is changing downloads."
                ),
                detail,
            ));
        }
        (Some(c), Some(Failure::Offline)) if c != 0 => {
            return Err(SetupError::new(
                Failure::Offline,
                format!("Emmet couldn't reach {who}'s download server. Check that you're online, then try again."),
                detail,
            ));
        }
        (Some(0), _) => {}
        (code, _) => {
            return Err(SetupError::new(
                Failure::Failed,
                format!(
                    "{who}'s installer stopped with an error{}. Try again.",
                    code.map(|c| format!(" (exit code {c})")).unwrap_or_default()
                ),
                detail,
            ));
        }
    }
    let after = detect::detect(kind, env, has_key);
    let Some(bin) = after.path.clone() else {
        return Err(SetupError::new(
            Failure::Failed,
            format!("The installer finished, but Emmet can't find {name} where it installs. Try again."),
            detail,
        ));
    };
    if kind == AgentKind::Claude {
        verify_signature(&bin, env)?;
    }
    Ok(after)
}

/// Checks that Claude Code at `bin` is signed by Anthropic, as its setup page says it is: Developer ID team
/// Q6L2SF6YDW on macOS (codesign), "Anthropic, PBC" on Windows (Get-AuthenticodeSignature).
pub fn verify_signature(bin: &Path, env: &[(String, String)]) -> Result<(), SetupError> {
    if cfg!(debug_assertions) && env::get(env, "EMMET_FAKE_SIGNATURE") == Some("ok") {
        return Ok(());
    }
    let real = std::fs::canonicalize(bin).unwrap_or_else(|_| bin.to_path_buf());
    // Whether it's signed by Anthropic, or why Emmet couldn't tell.
    let checked: Result<bool, String> = if cfg!(target_os = "macos") {
        Command::new("/usr/bin/codesign")
            .args(["-d", "--verbose=2"])
            .arg(&real)
            .output()
            .map_err(|e| format!("codesign: {e}"))
            .and_then(|team| {
                let strict = Command::new("/usr/bin/codesign")
                    .args(["--verify", "--strict"])
                    .arg(&real)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map_err(|e| format!("codesign: {e}"))?;
                Ok(strict.success()
                    && team_id(&String::from_utf8_lossy(&team.stderr)).as_deref() == Some(ANTHROPIC_TEAM))
            })
    } else if cfg!(windows) {
        let mut cmd = Command::new(powershell());
        cmd.args([
            "-NoProfile",
            "-Command",
            "$s = Get-AuthenticodeSignature -LiteralPath $env:EMMET_SIGNED; \"$($s.Status)|$($s.SignerCertificate.Subject)\"",
        ])
        .env("EMMET_SIGNED", &real)
        .stdin(Stdio::null());
        crate::session::no_window(&mut cmd);
        match cmd.output() {
            Ok(o) if String::from_utf8_lossy(&o.stdout).contains('|') => {
                Ok(authenticode_ok(&String::from_utf8_lossy(&o.stdout)))
            }
            // PowerShell blocked (AppLocker, Constrained Language Mode) says nothing about the signature.
            Ok(o) => Err(last_line(&[String::from_utf8_lossy(&o.stderr).to_string()])
                .unwrap_or_else(|| "PowerShell gave no answer".into())),
            Err(e) => Err(format!("PowerShell: {e}")),
        }
    } else {
        Ok(true)
    };
    match checked {
        Ok(true) => Ok(()),
        Ok(false) => Err(SetupError::new(
            Failure::Signature,
            "Claude Code is installed, but the copy Emmet found isn't signed by Anthropic. Don't use it: delete it, \
             then install again.",
            Some(real.display().to_string()),
        )),
        Err(why) => Err(SetupError::new(
            Failure::Failed,
            "Emmet couldn't check Claude Code's signature, so it can't tell whether this copy is Anthropic's. Try \
             again.",
            Some(why),
        )),
    }
}

/// `TeamIdentifier=…` from `codesign -d --verbose=2`'s output.
fn team_id(codesign: &str) -> Option<String> {
    codesign
        .lines()
        .find_map(|l| l.strip_prefix("TeamIdentifier="))
        .map(|t| t.trim().to_string())
        .filter(|t| t != "not set")
}

/// `Valid|<subject>` from Get-AuthenticodeSignature, with Anthropic's name as the subject's common name, which leads
/// the subject (the name itself has a comma in it).
fn authenticode_ok(out: &str) -> bool {
    let line = out.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("");
    let Some((status, subject)) = line.split_once('|') else {
        return false;
    };
    status == "Valid"
        && subject
            .strip_prefix(ANTHROPIC_SIGNER)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with(", "))
}

/// Signs in to Claude Code with its own `claude auth login` (`--console` for a Claude Console account instead of a
/// Claude plan). The CLI opens the browser and waits for its `localhost` callback; Emmet waits for it to exit. Its
/// output isn't streamed to the window, since one line carries the sign-in link; on a failure, Claude Code's last
/// line without a link becomes the detail.
pub fn sign_in(env: &[(String, String)], console: bool, cancel: &Cancel) -> Result<Detection, SetupError> {
    let Some(bin) = detect::find(AgentKind::Claude, env) else {
        return Err(SetupError::new(
            Failure::Failed,
            "Claude Code isn't installed yet. Install it first.",
            None,
        ));
    };
    let mut cmd = detect::command(&bin);
    cmd.args(["auth", "login"]);
    if console {
        cmd.arg("--console");
    }
    crate::session::set_env(&mut cmd, env);
    let ended = run(cmd, cancel, SIGN_IN_LIMIT, &mut |_| {}).map_err(|e| {
        SetupError::new(
            Failure::Failed,
            format!("Emmet couldn't start Claude Code's sign-in: {e}."),
            None,
        )
    })?;
    if cancel.canceled() && ended.code != Some(0) {
        return Err(SetupError::new(Failure::Canceled, "Sign-in canceled.", None));
    }
    if ended.timed_out {
        return Err(SetupError::new(
            Failure::TimedOut,
            "Emmet waited 10 minutes and the sign-in didn't finish, so it stopped waiting. Try again.",
            None,
        ));
    }
    let after = detect::detect(AgentKind::Claude, env, false);
    if ended.code == Some(0) && after.signed_in {
        return Ok(after);
    }
    if ended.code == Some(0) {
        return Err(SetupError::new(
            Failure::Failed,
            "Claude Code finished its sign-in, but `claude auth status` doesn't show you signed in. Try again.",
            sign_in_detail(&ended.lines),
        ));
    }
    Err(SetupError::new(
        Failure::Failed,
        "Claude Code's sign-in didn't finish. Try again.",
        sign_in_detail(&ended.lines),
    ))
}

/// Claude Code's own words for why sign-in ended, never a line with a URL in it (the sign-in link).
fn sign_in_detail(lines: &[String]) -> Option<String> {
    // The paste prompt has no newline, so what Claude Code says next shares its line.
    let said: Vec<String> = lines
        .iter()
        .map(|l| l.rsplit_once("if prompted >").map_or(l.as_str(), |(_, rest)| rest))
        .filter(|l| !l.contains("://"))
        .map(String::from)
        .collect();
    last_line(&said).filter(|l| !l.starts_with("Opening browser"))
}

/// What Emmet found of the Antigravity CLI (D-42): it has no status command, so sign-in is agy's own first-run step.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Antigravity {
    pub path: Option<PathBuf>,
    pub version: Option<String>,
}

/// Finds `agy`: `EMMET_AGY_BIN`, PATH, then where Google's installer puts it (`~/.local/bin`, and on Windows
/// `%LOCALAPPDATA%\agy\bin`).
pub fn antigravity(env: &[(String, String)]) -> Antigravity {
    let path = detect::find_named(
        "EMMET_AGY_BIN",
        if cfg!(windows) { &["agy.exe"] } else { &["agy"] },
        env,
    );
    let version = path
        .as_ref()
        .and_then(|p| detect::output(p, &["--version"], env))
        .and_then(|v| detect::parse_version(&v))
        .map(|(a, b, c)| format!("{a}.{b}.{c}"));
    Antigravity { path, version }
}

/// The project file Antigravity reads its workspace MCP servers from (antigravity.google/docs/mcp).
pub fn antigravity_config(project: &Path) -> PathBuf {
    project.join(".agents").join("mcp_config.json")
}

/// Adds Emmet's MCP server to the project's `.agents/mcp_config.json` as `emmet`, keeping every other server and
/// setting the file has. A file that isn't a JSON object is left alone, and so is a `.agents` or a file that's a link
/// or leads outside the project: a project from elsewhere could point it at `~/.gemini`.
pub fn write_antigravity_config(project: &Path, emmet: &Path) -> Result<PathBuf, SetupError> {
    let path = antigravity_config(project);
    if antigravity_linked(project) {
        return Err(SetupError::new(
            Failure::Failed,
            "The project's .agents folder or its mcp_config.json is a link, so Emmet left it alone. Replace it with a \
             plain folder, then try again.",
            None,
        ));
    }
    let mut doc = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice::<Value>(&bytes)
            .ok()
            .filter(Value::is_object)
            .ok_or_else(|| {
                SetupError::new(
                    Failure::Failed,
                    "The project's .agents/mcp_config.json isn't a JSON object, so Emmet left it alone. Fix or remove \
                     it, then try again.",
                    None,
                )
            })?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => json!({}),
        Err(e) => {
            return Err(SetupError::new(
                Failure::Failed,
                format!("Emmet couldn't read .agents/mcp_config.json: {e}."),
                None,
            ));
        }
    };
    if !doc["mcpServers"].is_object() {
        doc["mcpServers"] = json!({});
    }
    doc["mcpServers"]["emmet"] = antigravity_entry(project, emmet);
    let mut bytes = serde_json::to_vec_pretty(&doc).expect("json");
    bytes.push(b'\n');
    emmet_core::fsx::write_atomic(&path, &bytes).map_err(|e| {
        SetupError::new(
            Failure::Failed,
            format!("Emmet couldn't write {}: {}.", path.display(), e.message),
            None,
        )
    })?;
    Ok(path)
}

/// Emmet's entry in `mcpServers`: the sidecar, with the project named outright.
fn antigravity_entry(project: &Path, emmet: &Path) -> Value {
    json!({"command": emmet, "args": ["mcp", "--project", project]})
}

/// Whether `.agents` or its `mcp_config.json` is a link, or `.agents` leads outside the project.
fn antigravity_linked(project: &Path) -> bool {
    let dir = project.join(".agents");
    let linked = |p: &Path| std::fs::symlink_metadata(p).is_ok_and(|m| m.file_type().is_symlink());
    let outside = || {
        let root = std::fs::canonicalize(project).ok();
        dir.exists() && std::fs::canonicalize(&dir).ok() != root.map(|r| r.join(".agents"))
    };
    linked(&dir) || linked(&antigravity_config(project)) || outside()
}

/// Whether the project's file already has exactly the entry [`write_antigravity_config`] would write: a moved project
/// or a moved Emmet needs it written again.
pub fn antigravity_configured(project: &Path, emmet: &Path) -> bool {
    !antigravity_linked(project)
        && std::fs::read(antigravity_config(project))
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .is_some_and(|d| d["mcpServers"]["emmet"] == antigravity_entry(project, emmet))
}

/// Emmet's own setup record in app data: whether the user accepted the provider notice (D-37).
#[derive(Debug, Default, Serialize, Deserialize)]
struct Record {
    #[serde(default)]
    provider_notice: bool,
}

fn record_path(data: &Path) -> PathBuf {
    data.join("setup.json")
}

/// Whether the user has accepted the provider notice (D-37); the window asks at each click until then.
pub fn notice_seen(data: &Path) -> bool {
    std::fs::read(record_path(data))
        .ok()
        .and_then(|b| serde_json::from_slice::<Record>(&b).ok())
        .is_some_and(|r| r.provider_notice)
}

pub fn see_notice(data: &Path) -> Result<(), SetupError> {
    let bytes = serde_json::to_vec(&Record { provider_notice: true }).expect("json");
    emmet_core::fsx::write_atomic(&record_path(data), &bytes)
        .map_err(|e| SetupError::new(Failure::Failed, e.message, None))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(String::from).collect()
    }

    #[test]
    fn installer_output_names_what_went_wrong() {
        let cases = [
            ("curl: (6) Could not resolve host: claude.ai", Some(Failure::Offline)),
            (
                "Failed to get latest version: No such host is known.",
                Some(Failure::Offline),
            ),
            (
                "Invoke-RestMethod : The remote name could not be resolved: 'chatgpt.com'",
                Some(Failure::Offline),
            ),
            ("Checksum verification failed", Some(Failure::Checksum)),
            (
                "Downloaded Codex archive checksum did not match expected digest.",
                Some(Failure::Checksum),
            ),
            ("Setting up Claude Code...\nError: EACCES", None),
        ];
        for (out, want) in cases {
            assert_eq!(classify(&lines(out)), want, "{out}");
        }
    }

    #[test]
    fn only_anthropics_signature_passes() {
        let mac = "Executable=/x\nIdentifier=com.anthropic.claude-code\nTeamIdentifier=Q6L2SF6YDW\n";
        assert_eq!(team_id(mac).as_deref(), Some(ANTHROPIC_TEAM));
        assert_eq!(team_id("Identifier=claude\nTeamIdentifier=not set\n"), None);
        assert!(authenticode_ok(
            "Valid|CN=\"Anthropic, PBC\", O=\"Anthropic, PBC\", L=San Francisco, S=California, C=US\r\n"
        ));
        assert!(!authenticode_ok("NotSigned|"));
        assert!(!authenticode_ok(
            "Valid|CN=\"Anthropic, PBC Tools\", O=\"Someone Else\""
        ));
        assert!(!authenticode_ok("HashMismatch|CN=\"Anthropic, PBC\""));
    }

    #[test]
    fn the_sign_in_link_never_reaches_the_detail() {
        let out = lines(
            "Opening browser to sign in…\nIf the browser didn't open, visit: https://claude.com/cai/oauth/authorize?state=s\nPaste code here if prompted > Login failed: access denied",
        );
        assert_eq!(sign_in_detail(&out).as_deref(), Some("Login failed: access denied"));
        assert_eq!(sign_in_detail(&out[..2]), None);
        let out = lines("Opening browser to sign in…\nOAuth error: the request was denied");
        assert_eq!(
            sign_in_detail(&out).as_deref(),
            Some("OAuth error: the request was denied")
        );
    }
}
