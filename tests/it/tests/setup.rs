// SPDX-License-Identifier: AGPL-3.0-or-later
//! Agent setup without a terminal (D-41, ADR 0012): installing Claude Code and Codex with their providers' installers
//! and signing in with Claude Code's own `claude auth login`, against fake-install and fake-claude in a fresh home
//! each: success, already installed, offline, a checksum mismatch, another error, cancel, an unsigned Claude Code,
//! and a failed or canceled sign-in. Each failure is a plain sentence with one retry; the retry works.

use std::path::{Path, PathBuf};
use std::time::Duration;

use emmet_agent::setup::{self, Cancel, Failure};
use emmet_agent::{AgentKind, Billing};
use emmet_it::agents::fake_bin;
use emmet_it::temp_dir;

struct Home {
    home: PathBuf,
    fake: PathBuf,
    env: Vec<(String, String)>,
}

/// A fresh home with neither CLI, the fakes' folder, and only the system's own folders on PATH.
fn home(name: &str) -> Home {
    let dir = temp_dir(&format!("setup-{name}"));
    let (home, fake) = (dir.join("home"), dir.join("fake"));
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&fake).unwrap();
    // Not signed in until fake-claude's `auth login` says so.
    std::fs::write(
        fake.join("claude-auth.json"),
        r#"{"loggedIn":false,"authMethod":"none"}"#,
    )
    .unwrap();
    let h = home.display().to_string();
    let mut env: Vec<(String, String)> = vec![
        ("HOME".into(), h.clone()),
        ("USERPROFILE".into(), h.clone()),
        ("APPDATA".into(), home.join("AppData/Roaming").display().to_string()),
        ("LOCALAPPDATA".into(), home.join("AppData/Local").display().to_string()),
        ("EMMET_FAKE_DIR".into(), fake.display().to_string()),
        ("EMMET_FAKE_INSTALL".into(), fake_bin("install").display().to_string()),
        ("EMMET_FAKE_SIGNATURE".into(), "ok".into()),
        ("EMMET_DETECT_HOME_ONLY".into(), "1".into()),
    ];
    if cfg!(windows) {
        let root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());
        env.push(("PATH".into(), format!("{root}\\System32")));
        env.push(("SystemRoot".into(), root));
    } else {
        env.push(("PATH".into(), "/usr/bin:/bin".into()));
    }
    Home { home, fake, env }
}

impl Home {
    fn mode(&self, cli: &str, mode: &str) {
        std::fs::write(self.fake.join(format!("install-{cli}.mode")), mode).unwrap();
    }

    fn attempts(&self, cli: &str) -> usize {
        std::fs::read_to_string(self.fake.join(format!("install-{cli}.log")))
            .map(|l| l.lines().count())
            .unwrap_or(0)
    }

    fn install(&self, kind: AgentKind) -> Result<emmet_agent::Detection, setup::SetupError> {
        setup::install(kind, &self.env, false, &Cancel::default(), &mut |_| {})
    }

    /// Where the real installer leaves the CLI, as fake-install does.
    fn installed(&self, kind: AgentKind) -> PathBuf {
        let exe = std::env::consts::EXE_SUFFIX;
        match (kind, cfg!(windows)) {
            (AgentKind::Codex, true) => self.home.join("AppData/Local/Programs/OpenAI/Codex/bin/codex.exe"),
            (AgentKind::Codex, false) => self.home.join(format!(".local/bin/codex{exe}")),
            (AgentKind::Claude, _) => self.home.join(format!(".local/bin/claude{exe}")),
        }
    }

    fn login(&self, mode: &str) {
        std::fs::write(self.fake.join("claude-login.mode"), mode).unwrap();
    }

    fn login_record(&self) -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(self.fake.join("claude-login.json")).unwrap()).unwrap()
    }
}

fn same(a: &Path, b: &Path) -> bool {
    emmet_core::fsx::canonical(a).ok() == emmet_core::fsx::canonical(b).ok()
}

