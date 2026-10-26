// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet eval` (plan section 16.1, D-37, D-38): one live run of a ladder rung through a real agent CLI, headless, on
//! Barry's API keys only, scored and written up in docs/evals/. It starts the CLI through the same adapters as the
//! app and answers for the app itself: the rung's answer script answers `ask_user`, and approvals allow Emmet's own
//! tools and edits to the model files and deny everything else. `--dry-run` is for the fake agents in CI: no keychain,
//! no question before it starts. A turn is one tool call; questions are `ask_user` calls, so they count.
//!
//! v0 runs rung 1; 5A adds the other rungs.

use std::collections::BTreeMap;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use emmet_agent::{
    AgentEvent, AgentKind, Answer, ApprovalKind, Billing, Decision, DoneReason, Hub, Question, SessionConfig, UserTurn,
};
use emmet_core::data::DataDir;
use emmet_core::envelope::{Error, ErrorCode, Result};
use emmet_core::project::{Project, Template};
use serde::Serialize;
use serde_json::Value;

/// The ladder (prompts, features, answer scripts), as the docs have it.
const LADDER: &str = include_str!("../../../docs/spec/ladder.md");

/// Rung 1's budget (plan section 16.1).
const BUDGET: Duration = Duration::from_secs(10 * 60);
const MAX_TURNS: usize = 40;
const MIN_PARTS: usize = 40;
const MIN_PARAMS: usize = 3;
/// After the budget, how long the run may go on before it's stopped (it fails the budget either way).
const GRACE: Duration = Duration::from_secs(2 * 60);

pub struct Options {
    pub rung: String,
    pub agent: AgentKind,
    pub dry_run: bool,
    /// Skip the question before a live run.
    pub yes: bool,
    /// Where the report and render go (default: docs/evals/ in the repo).
    pub out: Option<PathBuf>,
    /// Where to keep the run's project (default: a new temp folder).
    pub work: Option<PathBuf>,
}

/// One rung of docs/spec/ladder.md.
#[derive(Debug, Clone, PartialEq)]
pub struct Rung {
    pub title: String,
    pub prompt: String,
    pub features: String,
    /// (the words a question may mention, the answer)
    pub script: Vec<(Vec<String>, String)>,
}

