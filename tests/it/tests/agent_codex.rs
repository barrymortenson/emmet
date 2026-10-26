// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Codex adapter, replayed (plan sections 10 and 16): fake-codex plays each of 0B's transcripts and 1C's derived
//! ones through the real `emmet mcp` and `emmet mcp-host`, and each test pins the event sequence (an outline snapshot)
//! and the project's state. The adapter's messages are checked against the committed app-server schema.

use emmet_agent::codex::types;
use emmet_agent::{
    AgentErrorCode, AgentEvent, AgentKind, Answer, ApprovalKind, Billing, Decision, DoneReason, SteerPath, UserTurn,
    usage::UsageStatus,
};
use emmet_it::agents::{Events, Rig};
use emmet_it::repo;
use emmet_it::schema::validate;
use insta::assert_json_snapshot;
use serde_json::{Value, json};

const CODEX: AgentKind = AgentKind::Codex;
const KEY: &str = "sk-test-not-a-real-key";

fn turn(text: &str) -> UserTurn {
    UserTurn { text: text.into() }
}

/// A rig with an API key in its keychain, as Settings → Agents leaves it.
fn rig(name: &str, scenario: &str) -> Rig {
    let rig = Rig::new(name, "codex", scenario);
    rig.key(KEY);
    rig
}