#[test]
fn claude_installs_and_signs_in_without_a_terminal() {
    let h = home("claude-ok");
    let mut said = Vec::new();
    let d = setup::install(AgentKind::Claude, &h.env, false, &Cancel::default(), &mut |l| {
        said.push(l.to_string())
    })
    .expect("installs");
    assert!(
        same(d.path.as_deref().unwrap(), &h.installed(AgentKind::Claude)),
        "{:?}",
        d.path
    );
    assert_eq!(d.version.as_deref(), Some("2.1.310"));
    assert!(!d.signed_in && d.fix.as_deref() == Some("Claude Code isn't signed in yet."));
    assert!(
        said.iter().any(|l| l == "Setting up claude..."),
        "progress lines reach the caller: {said:?}"
    );

    h.login("ok");
    let d = setup::sign_in(&h.env, false, &Cancel::default()).expect("signs in");
    assert!(d.signed_in);
    assert_eq!(
        d.billing,
        Billing::Plan {
            name: Some("pro".into())
        }
    );
    // Claude Code's own command, unchanged, with nothing on its stdin: Emmet never handles a sign-in code.
    let rec = h.login_record();
    assert_eq!(rec["args"], serde_json::json!(["auth", "login"]));
    assert_eq!(rec["stdin_closed"], true);
}

#[test]
fn codex_installs_where_openais_installer_puts_it() {
    let h = home("codex-ok");
    let d = h.install(AgentKind::Codex).expect("installs");
    assert!(
        same(d.path.as_deref().unwrap(), &h.installed(AgentKind::Codex)),
        "{:?}",
        d.path
    );
    // Signed out until Continue with ChatGPT or an API key (1E).
    assert!(!d.signed_in);
}

#[test]
fn an_installed_cli_isnt_installed_again() {
    let h = home("already");
    h.install(AgentKind::Claude).expect("installs");
    let d = h.install(AgentKind::Claude).expect("already there");
    assert!(d.path.is_some());
    assert_eq!(h.attempts("claude"), 1, "the installer ran only once");
}

#[test]
fn offline_says_so_and_the_retry_works() {
    let h = home("offline");
    for (cli, kind, who) in [
        ("claude", AgentKind::Claude, "Anthropic"),
        ("codex", AgentKind::Codex, "OpenAI"),
    ] {
        h.mode(cli, "offline");
        let e = h.install(kind).expect_err("offline fails");
        assert_eq!(e.failure, Failure::Offline, "{cli}: {e:?}");
        assert_eq!(
            e.message,
            format!("Emmet couldn't reach {who}'s download server. Check that you're online, then try again.")
        );
        assert!(!h.installed(kind).exists());
        h.mode(cli, "ok");
        assert!(h.install(kind).expect("the retry installs").path.is_some());
    }
}

#[test]
fn a_checksum_mismatch_installs_nothing() {
    let h = home("checksum");
    for (cli, kind) in [("claude", AgentKind::Claude), ("codex", AgentKind::Codex)] {
        h.mode(cli, "checksum");
        let e = h.install(kind).expect_err("checksum fails");
        assert_eq!(e.failure, Failure::Checksum, "{cli}: {e:?}");
        assert!(
            e.message.starts_with("The download didn't match the checksum"),
            "{}",
            e.message
        );
        assert!(!h.installed(kind).exists());
    }
}

#[test]
fn another_installer_error_shows_its_own_words() {
    let h = home("fail");
    h.mode("claude", "fail");
    let e = h.install(AgentKind::Claude).expect_err("fails");
    assert_eq!(e.failure, Failure::Failed);
    assert_eq!(
        e.message,
        "Anthropic's installer stopped with an error (exit code 3). Try again."
    );
    assert_eq!(
        e.detail.as_deref(),
        Some("Error: the installer hit something unexpected (fail)")
    );
}