/// Reads rung `n` ("1") from the ladder's markdown: the blockquoted prompt, the features line, and the answer table.
pub fn rung(ladder: &str, n: &str) -> Option<Rung> {
    let head = format!("## Rung {n}:");
    let start = ladder.find(&head)?;
    let rest = &ladder[start..];
    let end = rest[head.len()..].find("\n## ").map_or(rest.len(), |i| i + head.len());
    let section = &rest[..end];
    let title = section.lines().next()?.trim_start_matches("## ").trim().to_string();
    let prompt = section
        .lines()
        .filter_map(|l| l.strip_prefix('>'))
        .map(str::trim)
        .collect::<Vec<_>>()
        .join(" ");
    // The features paragraph, which wraps over several lines.
    let features = section
        .split("\n\n")
        .find_map(|p| p.trim().strip_prefix("**Features:**"))
        .map(|p| p.split_whitespace().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    let mut script = Vec::new();
    let mut in_table = false;
    for line in section.lines() {
        if line.starts_with("**Answer script:**") {
            in_table = true;
            continue;
        }
        if !in_table || !line.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line.trim().trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 2 || cells[0].starts_with("When") || cells[0].starts_with("---") {
            continue;
        }
        let words = cells[0]
            .split(',')
            .map(|w| w.trim().to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();
        script.push((words, cells[1].to_string()));
    }
    (!prompt.is_empty()).then_some(Rung {
        title,
        prompt,
        features,
        script,
    })
}

/// The answer to an `ask_user` question (docs/spec/ladder.md, "How answer scripts work"): the first entry with a
/// phrase whose words all appear in the question or its options gives the answer; an answer naming an option chooses
/// it, any other is the Other text. No entry: the agent's recommended (first) option.
pub fn answer(script: &[(Vec<String>, String)], q: &Question) -> (Answer, String) {
    let mut text = q.question.to_lowercase();
    for o in &q.options {
        text.push(' ');
        text.push_str(&o.label.to_lowercase());
    }
    let words: Vec<&str> = text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    let mentions = |phrase: &str| phrase.split_whitespace().all(|w| words.contains(&w));
    let found = script.iter().find(|(phrases, _)| phrases.iter().any(|p| mentions(p)));
    let Some((_, reply)) = found else {
        let first = q.options.first().map(|o| o.label.clone()).unwrap_or_default();
        return (
            Answer {
                choices: vec![first.clone()],
                other: None,
            },
            format!("{first} (the recommended option; the script has no answer)"),
        );
    };
    let lower = reply.to_lowercase();
    match q.options.iter().find(|o| {
        lower == o.label.to_lowercase()
            || lower.contains(&o.label.to_lowercase())
            || o.label.to_lowercase().contains(&lower)
    }) {
        Some(o) => (
            Answer {
                choices: vec![o.label.clone()],
                other: None,
            },
            o.label.clone(),
        ),
        None => (
            Answer {
                choices: vec![],
                other: Some(reply.clone()),
            },
            format!("Other: {reply}"),
        ),
    }
}

/// The eval's rule for approval cards (D-38): Emmet's own tools, and edits to the project's own model files
/// (`<root>/model.py`, `<root>/models/*.py`), nothing else. A relative path is taken from the project root; any `..`
/// is refused.
pub fn allowed(root: &Path, kind: ApprovalKind, tool: &str, detail: &str) -> bool {
    if tool.starts_with("mcp__emmet") || tool.starts_with("emmet") {
        return true;
    }
    if kind != ApprovalKind::Edit {
        return false;
    }
    // Claude's card names one file; Codex's lists them, comma-separated, maybe with " (its reason)" after.
    let files = match detail.trim().rsplit_once(" (") {
        Some((files, reason)) if reason.ends_with(')') => files,
        _ => detail.trim(),
    };
    files.split(", ").all(|f| {
        let f = f.trim();
        if f.is_empty() {
            return false;
        }
        let path = Path::new(f);
        if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
            return false;
        }
        let Ok(rel) = (if path.is_absolute() {
            path.strip_prefix(root)
        } else {
            Ok(path)
        }) else {
            return false;
        };
        let parts: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        match parts.as_slice() {
            [name] => name == "model.py",
            [dir, name] => dir == "models" && name.ends_with(".py"),
            _ => false,
        }
    })
}

/// What happened in a run, for the report.
#[derive(Debug, Default, Serialize)]
pub struct Run {
    pub agent: String,
    pub cli_version: Option<String>,
    pub model: Option<String>,
    pub billing: Option<String>,
    pub seconds: u64,
    pub turns: usize,
    pub questions: Vec<(String, String)>,
    pub approvals: Vec<(String, bool)>,
    pub tokens: (u64, u64),
    pub ended: String,
    pub errors: Vec<String>,
    pub last_message: String,
    pub transcript: Option<String>,
}

/// One scored line of section 16.1.
#[derive(Debug, Serialize)]
pub struct Score {
    pub name: String,
    pub required: bool,
    pub pass: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub rung: String,
    pub run: Run,
    pub scores: Vec<Score>,
    pub pass: bool,
    pub render: Option<String>,
    pub project: String,
}

/// Runs the eval and writes its report. Returns the report.
pub fn eval(data: &DataDir, opts: &Options) -> Result<Report> {
    if opts.rung != "1" {
        return Err(usage(format!(
            "emmet eval runs rung 1 for now; rung {} arrives with 5A.",
            opts.rung
        )));
    }
    let rung = rung(LADDER, "1").ok_or_else(|| Error::internal("docs/spec/ladder.md has no rung 1"))?;
    // The bus: the eval answers questions and approvals itself, so no app may hold this data dir's bus.
    if emmet_bus::listening(data) {
        return Err(Error::new(
            ErrorCode::Internal,
            "An Emmet app is open on this data dir, and the eval needs its bus.",
            "Quit the Emmet app (or `just dev`), then run the eval again.",
        ));
    }
    // The CLI gets emmet's own environment: run from a terminal, that's already the login shell's. (The app reads the
    // login shell's with an interactive `$SHELL -ilc`, which here would take the terminal over and leave the eval
    // suspended at its first question.)
    let mut env: Vec<(String, String)> = std::env::vars().collect();
    let mut openai_key = None;
    if !opts.dry_run {
        let (service, var) = match opts.agent {
            AgentKind::Claude => ("emmet-eval-anthropic", "ANTHROPIC_API_KEY"),
            AgentKind::Codex => ("emmet-eval-openai", "OPENAI_API_KEY"),
        };
        let key = emmet_agent::keychain::eval_key(service)
            .map_err(|e| Error::internal(e.message))?
            .ok_or_else(|| {
                Error::new(
                    ErrorCode::Internal,
                    format!("There's no {service} item in the keychain."),
                    format!(
                        "Store the eval key first (H-21): security add-generic-password -s {service} -a \"$USER\" -w"
                    ),
                )
            })?;
        env.retain(|(k, _)| k != var);
        match opts.agent {
            AgentKind::Claude => env.push((var.into(), key)),
            AgentKind::Codex => openai_key = Some(key),
        }
    }
    let adapter = emmet_agent::adapter(opts.agent, &env, data.root());
    let detection = adapter.detect();
    if detection.path.is_none() {
        return Err(Error::new(
            ErrorCode::Internal,
            format!("{} isn't installed.", opts.agent.label()),
            detection.fix.unwrap_or_default(),
        ));
    }
    if !opts.dry_run && !opts.yes && !confirm(opts.agent)? {
        return Err(Error::new(
            ErrorCode::Internal,
            "Canceled; nothing ran.",
            "Run it again when you're ready.",
        ));
    }

    // A fresh project in a temp folder.
    let work = match &opts.work {
        Some(w) => w.clone(),
        None => std::env::temp_dir().join(format!("emmet-eval-{}", emmet_core::fsx::random_hex(1))),
    };
    let root = work.join(format!("rung1-{}", name(opts.agent)));
    let created = emmet_core::ops::new_project(data, &root, Template::Empty)?;
    let project = Project::find(Some(Path::new(&created.project)))?;

    let hub = Hub::default();
    let route = hub.clone();
    let _bus = emmet_bus::Server::start(data, move |msg| {
        use emmet_bus::{Message, Reply};
        let answer = match msg {
            Message::Approve { session, request } => route.approve(&session, request),
            Message::Ask { session, question } => route.ask(&session, question),
            Message::Notes { session } => route.take_notes(&session),
            // render falls back to render mode, and highlight has no window to show.
            _ => return Reply::undelivered("emmet eval has no window"),
        };
        match answer {
            Some(a) => Reply {
                delivered: true,
                answer: Some(a),
                ..Default::default()
            },
            None => Reply::undelivered("that session isn't the eval's"),
        }
    })?;
    let emmet = std::env::current_exe().map_err(|e| Error::internal(format!("this emmet's path: {e}")))?;
    let session_id = format!("eval{}", emmet_core::fsx::random_hex(1));
    let cfg = SessionConfig {
        id: session_id.clone(),
        project: project.root.clone(),
        data: data.root().to_path_buf(),
        emmet,
        resume: None,
        model: None,
        env,
        hub: hub.clone(),
        native_steer: true,
        openai_key,
    };
    let mut session = adapter
        .start(cfg)
        .map_err(|e| Error::internal(format!("starting {}: {}", opts.agent.label(), e.message)))?;
    let events = session.events();
    let mut run = Run {
        agent: opts.agent.label().into(),
        cli_version: detection.version.clone(),
        ..Default::default()
    };
    eprintln!("eval: rung 1 on {} in {}", opts.agent.label(), project.root.display());
    let started = Instant::now();
    session
        .send(UserTurn {
            text: rung.prompt.clone(),
        })
        .map_err(|e| Error::internal(format!("sending the prompt: {}", e.message)))?;
    let mut stopped = false;
    loop {
        let left = (BUDGET + GRACE).saturating_sub(started.elapsed());
        let event = match events.recv_timeout(left.min(Duration::from_secs(5))) {
            Ok(e) => e,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if left.is_zero() && !stopped {
                    eprintln!("eval: over the time budget; stopping the turn");
                    let _ = session.interrupt();
                    stopped = true;
                }
                if started.elapsed() > BUDGET + GRACE * 2 {
                    run.ended = "never ended after Stop".into();
                    break;
                }
                continue;
            }
            Err(_) => {
                if run.ended.is_empty() {
                    run.ended = "the CLI exited".into();
                }
                break;
            }
        };
        match event {
            AgentEvent::Started { model, billing, .. } => {
                run.model = model;
                if let Some(b) = billing {
                    run.billing = Some(b.label());
                    // D-37: evals bill the eval key only. A plan in use means the key didn't take; stop at once.
                    if !opts.dry_run && matches!(b, Billing::Plan { .. }) {
                        let _ = session.interrupt();
                        session.close();
                        return Err(Error::new(
                            ErrorCode::Internal,
                            format!(
                                "{} started on your plan, not the eval API key; the eval stopped.",
                                opts.agent.label()
                            ),
                            "Check that nothing in your shell overrides the key, then run it again.",
                        ));
                    }
                }
            }
            AgentEvent::ToolCall { name, .. } => {
                // The last message is the text after the last tool call.
                run.last_message.clear();
                run.turns += 1;
                eprintln!("eval: [{}] {name}", run.turns);
                if run.turns > MAX_TURNS && !stopped {
                    eprintln!("eval: over {MAX_TURNS} turns; stopping the turn");
                    let _ = session.interrupt();
                    stopped = true;
                }
            }
            AgentEvent::Question { id, question } => {
                // ask_user is a tool call, so the question already counts as a turn (section 16.1).
                let (a, said) = answer(&rung.script, &question);
                eprintln!("eval: asked \"{}\" → {said}", question.question);
                run.questions.push((question.question.clone(), said));
                let _ = session.answer(&id, a);
            }
            AgentEvent::Approval { id, kind, tool, detail } => {
                let ok = allowed(&project.root, kind, &tool, &detail);
                run.approvals.push((format!("{tool}: {detail}"), ok));
                let _ = session.respond(&id, if ok { Decision::AllowOnce } else { Decision::Deny });
            }
            AgentEvent::Text { delta, .. } => run.last_message.push_str(&delta),
            AgentEvent::Usage { input, output } => {
                run.tokens.0 += input;
                run.tokens.1 += output;
            }
            AgentEvent::Error { message, .. } => run.errors.push(message),
            AgentEvent::TurnDone { reason } => {
                run.ended = match reason {
                    DoneReason::Completed => "completed",
                    DoneReason::Interrupted => "stopped (over budget)",
                    DoneReason::Failed => "failed",
                    DoneReason::Crashed => "crashed",
                }
                .into();
                break;
            }
            AgentEvent::Exited { .. } => {
                if run.ended.is_empty() {
                    run.ended = "the CLI exited".into();
                }
                break;
            }
            _ => {}
        }
    }
    run.seconds = started.elapsed().as_secs();
    session.close();
    let transcript = project
        .root
        .join(".emmet/transcripts")
        .join(format!("{session_id}.jsonl"));
    run.transcript = transcript.is_file().then(|| transcript.display().to_string());
    // The end of what the agent ended on is enough for the report.
    let last: Vec<char> = run.last_message.trim().chars().collect();
    run.last_message = last[last.len().saturating_sub(1200)..].iter().collect();

    let scores = score(data, &project, &run);
    let pass = scores.iter().filter(|s| s.required).all(|s| s.pass);
    let out_dir = match &opts.out {
        Some(o) => o.clone(),
        None => repo_dir()?.join("docs/evals"),
    };
    std::fs::create_dir_all(&out_dir).map_err(|e| Error::io(&out_dir, e))?;
    let date = emmet_agent::time::rfc3339(emmet_agent::time::now_ms())[..10].to_string();
    let png_name = format!("{date}-rung1-{}.png", name(opts.agent));
    let render = match crate::render::render(data, &project, "iso", 768, &[]) {
        Ok(png) => {
            crate::render::write_png(&out_dir.join(&png_name), &png)?;
            Some(png_name)
        }
        Err(e) => {
            eprintln!("eval: no render ({})", e.message);
            None
        }
    };
    let report = Report {
        rung: rung.title.clone(),
        run,
        scores,
        pass,
        render,
        project: project.root.display().to_string(),
    };
    write_report(
        &out_dir.join(format!("{date}-rung1.md")),
        opts.agent,
        &rung,
        &report,
        &date,
    )?;
    if !opts.dry_run && report.run.transcript.is_some() {
        offer_fixture(opts.agent, &report)?;
    }
    Ok(report)
}