fn is_tool(name: &'static str) -> impl Fn(&AgentEvent) -> bool {
    move |e| matches!(e, AgentEvent::ToolCall { name: n, .. } if n == name)
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

/// What Emmet sent as each turn's text (turn/start and turn/steer).
fn sent_texts(rig: &Rig, method: &str) -> Vec<String> {
    rig.written()
        .iter()
        .filter(|m| m["method"] == method)
        .map(|m| m["params"]["input"][0]["text"].as_str().unwrap_or("").to_string())
        .collect()
}

/// Answers each approval card in order with `decisions`, then waits for the turn to end.
fn turn_answering(s: &mut dyn emmet_agent::Session, ev: &mut Events, decisions: &mut Vec<Decision>) -> Vec<AgentEvent> {
    let mut all = vec![];
    loop {
        let got = ev.until("a card or the end of the turn", |e| {
            matches!(e, AgentEvent::Approval { .. } | AgentEvent::TurnDone { .. })
        });
        let last = got.last().cloned();
        all.extend(got);
        match last {
            Some(AgentEvent::Approval { id, .. }) => s.respond(&id, decisions.remove(0)).unwrap(),
            _ => return all,
        }
    }
}

#[test]
fn plain_turn() {
    let rig = rig("codex-plain", "plain-turn");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!(rig.outline(&events));
    assert!(matches!(
        &events[0],
        AgentEvent::Started {
            billing: Some(Billing::ApiKey { .. }),
            ..
        }
    ));
    // The handshake section 10 describes, in order.
    let methods: Vec<String> = rig
        .written()
        .iter()
        .filter_map(|m| m["method"].as_str().map(String::from))
        .collect();
    assert_eq!(
        methods,
        [
            "initialize",
            "initialized",
            "account/read",
            "thread/start",
            "turn/start"
        ]
    );
    let start = rig
        .written()
        .into_iter()
        .find(|m| m["method"] == "thread/start")
        .unwrap();
    assert_eq!(start["params"]["developerInstructions"], emmet_agent::APP_POINTER);
    assert_eq!(start["params"]["cwd"], rig.project.display().to_string());
}

/// Codex asks before every MCP tool call (0B); Emmet accepts for its own servers without a card.
#[test]
fn tool_call() {
    let rig = rig("codex-tool", "tool-call");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Use the emmet server's ping tool with the message hello."))
        .unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!(rig.outline(&events));
    let answer = rig
        .written()
        .into_iter()
        .find(|m| m["id"] == 0 && m.get("result").is_some())
        .unwrap();
    assert_eq!(answer["result"]["action"], "accept");
}

#[test]
fn build_edits_model_py_and_runs_the_real_model() {
    let rig = rig("codex-build", "derived-build");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Make the row 10 bricks wide, then build it.")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!(rig.outline(&events));
    assert!(rig.model_py().contains("WIDTH = 10"));
    let placements: Value =
        serde_json::from_str(&std::fs::read_to_string(rig.project.join("build/placements.json")).unwrap()).unwrap();
    assert_eq!(placements["parts"].as_array().unwrap().len(), 10);
}

/// `on-request` with the sandbox as the guard (D-40): only the write outside the project asks, and declining it
/// leaves the outside file alone.
#[test]
fn permissions_on_request() {
    let rig = rig("codex-permissions", "permissions");
    let (mut s, mut ev) = rig.start(CODEX);
    let mut decisions = vec![Decision::Deny];
    let mut all = vec![];
    for text in [
        "Change WIDTH in model.py from 8 to 10. Reply done when finished.",
        "Read the file ../outside.txt and tell me its first line.",
        "Create a file notes.txt in the current directory containing the word hi. Reply done when finished.",
        "Append the line bye to the file ../outside.txt. Reply done when finished.",
        "Run the shell command `echo hi` and tell me its output.",
    ] {
        s.send(turn(text)).unwrap();
        all.extend(turn_answering(s.as_mut(), &mut ev, &mut decisions));
    }
    s.close();
    assert_json_snapshot!(rig.outline(&all));
    assert!(decisions.is_empty(), "one card, as 0B found");
    let declined = rig
        .written()
        .into_iter()
        .find(|m| m["result"]["decision"].is_string())
        .unwrap();
    assert_eq!(declined["result"]["decision"], "decline");
}

/// `untrusted` (0B, by protocol): every command and file change is a card. Accepted file changes are written.
#[test]
fn every_approval_kind_answered() {
    let rig = rig("codex-untrusted", "permissions-untrusted");
    let (mut s, mut ev) = rig.start(CODEX);
    use Decision::*;
    let mut decisions = vec![Deny, AllowOnce, Deny, AllowSession, Deny, Deny];
    let mut all = vec![];
    for text in [
        "Change WIDTH in model.py from 8 to 10. Reply done when finished.",
        "Read the file ../outside.txt and tell me its first line.",
        "Create a file notes.txt in the current directory containing the word hi. Reply done when finished.",
        "Append the line bye to the file ../outside.txt. Reply done when finished.",
        "Run the shell command `echo hi` and tell me its output.",
    ] {
        s.send(turn(text)).unwrap();
        all.extend(turn_answering(s.as_mut(), &mut ev, &mut decisions));
    }
    s.close();
    assert_json_snapshot!(rig.outline(&all));
    let kinds: Vec<ApprovalKind> = all
        .iter()
        .filter_map(|e| match e {
            AgentEvent::Approval { kind, .. } => Some(*kind),
            _ => None,
        })
        .collect();
    use ApprovalKind::*;
    assert_eq!(kinds, [Command, Edit, Command, Edit, Command, Command]);
    assert!(rig.model_py().contains("WIDTH = 10"));
    assert_eq!(std::fs::read_to_string(rig.project.join("notes.txt")).unwrap(), "hi\n");
    let answers: Vec<String> = rig
        .written()
        .iter()
        .filter_map(|m| m["result"]["decision"].as_str().map(String::from))
        .collect();
    assert_eq!(
        answers,
        ["decline", "accept", "decline", "acceptForSession", "decline", "decline"]
    );
}

#[test]
fn interrupt() {
    let rig = rig("codex-interrupt", "interrupt");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn(
        "Use the emmet server's block tool with seconds=60, then say done.",
    ))
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
    let i = rig
        .written()
        .into_iter()
        .find(|m| m["method"] == "turn/interrupt")
        .unwrap();
    assert!(i["params"]["turnId"].is_string() && i["params"]["threadId"].is_string());
}

#[test]
fn interrupt_then_a_new_turn() {
    let rig = rig("codex-interrupt-new", "interrupt-new-turn");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn(
        "Use the emmet server's block tool with seconds=60, then say done.",
    ))
    .unwrap();
    ev.until("the block call", is_tool("mcp__emmet__block"));
    s.interrupt().unwrap();
    let mut all = ev.turn();
    s.send(turn("Reply with exactly the word: after")).unwrap();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
}

