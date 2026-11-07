// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet export --format pdf` (session 4E, ADR 0014): the PDF render mode draws and emmet lays out, read back. Every
//! main step is on a page once, in order, with its parts box's counts; every sub-assembly's sub-steps are in its box,
//! with "Nx" when the step puts on more than one; the pages are numbered from 2; the file's cross-reference table
//! points at its objects, and every picture is a JPEG. 4E's Done when 2 (with the page baselines in
//! tests/e2e/specs/instructions.e2e.mjs) and What good looks like 2 (a 500-part model in under 30 seconds).

use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

use emmet_it::{emmet_in, project};
use serde_json::Value;

/// A PDF's objects by number, from its cross-reference table (which this checks points at each one).
struct Pdf<'a> {
    bytes: &'a [u8],
    objects: HashMap<usize, usize>,
}

impl<'a> Pdf<'a> {
    fn parse(bytes: &'a [u8]) -> Self {
        assert!(bytes.starts_with(b"%PDF-1.4\n"), "not a PDF 1.4 file");
        let tail = std::str::from_utf8(&bytes[bytes.len().saturating_sub(64)..]).unwrap();
        let start: usize = tail
            .split("startxref\n")
            .nth(1)
            .and_then(|t| t.lines().next())
            .and_then(|n| n.trim().parse().ok())
            .expect("startxref");
        let xref = std::str::from_utf8(&bytes[start..]).unwrap();
        let mut lines = xref.lines();
        assert_eq!(lines.next(), Some("xref"));
        let count: usize = lines.next().unwrap().split(' ').nth(1).unwrap().parse().unwrap();
        let mut objects = HashMap::new();
        for (n, line) in lines.take(count).enumerate().skip(1) {
            let at: usize = line[..10].parse().unwrap();
            let head = format!("{n} 0 obj\n");
            assert!(
                bytes[at..].starts_with(head.as_bytes()),
                "xref entry {n} doesn't point at its object"
            );
            objects.insert(n, at);
        }
        Pdf { bytes, objects }
    }

    /// The object's dictionary (up to its stream or its end) and its stream's bytes, if it has one.
    fn object(&self, n: usize) -> (String, Option<&'a [u8]>) {
        let at = self.objects[&n];
        let body = &self.bytes[at..];
        let text = String::from_utf8_lossy(&body[..body.len().min(4096)]).into_owned();
        // Its dictionary ends at its stream or at its endobj, whichever comes first.
        let end = text.find("\nendobj").unwrap_or(text.len());
        let stream = text.find("\nstream\n").filter(|s| *s < end);
        let dict = text[..stream.unwrap_or(end)].to_string();
        if stream.is_none() {
            return (dict, None);
        }
        let len: usize = dict
            .split("/Length ")
            .nth(1)
            .unwrap()
            .split([' ', '>'])
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let from = at + dict.len() + "\nstream\n".len();
        (dict, Some(&self.bytes[from..from + len]))
    }

    fn pages(&self) -> Vec<usize> {
        let (pages, _) = self.object(2);
        let kids = pages.split("/Kids [").nth(1).unwrap().split(']').next().unwrap();
        kids.split(" 0 R").filter_map(|k| k.trim().parse().ok()).collect()
    }
}

/// A page's text in drawing order: (bold, size, text).
fn texts(content: &str) -> Vec<(bool, f64, String)> {
    content
        .lines()
        .filter(|l| l.starts_with("BT "))
        .map(|l| {
            let mut words = l.split(' ');
            let font = words.nth(1).unwrap();
            let size: f64 = words.next().unwrap().parse().unwrap();
            let open = l.find(") Tj").unwrap();
            let start = l[..open].rfind(" (").unwrap() + 2;
            (
                font == "/F2",
                size,
                l[start..open].replace("\\(", "(").replace("\\)", ")"),
            )
        })
        .collect()
}

/// The counts of a parts box ("2x" each), as steps.json lists them.
fn counts(parts: &Value) -> Vec<String> {
    parts
        .as_array()
        .unwrap()
        .iter()
        .map(|p| format!("{}x", p["count"]))
        .collect()
}

