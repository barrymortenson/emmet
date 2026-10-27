// SPDX-License-Identifier: AGPL-3.0-or-later
//! fake-install: stands in for Anthropic's and OpenAI's installers (`EMMET_FAKE_INSTALL`, ADR 0012). `fake-install
//! claude|codex` reads `install-<cli>.mode` in `EMMET_FAKE_DIR` (default `ok`), notes the attempt in
//! `install-<cli>.log`, and does what the real installer does in that case, with its words:
//!
//! - `ok`: installs fake-claude or fake-codex where the real installer puts the CLI, under the test's HOME (or
//!   LOCALAPPDATA for Codex on Windows), or at `EMMET_CLAUDE_BIN` / `EMMET_CODEX_BIN` when the test names the path.
//! - `offline`: the download fails as curl or PowerShell reports it.
//! - `checksum`: the installer's own SHA-256 check fails.
//! - `fail`: some other error.
//! - `slow`: a child process does the install three seconds later, so canceling must end the whole tree. The child
//!   writes `install-<cli>.started` first, so a test can cancel once it's running.
//! - `elsewhere`: succeeds, but leaves the CLI in a folder detection doesn't search.
//! - `linger`: installs and exits, leaving a helper of its own (in another process group) holding its output open for
//!   20 s, so a caller must go by the installer's exit, not by its output closing.

use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = args.first().map(String::as_str).unwrap_or("");
    if !matches!(cli, "claude" | "codex") {
        eprintln!("fake-install: usage: fake-install claude|codex");
        std::process::exit(2);
    }
    let dir = PathBuf::from(std::env::var_os("EMMET_FAKE_DIR").unwrap_or_default());
    match args.get(1).map(String::as_str) {
        Some("--late") => {
            let _ = std::fs::write(dir.join(format!("install-{cli}.started")), "");
            std::thread::sleep(Duration::from_secs(3));
            install(cli);
            return;
        }
        Some("--hold") => {
            std::thread::sleep(Duration::from_secs(20));
            return;
        }
        _ => {}
    }
    let mode = std::fs::read_to_string(dir.join(format!("install-{cli}.mode")))
        .map(|m| m.trim().to_string())
        .unwrap_or_else(|_| "ok".into());
    if let Ok(mut log) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(format!("install-{cli}.log")))
    {
        let _ = writeln!(log, "{mode}");
    }
    match mode.as_str() {
        "ok" => {
            println!("Setting up {cli}...");
            install(cli);
            println!("Installation complete!");
        }
        "offline" if cfg!(windows) => {
            eprintln!("Failed to get latest version: No such host is known. (downloads.claude.ai:443)");
            std::process::exit(1);
        }
        "offline" => {
            eprintln!("curl: (6) Could not resolve host: claude.ai");
            std::process::exit(6);
        }
        "checksum" if cli == "claude" => {
            eprintln!("Checksum verification failed");
            std::process::exit(1);
        }
        "checksum" => {
            eprintln!("Downloaded Codex archive checksum did not match expected digest.");
            std::process::exit(1);
        }
        "elsewhere" => {
            println!("Setting up {cli}...");
            let home = std::env::var_os("HOME")
                .or(std::env::var_os("USERPROFILE"))
                .expect("HOME");
            let to = PathBuf::from(home).join("somewhere-else");
            std::fs::create_dir_all(&to).expect("folder");
            let exe = std::env::consts::EXE_SUFFIX;
            let me = std::env::current_exe().expect("own path");
            std::fs::copy(
                me.with_file_name(format!("fake-{cli}{exe}")),
                to.join(format!("{cli}{exe}")),
            )
            .expect("copy");
            println!("Installation complete!");
        }
        "linger" => {
            println!("Setting up {cli}...");
            install(cli);
            let mut helper = std::process::Command::new(std::env::current_exe().expect("own path"));
            helper.args([cli, "--hold"]);
            #[cfg(unix)]
            std::os::unix::process::CommandExt::process_group(&mut helper, 0);
            // Never waited on: the helper is meant to outlive the installer, holding its output.
            #[allow(clippy::zombie_processes)]
            helper.spawn().expect("start the helper");
            println!("Installation complete!");
        }
        "slow" => {
            println!("Downloading {cli}...");
            let me = std::env::current_exe().expect("own path");
            let status = std::process::Command::new(me)
                .args([cli, "--late"])
                .status()
                .expect("start the download");
            std::process::exit(status.code().unwrap_or(1));
        }
        _ => {
            eprintln!("Error: the installer hit something unexpected ({mode})");
            std::process::exit(3);
        }
    }
}

/// Copies the fake CLI next to this binary to where the real installer leaves the CLI.
fn install(cli: &str) {
    let exe = std::env::consts::EXE_SUFFIX;
    let me = std::env::current_exe().expect("own path");
    let from = me.with_file_name(format!("fake-{cli}{exe}"));
    let var = |k: &str| std::env::var_os(k).map(PathBuf::from);
    let home = var("HOME").or(var("USERPROFILE")).expect("HOME");
    let named = var(if cli == "claude" {
        "EMMET_CLAUDE_BIN"
    } else {
        "EMMET_CODEX_BIN"
    });
    let to = if let Some(p) = named {
        p
    } else if cli == "codex" && cfg!(windows) {
        var("LOCALAPPDATA")
            .expect("LOCALAPPDATA")
            .join("Programs/OpenAI/Codex/bin/codex.exe")
    } else {
        home.join(".local/bin").join(format!("{cli}{exe}"))
    };
    std::fs::create_dir_all(to.parent().expect("folder")).expect("make the folder");
    std::fs::copy(&from, &to).expect("install the CLI");
}