/// Redirect, native path (ADR 0001): turn/steer during the tool call reaches the model in the same turn.
#[test]
fn a_redirect_delivered_natively_mid_turn() {
    let rig = rig("codex-steer", "mid-turn-steer");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn(
        "Use the emmet server's block tool with seconds=20. When it returns, reply with one short sentence.",
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
    assert_eq!(
        sent_texts(&rig, "turn/steer"),
        ["Also include the word banana in your reply."]
    );
    let steer = rig.written().into_iter().find(|m| m["method"] == "turn/steer").unwrap();
    assert!(steer["params"]["expectedTurnId"].is_string());
}

/// 0B tried a turn/start mid-turn too: Codex folds it into the running turn. Emmet sends turn/steer instead, with the
/// same result.
#[test]
fn a_mid_turn_message_folds_into_the_running_turn() {
    let rig = rig("codex-mid-start", "mid-turn-start");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn(
        "Use the emmet server's block tool with seconds=20. When it returns, reply with one short sentence.",
    ))
    .unwrap();
    ev.until("the block call", is_tool("mcp__emmet__block"));
    s.send(turn("Also include the word banana in your reply.")).unwrap();
    let events = ev.turn();
    s.close();
    assert!(text_of(&events).contains("banana"));
    assert_eq!(sent_texts(&rig, "turn/start").len(), 1, "no second turn");
}

#[test]
fn a_redirect_delivered_through_user_notes() {
    let mut rig = rig("codex-notes", "derived-build");
    rig.cfg.native_steer = false;
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Make the row 10 bricks wide, then build it.")).unwrap();
    let note = s.steer("Make it blue instead.").unwrap();
    let events = ev.turn();
    s.close();
    assert!(events.contains(&AgentEvent::NoteDelivered {
        id: note.id,
        path: SteerPath::Queued
    }));
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
}

#[test]
fn a_redirect_sent_as_the_next_turn() {
    let rig = rig("codex-next", "derived-next-turn");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    s.steer("And then say it again.").unwrap();
    let mut all = ev.turn();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
    assert_eq!(
        sent_texts(&rig, "turn/start"),
        ["Reply with exactly the word: ready", "And then say it again."]
    );
}

#[test]
fn a_redirect_still_pending_at_stop() {
    let mut rig = rig("codex-pending-stop", "interrupt-new-turn");
    rig.cfg.native_steer = false;
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn(
        "Use the emmet server's block tool with seconds=60, then say done.",
    ))
    .unwrap();
    s.steer("Reply with exactly the word: after").unwrap();
    ev.until("the block call", is_tool("mcp__emmet__block"));
    s.interrupt().unwrap();
    let mut all = ev.turn();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
    assert_eq!(
        sent_texts(&rig, "turn/start").last().unwrap(),
        "(You stopped the previous turn.)\n\nReply with exactly the word: after"
    );
}

#[test]
fn a_question_answered_in_the_same_turn() {
    let rig = rig("codex-ask", "derived-ask-user");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Build a wall.")).unwrap();
    let mut all = ev.until("the question", |e| matches!(e, AgentEvent::Question { .. }));
    let Some(AgentEvent::Question { id, .. }) = all.last().cloned() else {
        unreachable!()
    };
    s.answer(
        &id,
        Answer {
            choices: vec![],
            other: Some("Four bricks, please".into()),
        },
    )
    .unwrap();
    all.extend(ev.turn());
    s.close();
    assert_json_snapshot!(rig.outline(&all));
}

