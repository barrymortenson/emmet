// SPDX-License-Identifier: AGPL-3.0-or-later
//! The photo rung (docs/spec/ladder.md, rung P; session 4D): four real subjects, two angles each, in
//! `data/ladder/photo/subjects.toml`. Its live runs are Barry's own hand sessions in the app on his plans (Barry
//! 2026-11-04): Emmet never sends a turn for them. So `emmet eval --rung photo` does one of two things:
//!
//! - `--score <project>` reads a finished hand session (the project and its chat in `.emmet/chat/`), checks the
//!   subject's photos on this Mac against their hashes, and scores it: 0 validator errors, the part range, that it
//!   started from the photo, whether the agent asked for another angle, the time to the first draft, and Barry's
//!   three rubric scores. It sends nothing to an agent.
//! - `--dry-run` drives a fake agent the same way, for CI: angle a with "Build this.", a question answered from the
//!   scripts, and angle b as the next turn when the agent asks for another angle; generated images stand in for the
//!   photos, which never leave Barry's Mac.
//!
//! The report (`docs/evals/<date>-photo.md`) has the renders and the scores, never the photos.

use std::path::{Path, PathBuf};
use std::time::Instant;

use emmet_agent::{AgentEvent, AgentKind, Decision, DoneReason, Question, UserTurn};
use emmet_core::data::DataDir;
use emmet_core::envelope::{Error, ErrorCode, Result};
use emmet_core::project::{Project, Template};
use serde::Deserialize;
use serde_json::Value;

use crate::eval::{Report, Run, Score, answer, repo_dir, rung};

/// What the first draft should beat (plan section 14, "a draft in the first minute").
const DRAFT_SECONDS: u64 = 60;
/// A subject passes at 4 of the rubric's 6 (section 16.1).
const RUBRIC_PASS: u8 = 4;
/// The words that make a question, or the end of a turn, a request for another photo (the ladder's answer script).
const ANOTHER_ANGLE: &[&str] = &[
    "another angle",
    "another photo",
    "another view",
    "second photo",
    "second angle",
    "other side",
    "from the side",
    "from the back",
    "side view",
    "back view",
    "photo of the side",
    "photo of the back",
];
/// What the user says with angle b, unless the subject's note for it says more.
const ANGLE_B: &str = "Here's another angle.";

