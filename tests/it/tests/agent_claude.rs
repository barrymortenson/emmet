// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Claude Code adapter, replayed (plan sections 10 and 16): fake-claude plays each of 0B's transcripts and 1C's
//! derived ones (tests/replay/derive.mjs) through the real `emmet mcp` and `emmet mcp-host`, and each test pins the
//! event sequence (an outline snapshot) and the project's state.

use emmet_agent::{
    AgentEvent, AgentKind, Answer, ApprovalKind, Billing, Decision, DoneReason, SteerPath, UserTurn, claude,
    usage::UsageStatus,
};
use emmet_it::agents::Rig;
use insta::assert_json_snapshot;

const CLAUDE: AgentKind = AgentKind::Claude;

fn turn(text: &str) -> UserTurn {
    UserTurn { text: text.into() }
}

fn is_tool(name: &'static str) -> impl Fn(&AgentEvent) -> bool {
    move |e| matches!(e, AgentEvent::ToolCall { name: n, .. } if n == name)
}

fn delivered(events: &[AgentEvent]) -> Vec<SteerPath> {
    events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::NoteDelivered { path, .. } => Some(*path),
            _ => None,
        })
        .collect()
}

fn text_of(events: &[AgentEvent]) -> String {
    events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::Text { delta, .. } => Some(delta.as_str()),
            _ => None,
        })
        .collect()
}

fn user_texts(rig: &Rig) -> Vec<String> {
    rig.written()
        .iter()
        .filter(|m| m["type"] == "user")
        .map(|m| m["message"]["content"].as_str().unwrap_or("").to_string())
        .collect()
}

#[test]
fn plain_turn() {
    let rig = Rig::new("claude-plain", "claude", "plain-turn");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!(rig.outline(&events));
    assert_eq!(
        events[0],
        AgentEvent::Started {
            resume: s.resume_token().unwrap(),
            model: Some("claude-haiku-5-5".into()),
            billing: Some(Billing::ApiKey {
                source: Some("ANTHROPIC_API_KEY".into())
            }),
        }
    );
    // The transcript has both directions, as 0B's recordings do.
    assert_eq!(user_texts(&rig), ["Reply with exactly the word: ready"]);
}

/// A tool only 0B's stub had (`ping`) keeps its recorded result; ToolSearch comes first because MCP tools are
/// deferred.
#[test]
fn tool_call() {
    let rig = Rig::new("claude-tool", "claude", "tool-call");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn(
        "Use the ping tool with the message hello, then tell me exactly what it answered.",
    ))
    .unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!(rig.outline(&events));
}

#[test]
fn build_edits_model_py_and_runs_the_real_model() {
    let rig = Rig::new("claude-build", "claude", "derived-build");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Make the row 10 bricks wide, then build it.")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!(rig.outline(&events));
    assert!(rig.model_py().contains("WIDTH = 10"));
    let placements: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(rig.project.join("build/placements.json")).unwrap()).unwrap();
    assert_eq!(placements["parts"].as_array().unwrap().len(), 10);
}

/// 0B's four permission turns through Emmet's real approve: the edit to model.py is pre-allowed, the read outside
/// the project is a read (allowed, unlike 0B's stub), the new notes.txt is inside the project (allowed), and echo
/// ran under the sandbox without asking. No card appears.
#[test]
fn permissions() {
    let rig = Rig::new("claude-permissions", "claude", "permissions");
    std::fs::write(rig.project.parent().unwrap().join("outside.txt"), "the outside line\n").unwrap();
    let (mut s, mut ev) = rig.start(CLAUDE);
    let mut all = vec![];
    for text in [
        "Change WIDTH in model.py from 8 to 10. Reply done when finished.",
        "Read the file ../outside.txt and tell me its first line.",
        "Create a file notes.txt in the current directory containing the word hi. Reply done when finished.",
        "Run the shell command `echo hi` and tell me its output.",
    ] {
        s.send(turn(text)).unwrap();
        all.extend(ev.turn());
    }
    s.close();
    assert_json_snapshot!(rig.outline(&all));
    assert!(!all.iter().any(|e| matches!(e, AgentEvent::Approval { .. })));
    assert!(rig.model_py().contains("WIDTH = 10"));
    assert_eq!(std::fs::read_to_string(rig.project.join("notes.txt")).unwrap(), "hi\n");
}

