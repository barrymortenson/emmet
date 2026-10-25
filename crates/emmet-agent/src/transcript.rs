// SPDX-License-Identifier: AGPL-3.0-or-later
//! Transcript capture (plan section 10): every line of a session's protocol stream, both ways, in the project's
//! `.emmet/transcripts/<session>.jsonl` (local, never shared). The lines are the `{t, dir, msg}` of 0B's recordings,
//! so `scripts/scrub-transcript` turns a live session into a replay fixture for tests/replay/.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

use serde_json::{Value, json};

pub struct Transcript {
    out: Option<BufWriter<File>>,
    started: Instant,
}

impl Transcript {
    /// Opens (appends to) the session's transcript. A transcript that can't be written isn't fatal: the chat goes on
    /// without one.
    pub fn open(project: &Path, session: &str, meta: Value) -> Self {
        let dir = project.join(".emmet").join("transcripts");
        let out = std::fs::create_dir_all(&dir)
            .and_then(|_| {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(dir.join(format!("{session}.jsonl")))
            })
            .ok()
            .map(BufWriter::new);
        let mut t = Self {
            out,
            started: Instant::now(),
        };
        t.write("note", json!({ "meta": meta }));
        t
    }

    /// One line: `dir` is "in" (to the CLI), "out" (from it), "err" (its stderr), "bad" (a line that isn't JSON),
    /// or "note".
    pub fn write(&mut self, dir: &str, msg: Value) {
        let Some(out) = self.out.as_mut() else { return };
        let line = json!({ "t": self.started.elapsed().as_millis() as u64, "dir": dir, "msg": msg });
        let ok = writeln!(out, "{line}").and_then(|_| out.flush());
        if ok.is_err() {
            self.out = None;
        }
    }
}