/// Reads the PDF back against build/steps.json: returns the page count.
fn check_book(dir: &Path, pdf: &[u8]) -> usize {
    let steps: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("build/steps.json")).unwrap()).unwrap();
    let pdf = Pdf::parse(pdf);
    let pages = pdf.pages();
    // Every picture is a JPEG.
    let mut images = 0;
    for n in pdf.objects.keys() {
        let (dict, stream) = pdf.object(*n);
        if dict.contains("/Subtype /Image") {
            assert!(dict.contains("/DCTDecode"), "{dict}");
            assert!(stream.unwrap().starts_with(&[0xFF, 0xD8]), "image {n} isn't a JPEG");
            images += 1;
        }
    }
    assert!(images > 0, "no pictures");
    // Walk the pages: counts come before the step number they belong to.
    let mut main = Vec::new();
    let mut subs = Vec::new();
    let mut callout_counts = Vec::new();
    let mut pending: Vec<String> = Vec::new();
    for (i, page) in pages.iter().enumerate() {
        let (dict, _) = pdf.object(*page);
        let contents: usize = dict
            .split("/Contents ")
            .nth(1)
            .unwrap()
            .split(' ')
            .next()
            .unwrap()
            .parse()
            .unwrap();
        let (_, stream) = pdf.object(contents);
        let content = String::from_utf8_lossy(stream.unwrap()).into_owned();
        let mut numbered = None;
        for (bold, size, text) in texts(&content) {
            match (bold, size as u32) {
                (false, 32) => main.push((text.parse::<u32>().unwrap(), std::mem::take(&mut pending))),
                (false, 18) => subs.push((text.parse::<u32>().unwrap(), std::mem::take(&mut pending))),
                (true, 18) => callout_counts.push(text),
                (true, 9) => pending.push(text),
                (false, 11) => numbered = Some(text),
                _ => {}
            }
        }
        if i == 0 {
            assert_eq!(numbered, None, "the cover has no page number");
        } else {
            assert_eq!(
                numbered,
                Some((i + 1).to_string()),
                "page {} is numbered {numbered:?}",
                i + 1
            );
        }
    }
    // The main steps, once each and in order, each with its parts box.
    let want: Vec<(u32, Vec<String>)> = steps["steps"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| (s["n"].as_u64().unwrap() as u32, counts(&s["parts"])))
        .collect();
    assert_eq!(main, want, "the main steps on the pages aren't steps.json's");
    // Each sub-assembly's steps, in its box before the step that puts it on, each time a step puts it on; a
    // sub-assembly inside one gets its own box first (a chair before the table set it goes into).
    fn walk(steps: &Value, placed: &Value, subs: &mut Vec<(u32, Vec<String>)>, callouts: &mut Vec<String>) {
        let model = steps["submodels"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["file"] == placed["file"])
            .unwrap();
        for sub in model["steps"].as_array().unwrap() {
            for inner in sub["submodels"].as_array().unwrap() {
                walk(steps, inner, subs, callouts);
            }
        }
        for sub in model["steps"].as_array().unwrap() {
            subs.push((sub["n"].as_u64().unwrap() as u32, counts(&sub["parts"])));
        }
        if placed["count"].as_u64().unwrap() > 1 {
            callouts.push(format!("{}x", placed["count"]));
        }
    }
    let mut want_subs = Vec::new();
    let mut want_callouts = Vec::new();
    for s in steps["steps"].as_array().unwrap() {
        for placed in s["submodels"].as_array().unwrap() {
            walk(&steps, placed, &mut want_subs, &mut want_callouts);
        }
    }
    assert_eq!(
        subs, want_subs,
        "the sub-assemblies' steps on the pages aren't steps.json's"
    );
    // A box split over two pages says its "Nx" on each.
    let mut said = callout_counts.clone();
    said.dedup();
    let mut want = want_callouts.clone();
    want.dedup();
    assert_eq!(said, want, "the sub-assembly boxes' counts");
    pages.len()
}

fn export(dir: &Path, paper: &str) -> (Value, Vec<u8>) {
    let (v, code) = emmet_in(dir, &["export", "--format", "pdf", "--paper", paper]);
    assert_eq!(code, 0, "{v}");
    let path = v["data"]["path"].as_str().unwrap().to_string();
    (v, std::fs::read(path).unwrap())
}

#[test]
fn a_sub_assembly_built_four_times_is_one_box_marked_4x() {
    let dir = project("submodel-x4");
    let (v, pdf) = export(&dir, "letter");
    assert_eq!(v["data"]["paper"], "letter");
    let pages = check_book(&dir, &pdf);
    assert_eq!(v["data"]["pages"], pages);
    // A4 is a different page.
    let (_, a4) = export(&dir, "a4");
    assert!(String::from_utf8_lossy(&a4).contains("/MediaBox [0 0 595.28 841.89]"));
    assert!(String::from_utf8_lossy(&pdf).contains("/MediaBox [0 0 612 792]"));
}

// A table set built apart, with its chair built apart inside it: the chair's box (2x) comes before the table set's,
// and both before the step that puts the table set on. The chair's steps were drawn but never laid out.
#[test]
fn a_sub_assembly_inside_a_sub_assembly_gets_its_own_box() {
    let dir = project("nested-submodel");
    let (_, pdf) = export(&dir, "letter");
    check_book(&dir, &pdf);
    let steps: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("build/steps.json")).unwrap()).unwrap();
    let nested = steps["submodels"].as_array().unwrap().iter().any(|m| {
        m["steps"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| !s["submodels"].as_array().unwrap().is_empty())
    });
    assert!(nested, "the fixture should nest a sub-assembly: {steps}");
}