/// The ask rule (Windows, D-40): each shell command reaches approve and becomes a card. Denied, the tool result is the
/// denial; allowed, the command runs.
#[test]
fn an_approval_card_denied_then_allowed() {
    let rig = Rig::new("claude-ask-bash", "claude", "permissions-ask-bash");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Run the shell command `ls` and tell me its output."))
        .unwrap();
    let card = ev.until("an approval card", |e| matches!(e, AgentEvent::Approval { .. }));
    let Some(AgentEvent::Approval { id, kind, tool, detail }) = card.last().cloned() else {
        unreachable!()
    };
    assert_eq!(
        (kind, tool.as_str(), detail.as_str()),
        (ApprovalKind::Command, "Bash", "ls")
    );
    s.respond(&id, Decision::Deny).unwrap();
    let mut all = card;
    all.extend(ev.turn());
    s.send(turn("Run the shell command `echo hi` and tell me its output."))
        .unwrap();
    let card = ev.until("an approval card", |e| matches!(e, AgentEvent::Approval { .. }));
    let Some(AgentEvent::Approval { id, .. }) = card.last().cloned() else {
        unreachable!()
    };
    s.respond(&id, Decision::AllowOnce).unwrap();
    all.extend(card);
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
}

#[test]
fn interrupt() {
    let rig = Rig::new("claude-interrupt", "claude", "interrupt");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Use the block tool with seconds=60, then say done."))
        .unwrap();
    ev.until("the block call", is_tool("mcp__emmet__block"));
    s.interrupt().unwrap();
    let events = ev.turn();
    s.close();
    assert_eq!(
        events.last(),
        Some(&AgentEvent::TurnDone {
            reason: DoneReason::Interrupted
        })
    );
    assert!(rig.written().iter().any(|m| m["request"]["subtype"] == "interrupt"));
}

#[test]
fn interrupt_then_a_new_turn() {
    let rig = Rig::new("claude-interrupt-new", "claude", "interrupt-new-turn");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Use the block tool with seconds=60, then say done."))
        .unwrap();
    ev.until("the block call", is_tool("mcp__emmet__block"));
    s.interrupt().unwrap();
    let mut all = ev.turn();
    s.send(turn("Reply with exactly the word: after")).unwrap();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
}

/// Redirect, native path (ADR 0001): written during the tool call, it reaches the model in the same turn.
#[test]
fn a_redirect_delivered_natively_mid_turn() {
    let rig = Rig::new("claude-steer", "claude", "mid-turn-message");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn(
        "Use the block tool with seconds=20. When it returns, reply with one short sentence.",
    ))
    .unwrap();
    ev.until("the block call", is_tool("mcp__emmet__block"));
    let note = s.steer("Also include the word banana in your reply.").unwrap();
    let events = ev.turn();
    s.close();
    assert!(events.contains(&AgentEvent::NoteDelivered {
        id: note.id,
        path: SteerPath::Native
    }));
    assert!(text_of(&events).contains("banana"));
    assert_eq!(user_texts(&rig).len(), 2, "one turn plus the redirect, no second turn");
}

/// Redirect, queued path: with native delivery off, the note waits for the next Emmet tool result, where `emmet mcp
/// --session` attaches it as user_notes.
#[test]
fn a_redirect_delivered_through_user_notes() {
    let mut rig = Rig::new("claude-notes", "claude", "derived-build");
    rig.cfg.native_steer = false;
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Make the row 10 bricks wide, then build it.")).unwrap();
    let note = s.steer("Make it blue instead.").unwrap();
    let events = ev.turn();
    s.close();
    assert_eq!(delivered(&events), [SteerPath::Queued]);
    let run = events
        .iter()
        .rev()
        .find_map(|e| match e {
            AgentEvent::ToolResult { summary, .. } => Some(summary.clone()),
            _ => None,
        })
        .unwrap();
    assert!(
        run.starts_with("The user just said: \"Make it blue instead.\""),
        "{run}"
    );
    assert!(events.contains(&AgentEvent::NoteDelivered {
        id: note.id,
        path: SteerPath::Queued
    }));
}

