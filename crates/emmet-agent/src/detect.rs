// SPDX-License-Identifier: AGPL-3.0-or-later
//! Finding the CLIs (plan section 10): on PATH and in the usual install folders, their `--version`, the version range
//! each adapter was tested with, and sign-in through the CLI's own status command. Emmet never opens token files.
//!
//! `EMMET_CLAUDE_BIN` and `EMMET_CODEX_BIN` (in the session's environment, or the app's) name the binary outright;
//! the tests point them at the fake agents.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;

use crate::{AgentKind, Billing, Detection, env};

/// The version ranges each adapter was tested with: [low, high). Outside them the chat still works, with a note.
pub const CLAUDE_TESTED: ((u64, u64, u64), (u64, u64, u64)) = ((2, 1, 310), (2, 2, 0));
pub const CODEX_TESTED: ((u64, u64, u64), (u64, u64, u64)) = ((0, 160, 0), (0, 161, 0));

fn names(kind: AgentKind) -> &'static [&'static str] {
    match (kind, cfg!(windows)) {
        (AgentKind::Claude, false) => &["claude"],
        (AgentKind::Codex, false) => &["codex"],
        (AgentKind::Claude, true) => &["claude.exe", "claude.cmd"],
        (AgentKind::Codex, true) => &["codex.exe", "codex.cmd"],
    }
}

/// The usual install folders, after PATH: the native installers, Homebrew, npm and bun globals, and on Windows npm,
/// WinGet, and Scoop.
fn usual_dirs(env: &[(String, String)]) -> Vec<PathBuf> {
    let home = env::get(env, "HOME")
        .or(env::get(env, "USERPROFILE"))
        .map(PathBuf::from);
    let mut dirs = Vec::new();
    if let Some(h) = &home {
        dirs.push(h.join(".local/bin"));
        dirs.push(h.join(".claude/local"));
        dirs.push(h.join(".npm-global/bin"));
        dirs.push(h.join(".bun/bin"));
    }
    if cfg!(windows) {
        if let Some(a) = env::get(env, "APPDATA") {
            dirs.push(Path::new(a).join("npm"));
        }
        if let Some(l) = env::get(env, "LOCALAPPDATA") {
            dirs.push(Path::new(l).join("Microsoft/WinGet/Links"));
            dirs.push(Path::new(l).join("Programs/claude"));
        }
        if let Some(h) = &home {
            dirs.push(h.join("scoop/shims"));
        }
    } else {
        dirs.push("/opt/homebrew/bin".into());
        dirs.push("/usr/local/bin".into());
    }
    dirs
}

/// The CLI's path: the override, else the first match on PATH, else in the usual folders.
pub fn find(kind: AgentKind, env: &[(String, String)]) -> Option<PathBuf> {
    let var = match kind {
        AgentKind::Claude => "EMMET_CLAUDE_BIN",
        AgentKind::Codex => "EMMET_CODEX_BIN",
    };
    if let Some(p) = env::get(env, var)
        .map(PathBuf::from)
        .or(std::env::var_os(var).map(PathBuf::from))
    {
        return Some(p);
    }
    let path = env::get(env, "PATH").unwrap_or_default();
    std::env::split_paths(path)
        .chain(usual_dirs(env))
        .flat_map(|d| names(kind).iter().map(move |n| d.join(n)))
        .find(|p| p.is_file())
}

/// The command that starts the CLI at `bin`. An npm install on Windows leaves a `.cmd` shim, and a batch file can't be
/// given arguments like Codex's `-c key="value"` safely, so Emmet runs the package's own launcher with node instead.
pub fn command(bin: &Path) -> Command {
    let shim = bin
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"));
    let launcher = match bin
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("codex") => "node_modules/@openai/codex/bin/codex.js",
        Some("claude") => "node_modules/@anthropic-ai/claude-code/cli.js",
        _ => "",
    };
    if shim
        && !launcher.is_empty()
        && let Some(js) = bin.parent().map(|d| d.join(launcher)).filter(|js| js.is_file())
    {
        let mut cmd = Command::new("node");
        cmd.arg(js);
        return cmd;
    }
    Command::new(bin)
}

/// Runs the CLI with `args` and the user's environment, and returns its stdout, or None if it fails or takes more
/// than 15 s.
pub(crate) fn output(bin: &Path, args: &[&str], env: &[(String, String)]) -> Option<String> {
    let mut cmd = command(bin);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    crate::session::set_env(&mut cmd, env);
    crate::session::no_window(&mut cmd);
    let mut child = cmd.spawn().ok()?;
    let mut stdout = child.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::Read::read_to_string(&mut stdout, &mut s);
        let _ = tx.send(s);
    });
    match rx.recv_timeout(Duration::from_secs(15)) {
        Ok(s) => {
            let ok = child.wait().is_ok_and(|s| s.success());
            ok.then_some(s)
        }
        Err(_) => {
            let _ = child.kill();
            None
        }
    }
}