#[test]
fn a_question_canceled_by_stop() {
    let rig = rig("codex-ask-stop", "derived-ask-user-stopped");
    let (mut s, mut ev) = rig.start(CODEX);
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

#[test]
fn a_crash_mid_stream_then_resume() {
    let rig = rig("codex-crash", "derived-crash-mid-stream");
    let before = rig.model_py();
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Use the emmet server's ping tool with the message hello."))
        .unwrap();
    let events = ev.exit();
    assert_json_snapshot!(rig.outline(&events));
    assert_eq!(rig.model_py(), before);
    let resume = s.resume_token().unwrap();
    let mut cfg = rig.cfg.clone();
    cfg.resume = Some(resume.clone());
    let (mut s, mut ev) = rig.start_with(CODEX, cfg);
    s.send(turn("What was the code word? Reply with just the word."))
        .unwrap();
    let events = ev.turn();
    s.close();
    assert_eq!(
        events.last(),
        Some(&AgentEvent::TurnDone {
            reason: DoneReason::Completed
        })
    );
    let r = rig
        .written()
        .into_iter()
        .find(|m| m["method"] == "thread/resume")
        .unwrap();
    assert_eq!(r["params"]["threadId"], resume);
}

#[test]
fn a_malformed_line_is_skipped() {
    let rig = rig("codex-malformed", "derived-malformed-line");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!("plain_turn", rig.outline(&events));
}

#[test]
fn unknown_event_types_are_ignored() {
    let rig = rig("codex-unknown", "derived-unknown-event");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!("plain_turn", rig.outline(&events));
}

#[test]
fn resume_continues_the_thread() {
    let rig = rig("codex-resume", "resume");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Remember the code word teal. Reply with exactly: ok"))
        .unwrap();
    ev.turn();
    let thread = s.resume_token().unwrap();
    s.close();
    ev.exit();
    let mut cfg = rig.cfg.clone();
    cfg.resume = Some(thread.clone());
    let (mut s, mut ev) = rig.start_with(CODEX, cfg);
    s.send(turn("What was the code word? Reply with just the word."))
        .unwrap();
    let events = ev.turn();
    s.close();
    assert!(text_of(&events).contains("teal"));
    let r = rig
        .written()
        .into_iter()
        .find(|m| m["method"] == "thread/resume")
        .unwrap();
    assert_eq!(r["params"]["threadId"], thread);
}

/// 0B: Codex's default MCP tool timeout is 300 s (the call fails); emmet_host's is raised to an hour.
#[test]
fn tool_timeouts() {
    let r = rig("codex-block-default", "block-default");
    let (mut s, mut ev) = r.start(CODEX);
    s.send(turn("Use the emmet server's block tool with seconds=600."))
        .unwrap();
    let events = ev.turn();
    s.close();
    let failed = events.iter().find_map(|e| match e {
        AgentEvent::ToolResult { ok: false, summary, .. } => Some(summary.clone()),
        _ => None,
    });
    assert!(failed.unwrap().contains("timed out awaiting tools/call after 300s"));

    let r = rig("codex-block-raised", "block-raised");
    let (mut s, mut ev) = r.start(CODEX);
    s.send(turn("Use the emmet_host server's block tool with seconds=600."))
        .unwrap();
    let events = ev.turn();
    s.close();
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::ToolResult { ok: true, .. }))
    );
    let args = r.record("codex", 0)["args"].clone();
    assert!(
        args.as_array()
            .unwrap()
            .contains(&"mcp_servers.emmet_host.tool_timeout_sec=3600".into())
    );
}

#[test]
fn a_refused_key_is_one_error() {
    let rig = rig("codex-401", "auth-401");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert_json_snapshot!(rig.outline(&events));
    let errors: Vec<AgentErrorCode> = events
        .iter()
        .filter_map(|e| match e {
            AgentEvent::Error { code, .. } => Some(*code),
            _ => None,
        })
        .collect();
    assert_eq!(errors, [AgentErrorCode::AuthFailed], "the retries aren't errors");
}

#[test]
fn no_credit_is_quota_exceeded() {
    let rig = rig("codex-quota", "quota-exceeded");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert!(events.iter().any(|e| matches!(
        e,
        AgentEvent::Error {
            code: AgentErrorCode::QuotaExceeded,
            ..
        }
    )));
    assert_eq!(
        events.last(),
        Some(&AgentEvent::TurnDone {
            reason: DoneReason::Failed
        })
    );
}