/// Redirect, third path: the turn ends before a step, so the note goes out as the next turn.
#[test]
fn a_redirect_sent_as_the_next_turn() {
    let rig = Rig::new("claude-next", "claude", "derived-next-turn");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    s.steer("And then say it again.").unwrap();
    let mut all = ev.turn();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
    assert_eq!(
        user_texts(&rig),
        ["Reply with exactly the word: ready", "And then say it again."]
    );
}

/// A note still pending at Stop goes out as the next turn, prefixed so the agent knows why its turn ended.
#[test]
fn a_redirect_still_pending_at_stop() {
    let mut rig = Rig::new("claude-pending-stop", "claude", "interrupt-new-turn");
    rig.cfg.native_steer = false;
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Use the block tool with seconds=60, then say done."))
        .unwrap();
    s.steer("Reply with exactly the word: after").unwrap();
    ev.until("the block call", is_tool("mcp__emmet__block"));
    s.interrupt().unwrap();
    let mut all = ev.turn();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
    assert_eq!(
        user_texts(&rig).last().unwrap(),
        "(You stopped the previous turn.)\n\nReply with exactly the word: after"
    );
}

#[test]
fn a_question_answered_in_the_same_turn() {
    let rig = Rig::new("claude-ask", "claude", "derived-ask-user");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Build a wall.")).unwrap();
    let mut all = ev.until("the question", |e| matches!(e, AgentEvent::Question { .. }));
    let Some(AgentEvent::Question { id, question }) = all.last().cloned() else {
        unreachable!()
    };
    assert_eq!(question.options[0].label, "6 bricks");
    assert_eq!(question.header.as_deref(), Some("Height"));
    s.answer(
        &id,
        Answer {
            choices: vec!["6 bricks".into()],
            other: None,
        },
    )
    .unwrap();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
}

#[test]
fn a_question_canceled_by_stop() {
    let rig = Rig::new("claude-ask-stop", "claude", "derived-ask-user-stopped");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Build a wall.")).unwrap();
    let mut all = ev.until("the question", |e| matches!(e, AgentEvent::Question { .. }));
    s.interrupt().unwrap();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
    assert_eq!(
        all.last(),
        Some(&AgentEvent::TurnDone {
            reason: DoneReason::Interrupted
        })
    );
}

/// The CLI dies mid-turn: the chat says so with its last words, the project is whole, and resume continues the session
/// in a new process.
#[test]
fn a_crash_mid_stream_then_resume() {
    let rig = Rig::new("claude-crash", "claude", "derived-crash-mid-stream");
    let before = rig.model_py();
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn(
        "Use the ping tool with the message hello, then tell me exactly what it answered.",
    ))
    .unwrap();
    let events = ev.exit();
    assert_json_snapshot!(rig.outline(&events));
    let crash = events.iter().find_map(|e| match e {
        AgentEvent::Error { message, .. } => Some(message.clone()),
        _ => None,
    });
    assert!(crash.unwrap().contains("crashed on purpose"));
    assert_eq!(rig.model_py(), before);
    assert!(s.send(turn("again")).is_err(), "a dead session refuses turns");

    let resume = s.resume_token().unwrap();
    let mut cfg = rig.cfg.clone();
    cfg.resume = Some(resume.clone());
    let (mut s, mut ev) = rig.start_with(CLAUDE, cfg);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert_eq!(
        events.last(),
        Some(&AgentEvent::TurnDone {
            reason: DoneReason::Completed
        })
    );
    let args = rig.record("claude", 1)["args"].clone();
    let args: Vec<&str> = args.as_array().unwrap().iter().filter_map(|a| a.as_str()).collect();
    assert!(args.contains(&format!("--resume={resume}").as_str()), "{args:?}");
}

#[test]
fn a_malformed_line_is_skipped() {
    let rig = Rig::new("claude-malformed", "claude", "derived-malformed-line");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!("plain_turn", rig.outline(&events));
}

#[test]
fn unknown_event_types_are_ignored() {
    let rig = Rig::new("claude-unknown", "claude", "derived-unknown-event");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!("plain_turn", rig.outline(&events));
}

