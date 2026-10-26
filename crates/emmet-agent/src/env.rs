// SPDX-License-Identifier: AGPL-3.0-or-later
//! The environment the CLIs get (D-37, plan section 10): the one the user's terminal would give them, unchanged. An
//! app opened from the Finder doesn't inherit the login shell's variables (PATH, ANTHROPIC_API_KEY, proxies), so on
//! macOS Emmet reads them from the login shell once; on Windows a program's environment already comes from the
//! user's registry environment, as a terminal's does.
//!
//! Nothing here adds, removes, or changes a credential variable: a variable that would pick which account pays is
//! exactly as the user's shell has it.

use std::time::Duration;

/// What separates the shell's own startup output from the environment it prints.
const MARK: &str = "\u{0}EMMET-ENV\u{0}";
/// Variables that describe the probe's shell process, not the user's environment.
const PROCESS: &[&str] = &["PWD", "OLDPWD", "SHLVL", "_"];

/// The login shell's environment, or this process's when it can't be read (or `EMMET_AGENT_ENV=inherit`, which the
/// E2E tests set so the fake agents get their settings).
pub fn login_env() -> Vec<(String, String)> {
    if cfg!(windows) || std::env::var("EMMET_AGENT_ENV").as_deref() == Ok("inherit") {
        return std::env::vars().collect();
    }
    shell_env().unwrap_or_else(|| std::env::vars().collect())
}

fn shell_env() -> Option<Vec<(String, String)>> {
    let shell = std::env::var("SHELL").ok().filter(|s| s.starts_with('/'))?;
    // -i and -l, as Terminal starts it, so .zshrc and .zprofile (or .bash_profile) both run.
    let script = format!("printf '{}'; /usr/bin/env -0", MARK.replace('\u{0}', "\\0"));
    let mut child = std::process::Command::new(&shell)
        .args(["-ilc", &script])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let mut stdout = child.stdout.take()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = std::io::Read::read_to_end(&mut stdout, &mut buf);
        let _ = tx.send(buf);
    });
    let out = match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(out) => out,
        Err(_) => {
            let _ = child.kill();
            return None;
        }
    };
    let _ = child.wait();
    parse(&String::from_utf8_lossy(&out))
}

/// The `KEY=value` pairs after the mark, NUL-separated.
fn parse(out: &str) -> Option<Vec<(String, String)>> {
    let (_, vars) = out.split_once(MARK)?;
    let env: Vec<(String, String)> = vars
        .split('\0')
        .filter_map(|kv| kv.split_once('='))
        .filter(|(k, _)| !k.is_empty() && !PROCESS.contains(k))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    (!env.is_empty()).then_some(env)
}

/// A variable's value in `env`.
pub fn get<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
    env.iter()
        .rev()
        .find(|(k, _)| {
            if cfg!(windows) {
                k.eq_ignore_ascii_case(key)
            } else {
                k == key
            }
        })
        .map(|(_, v)| v.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_noise_and_the_probes_own_variables_are_dropped() {
        let out = format!("Welcome!\n{MARK}HOME=/h\0PATH=/a:/b\0SHLVL=2\0ODD=x=y\0_=/usr/bin/env\0");
        assert_eq!(
            parse(&out).unwrap(),
            [("HOME", "/h"), ("PATH", "/a:/b"), ("ODD", "x=y")].map(|(k, v)| (k.to_string(), v.to_string()))
        );
        assert_eq!(parse("no mark here"), None);
    }
}
