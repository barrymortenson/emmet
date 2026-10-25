// SPDX-License-Identifier: AGPL-3.0-or-later
//! No write to user-level agent config (D-26, plan section 10): a test home holds sentinel `.claude/`, `.claude.json`,
//! and `.codex/` files, and every file in it is hashed before and after both adapters run detection, a turn with tool
//! calls and an edit, approval cards, a question, the usage probe, a crash, and resume. Nothing in the home may be
//! added, changed, or removed: not by the adapters, the emmet MCP servers they start, or the runs those make.

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
        rig.scenario(cli, "", "plain-turn");
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

#[test]
fn user_level_agent_configs_are_untouched() {
    let home = temp_dir("sentinel-home");
    for (rel, text) in SENTINELS {
        let p = home.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }
    let before = hash_tree(&home);
    assert_eq!(before.len(), SENTINELS.len());

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

    let after = hash_tree(&home);
    let changed: Vec<String> = after
        .iter()
        .filter(|(k, v)| before.get(*k) != Some(*v))
        .map(|(k, _)| k.clone())
        .chain(before.keys().filter(|k| !after.contains_key(*k)).cloned())
        .collect();
    assert!(changed.is_empty(), "files in the home changed: {changed:?}");
}