#[test]
fn resume_remembers_the_first_process() {
    let rig = Rig::new("claude-resume", "claude", "resume");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Remember the code word teal. Reply with exactly: ok"))
        .unwrap();
    ev.turn();
    let resume = s.resume_token().unwrap();
    s.close();
    ev.exit();
    let mut cfg = rig.cfg.clone();
    cfg.resume = Some(resume.clone());
    let (mut s, mut ev) = rig.start_with(CLAUDE, cfg);
    s.send(turn("What was the code word? Reply with just the word."))
        .unwrap();
    let events = ev.turn();
    s.close();
    assert!(matches!(&events[0], AgentEvent::Started { resume: r, .. } if *r == resume));
    assert_eq!(text_of(&events).trim(), "teal");
}

/// 0B's long tool calls (600 s, and a timeout past 32 bits) complete; `tool_progress` heartbeats don't show; the
/// raised MCP tool timeout is in the CLI's environment.
#[test]
fn long_tool_calls_complete() {
    for scenario in ["block-default", "block-raised", "block-overflow"] {
        let rig = Rig::new(&format!("claude-{scenario}"), "claude", scenario);
        let (mut s, mut ev) = rig.start(CLAUDE);
        s.send(turn("Use the block tool, then tell me exactly what it returned."))
            .unwrap();
        let events = ev.turn();
        s.close();
        assert_eq!(
            events.last(),
            Some(&AgentEvent::TurnDone {
                reason: DoneReason::Completed
            }),
            "{scenario}"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, AgentEvent::ToolResult { ok: true, .. })),
            "{scenario}"
        );
        assert_eq!(
            rig.record("claude", 0)["env"]["MCP_TOOL_TIMEOUT"],
            claude::MCP_TOOL_TIMEOUT
        );
    }
}

/// The hand-check fixtures (0B, reduced) have no input line: the fake plays them as soon as it starts.
#[test]
fn billing_on_a_plan_with_live_limits() {
    let rig = Rig::new("claude-plan", "claude", "plan-turn");
    let (mut s, mut ev) = rig.start(CLAUDE);
    let events = ev.turn();
    s.close();
    assert_json_snapshot!(rig.outline(&events));
    assert!(matches!(
        &events[0],
        AgentEvent::Started {
            billing: Some(Billing::Plan { .. }),
            ..
        }
    ));
}

#[test]
fn billing_on_an_api_key_that_wins_over_the_plan() {
    let rig = Rig::new("claude-plan-key", "claude", "plan-and-key-turn");
    let (mut s, mut ev) = rig.start(CLAUDE);
    let events = ev.turn();
    s.close();
    assert!(matches!(
        &events[0],
        AgentEvent::Started { billing: Some(Billing::ApiKey { source: Some(src) }), .. } if src == "ANTHROPIC_API_KEY"
    ));
    assert!(!events.iter().any(|e| matches!(e, AgentEvent::Limits { .. })));
}

/// The usage probe (D-36): `/usage` in a short-lived process with no MCP servers and no saved session, in an empty
/// folder in app data. Rows keep Claude Code's order.
#[test]
fn usage_report_meters() {
    let rig = Rig::new("claude-usage", "claude", "plain-turn");
    rig.scenario("claude", "-usage", "usage-report");
    let snap = rig.adapter(CLAUDE).usage().unwrap();
    assert_eq!(snap.status, UsageStatus::Ok);
    assert_eq!(snap.billing, Billing::Plan { name: None });
    let rows: Vec<(String, Option<f64>)> = snap.windows.iter().map(|w| (w.label.clone(), w.percent)).collect();
    assert_eq!(
        rows,
        [
            ("Session".into(), Some(4.0)),
            ("Week".into(), Some(13.0)),
            ("Week · Fable".into(), Some(0.0))
        ]
    );
    assert_eq!(
        snap.windows[0].resets_at,
        emmet_agent::time::parse_rfc3339("2026-10-25T05:19:59Z")
    );
    let rec = rig.record("claude", 0);
    let args: Vec<&str> = rec["args"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|a| a.as_str())
        .collect();
    assert!(args.contains(&"--no-session-persistence") && args.contains(&"--strict-mcp-config"));
    assert!(
        !args.contains(&"--mcp-config"),
        "the probe has no MCP servers: {args:?}"
    );
    assert!(rec["cwd"].as_str().unwrap().ends_with("usage-probe"));
    assert_eq!(
        rig.written(),
        Vec::<serde_json::Value>::new(),
        "the probe isn't a chat session"
    );
}