/// D-35: Codex runs only on the key from the keychain, through Emmet's provider; without one it doesn't start, and
/// detection says how to add it.
#[test]
fn the_api_key_comes_from_the_keychain() {
    let rig = Rig::new("codex-no-key", "codex", "plain-turn");
    let d = rig.adapter(CODEX).detect();
    assert!(!d.signed_in);
    assert_eq!(
        d.fix.as_deref(),
        Some("Continue with ChatGPT, or add your OpenAI API key, in Settings → Agents.")
    );
    let err = rig.adapter(CODEX).start(rig.cfg.clone()).err().unwrap();
    assert_eq!(err.code, AgentErrorCode::NotSignedIn);

    rig.key(KEY);
    let d = rig.adapter(CODEX).detect();
    assert!(d.signed_in && d.supported);
    assert_eq!(d.version.as_deref(), Some("0.160.1"));
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    ev.turn();
    s.close();
    let rec = rig.record("codex", 0);
    let env: std::collections::BTreeMap<String, String> = serde_json::from_value(rec["env"].clone()).unwrap();
    assert_eq!(env.get(emmet_agent::codex::KEY_VAR).map(String::as_str), Some(KEY));
    let args: Vec<String> = serde_json::from_value(rec["args"].clone()).unwrap();
    for o in [
        "model_provider=\"emmet_api_key\"",
        "model_providers.emmet_api_key.env_key=\"OPENAI_API_KEY\"",
        "model_providers.emmet_api_key.requires_openai_auth=false",
        "sandbox_mode=\"workspace-write\"",
        "approval_policy=\"on-request\"",
        "sandbox_workspace_write.exclude_slash_tmp=true",
        "sandbox_workspace_write.exclude_tmpdir_env_var=true",
    ] {
        assert!(args.iter().any(|a| a == o), "missing -c {o}");
    }
    // The key never reaches the transcript.
    let transcript = std::fs::read_to_string(
        rig.project
            .join(".emmet/transcripts")
            .join(format!("{}.jsonl", rig.cfg.id)),
    )
    .unwrap();
    assert!(!transcript.contains(KEY));
    let usage = rig.adapter(CODEX).usage().unwrap();
    assert_eq!(usage.status, UsageStatus::ApiKey);
}

/// H-04, Codex 0.164 on Windows: under on-request every PowerShell command asks (even a read), while Emmet's own tool
/// calls are accepted without a card. Replayed from Barry's VM session, scrubbed.
#[test]
fn windows_every_command_asks() {
    let rig = rig("codex-windows", "windows-approvals");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("make it 10 bricks high")).unwrap();
    let mut decisions = vec![Decision::AllowOnce, Decision::AllowSession];
    let events = turn_answering(s.as_mut(), &mut ev, &mut decisions);
    s.close();
    assert_json_snapshot!(rig.outline(&events));
    assert!(decisions.is_empty(), "two command cards");
    let accepted = rig
        .written()
        .iter()
        .filter(|m| m["result"]["action"] == "accept")
        .count();
    assert_eq!(accepted, 5, "Emmet's five tool calls, accepted without a card");
}

/// What Emmet sends validates against the committed app-server schema, what it reads is in it, and the methods it
/// uses are the schema's: a Codex update that changes them fails here, not in a chat (plan section 10).
#[test]
fn codex_types_match_the_committed_schema() {
    for version in types::SCHEMAS {
        let dir = repo().join("crates/emmet-agent/schema/codex").join(version);
        let load = |f: &str| -> Value {
            serde_json::from_str(&std::fs::read_to_string(dir.join(f)).unwrap_or_else(|e| panic!("{f}: {e}"))).unwrap()
        };
        for (file, value) in types::sent_examples() {
            let problems = validate(&load(file), &value);
            assert!(
                problems.is_empty(),
                "{version} {file}: {value} doesn't validate:\n{}",
                problems.join("\n")
            );
        }
        for (file, path) in types::READS {
            let schema = load(file);
            assert!(resolves(&schema, &schema, path), "{version} {file} has no {path}");
        }
        let methods = |f: &str| -> Vec<String> {
            load(f)["oneOf"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|v| v["properties"]["method"]["enum"][0].as_str().map(String::from))
                .collect()
        };
        let client = methods("ClientRequest.json");
        for m in types::CLIENT_METHODS {
            assert!(client.contains(&m.to_string()), "{version}: no client request {m}");
        }
        let server = methods("ServerRequest.json");
        for m in types::SERVER_REQUESTS {
            assert!(server.contains(&m.to_string()), "{version}: no server request {m}");
        }
        assert!(methods("ClientNotification.json").contains(&"initialized".to_string()));
        let statuses = load("v2/TurnCompletedNotification.json")["definitions"]["TurnStatus"]["enum"].clone();
        for st in types::TURN_STATUSES {
            assert!(
                statuses.as_array().unwrap().contains(&(*st).into()),
                "{version}: no turn status {st}"
            );
        }
    }
}