/// The run scored per section 16.1 (rung 1).
fn score(data: &DataDir, project: &Project, run: &Run) -> Vec<Score> {
    let mut out = Vec::new();
    let mut push = |name: &str, required: bool, pass: bool, detail: String| {
        out.push(Score {
            name: name.into(),
            required,
            pass,
            detail,
        })
    };
    let base = emmet_core::ops::run_model(data, project, &BTreeMap::new()).0;
    let checked = base
        .as_ref()
        .ok()
        .and(emmet_core::ops::check_build(data, project, &[]).ok());
    match (&base, &checked) {
        (Ok(r), Some(v)) => push(
            "Runs and validates with 0 errors",
            true,
            v.errors() == 0,
            format!(
                "{} parts, {} findings ({} errors)",
                r.parts,
                v.findings.len(),
                v.errors()
            ),
        ),
        (Ok(_), None) => push(
            "Runs and validates with 0 errors",
            true,
            false,
            "the validator couldn't run".into(),
        ),
        (Err(e), _) => push(
            "Runs and validates with 0 errors",
            true,
            false,
            format!("{:?}: {}", e.code, e.message),
        ),
    }
    let (placements, _) = emmet_core::ops::built_placements(project).unwrap_or_default();
    let index = emmet_core::index::Index::open(data).ok();
    let category = |part: &str| {
        index
            .as_ref()
            .and_then(|i| i.part(part).ok().flatten())
            .map(|p| (p.category, p.name))
            .unwrap_or_default()
    };
    let has = |want: &str| {
        placements.iter().any(|p| {
            let (cat, name) = category(&p.part);
            cat == want || name.starts_with(want)
        })
    };
    let (door, window, slope) = (has("Door"), has("Window") || has("Glass"), has("Slope"));
    let baseplate = placements.iter().any(|p| p.part == "3811" && p.color == 2);
    let parts = placements.len();
    push(
        "Size and features",
        true,
        parts >= MIN_PARTS && door && window && slope,
        format!(
            "{parts} parts (≥ {MIN_PARTS}); door {}; windows {}; sloped roof {}; green 32 x 32 baseplate {}",
            yes(door),
            yes(window),
            yes(slope),
            yes(baseplate)
        ),
    );
    let (changing, detail) = params_that_change(data, project, &placements);
    push(
        "Parameters",
        true,
        changing >= MIN_PARAMS,
        format!("{changing} change the model (≥ {MIN_PARAMS}): {detail}"),
    );
    push(
        "Availability",
        false,
        true,
        "n/a: color availability arrives with the parts data pack (3B)".into(),
    );
    push(
        "Budget",
        true,
        run.seconds <= BUDGET.as_secs() && run.turns <= MAX_TURNS && run.ended == "completed",
        format!(
            "{} min {} s (≤ 10 min), {} turns (≤ {MAX_TURNS}), ended: {}",
            run.seconds / 60,
            run.seconds % 60,
            run.turns,
            run.ended
        ),
    );
    out
}