#[test]
fn usage_is_unavailable_when_the_cli_reports_none() {
    let rig = Rig::new("claude-usage-none", "claude", "plain-turn");
    rig.scenario("claude", "-usage", "plain-turn");
    let snap = rig.adapter(CLAUDE).usage().unwrap();
    assert_eq!(
        snap.status,
        UsageStatus::ApiKey,
        "plain-turn bills ANTHROPIC_API_KEY, which has no meters"
    );
    // On the plan, but no usage report: unavailable, never 0 %.
    rig.scenario("claude", "-usage", "plan-turn");
    let snap = rig.adapter(CLAUDE).usage().unwrap();
    assert_eq!(snap.status, UsageStatus::Unavailable);
    assert!(snap.windows.is_empty() && snap.failure.is_some());
}

/// D-37: the process gets the environment Emmet was given, exactly, plus MCP_TOOL_TIMEOUT; no credential variable is
/// added, removed, or changed; and never --bare or --setting-sources. The same for the usage probe.
#[test]
fn the_claude_process_gets_the_login_environment_unchanged() {
    let mut rig = Rig::new("claude-env", "claude", "plain-turn");
    rig.scenario("claude", "-usage", "usage-report");
    for (k, v) in [
        ("ANTHROPIC_API_KEY", "sk-ant-test-not-a-key"),
        ("ANTHROPIC_AUTH_TOKEN", "test-token"),
        ("ANTHROPIC_BASE_URL", "https://example.invalid"),
        ("CLAUDE_CODE_OAUTH_TOKEN", "test-oauth"),
        ("CLAUDE_CODE_USE_BEDROCK", "0"),
        ("AWS_PROFILE", "test"),
    ] {
        rig.cfg.env.push((k.into(), v.into()));
    }
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    ev.turn();
    s.close();
    rig.adapter(CLAUDE).usage().unwrap();
    let given: std::collections::BTreeMap<String, String> = rig.cfg.env.iter().cloned().collect();
    for (n, extra) in [(0, Some(claude::MCP_TOOL_TIMEOUT)), (1, None)] {
        let rec = rig.record("claude", n);
        let mut got: std::collections::BTreeMap<String, String> = serde_json::from_value(rec["env"].clone()).unwrap();
        // What the OS adds to every process on its own.
        for k in ["__CF_USER_TEXT_ENCODING", "__CFBundleIdentifier"] {
            if !given.contains_key(k) {
                got.remove(k);
            }
        }
        let mut want = given.clone();
        if let Some(t) = extra {
            want.insert("MCP_TOOL_TIMEOUT".into(), t.into());
        }
        assert_eq!(got, want, "start {n}: the environment changed");
        let args: Vec<&str> = rec["args"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|a| a.as_str())
            .collect();
        for banned in ["--bare", "--setting-sources", "--dangerously-skip-permissions"] {
            assert!(!args.contains(&banned), "start {n} passed {banned}");
        }
    }
    let args = rig.record("claude", 0)["args"].clone();
    let args: Vec<&str> = args.as_array().unwrap().iter().filter_map(|a| a.as_str()).collect();
    let after = |flag: &str| args.iter().position(|a| *a == flag).map(|i| args[i + 1]);
    assert_eq!(after("--permission-prompt-tool"), Some("mcp__emmet_host__approve"));
    assert_eq!(after("--disallowedTools"), Some("AskUserQuestion"));
    assert_eq!(after("--settings"), Some(claude::settings()));
    assert_eq!(after("--append-system-prompt"), Some(emmet_agent::APP_POINTER));
    assert!(args.contains(&"--strict-mcp-config"));
}

