// SPDX-License-Identifier: AGPL-3.0-or-later
//! ADR 0006: on Windows, both subsystems serve stdio over pipes (how an agent spawns an MCP server), but only the
//! console-subsystem exe has a console to print to when run from a terminal. Elsewhere the subsystem doesn't exist,
//! so the pipe test runs everywhere and the console test only on Windows.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

fn exchange(bin: &str, lines: &[&str]) -> Vec<String> {
    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn");
    let mut stdin = child.stdin.take().unwrap();
    for l in lines {
        writeln!(stdin, "{l}").unwrap();
    }
    drop(stdin);
    let out = BufReader::new(child.stdout.take().unwrap())
        .lines()
        .map(Result::unwrap)
        .collect();
    assert!(child.wait().unwrap().success());
    out
}

#[test]
fn both_subsystems_answer_over_pipes() {
    for (bin, kind) in [
        (env!("CARGO_BIN_EXE_console-echo"), "console"),
        (env!("CARGO_BIN_EXE_gui-echo"), "gui"),
    ] {
        let out = exchange(bin, &[r#"{"id":1,"method":"initialize"}"#, "ping"]);
        assert_eq!(
            out,
            [
                format!(r#"{{"id":1,"result":{{"serverInfo":{{"name":"{kind}"}}}}}}"#),
                "ping".to_string()
            ],
            "{kind}"
        );
    }
}

#[cfg(windows)]
#[test]
fn only_the_console_subsystem_has_a_console() {
    let console = exchange(env!("CARGO_BIN_EXE_console-echo"), &["console?"]);
    let gui = exchange(env!("CARGO_BIN_EXE_gui-echo"), &["console?"]);
    eprintln!("console-echo: {console:?}, gui-echo: {gui:?}");
    assert_eq!(console, ["console=true"]);
    assert_eq!(gui, ["console=false"]);
}
