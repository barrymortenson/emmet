// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet eval --rung 1 --dry-run` with each fake agent (plan section 16.1, 1D's Done when 3): the fake replays a
//! derived rung 1 transcript through the real `emmet mcp` and `emmet mcp-host`, the eval answers the style question
//! from the rung's script and approves only Emmet's tools and model.py, then scores the run and writes the report.
//! Both agents run in one test, one after the other: each run holds the data dir's bus.

use std::path::PathBuf;

use emmet_it::agents::fake_bin;
use emmet_it::{emmet, envelope, eval_data, temp_dir};

/// Each run holds the eval data dir's bus, so the tests that run one take turns.
static BUS: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn a_dry_run_scores_rung_1_with_each_fake_agent() {
    let _bus = BUS.lock().unwrap_or_else(|e| e.into_inner());
    for cli in ["claude", "codex"] {
        let dir = temp_dir(&format!("eval-{cli}"));
        let fake = dir.join("fake");
        let (out, work) = (dir.join("out"), dir.join("work"));
        std::fs::create_dir_all(&fake).unwrap();
        let transcript = emmet_it::repo().join(format!("tests/replay/{cli}/derived-rung1.jsonl"));
        // The fixture's prompt is rung 1's alone (derive.mjs once took every quoted line after it, the later rungs too).
        let fixture = std::fs::read_to_string(&transcript).unwrap();
        let first = fixture
            .lines()
            .find(|l| l.contains(r#""dir":"in""#) && (l.contains("turn/start") || l.contains(r#""type":"user""#)))
            .unwrap();
        assert!(first.contains("Build a small cottage"), "{cli}: {first}");
        assert!(
            !first.contains("train station") && !first.contains("Build this"),
            "{cli}: the fixture's prompt is rung 1's alone"
        );
        std::fs::write(fake.join(format!("{cli}.scenario")), transcript.display().to_string()).unwrap();
        // Codex reads its key from the keychain; tests use a file instead (EMMET_FAKE_KEYCHAIN, debug builds only).
        std::fs::write(fake.join("keychain"), "sk-test-not-a-key").unwrap();
        let run = emmet()
            .arg("--json")
            .arg("--data-dir")
            .arg(eval_data())
            .args(["eval", "--rung", "1", "--agent", cli, "--dry-run", "--out"])
            .arg(&out)
            .arg("--work")
            .arg(&work)
            .env("EMMET_FAKE_DIR", &fake)
            .env("EMMET_CLAUDE_BIN", fake_bin("claude"))
            .env("EMMET_CODEX_BIN", fake_bin("codex"))
            .env("EMMET_FAKE_KEYCHAIN", fake.join("keychain"))
            // No render mode: a desktop build beside emmet would open a window and wait out its timeout (200 s here).
            .env("EMMET_APP", "/nonexistent/emmet-app")
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
fn a_rung_that_isnt_on_the_ladder_is_a_usage_error() {
    let (v, code) = envelope(
        &emmet()
            .arg("--json")
            .arg("--data-dir")
            .arg(eval_data())
            .args(["eval", "--rung", "4", "--agent", "claude", "--dry-run"])
            .output()
            .unwrap(),
    );
    assert_eq!((code, v["error"]["code"].as_str()), (3, Some("PARAM_INVALID")), "{v}");
}

/// The chat the app keeps for a hand session, written for a test: the prompt at 0 s, `calls` tool calls a few seconds
/// apart, and the turn done at `seconds`.
fn hand_session(project: &std::path::Path, cli: &str, prompt: &str, calls: usize, seconds: u64) {
    let chat = project.join(".emmet/chat");
    std::fs::create_dir_all(&chat).unwrap();
    std::fs::write(
        chat.join("sessions.json"),
        format!(r#"{{"sessions":[{{"id":"s1","kind":"{cli}","started":0}}]}}"#),
    )
    .unwrap();
    let t0 = 1_800_000_000_000.0_f64;
    let mut records = vec![
        serde_json::json!({ "at": t0, "user": prompt }),
        serde_json::json!({ "at": t0 + 1_000.0, "event": { "type": "started", "resume": "r", "model": "the-model", "billing": { "kind": "plan", "name": "max" } } }),
    ];
    for i in 0..calls {
        let at = t0 + 2_000.0 + i as f64 * 3_000.0;
        records.push(serde_json::json!({ "at": at, "event": { "type": "tool_call", "id": format!("t{i}"), "name": "mcp__emmet__run_model", "input": {} } }));
        records.push(serde_json::json!({ "at": at + 1_000.0, "event": { "type": "tool_result", "id": format!("t{i}"), "ok": true, "summary": "built" } }));
    }
    let end = t0 + seconds as f64 * 1000.0;
    records.push(serde_json::json!({ "at": end - 1_000.0, "event": { "type": "text", "message": "m1", "delta": "It's built." } }));
    records.push(serde_json::json!({ "at": end, "event": { "type": "turn_done", "reason": "completed" } }));
    std::fs::write(
        chat.join("s1.jsonl"),
        records.iter().map(|r| format!("{r}\n")).collect::<String>(),
    )
    .unwrap();
}

/// `emmet eval --rung <rung> --agent <cli> --score <project>`: the envelope and the exit code. Nothing may start an
/// agent or open a window.
fn score_rung(
    data: &std::path::Path,
    rung: &str,
    cli: &str,
    project: &std::path::Path,
    out: &std::path::Path,
) -> (serde_json::Value, i32) {
    let run = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(data)
        .args(["eval", "--rung", rung, "--agent", cli, "--score"])
        .arg(project)
        .arg("--out")
        .arg(out)
        .env("EMMET_APP", "/nonexistent/emmet-app")
        .env("EMMET_CLAUDE_BIN", "/nonexistent/claude")
        .env("EMMET_CODEX_BIN", "/nonexistent/codex")
        .output()
        .unwrap();
    envelope(&run)
}

fn score_detail(v: &serde_json::Value, name: &str) -> (bool, String) {
    let s = v["data"]["scores"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == name)
        .unwrap_or_else(|| panic!("no score '{name}': {v:#}"));
    (s["pass"] == true, s["detail"].as_str().unwrap().to_string())
}

/// Rung 2 (5A): Barry's two hand sessions of 4B, the projects they left, scored as G2 scores one. The track loop
/// closes through its connectors, the station names its platform and canopy, and five or more knobs change the model,
/// the platform's length among them; Codex's station is short of rung 2's size and fails on that alone. The Cottage
/// scored as rung 2 has none of it, and a session that ran long fails its budget.
#[test]
fn a_hand_session_of_rung_2_is_scored_on_its_track_its_station_and_its_knobs() {
    let dir = temp_dir("eval-rung2");
    for cli in ["claude", "codex"] {
        let project = emmet_it::project(&format!("rung2-{cli}"));
        hand_session(
            &project,
            cli,
            "Build a small train station next to a loop of track.",
            30,
            12 * 60,
        );
        let (v, code) = score_rung(emmet_it::shared_data(), "2", cli, &project, &dir.join("out"));
        // Claude Code's station passes. Codex's has the loop, the platform, and the canopy, but 216 parts where rung 2
        // asks for 250 (ladder.rs says why), so its run fails on size alone, and a failed run exits 1.
        let passes = cli == "claude";
        assert_eq!(code, if passes { 0 } else { 1 }, "{cli}: {v:#}");
        assert_eq!(v["data"]["pass"], passes, "{cli}: {v:#}");
        let (ok, features) = score_detail(&v, "Size and features");
        assert_eq!(ok, passes, "{cli}: {features}");
        assert!(features.contains("track loop closes yes"), "{cli}: {features}");
        assert!(features.contains("platform yes; canopy yes"), "{cli}: {features}");
        if !passes {
            assert!(features.starts_with("216 parts (≥ 250)"), "{features}");
        }
        let (ok, params) = score_detail(&v, "Parameters");
        assert!(
            ok && params.contains("the platform's length among them yes"),
            "{cli}: {params}"
        );
        assert_eq!(
            score_detail(&v, "Budget"),
            (true, "12 min 0 s (≤ 30 min), 30 turns (≤ 120), ended: completed".into()),
            "{cli}"
        );
        // Scoring puts build/ back at the model's defaults and leaves the sources alone.
        assert_eq!(
            std::fs::read(project.join("model.py")).unwrap(),
            std::fs::read(emmet_it::fixture(&format!("projects/rung2-{cli}/model.py"))).unwrap()
        );
    }
    // Both agents are in the day's report, each with its scores, and nothing in it names this machine.
    let report = std::fs::read_dir(dir.join("out"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| p.to_string_lossy().ends_with("-rung2.md"))
        .expect("the rung 2 report");
    let text = std::fs::read_to_string(&report).unwrap();
    assert!(
        text.starts_with("# Live eval: Rung 2: Train station with a track loop"),
        "{text}"
    );
    assert!(
        text.contains("## Claude Code — pass") && text.contains("## Codex — fail"),
        "{text}"
    );
    assert!(text.contains("hand sessions in the app"), "{text}");
    assert!(text.contains("## Trend"), "two runs make a trend: {text}");
    let home = std::env::var("HOME").unwrap_or_default();
    assert!(
        home.is_empty() || !text.contains(&home),
        "the report names no personal path"
    );

    // A cottage isn't a station: no track, no platform, no canopy.
    let cottage = dir.join("cottage");
    let made = emmet()
        .arg("--data-dir")
        .arg(emmet_it::shared_data())
        .arg("new")
        .arg(&cottage)
        .output()
        .unwrap();
    assert!(made.status.success(), "{}", emmet_it::text(&made));
    // 31 minutes: over rung 2's 30.
    hand_session(&cottage, "claude", "Build a small train station.", 5, 31 * 60);
    let (v, code) = score_rung(
        emmet_it::shared_data(),
        "2",
        "claude",
        &cottage,
        &dir.join("out-cottage"),
    );
    assert_eq!(code, 1, "a failed run exits 1: {v:#}");
    let (ok, features) = score_detail(&v, "Size and features");
    assert!(!ok, "{features}");
    assert!(
        features.contains("track loop closes no (0 pieces of track") && features.contains("platform no; canopy no"),
        "{features}"
    );
    assert!(!score_detail(&v, "Budget").0);
    assert!(score_detail(&v, "Runs and validates with 0 errors").0);
}

/// One LDraw part line for `p`, turned a quarter about the vertical and slid by `slide`.
fn turned_line(p: &emmet_core::ldraw::placements::Placement, color: u32, slide: [f64; 3]) -> String {
    let m = &p.matrix;
    // (x, y, z) → (-z, y, x), for the place and for each column of the rotation.
    let at = [-m[14] + slide[0], m[13] + slide[1], m[12] + slide[2]];
    let col = |c: usize| [-m[c * 4 + 2], m[c * 4 + 1], m[c * 4]];
    let (a, b, c) = (col(0), col(1), col(2));
    format!(
        "1 {color} {} {} {} {} {} {} {} {} {} {} {} {} {}.dat",
        at[0], at[1], at[2], a[0], b[0], c[0], a[1], b[1], c[1], a[2], b[2], c[2], p.part
    )
}

/// Rung 3 (5A): a hand session's model scored against the set it rebuilds, 6743-1 Street Speeder: its parts list
/// against the set's Rebrickable inventory (F1 over part × color) and its placements against the OMR model. The
/// model here is the OMR model itself without the parts ldraw-mini lacks, turned a quarter and moved, so the numbers
/// are exact: every piece it has is the set's and in place, and the pieces it lacks cost recall. Then with ten pieces
/// in the wrong color and five lifted off the model.
#[test]
fn a_hand_session_of_rung_3_is_scored_against_the_set() {
    let truth_file = emmet_it::fixture("ladder/rung3/6743-1.mpd");
    // The ground truth is the OMR's file, unmodified: the nightly corpus lists the same bytes.
    let nightly: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(emmet_it::fixture("omr/nightly.json")).unwrap()).unwrap();
    let listed = nightly["models"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["file"] == "6743-1.mpd")
        .unwrap();
    assert_eq!(
        emmet_it::sha256(&std::fs::read(&truth_file).unwrap()),
        listed["sha256"].as_str().unwrap()
    );
    let truth = emmet_it::corpus::placements_of(&truth_file);
    assert_eq!(truth.len(), 165, "the set's 165 pieces");
    let data = emmet_it::shared_data_with_parts();
    let have = |part: &str| {
        emmet_it::fixture("ldraw-mini/parts")
            .join(format!("{part}.dat"))
            .is_file()
    };
    let kept: Vec<_> = truth.iter().filter(|p| have(&p.part)).collect();
    let k = kept.len();
    assert!((120..165).contains(&k), "{k} of the set's pieces are in ldraw-mini");

    let dir = temp_dir("eval-rung3");
    let build = |name: &str, lines: Vec<String>| {
        let ldr = dir.join(format!("{name}.ldr"));
        std::fs::write(
            &ldr,
            format!("0 Street Speeder\n0 Name: {name}.ldr\n{}\n", lines.join("\n")),
        )
        .unwrap();
        let project = dir.join(name);
        let made = emmet()
            .arg("--data-dir")
            .arg(data)
            .arg("new")
            .arg(&project)
            .arg("--import")
            .arg(&ldr)
            .output()
            .unwrap();
        assert!(made.status.success(), "{}", emmet_it::text(&made));
        hand_session(&project, "codex", "Rebuild the official set 6743-1.", 20, 20 * 60);
        project
    };
    let slide = [140.0, -48.0, 60.0];
    let exact = build("exact", kept.iter().map(|p| turned_line(p, p.color, slide)).collect());
    let (v, _) = score_rung(data, "3", "codex", &exact, &dir.join("out"));
    let f1 = 2.0 * k as f64 / (k as f64 + 165.0);
    assert_eq!(
        score_detail(&v, "Parts list against the set (BOM F1)"),
        (
            f1 >= 0.8,
            format!(
                "F1 {f1:.2} (≥ 0.80): {k} of the model's {k} pieces are the set's, and {k} of the set's 165 are in the model"
            )
        ),
        "{v:#}"
    );
    let share = k as f64 / 165.0;
    assert_eq!(
        score_detail(&v, "Placements against the set"),
        (
            share >= 0.6,
            format!(
                "{:.0} % (≥ 60 %): {k} of the set's 165 parts have a part of the same id within a stud",
                share * 100.0
            )
        ),
        "{v:#}"
    );
    assert_eq!(
        score_detail(&v, "Budget"),
        (true, "20 min 0 s (≤ 45 min), 20 turns, ended: completed".into())
    );

    // Ten pieces in a color the set doesn't have them in (bright pink), and five more lifted three bricks off.
    let lines: Vec<String> = kept
        .iter()
        .enumerate()
        .map(|(i, p)| match i {
            0..10 => turned_line(p, 5, slide),
            10..15 => turned_line(p, p.color, [slide[0], slide[1] - 72.0, slide[2]]),
            _ => turned_line(p, p.color, slide),
        })
        .collect();
    let off = build("off", lines);
    let (v, _) = score_rung(data, "3", "codex", &off, &dir.join("out"));
    let (_, bom) = score_detail(&v, "Parts list against the set (BOM F1)");
    assert!(
        bom.ends_with(&format!(
            ": {} of the model's {k} pieces are the set's, and {} of the set's 165 are in the model",
            k - 10,
            k - 10
        )),
        "{bom}"
    );
    let (_, placed) = score_detail(&v, "Placements against the set");
    assert!(
        placed.contains(&format!(": {} of the set's 165 parts", k - 5)),
        "{placed}"
    );
}

/// A photo rung folder for the tests (EMMET_PHOTO_DIR): one subject whose range fits the cottage the fakes build
/// (905 parts), with the generated photos as its two angles, hashed.
fn photo_dir(dir: &std::path::Path) -> PathBuf {
    let photos = dir.join("photo");
    std::fs::create_dir_all(photos.join("local")).unwrap();
    let fixture =
        |a: &str| std::fs::read(emmet_it::repo().join(format!("tests/fixtures/photos/angle-{a}.jpg"))).unwrap();
    for a in ["a", "b"] {
        std::fs::write(photos.join(format!("local/truck-{a}.jpg")), fixture(a)).unwrap();
    }
    std::fs::write(
        photos.join("subjects.toml"),
        format!(
            "[[subject]]\nid = \"truck\"\ndescription = \"A small pickup truck\"\nparts = [500, 1500]\nimages = [\n  \
             {{ angle = \"a\", file = \"truck-a.jpg\", sha256 = \"{}\" }},\n  {{ angle = \"b\", file = \"truck-b.jpg\", sha256 = \"{}\" }},\n]\n",
            emmet_core::fsx::sha256_hex(&fixture("a")),
            emmet_core::fsx::sha256_hex(&fixture("b"))
        ),
    )
    .unwrap();
    photos
}

/// The repo's own subjects.toml reads whole: the photo rung names its four subjects when it's given one it hasn't.
#[test]
fn the_photo_rungs_subjects_are_the_four_in_its_manifest() {
    let run = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(eval_data())
        .args([
            "eval",
            "--rung",
            "photo",
            "--subject",
            "nope",
            "--agent",
            "claude",
            "--dry-run",
        ])
        .output()
        .unwrap();
    let (v, code) = envelope(&run);
    assert_eq!(code, 3, "{v}");
    assert_eq!(
        v["error"]["hint"], "Pick one with --subject: lamp, e-bike, bradbury, truck.",
        "{v}"
    );
}

/// The photo rung's dry run (4D's Done when 3) with each fake agent: angle a goes with "Build this.", the fake builds a
/// first draft and ends its turn asking for another angle, angle b goes as the next turn, and the build is scored and
/// written up with its render and without a photo.
#[test]
fn a_dry_run_scores_the_photo_rung_with_each_fake_agent() {
    let _bus = BUS.lock().unwrap_or_else(|e| e.into_inner());
    for cli in ["claude", "codex"] {
        let dir = temp_dir(&format!("eval-photo-{cli}"));
        let photos = photo_dir(&dir);
        let fake = dir.join("fake");
        let (out, work) = (dir.join("out"), dir.join("work"));
        std::fs::create_dir_all(&fake).unwrap();
        let transcript = emmet_it::repo().join(format!("tests/replay/{cli}/derived-photo.jsonl"));
        std::fs::write(fake.join(format!("{cli}.scenario")), transcript.display().to_string()).unwrap();
        std::fs::write(fake.join("keychain"), "sk-test-not-a-key").unwrap();
        let run = emmet()
            .arg("--json")
            .arg("--data-dir")
            .arg(eval_data())
            .args([
                "eval",
                "--rung",
                "photo",
                "--subject",
                "truck",
                "--agent",
                cli,
                "--dry-run",
                "--rubric",
                "2,2,1",
                "--out",
            ])
            .arg(&out)
            .arg("--work")
            .arg(&work)
            .env("EMMET_FAKE_DIR", &fake)
            .env("EMMET_CLAUDE_BIN", fake_bin("claude"))
            .env("EMMET_CODEX_BIN", fake_bin("codex"))
            .env("EMMET_FAKE_KEYCHAIN", fake.join("keychain"))
            .env("EMMET_PHOTO_DIR", &photos)
            // No render mode: a desktop build beside emmet would open a window for each render.
            .env("EMMET_APP", "/nonexistent/emmet-app")
            .output()
            .unwrap();
        let (v, code) = envelope(&run);
        let stderr = String::from_utf8_lossy(&run.stderr);
        assert_eq!(code, 0, "{cli}: {v}\n{stderr}");
        let data = &v["data"];
        assert_eq!(data["pass"], true, "{cli}: {data:#}");
        let scores: Vec<(&str, bool, &str)> = data["scores"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| {
                (
                    s["name"].as_str().unwrap(),
                    s["pass"] == true,
                    s["detail"].as_str().unwrap(),
                )
            })
            .collect();
        let names: Vec<&str> = scores.iter().map(|s| s.0).collect();
        assert_eq!(
            names,
            [
                "Runs and validates with 0 errors",
                "Part count",
                "Built from the photo",
                "Asked for another angle",
                "First draft within a minute",
                "Barry's rubric"
            ]
        );
        assert!(scores.iter().all(|s| s.1), "{cli}: {scores:?}");
        // The count is a reference beside what a display set has, never a gate (Barry 2026-11-10).
        let count = &v["data"]["scores"].as_array().unwrap()[1];
        assert_eq!(count["required"], false, "{cli}: {count}");
        assert!(scores[1].2.contains("a display set of it has "), "{cli}: {scores:?}");
        assert_eq!(scores[3].2, "asked, and got angle b");
        assert_eq!(
            scores[5].2,
            "proportions 2, recognizability 2, buildability 1: 5 of 6 (≥ 4)"
        );
        // Both photos reached the CLI, cleaned, each with its turn.
        let project = work.join(format!("photo-truck-{cli}"));
        let refs: Vec<PathBuf> = std::fs::read_dir(project.join("refs"))
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(refs.len(), 2, "{cli}: {refs:?}");
        for r in &refs {
            assert_eq!(crate::refs::has_metadata(&std::fs::read(r).unwrap()), None);
        }
        let stdin = std::fs::read_to_string(fake.join("stdin.jsonl")).unwrap();
        let turns: Vec<serde_json::Value> = stdin
            .lines()
            .map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap())
            .filter(|m| m["type"] == "user" || m["method"] == "turn/start")
            .collect();
        assert_eq!(turns.len(), 2, "{cli}: two turns");
        let texts: Vec<String> = turns.iter().map(|t| t.to_string()).collect();
        assert!(
            texts[0].contains("Build this.") && texts[1].contains("Here's another angle."),
            "{texts:?}"
        );
        let kind = if cli == "claude" {
            r#""type":"image""#
        } else {
            r#""type":"localImage""#
        };
        assert!(
            texts.iter().all(|t| t.contains(kind)),
            "{cli}: each turn carries its photo"
        );
        // The report: the subject's section and its render; no photo, and nothing from the photos' folder.
        let report = std::fs::read_to_string(
            std::fs::read_dir(&out)
                .unwrap()
                .map(|e| e.unwrap().path())
                .find(|p| p.extension().is_some_and(|x| x == "md"))
                .unwrap(),
        )
        .unwrap();
        let label = if cli == "claude" { "Claude Code" } else { "Codex" };
        assert!(report.contains(&format!("## {label}: truck — pass")), "{report}");
        // The render, where this machine can make one (the eval says "no render" and goes on when it can't).
        if let Some(png) = data["render"].as_str() {
            assert!(
                png.contains("-photo-truck-") && report.contains(&format!("({png})")),
                "{report}"
            );
        }
        assert!(!report.contains("local/") && !report.contains(".jpg"), "{report}");
        let files: Vec<String> = std::fs::read_dir(&out)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(
            files.iter().all(|f| f.ends_with(".md") || f.ends_with(".png")),
            "{files:?}"
        );
    }
}

/// `--score` on a hand session (G2's runs, Barry 2026-11-04): it reads the chat the app kept, times the first draft,
/// sees the request for another angle, takes the rubric, and sends nothing to an agent. It refuses to score when the
/// subject's photos on this Mac aren't the ones subjects.toml names.
#[test]
fn scoring_a_hand_session_reads_its_chat_and_checks_the_photos() {
    let dir = temp_dir("eval-photo-score");
    let photos = photo_dir(&dir);
    let fixture =
        |a: &str| std::fs::read(emmet_it::repo().join(format!("tests/fixtures/photos/angle-{a}.jpg"))).unwrap();
    // A built project with the chat the app keeps: the photo at 0 s, the first build done at 20 s, a turn that ends
    // asking for the side, angle b at 60 s.
    let project = dir.join("truck");
    let made = emmet()
        .arg("--data-dir")
        .arg(eval_data())
        .arg("new")
        .arg(&project)
        .output()
        .unwrap();
    assert!(made.status.success(), "{}", emmet_it::text(&made));
    let chat = project.join(".emmet/chat");
    std::fs::create_dir_all(&chat).unwrap();
    std::fs::write(
        chat.join("sessions.json"),
        r#"{"sessions":[{"id":"s1","kind":"claude","started":0}]}"#,
    )
    .unwrap();
    let t0 = 1_800_000_000_000.0_f64;
    let records = [
        serde_json::json!({ "at": t0, "user": "Build this.", "photos": ["refs/truck-a-0123456789ab.jpg"] }),
        serde_json::json!({ "at": t0 + 1_000.0, "event": { "type": "started", "resume": "r", "model": "claude-opus-5-5", "billing": { "kind": "plan", "name": "max" } } }),
        serde_json::json!({ "at": t0 + 5_000.0, "event": { "type": "tool_call", "id": "t1", "name": "mcp__emmet__run_model", "input": {} } }),
        serde_json::json!({ "at": t0 + 20_000.0, "event": { "type": "tool_result", "id": "t1", "ok": true, "summary": "built" } }),
        serde_json::json!({ "at": t0 + 21_000.0, "event": { "type": "text", "message": "m1", "delta": "A first draft is up. Can you send a photo from the side?" } }),
        serde_json::json!({ "at": t0 + 22_000.0, "event": { "type": "turn_done", "reason": "completed" } }),
        serde_json::json!({ "at": t0 + 60_000.0, "user": "Here's another angle.", "photos": ["refs/truck-b-0123456789ab.jpg"] }),
        serde_json::json!({ "at": t0 + 90_000.0, "event": { "type": "turn_done", "reason": "completed" } }),
    ];
    std::fs::write(
        chat.join("s1.jsonl"),
        records.iter().map(|r| format!("{r}\n")).collect::<String>(),
    )
    .unwrap();
    let score = |rubric: &str| {
        emmet()
            .arg("--json")
            .arg("--data-dir")
            .arg(eval_data())
            .args([
                "eval",
                "--rung",
                "photo",
                "--subject",
                "truck",
                "--agent",
                "claude",
                "--rubric",
                rubric,
                "--score",
            ])
            .arg(&project)
            .arg("--out")
            .arg(dir.join("out"))
            .env("EMMET_PHOTO_DIR", &photos)
            // No render mode: a desktop build beside emmet would open a window for each render.
            .env("EMMET_APP", "/nonexistent/emmet-app")
            // Nothing may start an agent: a CLI that would fail the test if it ran.
            .env("EMMET_CLAUDE_BIN", "/nonexistent/claude")
            .output()
            .unwrap()
    };
    let (v, code) = envelope(&score("1,1,1"));
    assert_eq!(code, 1, "a 3 of 6 rubric fails the subject: {v}");
    let scores = &v["data"]["scores"];
    let detail = |name: &str| {
        scores.as_array().unwrap().iter().find(|s| s["name"] == name).unwrap()["detail"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(
        detail("First draft within a minute"),
        "the first build ran 20 s after the first message (≤ 60 s)"
    );
    assert_eq!(detail("Asked for another angle"), "asked, and got angle b");
    assert_eq!(
        detail("Built from the photo"),
        "the session's first message carried the photo"
    );
    assert_eq!(v["data"]["run"]["billing"], "Max plan", "{v}");
    let (v, code) = envelope(&score("2,1,1"));
    assert_eq!((code, &v["data"]["pass"]), (0, &serde_json::json!(true)), "{v}");
    // A session that opened with words and a reply, the photo after: not built from the photo. A line this build can't
    // read is reported, not passed over.
    let opened = [
        serde_json::json!({ "at": t0, "user": "Can you build from photos?" }),
        serde_json::json!({ "at": t0 + 2_000.0, "event": { "type": "text", "message": "m0", "delta": "Yes: send one." } }),
        serde_json::json!({ "at": t0 + 3_000.0, "event": { "type": "turn_done", "reason": "completed" } }),
        serde_json::json!({ "at": t0 + 4_000.0, "event": { "type": "from_a_newer_emmet" } }),
    ];
    std::fs::write(
        chat.join("s1.jsonl"),
        opened
            .iter()
            .chain(&records)
            .map(|r| format!("{r}\n"))
            .collect::<String>(),
    )
    .unwrap();
    let (v, _) = envelope(&score("2,1,1"));
    let scores = &v["data"]["scores"];
    let built = scores
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "Built from the photo")
        .unwrap();
    assert_eq!(built["pass"], false, "{v}");
    assert!(
        v["data"]["run"]["errors"]
            .to_string()
            .contains("1 line of the chat couldn't be read"),
        "{v}"
    );
    std::fs::write(
        chat.join("s1.jsonl"),
        records.iter().map(|r| format!("{r}\n")).collect::<String>(),
    )
    .unwrap();
    // A photo on this Mac that isn't the one the manifest names: nothing is scored.
    std::fs::write(photos.join("local/truck-b.jpg"), fixture("a")).unwrap();
    let (v, code) = envelope(&score("2,2,2"));
    assert_ne!(code, 0);
    assert!(
        v["error"]["message"].as_str().unwrap().contains("its hash differs"),
        "{v}"
    );
}