/// Follows `path` (see types::READS) from `schema`, through `$ref`s and unions.
fn resolves(root: &Value, schema: &Value, path: &str) -> bool {
    let Some((step, rest)) = path.split_once('.').map(|(a, b)| (a, Some(b))).or(Some((path, None))) else {
        return false;
    };
    for s in candidates(root, schema) {
        let next = if let Some(tag) = step.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
            let t = &s["properties"]["type"];
            let matches = t["enum"].as_array().is_some_and(|e| e.contains(&tag.into())) || t["const"] == tag;
            matches.then(|| s.clone())
        } else {
            s["properties"].get(step).cloned()
        };
        if let Some(n) = next {
            match rest {
                None => return true,
                Some(r) => {
                    if resolves(root, &n, r) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// The schema itself and everything it stands for: its `$ref` target and each `allOf`, `anyOf`, and `oneOf` branch.
fn candidates(root: &Value, schema: &Value) -> Vec<Value> {
    let mut out = vec![];
    let mut todo = vec![schema.clone()];
    while let Some(s) = todo.pop() {
        if let Some(r) = s["$ref"].as_str().and_then(|r| r.strip_prefix("#/"))
            && let Some(t) = r.split('/').try_fold(root, |v, k| v.get(k))
        {
            todo.push(t.clone());
        }
        for key in ["allOf", "anyOf", "oneOf"] {
            for b in s[key].as_array().into_iter().flatten() {
                todo.push(b.clone());
            }
        }
        out.push(s);
    }
    out
}

/// A finding sent by "Ask agent to fix" (1D): the agent validates, gets broken-collision's real F-0001, edits the line
/// it names, runs, and validates clean. Derived from the recordings until a live run is recorded (plan section 21).
#[test]
fn a_collision_finding_is_fixed_at_its_line() {
    let rig = rig("codex-fix-collision", "derived-fix-collision");
    let broken = emmet_it::fixture("projects/broken-collision/model.py");
    std::fs::copy(&broken, rig.project.join("model.py")).unwrap();
    let (mut s, mut ev) = rig.start(CODEX);
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

// ------------------------------------------------------------------------------------- Codex on a ChatGPT plan (1E)

const PLAN_ID: &str = "oaiapp_test1";

/// A rig signed in to fake-openai's account, with ChatGPT chosen for Codex, as Continue with ChatGPT leaves it.
fn plan_rig(name: &str, scenario: &str) -> (emmet_it::chatgpt::Plan, Rig) {
    let plan = emmet_it::chatgpt::Plan::new(name);
    plan.signed_in(None);
    let rig = Rig::new(name, "codex", scenario).with_plan(&plan);
    (plan, rig)
}

fn started_env(rig: &Rig, n: usize) -> std::collections::BTreeMap<String, String> {
    serde_json::from_value(rig.record("codex", n)["env"].clone()).unwrap()
}

/// OpenAI's Codex app-server page: its provider block word for word, the access token in ACCESS_TOKEN, and a
/// clientInfo.name that matches the registration's agent_name_hint.
#[test]
fn plan_turn() {
    let (plan, rig) = plan_rig("codex-plan-turn", "plain-turn");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    assert!(
        matches!(&events[0], AgentEvent::Started { billing: Some(b), .. } if *b == emmet_agent::codex::plan_billing())
    );
    assert!(matches!(
        events.last(),
        Some(AgentEvent::TurnDone {
            reason: DoneReason::Completed
        })
    ));
    let args: Vec<String> = serde_json::from_value(rig.record("codex", 0)["args"].clone()).unwrap();
    let provider: Vec<&str> = args
        .iter()
        .filter(|a| a.starts_with("model_provider"))
        .map(String::as_str)
        .collect();
    assert_eq!(
        provider,
        [
            "model_provider=\"openai_chatgpt_plan\"",
            "model_providers.openai_chatgpt_plan.name=\"ChatGPT plan\"",
            "model_providers.openai_chatgpt_plan.base_url=\"https://api.openai.com/v1\"",
            "model_providers.openai_chatgpt_plan.env_key=\"ACCESS_TOKEN\"",
            "model_providers.openai_chatgpt_plan.wire_api=\"responses\"",
            "model_providers.openai_chatgpt_plan.requires_openai_auth=false",
            "model_providers.openai_chatgpt_plan.supports_websockets=false",
        ]
    );
    let token = plan.record(PLAN_ID).unwrap()["access_token"].clone();
    assert_eq!(
        started_env(&rig, 0).get(emmet_agent::codex::TOKEN_VAR).cloned(),
        token.as_str().map(String::from)
    );
    let init = rig.written().into_iter().find(|m| m["method"] == "initialize").unwrap();
    assert_eq!(init["params"]["clientInfo"]["name"], "Emmet");
    let d = rig.adapter(CODEX).detect();
    assert!(d.signed_in && d.billing == emmet_agent::codex::plan_billing(), "{d:?}");
}

/// OpenAI's renewal rule: a renewed token restarts the app-server with the new ACCESS_TOKEN, which initializes and
/// resumes the saved thread. The chat sees one session: the old process's exit isn't passed on.
#[test]
fn plan_token_renews_between_turns() {
    let (plan, rig) = plan_rig("codex-plan-renew", "resume");
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let mut events = ev.turn();
    let thread = s.resume_token().expect("a thread");
    let first = plan.record(PLAN_ID).unwrap()["access_token"].clone();
    // Time passes: the token has 10 minutes left, less than a turn may need.
    plan.edit_record(PLAN_ID, |r| {
        r["expires_at"] = json!(emmet_agent::time::now_ms() as i64 / 1000 + 600)
    });
    s.send(turn("Reply with exactly the word: again")).unwrap();
    events.extend(ev.turn());
    s.close();
    let renewed = plan.record(PLAN_ID).unwrap()["access_token"].clone();
    assert_ne!(renewed, first);
    let env = |n| {
        started_env(&rig, n)
            .get(emmet_agent::codex::TOKEN_VAR)
            .cloned()
            .map(Value::from)
    };
    assert_eq!(
        (env(0), env(1)),
        (Some(first), Some(renewed)),
        "a new app-server on the new token"
    );
    let resumed = rig
        .written()
        .into_iter()
        .find(|m| m["method"] == "thread/resume")
        .expect("thread/resume");
    assert_eq!(resumed["params"]["threadId"], thread.as_str());
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, AgentEvent::Exited { .. } | AgentEvent::Error { .. })),
        "the restart is invisible: {events:?}"
    );
    let done = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                AgentEvent::TurnDone {
                    reason: DoneReason::Completed
                }
            )
        })
        .count();
    assert_eq!(done, 2);
}