/// Cancel ends the installer and the downloader it started, so nothing lands afterward.
#[test]
fn cancel_stops_the_whole_install() {
    let h = home("cancel");
    h.mode("codex", "slow");
    let cancel = Cancel::default();
    let c = cancel.clone();
    let started = h.fake.join("install-codex.started");
    let t = std::thread::spawn(move || {
        // Once the downloader the installer started is running, so the cancel has a tree to end.
        while !started.exists() {
            std::thread::sleep(Duration::from_millis(50));
        }
        c.cancel();
    });
    let e = setup::install(AgentKind::Codex, &h.env, false, &cancel, &mut |_| {}).expect_err("canceled");
    t.join().unwrap();
    assert_eq!(e.failure, Failure::Canceled);
    assert_eq!(
        e.message,
        "Install canceled. Codex isn't set up; install it again whenever you like."
    );
    std::thread::sleep(Duration::from_secs(4));
    assert!(
        !h.installed(AgentKind::Codex).exists(),
        "the downloader outlived the cancel"
    );
}

/// Anthropic publishes who signs Claude Code; an unsigned copy (the fake) is refused with the OS's own check.
#[cfg(any(target_os = "macos", windows))]
#[test]
fn an_unsigned_claude_code_is_refused() {
    let mut h = home("unsigned");
    h.env.retain(|(k, _)| k != "EMMET_FAKE_SIGNATURE");
    let e = h.install(AgentKind::Claude).expect_err("unsigned");
    assert_eq!(e.failure, Failure::Signature, "{e:?}");
    assert!(e.message.contains("isn't signed by Anthropic"), "{}", e.message);
    // Try again finds the refused copy still there, and refuses it again.
    let e = h.install(AgentKind::Claude).expect_err("still unsigned");
    assert_eq!(e.failure, Failure::Signature, "{e:?}");
}

#[test]
fn an_install_emmet_cant_find_says_so_and_the_retry_works() {
    let h = home("elsewhere");
    h.mode("claude", "elsewhere");
    let e = h.install(AgentKind::Claude).expect_err("not where Emmet looks");
    assert_eq!(e.failure, Failure::Failed);
    assert_eq!(
        e.message,
        "The installer finished, but Emmet can't find Claude Code where it installs. Try again."
    );
    h.mode("claude", "ok");
    assert!(h.install(AgentKind::Claude).expect("the retry installs").path.is_some());
}

/// An installer that exits while a helper of its own still holds its output: Emmet goes by the exit.
#[test]
fn an_install_ends_when_the_installer_does() {
    let h = home("linger");
    h.mode("codex", "linger");
    let started = std::time::Instant::now();
    h.install(AgentKind::Codex).expect("installs");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "waited {:?} for a helper that holds the output for 20 s",
        started.elapsed()
    );
}

#[test]
fn a_failed_or_canceled_sign_in_says_so() {
    let h = home("login-fail");
    h.install(AgentKind::Claude).expect("installs");
    h.login("fail");
    let e = setup::sign_in(&h.env, true, &Cancel::default()).expect_err("fails");
    assert_eq!(e.failure, Failure::Failed);
    assert_eq!(e.message, "Claude Code's sign-in didn't finish. Try again.");
    // Claude Code's words, never the line with its sign-in link.
    assert_eq!(e.detail.as_deref(), Some("Login failed: the authorization was denied"));
    assert_eq!(
        h.login_record()["args"],
        serde_json::json!(["auth", "login", "--console"])
    );

    h.login("wait");
    let cancel = Cancel::default();
    let c = cancel.clone();
    let t = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(800));
        c.cancel();
    });
    let started = std::time::Instant::now();
    let e = setup::sign_in(&h.env, false, &cancel).expect_err("canceled");
    t.join().unwrap();
    assert_eq!(e.failure, Failure::Canceled);
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "cancel ends the wait at once"
    );

    // Claude Code exits 0, but its own status still says signed out.
    h.login("unconfirmed");
    let e = setup::sign_in(&h.env, false, &Cancel::default()).expect_err("unconfirmed");
    assert!(e.message.contains("doesn't show you signed in"), "{}", e.message);

    h.login("ok");
    assert!(
        setup::sign_in(&h.env, false, &Cancel::default())
            .expect("the retry signs in")
            .signed_in
    );
}
