// SPDX-License-Identifier: AGPL-3.0-or-later
//! ADR 0006 spike. Both binaries run [`serve`]: a line-oriented echo that answers a JSON-RPC style `initialize`
//! the way a stdio MCP server would, and reports whether the process can reach a console.

use std::io::{BufRead, Write};

/// Whether this process has a console to print to: opening `CONOUT$` only works when one is attached.
pub fn has_console() -> bool {
    cfg!(windows) && std::fs::OpenOptions::new().write(true).open("CONOUT$").is_ok()
}

/// Reads lines from stdin; answers `{"id":N,"method":"initialize"}` and `console?`, echoes anything else.
pub fn serve(kind: &str) {
    let stdin = std::io::stdin();
    let mut out = std::io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let reply = if line.contains("\"initialize\"") {
            let id = line
                .split("\"id\":")
                .nth(1)
                .and_then(|s| s.split([',', '}']).next())
                .unwrap_or("null");
            format!(r#"{{"id":{id},"result":{{"serverInfo":{{"name":"{kind}"}}}}}}"#)
        } else if line == "console?" {
            format!("console={}", has_console())
        } else {
            line
        };
        if writeln!(out, "{reply}").and_then(|()| out.flush()).is_err() {
            break;
        }
    }
}