pub struct Options {
    pub agent: AgentKind,
    pub subject: String,
    pub dry_run: bool,
    /// A hand session's project to score, instead of a run.
    pub score: Option<PathBuf>,
    /// Barry's scores: proportions, recognizability, buildability, 0 to 2 each.
    pub rubric: Option<[u8; 3]>,
    pub out: Option<PathBuf>,
    pub work: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct Subjects {
    subject: Vec<Subject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Subject {
    pub id: String,
    pub description: String,
    pub parts: [usize; 2],
    #[serde(default)]
    pub answers: Vec<SubjectAnswer>,
    pub images: Vec<Image>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubjectAnswer {
    pub when: Vec<String>,
    pub answer: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub angle: String,
    pub file: String,
    pub sha256: String,
    #[serde(default)]
    pub note: Option<String>,
}

fn usage(message: impl Into<String>, hint: impl Into<String>) -> Error {
    Error::new(ErrorCode::ParamInvalid, message, hint)
}

/// The rung's folder: `subjects.toml` and the gitignored `local/` with the photos. `EMMET_PHOTO_DIR` points elsewhere
/// (the tests' own subjects and photos).
fn photo_dir() -> Result<PathBuf> {
    match std::env::var_os("EMMET_PHOTO_DIR") {
        Some(d) => Ok(PathBuf::from(d)),
        None => Ok(repo_dir()?.join("data/ladder/photo")),
    }
}

/// The subjects, from `subjects.toml`.
pub fn subjects() -> Result<Vec<Subject>> {
    let path = photo_dir()?.join("subjects.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
    let s: Subjects = toml::from_str(&text).map_err(|e| Error::internal(format!("{}: {e}", path.display())))?;
    Ok(s.subject)
}

impl Subject {
    fn image(&self, angle: &str) -> Option<&Image> {
        self.images.iter().find(|i| i.angle == angle)
    }

    /// The subject's own answers first, then the rung's shared script.
    fn script(&self, shared: &[(Vec<String>, String)]) -> Vec<(Vec<String>, String)> {
        let own = self
            .answers
            .iter()
            .map(|a| (a.when.iter().map(|w| w.to_lowercase()).collect(), a.answer.clone()));
        own.chain(shared.iter().cloned()).collect()
    }
}

/// The subject's photo for `angle` on this Mac (`data/ladder/photo/local/`), checked against its hash.
fn local_photo(subject: &Subject, angle: &str) -> Result<Vec<u8>> {
    let image = subject
        .image(angle)
        .ok_or_else(|| Error::internal(format!("{} has no angle {angle}", subject.id)))?;
    let path = photo_dir()?.join("local").join(&image.file);
    let bytes = std::fs::read(&path).map_err(|_| {
        Error::new(
            ErrorCode::IoError,
            format!("The photo {} isn't on this Mac.", image.file),
            "The photo rung's photos stay out of the repo: copy them into data/ladder/photo/local/ (H-23).",
        )
    })?;
    if emmet_core::fsx::sha256_hex(&bytes) != image.sha256 {
        return Err(Error::new(
            ErrorCode::IoError,
            format!("{} isn't the photo subjects.toml names (its hash differs).", image.file),
            "Put the original photo back, or record the new one's hash in subjects.toml with Barry's say-so.",
        ));
    }
    Ok(bytes)
}

/// What a session's chat shows, for scoring: the user's messages (with how many photos) and the agent's events, each
/// with its time (Unix ms).
#[derive(Debug)]
enum Seen {
    User { photos: usize },
    Event(AgentEvent),
}

/// A session's messages and events in order, each with its time (Unix ms).
type Timeline = Vec<(f64, Seen)>;

/// What the scores read from a session.
#[derive(Debug, Default)]
struct Observed {
    from_photo: bool,
    asked_angle: bool,
    angle_sent: bool,
    draft_seconds: Option<u64>,
    seconds: u64,
    turns: usize,
    questions: Vec<String>,
    last_message: String,
}

fn mentions_another_angle(text: &str) -> bool {
    let t = text.to_lowercase();
    ANOTHER_ANGLE.iter().any(|p| t.contains(p))
}

fn observe(seen: &[(f64, Seen)]) -> Observed {
    let mut o = Observed::default();
    let Some(start) = seen
        .iter()
        .find(|(_, s)| matches!(s, Seen::User { .. }))
        .map(|(at, _)| *at)
    else {
        return o;
    };
    let mut runs: Vec<String> = Vec::new();
    let mut photo_turns = 0;
    let mut users = 0;
    // The agent's words since the last message: a turn that ends asking for another angle.
    let mut words = String::new();
    for (at, s) in seen {
        match s {
            Seen::User { photos } => {
                users += 1;
                if *photos > 0 {
                    photo_turns += 1;
                    // Built from the photo: the session's first message carried it, before any step or question.
                    if users == 1 && o.turns == 0 && o.questions.is_empty() {
                        o.from_photo = true;
                    }
                    if photo_turns >= 2 {
                        o.angle_sent = true;
                    }
                }
                words.clear();
            }
            Seen::Event(e) => match e {
                AgentEvent::ToolCall { id, name, .. } => {
                    o.turns += 1;
                    words.clear();
                    if name.ends_with("run_model") {
                        runs.push(id.clone());
                    }
                }
                AgentEvent::ToolResult { id, ok: true, .. } if runs.contains(id) && o.draft_seconds.is_none() => {
                    o.draft_seconds = Some(((at - start) / 1000.0).max(0.0) as u64);
                }
                AgentEvent::Question { question, .. } => {
                    o.questions.push(question.question.clone());
                    if photo_turns < 2 && mentions_another_angle(&question.question) {
                        o.asked_angle = true;
                    }
                }
                AgentEvent::Text { delta, .. } => words.push_str(delta),
                AgentEvent::TurnDone { .. } => {
                    if photo_turns < 2 && mentions_another_angle(&words) {
                        o.asked_angle = true;
                    }
                    if !words.trim().is_empty() {
                        o.last_message = words.trim().to_string();
                    }
                }
                _ => {}
            },
        }
        o.seconds = ((at - start) / 1000.0).max(0.0) as u64;
    }
    o
}

/// Runs the photo rung (`--dry-run`) or scores a hand session (`--score`), and writes the report.
pub fn eval(data: &DataDir, opts: &Options) -> Result<Report> {
    let subjects = subjects()?;
    let subject = subjects.iter().find(|s| s.id == opts.subject).cloned().ok_or_else(|| {
        let ids: Vec<&str> = subjects.iter().map(|s| s.id.as_str()).collect();
        usage(
            format!("There's no photo subject '{}'.", opts.subject),
            format!("Pick one with --subject: {}.", ids.join(", ")),
        )
    })?;
    let ladder = rung(crate::eval::LADDER, "P").ok_or_else(|| Error::internal("docs/spec/ladder.md has no rung P"))?;
    let (project, run, observed) = match (&opts.score, opts.dry_run) {
        (Some(dir), false) => {
            // The photos on this Mac must be the ones the subject names, so the score is for the same photos.
            local_photo(&subject, "a")?;
            local_photo(&subject, "b")?;
            let project = Project::find(Some(dir))?;
            let (run, seen) = read_session(&project, opts.agent)?;
            (project, run, observe(&seen))
        }
        (None, true) => {
            let (project, run, seen) = dry_run(data, opts, &subject, &ladder.script)?;
            (project, run, observe(&seen))
        }
        (Some(_), true) => {
            return Err(usage(
                "--score and --dry-run don't go together.",
                "Score a hand session, or run the dry run.",
            ));
        }
        (None, false) => {
            return Err(usage(
                "The photo rung's live runs are your hand sessions in the app (Barry 2026-11-04): Emmet doesn't run them.",
                format!(
                    "Run it in the app on your plan (drop angle a, type \"Build this.\", answer as the rung's script says), \
                     then: emmet eval --rung photo --subject {} --agent {} --score <project>",
                    subject.id,
                    crate::eval::name(opts.agent)
                ),
            ));
        }
    };
    let rubric = match opts.rubric {
        Some(r) => Some(r),
        None if opts.score.is_some() => ask_rubric(&subject)?,
        None => None,
    };
    let mut run = run;
    run.turns = observed.turns;
    run.seconds = run.seconds.max(observed.seconds);
    run.questions = observed
        .questions
        .iter()
        .map(|q| (q.clone(), String::new()))
        .collect::<Vec<_>>();
    if run.last_message.is_empty() {
        run.last_message = observed.last_message.clone();
    }
    let scores = score(data, &project, &subject, &observed, rubric);
    let pass = scores.iter().filter(|s| s.required).all(|s| s.pass);

    let out_dir = match &opts.out {
        Some(o) => o.clone(),
        None => repo_dir()?.join("docs/evals"),
    };
    std::fs::create_dir_all(&out_dir).map_err(|e| Error::io(&out_dir, e))?;
    let date = emmet_agent::time::rfc3339(emmet_agent::time::now_ms())[..10].to_string();
    let stem = format!("{date}-photo-{}-{}", subject.id, crate::eval::name(opts.agent));
    let png_name = (1..)
        .map(|n| {
            if n == 1 {
                format!("{stem}.png")
            } else {
                format!("{stem}-{n}.png")
            }
        })
        .find(|f| !out_dir.join(f).exists())
        .expect("a free name");
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
        rung: format!("{}: {}", ladder.title, subject.id),
        run,
        scores,
        pass,
        render,
        project: project.root.display().to_string(),
    };
    write_report(
        &out_dir.join(format!("{date}-photo.md")),
        &subject,
        &report,
        &observed,
        opts,
        &date,
    )?;
    Ok(report)
}

/// The newest conversation with `agent` in the project's `.emmet/chat/`, as the app wrote it.
fn read_session(project: &Project, agent: AgentKind) -> Result<(Run, Timeline)> {
    let chat = project.root.join(".emmet/chat");
    let sessions_path = chat.join("sessions.json");
    let sessions: Value = std::fs::read_to_string(&sessions_path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .ok_or_else(|| {
            Error::new(
                ErrorCode::IoError,
                format!("{} has no chat to score.", project.root.display()),
                "Run the subject in the app first: the chat is kept in the project's .emmet/chat/.",
            )
        })?;
    let want = crate::eval::name(agent);
    let id = sessions["sessions"]
        .as_array()
        .into_iter()
        .flatten()
        .rev()
        .find(|s| s["kind"] == want)
        .and_then(|s| s["id"].as_str())
        .ok_or_else(|| {
            Error::new(
                ErrorCode::IoError,
                format!("The project has no conversation with {}.", agent.label()),
                "Score the agent the session ran with (--agent).",
            )
        })?
        .to_string();
    let log = chat.join(format!("{id}.jsonl"));
    let text = std::fs::read_to_string(&log).map_err(|e| Error::io(&log, e))?;
    let mut seen = Vec::new();
    let mut run = Run {
        agent: agent.label().into(),
        ended: "completed".into(),
        transcript: Some(
            project
                .root
                .join(".emmet/transcripts")
                .join(format!("{id}.jsonl"))
                .display()
                .to_string(),
        ),
        ..Default::default()
    };
    // Lines this build can't read (a chat from an older or newer Emmet) are counted and reported, so a score never
    // quietly misses them.
    let mut unread = 0;
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let Some((r, at)) = serde_json::from_str::<Value>(line)
            .ok()
            .and_then(|r| r["at"].as_f64().map(|at| (r, at)))
        else {
            unread += 1;
            continue;
        };
        if r["user"].is_string() {
            let photos = r["photos"].as_array().map_or(0, Vec::len);
            seen.push((at, Seen::User { photos }));
        } else if let Some(Ok(e)) = r
            .get("event")
            .filter(|e| !e.is_null())
            .map(|e| serde_json::from_value::<AgentEvent>(e.clone()))
        {
            match &e {
                AgentEvent::Started { model, billing, .. } => {
                    run.model = model.clone().or(run.model.take());
                    run.billing = billing.as_ref().map(|b| b.label()).or(run.billing.take());
                }
                AgentEvent::Usage { input, output } => {
                    run.tokens.0 += input;
                    run.tokens.1 += output;
                }
                AgentEvent::Error { message, .. } => run.errors.push(message.clone()),
                _ => {}
            }
            seen.push((at, Seen::Event(e)));
        } else if r.get("event").is_some_and(|e| !e.is_null()) {
            unread += 1;
        }
    }
    if unread > 0 {
        let lines = if unread == 1 { "line" } else { "lines" };
        run.errors.push(format!(
            "{unread} {lines} of the chat couldn't be read, so the scores may miss steps"
        ));
    }
    Ok((run, seen))
}

/// The dry run with a fake agent: angle a with the rung's prompt, questions answered from the scripts, and angle b as
/// the next turn when the agent asks for another angle.
fn dry_run(
    data: &DataDir,
    opts: &Options,
    subject: &Subject,
    shared: &[(Vec<String>, String)],
) -> Result<(Project, Run, Timeline)> {
    let env: Vec<(String, String)> = std::env::vars().collect();
    let adapter = emmet_agent::adapter(opts.agent, &env, data.root());
    if adapter.detect().path.is_none() {
        return Err(Error::internal(format!("{} isn't installed.", opts.agent.label())));
    }
    let work = match &opts.work {
        Some(w) => w.clone(),
        None => std::env::temp_dir().join(format!("emmet-eval-{}", emmet_core::fsx::random_hex(1))),
    };
    let root = work.join(format!("photo-{}-{}", subject.id, crate::eval::name(opts.agent)));
    let created = emmet_core::ops::new_project(data, &root, Template::Empty)?;
    let project = Project::find(Some(Path::new(&created.project)))?;
    // Generated stand-ins: the subject's photos stay on Barry's Mac.
    let fixture = |angle: &str| -> Result<PathBuf> {
        let src = repo_dir()?.join(format!("tests/fixtures/photos/angle-{angle}.jpg"));
        let bytes = std::fs::read(&src).map_err(|e| Error::io(&src, e))?;
        let saved = emmet_core::refs::save(&project.root, &format!("angle-{angle}.jpg"), &bytes)?;
        Ok(project.root.join(saved.path))
    };
    let script = subject.script(shared);
    let live = crate::eval::start(data, opts.agent, adapter.as_ref(), &project.root, env, None)?;
    let (mut session, events, _bus) = (live.session, live.events, live.bus);
    let mut run = Run {
        agent: opts.agent.label().into(),
        cli_version: adapter.detect().version,
        ..Default::default()
    };
    let started = Instant::now();
    let now = || emmet_agent::time::now_ms() as f64;
    let mut seen = vec![(now(), Seen::User { photos: 1 })];
    let first = UserTurn {
        text: "Build this.".into(),
        images: vec![fixture("a")?],
    };
    session
        .send(first)
        .map_err(|e| Error::internal(format!("sending angle a: {}", e.message)))?;
    let mut words = String::new();
    let mut angle_sent = false;
    let mut wants_angle = false;
    loop {
        let event = match events.recv_timeout(std::time::Duration::from_secs(120)) {
            Ok(e) => e,
            Err(_) => {
                run.ended = "the fake agent went quiet".into();
                break;
            }
        };
        match &event {
            AgentEvent::Started { model, billing, .. } => {
                run.model = model.clone();
                run.billing = billing.as_ref().map(|b| b.label());
            }
            AgentEvent::Question { id, question } => {
                let (a, said) = reply(&script, question, &mut wants_angle);
                eprintln!("eval: asked \"{}\" → {said}", question.question);
                let _ = session.answer(id, a);
            }
            AgentEvent::Approval { id, kind, tool, detail } => {
                let ok = crate::eval::allowed(&project.root, *kind, tool, detail);
                run.approvals.push((format!("{tool}: {detail}"), ok));
                let _ = session.respond(id, if ok { Decision::AllowOnce } else { Decision::Deny });
            }
            AgentEvent::Text { delta, .. } => words.push_str(delta),
            AgentEvent::ToolCall { .. } => words.clear(),
            AgentEvent::Error { message, .. } => run.errors.push(message.clone()),
            _ => {}
        }
        let done = match &event {
            AgentEvent::TurnDone { reason } => Some(*reason),
            AgentEvent::Exited { .. } => Some(DoneReason::Crashed),
            _ => None,
        };
        seen.push((now(), Seen::Event(event)));
        let Some(reason) = done else { continue };
        if !angle_sent && reason == DoneReason::Completed && (wants_angle || mentions_another_angle(&words)) {
            angle_sent = true;
            words.clear();
            seen.push((now(), Seen::User { photos: 1 }));
            // The words Barry sends with angle b: the subject's note for it, if it has one.
            let next = UserTurn {
                text: subject
                    .image("b")
                    .and_then(|i| i.note.clone())
                    .unwrap_or_else(|| ANGLE_B.into()),
                images: vec![fixture("b")?],
            };
            session
                .send(next)
                .map_err(|e| Error::internal(format!("sending angle b: {}", e.message)))?;
            continue;
        }
        run.ended = match reason {
            DoneReason::Completed => "completed",
            DoneReason::Interrupted => "stopped",
            DoneReason::Failed => "failed",
            DoneReason::Crashed => "crashed",
        }
        .into();
        break;
    }
    run.seconds = started.elapsed().as_secs();
    session.close();
    Ok((project, run, seen))
}

/// The scripts' answer to a question. A request for another angle can't be answered with a photo on a card: the
/// answer says it comes next, and angle b goes as the next turn (as in the app).
fn reply(script: &[(Vec<String>, String)], q: &Question, wants_angle: &mut bool) -> (emmet_agent::Answer, String) {
    let (a, said) = answer(script, q);
    if said.contains("(the subject's second photo)") || mentions_another_angle(&q.question) {
        *wants_angle = true;
        let text = "I'll send another photo as my next message.".to_string();
        return (
            emmet_agent::Answer {
                choices: vec![],
                other: Some(text.clone()),
            },
            format!("Other: {text}"),
        );
    }
    (a, said)
}

/// Barry's three scores, asked on the terminal.
fn ask_rubric(subject: &Subject) -> Result<Option<[u8; 3]>> {
    use std::io::{BufRead, IsTerminal, Write};
    if !std::io::stdin().is_terminal() {
        return Err(usage(
            "The rubric needs your three scores.",
            "Pass them as --rubric P,R,B (proportions, recognizability, buildability; 0 to 2 each).",
        ));
    }
    let mut out = [0u8; 3];
    eprintln!(
        "Score {} with the photo beside the render (0, 1, or 2 each):",
        subject.description
    );
    for (i, what) in ["Proportions", "Recognizability", "Buildability"].iter().enumerate() {
        loop {
            eprint!("  {what}: ");
            let _ = std::io::stderr().flush();
            let mut line = String::new();
            std::io::stdin()
                .lock()
                .read_line(&mut line)
                .map_err(|e| Error::internal(e.to_string()))?;
            match line.trim().parse::<u8>() {
                Ok(n) if n <= 2 => {
                    out[i] = n;
                    break;
                }
                _ => eprintln!("  0, 1, or 2."),
            }
        }
    }
    Ok(Some(out))
}

/// `--rubric 2,1,2`.
pub fn parse_rubric(s: &str) -> std::result::Result<[u8; 3], String> {
    let n: Vec<u8> = s
        .split(',')
        .map(|p| p.trim().parse::<u8>().ok().filter(|n| *n <= 2))
        .collect::<Option<_>>()
        .ok_or("each score is 0, 1, or 2")?;
    n.try_into()
        .map_err(|_| "three scores: proportions, recognizability, buildability".to_string())
}

fn score(data: &DataDir, project: &Project, subject: &Subject, o: &Observed, rubric: Option<[u8; 3]>) -> Vec<Score> {
    let mut out = Vec::new();
    let mut push = |name: &str, required: bool, pass: bool, detail: String| {
        out.push(Score {
            name: name.into(),
            required,
            pass,
            detail,
        })
    };
    let built = emmet_core::ops::run_model(data, project, &std::collections::BTreeMap::new()).0;
    let checked = built
        .as_ref()
        .ok()
        .map(|_| emmet_core::ops::check_build(data, project, &[]));
    let parts = built.as_ref().map(|r| r.parts).unwrap_or(0);
    match (&built, &checked) {
        (Ok(r), Some(Ok(v))) => push(
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
        (Ok(_), Some(Err(e))) => push(
            "Runs and validates with 0 errors",
            true,
            false,
            format!("the validator couldn't run: {}", e.message),
        ),
        (Ok(_), None) => unreachable!("a model that built is checked"),
        (Err(e), _) => push(
            "Runs and validates with 0 errors",
            true,
            false,
            format!("{:?}: {}", e.code, e.message),
        ),
    }
    let [lo, hi] = subject.parts;
    push(
        "Parts in range",
        true,
        (lo..=hi).contains(&parts),
        format!("{parts} parts ({lo} to {hi})"),
    );
    push(
        "Built from the photo",
        true,
        o.from_photo,
        if o.from_photo {
            "the session's first message carried the photo".into()
        } else {
            "the session's first message had no photo, or a step came before it".into()
        },
    );
    push(
        "Asked for another angle",
        false,
        o.asked_angle,
        match (o.asked_angle, o.angle_sent) {
            (true, true) => "asked, and got angle b".into(),
            (true, false) => "asked; angle b wasn't sent".into(),
            (false, true) => "angle b was sent without a request".into(),
            (false, false) => "didn't ask".into(),
        },
    );
    push(
        "First draft within a minute",
        false,
        o.draft_seconds.is_some_and(|s| s <= DRAFT_SECONDS),
        match o.draft_seconds {
            Some(s) => format!("the first build ran {s} s after the first message (≤ {DRAFT_SECONDS} s)"),
            None => "no build ran".into(),
        },
    );
    match rubric {
        Some([p, r, b]) => push(
            "Barry's rubric",
            true,
            p + r + b >= RUBRIC_PASS,
            format!(
                "proportions {p}, recognizability {r}, buildability {b}: {} of 6 (≥ {RUBRIC_PASS})",
                p + r + b
            ),
        ),
        None => push("Barry's rubric", true, false, "not scored yet (--rubric P,R,B)".into()),
    }
    out
}

fn write_report(path: &Path, subject: &Subject, r: &Report, o: &Observed, opts: &Options, date: &str) -> Result<()> {
    let agent = crate::eval::name(opts.agent);
    let tag = format!("photo-{}-{agent}", subject.id);
    let how = if opts.dry_run {
        "the dry run with the fake agent and generated photos"
    } else {
        "Barry's hand session in the app"
    };
    let mut s = vec![
        format!("<!-- eval:{tag} -->"),
        format!(
            "## {}: {} — {}",
            r.run.agent,
            subject.id,
            if r.pass { "pass" } else { "fail" }
        ),
        String::new(),
        format!(
            "{} ({} to {} parts), from {how}. {} {}, model {}, billed to {}. Emmet {}.",
            subject.description,
            subject.parts[0],
            subject.parts[1],
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
    for sc in &r.scores {
        s.push(format!(
            "| {} | {} | {} | {} |",
            sc.name,
            if sc.required { "yes" } else { "no" },
            if sc.pass { "pass" } else { "fail" },
            sc.detail.replace('|', "\\|")
        ));
    }
    s.push(String::new());
    if let Some(png) = &r.render {
        s.push(format!("![The model {} built of {}]({png})", r.run.agent, subject.id));
        s.push(String::new());
    }
    s.push(format!(
        "{} turns, {} min {} s; {} input and {} output tokens where the CLI reports them.",
        r.run.turns,
        r.run.seconds / 60,
        r.run.seconds % 60,
        r.run.tokens.0,
        r.run.tokens.1
    ));
    s.push(String::new());
    if o.questions.is_empty() {
        s.push("Questions: none.".into());
    } else {
        s.push("Questions:".into());
        s.push(String::new());
        for q in &o.questions {
            s.push(format!("- \"{q}\""));
        }
    }
    if !r.run.errors.is_empty() {
        s.push(String::new());
        s.push("Errors:".into());
        s.push(String::new());
        for e in &r.run.errors {
            s.push(format!("- {e}"));
        }
    }
    s.push(String::new());
    s.push("The agent's last message:".into());
    s.push(String::new());
    let last: Vec<char> = r.run.last_message.trim().chars().collect();
    let last: String = last[last.len().saturating_sub(1200)..].iter().collect();
    for line in last.lines() {
        s.push(format!("> {line}"));
    }
    s.push(format!("<!-- /eval:{tag} -->"));
    let section = s.join("\n");
    // Only a missing report is a new one: one that can't be read isn't written over.
    let existing = match std::fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(Error::internal(format!("{} can't be read: {e}", path.display()))),
    };
    let text = match existing {
        // An earlier run of this subject and agent today stays (D-38's one rerun after a fix): this one follows it.
        Some(text) if text.contains(&format!("<!-- eval:{tag} -->")) => {
            let n = (1..)
                .find(|n| !text.contains(&format!("<!-- eval:{tag}-run{n} -->")))
                .unwrap_or(1);
            let kept = text
                .replace(&format!("<!-- eval:{tag} -->"), &format!("<!-- eval:{tag}-run{n} -->"))
                .replace(
                    &format!("<!-- /eval:{tag} -->"),
                    &format!("<!-- /eval:{tag}-run{n} -->"),
                );
            let section = section.replacen(
                &format!("## {}: {}", r.run.agent, subject.id),
                &format!("## {}: {} (rerun)", r.run.agent, subject.id),
                1,
            );
            format!("{}\n\n{section}\n", kept.trim_end())
        }
        Some(text) => format!("{}\n\n{section}\n", text.trim_end()),
        None => format!(
            "# Live eval: Photo to model ({date})\n\nScored by `emmet eval --rung photo` (docs/spec/ladder.md, rung P): from \
             Barry's hand sessions in the app on his plans with `--score`, which sends nothing to an agent (Barry \
             2026-11-04), or with the fake agents with `--dry-run`. The prompt is \"Build this.\" with a subject's angle a; \
             angle b goes as the next message when the agent asks for another angle. A subject passes at 4 of 6 on the \
             rubric with 0 validator errors and its part count in range; the rung passes for an agent when 3 of the 4 \
             subjects pass. The photos stay on Barry's Mac, so only the renders are here.\n\n{section}\n"
        ),
    };
    emmet_core::fsx::write_atomic(path, text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rubric_flags() {
        assert_eq!(parse_rubric("2,1,2"), Ok([2, 1, 2]));
        assert_eq!(parse_rubric(" 0, 0 ,0"), Ok([0, 0, 0]));
        assert!(parse_rubric("3,1,2").is_err());
        assert!(parse_rubric("2,1").is_err());
        assert!(parse_rubric("2,1,2,2").is_err());
    }
}