// An agent changed the model while the window drew the book's pictures: the pictures and the parts boxes would come
// from two different builds, so writing refuses, before anything is laid out.
#[test]
fn a_model_changed_while_drawing_is_refused() {
    use emmet_core::{data::DataDir, instructions, project::Project};
    let dir = project("submodel-x4");
    let (_, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0);
    let data = DataDir::at(emmet_it::shared_data());
    let jobs = instructions::project_jobs(&data, &Project::find(Some(&dir)).unwrap()).unwrap();
    let (v, code) = emmet_in(&dir, &["params", "--set", "rows=5"]);
    assert_eq!(code, 0, "{v}");
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let picture = instructions::Picture {
        jpeg: String::new(),
        width: 1,
        height: 1,
    };
    let drawn = instructions::Drawn {
        cover: picture,
        runs: Vec::new(),
        parts: Vec::new(),
    };
    let project = Project::find(Some(&dir)).unwrap();
    let err = instructions::write(&data, &project, &jobs, &drawn, instructions::Paper::Letter, None).unwrap_err();
    assert_eq!(
        err.message, "The model changed while Emmet drew the instructions.",
        "{err:?}"
    );
    assert!(!dir.join("build/export").exists(), "nothing was written");
}

#[test]
fn the_rung_2_station_matches_its_steps() {
    let dir = project("rung2-claude");
    let (_, pdf) = export(&dir, "a4");
    check_book(&dir, &pdf);
}

/// What good looks like 2: a 500-part model exports in under 30 seconds. Rung 1's Cottage is the harder case: 905
/// parts in 160 steps and 13 sub-assemblies, every one of them a picture. Also 4E's Done when 2 for rung 1.
#[test]
fn the_cottage_exports_in_under_30_seconds_and_matches_its_steps() {
    let dir = emmet_it::temp_dir("instructions-cottage").join("cottage");
    let out = emmet_it::emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(emmet_it::shared_data())
        .arg("new")
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", emmet_it::text(&out));
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let parts = v["data"]["parts"].as_u64().unwrap();
    assert!(parts >= 500, "{parts} parts");
    let started = Instant::now();
    let (_, pdf) = export(&dir, "letter");
    let took = started.elapsed().as_secs_f64();
    let pages = check_book(&dir, &pdf);
    println!(
        "instructions for {parts} parts: {took:.1} s, {pages} pages, {:.1} MB",
        pdf.len() as f64 / 1e6
    );
    // Enforced on the self-hosted Mac; hosted runners (software WebGL) only report it, as perf.e2e.mjs does.
    if std::env::var("RUNNER_ENVIRONMENT").as_deref() != Ok("github-hosted") {
        assert!(took < 30.0, "the export took {took:.1} s");
    }
}

/// A step whose new parts are out of sight turns the model, and its page shows the turn icon (the only path drawn
/// 2.2 points wide).
#[test]
fn parts_behind_a_wall_turn_the_model() {
    let dir = project("turn-wall");
    let (_, pdf) = export(&dir, "a4");
    check_book(&dir, &pdf);
    let text = String::from_utf8_lossy(&pdf);
    assert_eq!(text.matches(" 2.2 w").count(), 1, "one turn icon");
    // Drawn just after step 3's number: the bricks behind the wall.
    let icon = text.find(" 2.2 w").unwrap();
    let before = &text[..icon];
    let last_step = before.rfind("/F1 32 Tf").unwrap();
    assert!(
        before[last_step..].contains("(3) Tj"),
        "the turn icon isn't with step 3"
    );
}

/// A shared project can carry a build/ of its own: an export never writes through a link in it.
#[cfg(unix)]
#[test]
fn an_export_never_writes_through_a_linked_folder() {
    let dir = project("submodel-x4");
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let elsewhere = emmet_it::temp_dir("linked-export");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let _ = std::fs::remove_dir_all(dir.join("build/export"));
    std::os::unix::fs::symlink(&elsewhere, dir.join("build/export")).unwrap();
    for format in ["ldr", "pdf"] {
        let (v, code) = emmet_in(&dir, &["export", "--format", format]);
        assert_ne!(code, 0, "{format}: {v}");
        assert_eq!(v["error"]["code"], "IO_ERROR", "{v}");
    }
    assert_eq!(
        std::fs::read_dir(&elsewhere).unwrap().count(),
        0,
        "something was written through the link"
    );
}
