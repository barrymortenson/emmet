// SPDX-License-Identifier: AGPL-3.0-or-later
//! No write to user-level agent config (D-26, plan section 10): a test home holds sentinel `.claude/`, `.claude.json`,
//! and `.codex/` files, and every file in it is hashed before and after both adapters run detection, a turn with tool
//! calls and an edit, approval cards, a question, the usage probe, a crash, and resume. Nothing in the home may be
//! added, changed, or removed: not by the adapters, the emmet MCP servers they start, or the runs those make. Setup
//! (1F) gets the same check: installing both CLIs and signing in change nothing in the home but the CLIs the
//! installers put there.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use emmet_agent::{AgentEvent, AgentKind, Answer, Decision, UserTurn};
use emmet_it::agents::{Events, Rig};
use emmet_it::{sha256, temp_dir};

const SENTINELS: &[(&str, &str)] = &[
    (".claude/settings.json", r#"{"permissions":{"allow":["Bash(ls)"]}}"#),
    (".claude/settings.local.json", "{}"),
    (".claude/CLAUDE.md", "# Sentinel\n"),
    (".claude/agents/sentinel.md", "sentinel\n"),
    (".claude/skills/sentinel/SKILL.md", "sentinel\n"),
    (".claude.json", r#"{"mcpServers":{}}"#),
    (".codex/config.toml", "model = \"sentinel\"\n"),
    (".codex/AGENTS.md", "# Sentinel\n"),
    (".codex/rules/sentinel.rules", "sentinel\n"),
    (".gemini/config/mcp_config.json", r#"{"mcpServers":{}}"#),
    (".gemini/settings.json", "{}"),
];

fn hash_tree(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        for e in std::fs::read_dir(&dir).expect("read home") {
            let p: PathBuf = e.expect("entry").path();
            if p.is_dir() {
                dirs.push(p);
            } else {
                let rel = p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
                out.insert(rel, sha256(&std::fs::read(&p).unwrap()));
            }
        }
    }
    out
}

fn turn(text: &str) -> UserTurn {
    UserTurn { text: text.into() }
}

/// Answers every card and question in a turn the first way it can.
fn answer_all(s: &mut dyn emmet_agent::Session, ev: &mut Events) {
    loop {
        let got = ev.until("the turn", |e| {
            matches!(
                e,
                AgentEvent::Approval { .. } | AgentEvent::Question { .. } | AgentEvent::TurnDone { .. }
            )
        });
        match got.last().cloned() {
            Some(AgentEvent::Approval { id, .. }) => s.respond(&id, Decision::AllowOnce).unwrap(),
            Some(AgentEvent::Question { id, question }) => s
                .answer(
                    &id,
                    Answer {
                        choices: vec![question.options[0].label.clone()],
                        other: None,
                    },
                )
                .unwrap(),
            _ => return,
        }
    }
}

fn run(home: &Path, cli: &str, kind: AgentKind, scenario: &str, texts: &[&str]) {
    let rig = Rig::in_home(&format!("sentinel-{cli}-{scenario}"), cli, scenario, home.to_path_buf());
    rig.key("sk-sentinel-not-a-key");
    let adapter = rig.adapter(kind);
    adapter.detect();
    let (mut s, mut ev) = rig.start(kind);
    for t in texts {
        s.send(turn(t)).unwrap();
        answer_all(s.as_mut(), &mut ev);
    }
    if let Some(resume) = s.resume_token() {
        s.close();
        ev.exit();
        let mut cfg = rig.cfg.clone();
        cfg.resume = Some(resume);
        // A resumed Codex sends thread/resume, which 0B's resume.jsonl answers in its second process.
        if kind == AgentKind::Codex {
            rig.scenario(cli, "", "resume");
            std::fs::write(rig.fake.join("codex.count"), "1").unwrap();
        } else {
            rig.scenario(cli, "", "plain-turn");
        }
        let (mut s, mut ev) = rig.start_with(kind, cfg);
        s.send(turn("Reply with exactly the word: ready")).unwrap();
        ev.turn();
        s.close();
    }
    if kind == AgentKind::Claude {
        rig.scenario(cli, "-usage", "usage-report");
        adapter.usage().unwrap();
    }
}

fn sentinel_home(name: &str) -> (PathBuf, BTreeMap<String, String>) {
    let home = temp_dir(name);
    for (rel, text) in SENTINELS {
        let p = home.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    let before = hash_tree(&home);
    assert_eq!(before.len(), SENTINELS.len());
    (home, before)
}

/// Every file in `home` added, changed, or removed since `before`.
fn changes(home: &Path, before: &BTreeMap<String, String>) -> Vec<String> {
    let after = hash_tree(home);
    after
        .iter()
        .filter(|(k, v)| before.get(*k) != Some(*v))
        .map(|(k, _)| k.clone())
        .chain(before.keys().filter(|k| !after.contains_key(*k)).cloned())
        .collect()
}

#[test]
fn user_level_agent_configs_are_untouched() {
    let (home, before) = sentinel_home("sentinel-home");

    let c = AgentKind::Claude;
    run(
        &home,
        "claude",
        c,
        "derived-build",
        &["Make the row 10 bricks wide, then build it."],
    );
    run(
        &home,
        "claude",
        c,
        "permissions-ask-bash",
        &["Run `ls`.", "Run `echo hi`."],
    );
    run(&home, "claude", c, "derived-ask-user", &["Build a wall."]);
    run(
        &home,
        "claude",
        c,
        "permissions",
        &["Change WIDTH.", "Read ../outside.txt.", "Create notes.txt.", "echo hi"],
    );
    let x = AgentKind::Codex;
    run(
        &home,
        "codex",
        x,
        "derived-build",
        &["Make the row 10 bricks wide, then build it."],
    );
    run(&home, "codex", x, "permissions-untrusted", &["a", "b", "c", "d", "e"]);
    run(&home, "codex", x, "derived-ask-user", &["Build a wall."]);
    run(&home, "claude", c, "derived-crash-mid-stream", &["Ping."]);
    run(&home, "codex", x, "derived-crash-mid-stream", &["Ping."]);

    let changed = changes(&home, &before);
    assert!(changed.is_empty(), "files in the home changed: {changed:?}");
}

#[test]
fn setup_writes_no_agent_config() {
    use emmet_agent::setup::{self, Cancel};
    let (home, before) = sentinel_home("sentinel-setup-home");
    let fake = temp_dir("sentinel-setup-fake");
    std::fs::write(fake.join("claude-auth.json"), r#"{"loggedIn":false}"#).unwrap();
    let h = home.display().to_string();
    let path = if cfg!(windows) {
        format!("{}\\System32", std::env::var("SystemRoot").unwrap_or_default())
    } else {
        "/usr/bin:/bin".into()
    };
    let env: Vec<(String, String)> = [
        ("HOME", h.clone()),
        ("USERPROFILE", h),
        ("APPDATA", home.join("AppData/Roaming").display().to_string()),
        ("LOCALAPPDATA", home.join("AppData/Local").display().to_string()),
        ("EMMET_FAKE_DIR", fake.display().to_string()),
        (
            "EMMET_FAKE_INSTALL",
            emmet_it::agents::fake_bin("install").display().to_string(),
        ),
        ("EMMET_FAKE_SIGNATURE", "ok".into()),
        ("EMMET_DETECT_HOME_ONLY", "1".into()),
        ("PATH", path),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .chain(std::env::var("SystemRoot").map(|v| ("SystemRoot".to_string(), v)))
    .collect();

    for kind in [AgentKind::Claude, AgentKind::Codex] {
        setup::install(kind, &env, false, &Cancel::default(), &mut |_| {}).expect("installs");
    }
    setup::sign_in(&env, false, &Cancel::default()).expect("signs in");

    // Only what the installers installed: the two CLIs.
    let exe = std::env::consts::EXE_SUFFIX;
    let codex = if cfg!(windows) {
        "AppData/Local/Programs/OpenAI/Codex/bin/codex.exe".to_string()
    } else {
        format!(".local/bin/codex{exe}")
    };
    let mut changed = changes(&home, &before);
    changed.sort();
    let mut want = vec![format!(".local/bin/claude{exe}"), codex];
    want.sort();
    assert_eq!(
        changed, want,
        "setup changed files in the home besides the installed CLIs"
    );
}