/// The first `x.y.z` in `--version`'s output ("2.1.310 (Claude Code)", "codex-cli 0.160.1").
pub fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    text.split(|c: char| !(c.is_ascii_digit() || c == '.')).find_map(|w| {
        let mut parts = w.split('.').map(|p| p.parse::<u64>().ok());
        match (parts.next(), parts.next(), parts.next()) {
            (Some(Some(a)), Some(Some(b)), Some(Some(c))) => Some((a, b, c)),
            _ => None,
        }
    })
}

fn in_range(v: (u64, u64, u64), range: ((u64, u64, u64), (u64, u64, u64))) -> bool {
    v >= range.0 && v < range.1
}

/// Detection for one CLI. `has_key` says whether the keychain holds Codex's API key.
pub fn detect(kind: AgentKind, env: &[(String, String)], has_key: bool) -> Detection {
    let mut d = Detection {
        kind,
        path: find(kind, env),
        version: None,
        supported: false,
        signed_in: false,
        billing: Billing::Unknown,
        fix: None,
        note: None,
    };
    let Some(bin) = d.path.clone() else {
        d.fix = Some(match kind {
            AgentKind::Claude => "Install Claude Code: https://code.claude.com/docs/en/setup".into(),
            AgentKind::Codex => "Install Codex: npm install -g @openai/codex".into(),
        });
        return d;
    };
    let version = output(&bin, &["--version"], env).and_then(|v| parse_version(&v));
    d.version = version.map(|(a, b, c)| format!("{a}.{b}.{c}"));
    let range = match kind {
        AgentKind::Claude => CLAUDE_TESTED,
        AgentKind::Codex => CODEX_TESTED,
    };
    d.supported = version.is_some_and(|v| in_range(v, range));
    if !d.supported {
        d.note = Some(format!(
            "untested {} {}",
            kind.label(),
            d.version.as_deref().unwrap_or("version")
        ));
    }
    match kind {
        AgentKind::Claude => {
            let status = output(&bin, &["auth", "status", "--json"], env)
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_default();
            let (signed_in, billing) = claude_auth(&status);
            d.signed_in = signed_in;
            d.billing = billing;
            if !signed_in {
                d.fix = Some("In your terminal, run claude, then /login.".into());
            }
        }
        AgentKind::Codex => {
            // Emmet runs Codex only on the user's API key, through its own provider block, never on the CLI's own
            // login (D-35); Sign in with ChatGPT comes in 1E.
            d.signed_in = has_key;
            d.billing = Billing::ApiKey {
                source: Some("keychain".into()),
            };
            if !has_key {
                d.fix = Some("Add your OpenAI API key in Settings → Agents.".into());
            }
        }
    }
    d
}

/// Sign-in and billing from `claude auth status --json`: `loggedIn`, and `authMethod` "claude.ai" with
/// `subscriptionType` on a plan, or `api_key` with `apiKeySource` on a key (ADR 0001).
pub fn claude_auth(status: &Value) -> (bool, Billing) {
    let method = status["authMethod"].as_str().unwrap_or("");
    let signed_in = status["loggedIn"]
        .as_bool()
        .unwrap_or(!method.is_empty() && method != "none");
    let source = status["apiKeySource"].as_str().filter(|s| *s != "none");
    let billing = if method == "api_key" || source.is_some() {
        Billing::ApiKey {
            source: source.map(String::from),
        }
    } else if signed_in {
        Billing::Plan {
            name: status["subscriptionType"].as_str().map(String::from),
        }
    } else {
        Billing::Unknown
    };
    (signed_in, billing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_npm_shim_runs_its_launcher_with_node() {
        let dir = std::env::temp_dir().join(format!("emmet-shim-{}", std::process::id()));
        let js = dir.join("node_modules/@openai/codex/bin/codex.js");
        std::fs::create_dir_all(js.parent().unwrap()).unwrap();
        std::fs::write(&js, "").unwrap();
        std::fs::write(dir.join("codex.cmd"), "").unwrap();
        let cmd = command(&dir.join("codex.cmd"));
        assert_eq!(cmd.get_program(), "node");
        assert_eq!(cmd.get_args().collect::<Vec<_>>(), [js.as_os_str()]);
        // A real program, or a shim whose package isn't there, runs as itself.
        assert_eq!(
            command(&dir.join("codex.exe")).get_program(),
            dir.join("codex.exe").as_os_str()
        );
        assert_eq!(
            command(&dir.join("claude.cmd")).get_program(),
            dir.join("claude.cmd").as_os_str()
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
