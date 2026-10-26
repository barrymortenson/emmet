// SPDX-License-Identifier: AGPL-3.0-or-later
//! What the app's bus server hands to the chat sessions (plan section 10): `emmet_host approve` (Claude Code's
//! permission prompt), `emmet_host ask_user` (question cards), and `emmet mcp --session` taking redirect notes. One
//! hub serves every session in the app; the bus handler calls [`Hub::approve`], [`Hub::ask`], and
//! [`Hub::take_notes`], each of which answers `None` for a session it doesn't know.
//!
//! `approve` and `ask` block the bus connection's thread until the user decides, Stop cancels, or the session ends.

use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use crate::{AgentEvent, Answer, ApprovalKind, Decision, Question, SteerPath, UserNote};

#[derive(Clone, Default)]
pub struct Hub(Arc<Mutex<HashMap<String, Entry>>>);

struct Entry {
    /// Which registration this is: only its own engine may unregister it.
    owner: u64,
    events: Sender<AgentEvent>,
    approvals: HashMap<String, Sender<Decision>>,
    /// What the user allowed for the rest of the session, by [`scope`]: never a whole shell or the whole web.
    allowed: HashSet<String>,
    question: Option<(String, Sender<Value>)>,
    /// Notes waiting for the next Emmet tool result (the queued path).
    notes: Vec<UserNote>,
    next: u64,
}

impl Hub {
    fn with<T>(&self, session: &str, f: impl FnOnce(&mut Entry) -> T) -> Option<T> {
        self.0.lock().expect("hub").get_mut(session).map(f)
    }

    /// `emmet_host approve`: an approval card for the session, held until the user decides. Answers Claude Code's
    /// `{behavior, message?}` (mcp-host adds `updatedInput`).
    pub fn approve(&self, session: &str, request: Value) -> Option<Value> {
        let tool = request["tool_name"].as_str().unwrap_or("a tool").to_string();
        let key = scope(&tool, &request["input"]);
        let (tx, rx) = channel();
        let allowed = self.with(session, |e| {
            if e.allowed.contains(&key) {
                return true;
            }
            e.next += 1;
            let id = format!("a{}", e.next);
            e.approvals.insert(id.clone(), tx);
            let (kind, detail) = describe(&tool, &request["input"]);
            let _ = e.events.send(AgentEvent::Approval {
                id,
                kind,
                tool: tool.clone(),
                detail,
            });
            false
        })?;
        if allowed {
            return Some(json!({ "behavior": "allow" }));
        }
        Some(match rx.recv() {
            Ok(Decision::AllowOnce) => json!({ "behavior": "allow" }),
            Ok(Decision::AllowSession) => {
                self.with(session, |e| e.allowed.insert(key));
                json!({ "behavior": "allow" })
            }
            Ok(Decision::Deny) => json!({ "behavior": "deny", "message": "The user denied this in Emmet." }),
            // Stop, or the session ended.
            Err(_) => json!({ "behavior": "deny", "message": "The user stopped the turn." }),
        })
    }

    /// `emmet_host ask_user`: a question card, held until the user answers or Stop cancels it. Answers
    /// `{answered, choices, other?}` or `{answered: false, reason}`.
    pub fn ask(&self, session: &str, question: Value) -> Option<Value> {
        let q: Question = serde_json::from_value(question).ok()?;
        let (tx, rx) = channel();
        self.with(session, |e| {
            e.next += 1;
            let id = format!("q{}", e.next);
            // One question at a time: a new one closes the old.
            if let Some((old, waiting)) = e.question.replace((id.clone(), tx)) {
                let _ = waiting.send(json!({ "answered": false, "reason": "stopped" }));
                let _ = e.events.send(AgentEvent::QuestionClosed {
                    id: old,
                    reason: Some("stopped".into()),
                });
            }
            let _ = e.events.send(AgentEvent::Question { id, question: q });
        })?;
        Some(
            rx.recv()
                .unwrap_or_else(|_| json!({ "answered": false, "reason": "stopped" })),
        )
    }

    /// `emmet mcp --session`: the notes waiting for this tool result. Each is marked delivered.
    pub fn take_notes(&self, session: &str) -> Option<Value> {
        self.with(session, |e| {
            let notes = std::mem::take(&mut e.notes);
            for n in &notes {
                let _ = e.events.send(AgentEvent::NoteDelivered {
                    id: n.id.clone(),
                    path: SteerPath::Queued,
                });
            }
            serde_json::to_value(notes).expect("notes serialize")
        })
    }

    // The engine's side.