/// OpenAI's usage-limit error: one error with OpenAI's recovery and the response's own code and request id, the turn
/// ends, and nothing restarts or switches to another way of paying.
#[test]
fn plan_usage_limit() {
    let (_plan, rig) = plan_rig("codex-plan-limit", "derived-plan-usage-limit");
    rig.key(KEY);
    let (mut s, mut ev) = rig.start(CODEX);
    s.send(turn("Reply with exactly the word: ready")).unwrap();
    let events = ev.turn();
    s.close();
    let errors: Vec<&AgentEvent> = events
        .iter()
        .filter(|e| matches!(e, AgentEvent::Error { .. }))
        .collect();
    let [AgentEvent::Error { code, message }] = errors.as_slice() else {
        panic!("one error: {events:?}")
    };
    assert_eq!(*code, AgentErrorCode::QuotaExceeded);
    assert!(message.starts_with(emmet_agent::codex::LIMIT), "{message}");
    assert!(message.contains("subscription_sharing_usage_limit_exceeded") && message.contains("req_plan_limit_1"));
    assert!(!message.contains("resets"), "no reset time is inferred");
    assert!(matches!(
        events.last(),
        Some(AgentEvent::TurnDone {
            reason: DoneReason::Failed
        })
    ));
    assert!(!rig.fake.join("codex-1.json").exists(), "no restart");
    assert!(
        !started_env(&rig, 0).contains_key(emmet_agent::codex::KEY_VAR),
        "the API key isn't used"
    );
}