#[test]
fn detection_reads_the_version_and_sign_in() {
    let rig = Rig::new("claude-detect", "claude", "plain-turn");
    let d = rig.adapter(CLAUDE).detect();
    assert_eq!(d.version.as_deref(), Some("2.1.310"));
    assert!(d.supported && d.signed_in && d.fix.is_none() && d.note.is_none());
    assert_eq!(
        d.billing,
        Billing::Plan {
            name: Some("max".into())
        }
    );
    assert_eq!(d.billing.label(), "Max plan");

    std::fs::write(
        rig.fake.join("claude-auth.json"),
        r#"{"loggedIn":false,"authMethod":"none"}"#,
    )
    .unwrap();
    let d = rig.adapter(CLAUDE).detect();
    assert!(!d.signed_in && !d.usable());
    assert_eq!(d.fix.as_deref(), Some("Claude Code isn't signed in yet."));
    std::fs::write(
        rig.fake.join("claude-auth.json"),
        r#"{"loggedIn":true,"authMethod":"api_key","apiKeySource":"ANTHROPIC_API_KEY"}"#,
    )
    .unwrap();
    assert_eq!(rig.adapter(CLAUDE).detect().billing.label(), "API key");
}

/// "Allow for this session" covers that one command, not the shell: the next, different command still asks.
#[test]
fn allowing_a_command_for_the_session_doesnt_allow_others() {
    let rig = Rig::new("claude-allow-session", "claude", "permissions-ask-bash");
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Run the shell command `ls` and tell me its output."))
        .unwrap();
    let card = ev.until("an approval card", |e| matches!(e, AgentEvent::Approval { .. }));
    let Some(AgentEvent::Approval { id, .. }) = card.last().cloned() else {
        unreachable!()
    };
    s.respond(&id, Decision::AllowSession).unwrap();
    ev.turn();
    s.send(turn("Run the shell command `echo hi` and tell me its output."))
        .unwrap();
    let card = ev.until("a second card", |e| matches!(e, AgentEvent::Approval { .. }));
    let Some(AgentEvent::Approval { id, detail, .. }) = card.last().cloned() else {
        unreachable!()
    };
    assert_eq!(detail, "echo hi");
    s.respond(&id, Decision::Deny).unwrap();
    ev.turn();
    s.close();
}

/// A session id or resume token comes back from the project's .emmet/, which may be someone else's: one that isn't a
/// plain id never reaches a path or the command line.
#[test]
fn a_resume_token_that_isnt_a_plain_id_is_refused() {
    let rig = Rig::new("claude-bad-resume", "claude", "plain-turn");
    for (id, resume) in [("../../escape", None), ("ok", Some("--bare")), ("ok", Some("a b"))] {
        let mut cfg = rig.cfg.clone();
        cfg.id = id.into();
        cfg.resume = resume.map(String::from);
        let err = rig.adapter(CLAUDE).start(cfg).err().expect("refused");
        assert!(
            err.message.contains("isn't one Emmet made"),
            "{id} {resume:?}: {}",
            err.message
        );
    }
    assert!(!rig.fake.join("claude-0.json").exists(), "the CLI never started");
}

