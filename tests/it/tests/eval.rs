// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet eval --rung 1 --dry-run` with each fake agent (plan section 16.1, 1D's Done when 3): the fake replays a
//! derived rung 1 transcript through the real `emmet mcp` and `emmet mcp-host`, the eval answers the style question
//! from the rung's script and approves only Emmet's tools and model.py, then scores the run and writes the report.
//! Both agents run in one test, one after the other: each run holds the data dir's bus.

use emmet_it::agents::fake_bin;
use emmet_it::{emmet, envelope, shared_data, temp_dir};

#[test]
fn a_dry_run_scores_rung_1_with_each_fake_agent() {
    for cli in ["claude", "codex"] {
        let dir = temp_dir(&format!("eval-{cli}"));
        let fake = dir.join("fake");
        let (out, work) = (dir.join("out"), dir.join("work"));
        std::fs::create_dir_all(&fake).unwrap();
        let transcript = emmet_it::repo().join(format!("tests/replay/{cli}/derived-rung1.jsonl"));
        std::fs::write(fake.join(format!("{cli}.scenario")), transcript.display().to_string()).unwrap();
        // Codex reads its key from the keychain; tests use a file instead (EMMET_FAKE_KEYCHAIN, debug builds only).
        std::fs::write(fake.join("keychain"), "sk-test-not-a-key").unwrap();
        let run = emmet()
            .arg("--json")
            .arg("--data-dir")
            .arg(shared_data())
            .args(["eval", "--rung", "1", "--agent", cli, "--dry-run", "--out"])
            .arg(&out)
            .arg("--work")
            .arg(&work)
            .env("EMMET_FAKE_DIR", &fake)
            .env("EMMET_CLAUDE_BIN", fake_bin("claude"))
            .env("EMMET_CODEX_BIN", fake_bin("codex"))
            .env("EMMET_FAKE_KEYCHAIN", fake.join("keychain"))
            .output()
            .unwrap();
        let (v, code) = envelope(&run);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert_eq!(code, 0, "{cli}: {v}\n{stderr}");
        let data = &v["data"];
        assert_eq!(data["pass"], true, "{cli}: {data:#}");
        let scores: Vec<(String, bool)> = data["scores"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| (s["name"].as_str().unwrap().to_string(), s["pass"] == true))
            .collect();
        assert_eq!(
            scores.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            [
                "Runs and validates with 0 errors",
                "Size and features",
                "Parameters",
                "Availability",
                "Budget"
            ]
        );
        // The question got the script's "style" answer, which names the first option.
        assert_eq!(
            data["run"]["questions"][0],
            serde_json::json!([
                "Which style should the cottage have?",
                "Classic: tan walls and a dark red roof"
            ]),
            "{cli}"
        );
        assert_eq!(data["run"]["ended"], "completed");
        // Claude: two ToolSearch, ask_user, Write, run_model, validate. Codex: ask_user, the file change, run_model,
        // validate.
        let turns = if cli == "claude" { 6 } else { 4 };
        assert_eq!(data["run"]["turns"], turns, "{cli}: {data:#}");
        let model = std::fs::read_to_string(work.join(format!("rung1-{cli}/model.py"))).unwrap();
        assert!(model.contains("def cottage("), "{cli}: the fake wrote the cottage");
        // The report: today's file, this agent's section, its scores and question.
        let report = std::fs::read_dir(&out)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.extension().is_some_and(|x| x == "md"))
            .expect("a report");
        let text = std::fs::read_to_string(&report).unwrap();
        let label = if cli == "claude" { "Claude Code" } else { "Codex" };
        assert!(text.contains(&format!("## {label} — pass")), "{text}");
        assert!(text.contains("| Parameters | yes | pass |"), "{text}");
        assert!(
            text.contains("\"Which style should the cottage have?\" → Classic"),
            "{text}"
        );
        assert!(text.starts_with("# Live eval: Rung 1: Cottage"), "{text}");
    }
}

#[test]
fn a_rung_without_an_eval_yet_is_a_usage_error() {
    let (v, code) = envelope(
        &emmet()
            .arg("--json")
            .arg("--data-dir")
            .arg(shared_data())
            .args(["eval", "--rung", "3", "--agent", "claude", "--dry-run"])
            .output()
            .unwrap(),
    );
    assert_eq!((code, v["error"]["code"].as_str()), (3, Some("PARAM_INVALID")), "{v}");
}