fn yes(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// How many params change the placements when set to another value, and which.
fn params_that_change(
    data: &DataDir,
    project: &Project,
    base: &[emmet_core::ldraw::placements::Placement],
) -> (usize, String) {
    let Ok(params) = emmet_core::ops::get_params(data, project) else {
        return (0, "the parameters couldn't be read".into());
    };
    let key = |list: &[emmet_core::ldraw::placements::Placement]| serde_json::to_string(list).unwrap_or_default();
    let before = key(base);
    let mut changing = Vec::new();
    let mut not = Vec::new();
    for spec in &params.params {
        let Some(other) = other_value(spec) else {
            not.push(format!("{} (no other value)", spec.name));
            continue;
        };
        let overrides = BTreeMap::from([(spec.name.clone(), other.clone())]);
        let changed = emmet_core::ops::run_model(data, project, &overrides).0.is_ok()
            && emmet_core::ops::built_placements(project).is_ok_and(|(p, _)| key(&p) != before);
        if changed {
            changing.push(format!("{} = {other}", spec.name));
        } else {
            not.push(spec.name.clone());
        }
    }
    // Put build/ back to the defaults.
    let _ = emmet_core::ops::run_model(data, project, &BTreeMap::new());
    let mut detail = changing.join(", ");
    if !not.is_empty() {
        detail.push_str(&format!("; no change: {}", not.join(", ")));
    }
    (changing.len(), detail)
}

/// A valid value other than the default: the far end of a range, the other bool, another choice or color.
fn other_value(spec: &emmet_core::ops::ParamSpec) -> Option<Value> {
    match spec.kind.as_str() {
        "bool" => Some(Value::Bool(!spec.default.as_bool()?)),
        "int" | "float" => {
            let d = spec.default.as_f64()?;
            let target = match (spec.min, spec.max) {
                (_, Some(max)) if max != d => max,
                (Some(min), _) if min != d => min,
                _ => d + spec.step.unwrap_or(1.0),
            };
            Some(if spec.kind == "int" {
                Value::from(target.round() as i64)
            } else {
                Value::from(target)
            })
        }
        "choice" => spec.choices.as_ref()?.iter().find(|c| **c != spec.default).cloned(),
        "color" => {
            let d = spec.default.as_str().unwrap_or("").to_lowercase();
            Some(Value::from(if d == "blue" { "Red" } else { "Blue" }))
        }
        _ => None,
    }
}

/// Writes (or updates) the agent's section of the day's report.
fn write_report(path: &Path, agent: AgentKind, rung: &Rung, r: &Report, date: &str) -> Result<()> {
    let tag = name(agent);
    let mut section = vec![
        format!("<!-- eval:{tag} -->"),
        format!("## {} — {}", r.run.agent, if r.pass { "pass" } else { "fail" }),
        String::new(),
        format!(
            "{} {}, model {}, billed to {}. Emmet {}.",
            r.run.agent,
            r.run.cli_version.as_deref().unwrap_or("(version unknown)"),
            r.run.model.as_deref().unwrap_or("(not reported)"),
            r.run.billing.as_deref().unwrap_or("(not reported)"),
            emmet_core::version()
        ),
        String::new(),
        "| Score | Required | Result | Detail |".into(),
        "|---|---|---|---|".into(),
    ];
    for s in &r.scores {
        section.push(format!(
            "| {} | {} | {} | {} |",
            s.name,
            if s.required { "yes" } else { "no" },
            if s.pass { "pass" } else { "fail" },
            s.detail.replace('|', "\\|")
        ));
    }
    section.push(String::new());
    if let Some(png) = &r.render {
        section.push(format!("![The model {} built]({png})", r.run.agent));
        section.push(String::new());
    }
    section.push(format!(
        "Usage: {} input and {} output tokens (where the CLI reports them; the provider's console has the cost).",
        r.run.tokens.0, r.run.tokens.1
    ));
    section.push(String::new());
    if r.run.questions.is_empty() {
        section.push("Questions: none.".into());
    } else {
        section.push("Questions, answered from the script:".into());
        section.push(String::new());
        for (q, a) in &r.run.questions {
            section.push(format!("- \"{q}\" → {a}"));
        }
    }
    let denied: Vec<&str> = r
        .run
        .approvals
        .iter()
        .filter(|(_, ok)| !ok)
        .map(|(w, _)| w.as_str())
        .collect();
    if !denied.is_empty() {
        section.push(String::new());
        section.push(format!("Denied: {}.", denied.join("; ")));
    }
    if !r.run.errors.is_empty() {
        section.push(String::new());
        section.push(format!("Errors from the CLI: {}.", r.run.errors.join("; ")));
    }
    section.push(String::new());
    section.push("The agent's last message:".into());
    section.push(String::new());
    for line in r.run.last_message.lines() {
        section.push(format!("> {line}"));
    }
    section.push(format!("<!-- /eval:{tag} -->"));
    let section = section.join("\n");

    let existing = std::fs::read_to_string(path).ok();
    let text = match existing {
        Some(text) => {
            let (open, close) = (format!("<!-- eval:{tag} -->"), format!("<!-- /eval:{tag} -->"));
            match (text.find(&open), text.find(&close)) {
                (Some(a), Some(b)) if b > a => format!("{}{section}{}", &text[..a], &text[b + close.len()..]),
                _ => format!("{}\n\n{section}\n", text.trim_end()),
            }
        }
        None => format!(
            "# Live eval: {} ({date})\n\nRun by `emmet eval --rung 1` (plan section 16.1, D-38): one run per agent on the eval API \
             keys, headless, with questions answered from the rung's script in docs/spec/ladder.md.\n\n**Prompt:** {}\n\n\
             **Features:** {}\n\n{section}\n",
            rung.title, rung.prompt, rung.features
        ),
    };
    emmet_core::fsx::write_atomic(path, text.as_bytes())
}

/// Asks before a live run (D-38): what it runs, what it bills, and what it costs.
fn confirm(agent: AgentKind) -> Result<bool> {
    let (key, provider) = match agent {
        AgentKind::Claude => ("emmet-eval-anthropic", "Anthropic"),
        AgentKind::Codex => ("emmet-eval-openai", "OpenAI"),
    };
    eprintln!(
        "Live eval: rung 1 on {}, 1 run, billed to your {provider} API key (keychain item {key}).\nExpected: 5 to 10 \
         minutes and about $1 to $5. It stops at 10 minutes (plus 2 to finish) or {MAX_TURNS} turns.\nRun it? [y/N] ",
        agent.label()
    );
    let mut line = String::new();
    std::io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| Error::internal(format!("reading the answer: {e}")))?;
    Ok(matches!(line.trim(), "y" | "Y" | "yes"))
}