    /// Registers the session's engine, and returns its owner token for [`Hub::unregister`].
    pub(crate) fn register(&self, session: &str, events: Sender<AgentEvent>) -> u64 {
        static OWNER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let owner = OWNER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.0.lock().expect("hub").insert(
            session.to_string(),
            Entry {
                owner,
                events,
                approvals: HashMap::new(),
                allowed: HashSet::new(),
                question: None,
                notes: vec![],
                next: 0,
            },
        );
        owner
    }

    /// Forgets the session, if `owner` still registers it: a ChatGPT plan session restarted on a renewed token
    /// (1E) registers the same id again, and the old engine's late exit mustn't remove the new one. Anything still
    /// waiting on it is answered as stopped (its senders drop).
    pub(crate) fn unregister(&self, session: &str, owner: u64) {
        let mut all = self.0.lock().expect("hub");
        if all.get(session).is_some_and(|e| e.owner == owner) {
            all.remove(session);
        }
    }

    /// True if the approval was one of the hub's (Claude's) and is now decided.
    pub(crate) fn decide(&self, session: &str, id: &str, d: Decision) -> bool {
        self.with(session, |e| e.approvals.remove(id).map(|tx| tx.send(d).is_ok()))
            .flatten()
            .unwrap_or(false)
    }

    pub(crate) fn answer(&self, session: &str, id: &str, a: &Answer) -> bool {
        self.with(session, |e| match e.question.take() {
            Some((qid, tx)) if qid == id => {
                let _ = tx.send(json!({ "answered": true, "choices": a.choices, "other": a.other }));
                let _ = e.events.send(AgentEvent::QuestionClosed { id: qid, reason: None });
                true
            }
            other => {
                e.question = other;
                false
            }
        })
        .unwrap_or(false)
    }

    /// Stop: the open question is answered "stopped" and every approval card is denied, so the agent's tool calls
    /// return and the interrupt can land.
    pub(crate) fn stop(&self, session: &str) {
        self.with(session, |e| {
            if let Some((id, tx)) = e.question.take() {
                let _ = tx.send(json!({ "answered": false, "reason": "stopped" }));
                let _ = e.events.send(AgentEvent::QuestionClosed {
                    id,
                    reason: Some("stopped".into()),
                });
            }
            for (id, _tx) in e.approvals.drain() {
                let _ = e.events.send(AgentEvent::ApprovalClosed { id });
            }
        });
    }

    pub(crate) fn queue_note(&self, session: &str, note: UserNote) {
        self.with(session, |e| e.notes.push(note));
    }

    /// Takes back every queued note (the turn ended before a tool result took them).
    pub(crate) fn drain_notes(&self, session: &str) -> Vec<UserNote> {
        self.with(session, |e| std::mem::take(&mut e.notes)).unwrap_or_default()
    }

    /// Changes or (with `None`) withdraws a queued note. False if it's gone.
    pub(crate) fn edit_note(&self, session: &str, id: &str, text: Option<&str>) -> bool {
        self.with(session, |e| {
            let Some(i) = e.notes.iter().position(|n| n.id == id) else {
                return false;
            };
            match text {
                Some(t) => e.notes[i].text = t.to_string(),
                None => {
                    e.notes.remove(i);
                }
            }
            true
        })
        .unwrap_or(false)
    }
}

/// What "Allow for this session" covers: the exact command for a shell, the host for a fetch, the file for an edit,
/// and the tool itself for anything else, so allowing one `ls` never allows every command.
fn scope(tool: &str, input: &Value) -> String {
    let s = |k: &str| input[k].as_str().unwrap_or_default();
    let what = match tool {
        "Bash" | "PowerShell" => s("command").trim().to_string(),
        "WebFetch" => {
            let url = s("url");
            let rest = url.split_once("://").map_or(url, |(_, r)| r);
            rest.split(['/', '?', '#']).next().unwrap_or_default().to_lowercase()
        }
        "WebSearch" => s("query").to_string(),
        "NotebookEdit" => s("notebook_path").to_string(),
        "Edit" | "Write" | "MultiEdit" => s("file_path").to_string(),
        _ => String::new(),
    };
    format!("{tool}\u{0}{what}")
}

/// The card's kind and one line for Claude Code's request.
fn describe(tool: &str, input: &Value) -> (ApprovalKind, String) {
    let s = |k: &str| input[k].as_str().unwrap_or_default().to_string();
    match tool {
        "Bash" | "PowerShell" => (ApprovalKind::Command, s("command")),
        "NotebookEdit" => (ApprovalKind::Edit, s("notebook_path")),
        "Edit" | "Write" | "MultiEdit" => (ApprovalKind::Edit, s("file_path")),
        "WebFetch" => (ApprovalKind::Permission, s("url")),
        "WebSearch" => (ApprovalKind::Permission, s("query")),
        _ => (ApprovalKind::Tool, serde_json::to_string(input).unwrap_or_default()),
    }
}
