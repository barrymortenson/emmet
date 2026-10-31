// SPDX-License-Identifier: AGPL-3.0-or-later
//! Transcript capture (plan section 10): every line of a session's protocol stream, both ways, in the project's
//! `.emmet/transcripts/<session>.jsonl` (local, never shared). The lines are the `{t, dir, msg}` of 0B's recordings,
//! so `scripts/scrub-transcript` turns a live session into a replay fixture for tests/replay/. The session's secrets
//! (an API key, a ChatGPT access token) are replaced before a line is written, wherever the CLI echoed them within
//! that line. A secret split across two streamed deltas would get through, which is why Codex is also told never to
//! pass its credential variables to the agent's commands (codex.rs).

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use serde_json::{Value, json};

pub struct Transcript {
    out: Option<BufWriter<File>>,
    started: Instant,
    secrets: Vec<String>,
}

/// What a secret becomes in a transcript, an event, or an error.
pub const REDACTED: &str = "<redacted>";

/// `s` with each of `secrets` replaced, or None if none is in it.
pub fn redact(s: &str, secrets: &[String]) -> Option<String> {
    let found: Vec<&String> = secrets
        .iter()
        .filter(|k| !k.is_empty() && s.contains(k.as_str()))
        .collect();
    if found.is_empty() {
        return None;
    }
    Some(
        found
            .iter()
            .fold(s.to_string(), |acc, k| acc.replace(k.as_str(), REDACTED)),
    )
}

impl Transcript {
    /// Opens (appends to) the session's transcript. A transcript that can't be written isn't fatal: the chat goes on
    /// without one.
    pub fn open(project: &Path, session: &str, meta: Value, secrets: Vec<String>) -> Self {
        // Never through a link the project carries (it may be someone else's).
        let out = emmet_core::fsx::own_file(project, &[".emmet", "transcripts"], &format!("{session}.jsonl"))
            .and_then(|path| std::fs::OpenOptions::new().create(true).append(true).open(path).ok())
            .map(BufWriter::new);
        let mut t = Self {
            out,
            started: Instant::now(),
            secrets,
        };
        t.write("note", json!({ "meta": meta }));
        t
    }

    /// One line: `dir` is "in" (to the CLI), "out" (from it), "err" (its stderr), "bad" (a line that isn't JSON),
    /// or "note".
    pub fn write(&mut self, dir: &str, msg: Value) {
        let Some(out) = self.out.as_mut() else { return };
        let line = json!({ "t": self.started.elapsed().as_millis() as u64, "dir": dir, "msg": msg }).to_string();
        let line = redact(&line, &self.secrets).unwrap_or(line);
        let ok = writeln!(out, "{line}").and_then(|_| out.flush());
        if ok.is_err() {
            self.out = None;
        }
    }
}