/// Offers to save the run's transcript, scrubbed, as a replay fixture (plan section 10).
fn offer_fixture(agent: AgentKind, r: &Report) -> Result<()> {
    let Some(transcript) = &r.run.transcript else {
        return Ok(());
    };
    let repo = repo_dir()?;
    let out = repo.join(format!("tests/replay/{}/eval-rung1.jsonl", name(agent)));
    eprint!(
        "Save the scrubbed transcript as {}? [y/N] ",
        out.strip_prefix(&repo).unwrap_or(&out).display()
    );
    let _ = std::io::stderr().flush();
    let mut line = String::new();
    let _ = std::io::stdin().lock().read_line(&mut line);
    if !matches!(line.trim(), "y" | "Y" | "yes") {
        return Ok(());
    }
    let status = std::process::Command::new("node")
        .arg(repo.join("scripts/scrub-transcript"))
        .arg(transcript)
        .arg(&out)
        .arg("--path")
        .arg(format!("{}=<WS>/project", r.project))
        .status()
        .map_err(|e| Error::internal(format!("running scripts/scrub-transcript: {e}")))?;
    if !status.success() {
        return Err(Error::internal("scripts/scrub-transcript failed; nothing was saved."));
    }
    eprintln!("Saved {}.", out.display());
    Ok(())
}