/// Plan section 21, 3A's Done when 5: the agent validates replay-floating-step-order, gets the v1 summary with a
/// floating and a step_order finding, applies both hints (lower the brick a plate, put the post's step before the
/// lamp's), runs, and validates clean. Derived from the recordings until a live run is recorded.
#[test]
fn a_floating_and_a_step_order_finding_are_fixed_from_the_summary() {
    let rig = Rig::new("claude-fix-two", "claude", "derived-fix-floating-step-order");
    let model = emmet_it::fixture("projects/replay-floating-step-order/model.py");
    std::fs::copy(&model, rig.project.join("model.py")).unwrap();
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Validate the model and fix what it finds.")).unwrap();
    let events = ev.turn();
    s.close();
    let results: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            emmet_agent::AgentEvent::ToolResult { summary, .. } => Some(summary.clone()),
            _ => None,
        })
        .collect();
    let first = results
        .iter()
        .find(|r| r.contains("F-0001"))
        .expect("validate reported the findings");
    assert!(first.starts_with("2 errors: 1 floating, 1 step_order."), "{first}");
    assert!(first.contains("F-0001 error floating model.py:14"), "{first}");
    assert!(first.contains("lower it by 1 plate"), "{first}");
    assert!(first.contains("F-0002 error step_order model.py:16"), "{first}");
    assert!(results.last().unwrap().contains("No findings"), "{results:?}");
    let model = rig.model_py();
    assert!(model.contains(r#"at=(0, 0, 4), id="tower/2""#), "{model}");
    assert!(model.find(r#"id="post""#) < model.find(r#"id="lamp""#), "{model}");
    let (v, code) = emmet_it::emmet_json(&["check", "--strict", "--project", rig.project.to_str().unwrap()]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]));
}

/// A finding sent by "Ask agent to fix" (1D): the agent validates, gets broken-collision's real F-0001, edits the line
/// it names, runs, and validates clean. Derived from the recordings until a live run is recorded (plan section 21).
#[test]
fn a_collision_finding_is_fixed_at_its_line() {
    let rig = Rig::new("claude-fix-collision", "claude", "derived-fix-collision");
    let broken = emmet_it::fixture("projects/broken-collision/model.py");
    std::fs::copy(&broken, rig.project.join("model.py")).unwrap();
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Fix F-0001 (collision, parts wall/a, wall/b at model.py:10, model.py:11): wall/a (Brick 2 x 4) and wall/b (Brick 2 x 4) overlap by 1 stud. They share 1 stud side by side. Move wall/b by 1 stud, or use a shorter part. Then run validate again.")).unwrap();
    let events = ev.turn();
    s.close();
    let results: Vec<String> = events
        .iter()
        .filter_map(|e| match e {
            emmet_agent::AgentEvent::ToolResult { summary, .. } => Some(summary.clone()),
            _ => None,
        })
        .collect();
    let first = results
        .iter()
        .find(|r| r.contains("F-0001"))
        .expect("validate reported the collision");
    assert!(first.contains("collision model.py:10"), "{first}");
    assert!(results.last().unwrap().contains("No findings"), "{results:?}");
    assert!(rig.model_py().contains(r#"at=(2, 0, 1), id="wall/b""#));
    let (v, code) = emmet_it::emmet_json(&["check", "--strict", "--project", rig.project.to_str().unwrap()]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["findings"], serde_json::json!([]));
}

/// The model and effort picker (4B): a documented alias and level reach Claude Code as `--model` and `--effort`, and a
/// name it doesn't document never reaches its command line.
#[test]
fn the_model_and_effort_reach_the_cli() {
    let rig = Rig::new("claude-model", "claude", "plain-turn");
    let args = |n: usize| -> Vec<String> {
        rig.record("claude", n)["args"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|a| a.as_str().map(String::from))
            .collect()
    };
    let after = |a: &[String], flag: &str| a.iter().position(|x| x == flag).map(|i| a[i + 1].clone());
    for (n, (model, effort)) in [("opus", "high"), ("gpt-6-luna", "ultra")].into_iter().enumerate() {
        let mut cfg = rig.cfg.clone();
        cfg.model = Some(model.into());
        cfg.effort = Some(effort.into());
        let (mut s, mut ev) = rig.start_with(CLAUDE, cfg);
        s.send(turn("Reply with exactly the word: ready")).unwrap();
        let events = ev.turn();
        s.close();
        let a = args(n);
        if n == 0 {
            assert_eq!(after(&a, "--model").as_deref(), Some("opus"));
            assert_eq!(after(&a, "--effort").as_deref(), Some("high"));
            let Some(AgentEvent::Models { models }) = events.iter().find(|e| matches!(e, AgentEvent::Models { .. }))
            else {
                panic!("no Models event");
            };
            let ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
            assert_eq!(ids, ["", "fable", "opus", "sonnet", "haiku"]);
        } else {
            assert!(!a.iter().any(|x| x == "--model" || x == "--effort"), "{a:?}");
        }
    }
}

/// A project that carries `.emmet/transcripts` as a link elsewhere (it may be someone else's) gets no transcript
/// written through it, and the chat goes on (4B's review; the chat's own `.emmet/chat/` uses the same check).
#[cfg(unix)]
#[test]
fn a_linked_transcripts_folder_isnt_written_through() {
    let rig = Rig::new("claude-linked-transcripts", "claude", "plain-turn");
    let elsewhere = emmet_it::temp_dir("linked-transcripts-target");
    std::fs::create_dir_all(rig.project.join(".emmet")).unwrap();
    std::os::unix::fs::symlink(&elsewhere, rig.project.join(".emmet/transcripts")).unwrap();
    let (mut s, mut ev) = rig.start(CLAUDE);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert!(events.iter().any(|e| matches!(e, AgentEvent::TurnDone { .. })));
    assert_eq!(
        std::fs::read_dir(&elsewhere).unwrap().count(),
        0,
        "nothing went through the link"
    );
}