/// The repo this emmet was built from (eval is a dev tool).
fn repo_dir() -> Result<PathBuf> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    emmet_core::fsx::canonical(&dir).map_err(|e| Error::io(&dir, e))
}

fn name(agent: AgentKind) -> &'static str {
    match agent {
        AgentKind::Claude => "claude",
        AgentKind::Codex => "codex",
    }
}

fn usage(message: String) -> Error {
    Error::new(
        ErrorCode::ParamInvalid,
        message,
        "Run emmet eval --rung 1 --agent claude|codex.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use emmet_agent::QuestionOption;

    fn q(text: &str, options: &[&str]) -> Question {
        Question {
            question: text.into(),
            header: None,
            options: options
                .iter()
                .map(|o| QuestionOption {
                    label: o.to_string(),
                    description: None,
                })
                .collect(),
            multi: false,
        }
    }

    #[test]
    fn rung_1_reads_from_the_ladder() {
        let r = rung(LADDER, "1").expect("rung 1");
        assert!(
            r.prompt
                .starts_with("Build a small cottage on a green 32 × 32 baseplate"),
            "{}",
            r.prompt
        );
        assert!(r.prompt.ends_with("and whether it has a chimney."));
        assert!(
            r.features.contains("3811") && r.features.ends_with("when the chimney parameter is on."),
            "{}",
            r.features
        );
        assert_eq!(r.script.len(), 6);
        assert_eq!(r.script[1].0, ["size", "footprint", "how big"]);
    }

    #[test]
    fn questions_get_the_scripted_answer() {
        let script = rung(LADDER, "1").unwrap().script;
        // An answer that names an option chooses it.
        let (a, said) = answer(
            &script,
            &q("Which style?", &["Modern", "Classic: tan walls and a dark red roof"]),
        );
        assert_eq!(a.choices, ["Classic: tan walls and a dark red roof"]);
        assert_eq!(said, "Classic: tan walls and a dark red roof");
        // One that doesn't is the Other text; a phrase matches when all its words appear.
        let (a, _) = answer(&script, &q("How big should the house be?", &["Small", "Large"]));
        assert_eq!(a.other.as_deref(), Some("8 studs wide and 6 deep"));
        // No entry: the recommended (first) option.
        let (a, said) = answer(&script, &q("Should the door face north?", &["Yes", "No"]));
        assert_eq!(a.choices, ["Yes"]);
        assert!(said.contains("recommended"));
    }

    #[test]
    fn approvals_allow_emmet_tools_and_the_projects_model_files_only() {
        let root = Path::new("/p/rung1");
        let ok = |kind, tool, detail| allowed(root, kind, tool, detail);
        assert!(ok(ApprovalKind::Tool, "mcp__emmet__run_model", ""));
        assert!(ok(ApprovalKind::Edit, "Edit", "model.py"));
        assert!(ok(ApprovalKind::Edit, "Edit", "/p/rung1/model.py"));
        assert!(ok(ApprovalKind::Edit, "fileChange", "/p/rung1/models/walls.py"));
        assert!(ok(
            ApprovalKind::Edit,
            "fileChange",
            "/p/rung1/model.py, /p/rung1/models/roof.py (a reason)"
        ));
        assert!(!ok(
            ApprovalKind::Edit,
            "fileChange",
            "/p/rung1/model.py, /p/rung1/CARD.md"
        ));
        assert!(!ok(ApprovalKind::Edit, "Write", "CARD.md"));
        assert!(!ok(ApprovalKind::Edit, "Write", "../model.py"));
        assert!(
            !ok(ApprovalKind::Edit, "Write", "/elsewhere/model.py"),
            "a model.py outside the project"
        );
        assert!(!ok(ApprovalKind::Edit, "Write", "/p/rung1/models/deep/x.py"));
        assert!(!ok(ApprovalKind::Edit, "Write", "/p/rung1/models/../../etc/model.py"));
        assert!(!ok(ApprovalKind::Command, "Bash", "rm -rf build"));
        assert!(!ok(ApprovalKind::Permission, "WebFetch", "https://example.com"));
    }
}
