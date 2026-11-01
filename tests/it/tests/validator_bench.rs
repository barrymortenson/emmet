// SPDX-License-Identifier: AGPL-3.0-or-later
//! The validator bench (plan section 11): validate, warm, on 500- and 2,000-part models, against ≤ 300 ms and ≤ 1.5 s
//! on the M-series runner. The models are the committed OMR fixtures side by side, as many copies as it takes, so the
//! part mix is a real one; each copy keeps four spaces clear, a door's and a seat's worth, so figure_clearance is timed
//! doing its work (3E). `just bench validator` runs it in a release build (the numbers mean nothing in a debug
//! one, where it only prints them) and writes tests/e2e/out/bench-validator-<os>.md.

use std::time::{Duration, Instant};

use emmet_core::check::KeepClear;
use emmet_core::ldraw::placements::Placement;
use emmet_it::corpus::{models_in, placements_of, validate_keeping};
use emmet_it::{fixture, repo, shared_library};

/// The fixtures side by side on a grid, 1,000 LDU apart, until there are at least `n` parts, and four spaces kept clear
/// around each copy's origin, 3 studs by 4 by 6 bricks, as a door's or a seat's are.
fn scene(n: usize) -> (Vec<Placement>, Vec<KeepClear>) {
    let models: Vec<Vec<Placement>> = models_in(&fixture("omr")).iter().map(|f| placements_of(f)).collect();
    let mut out = Vec::new();
    let mut keep = Vec::new();
    let mut k = 0;
    while out.len() < n {
        let (gx, gz) = ((k % 8) as f64 * 1000.0, (k / 8) as f64 * 1000.0);
        for (dx, dz) in [(-80.0, -80.0), (20.0, -80.0), (-80.0, 20.0), (20.0, 20.0)] {
            keep.push(KeepClear {
                boxes: vec![[gx + dx, -144.0, gz + dz, gx + dx + 60.0, -8.0, gz + dz + 80.0]],
                why: "a figure".into(),
                src: String::new(),
            });
        }
        for p in &models[k % models.len()] {
            let mut q = p.clone();
            q.id = format!("{k}/{}", p.id);
            q.matrix[12] += gx;
            q.matrix[14] += gz;
            out.push(q);
        }
        k += 1;
    }
    (out, keep)
}

/// The median of `runs` validations after one to warm the caches.
fn time((placements, keep): &(Vec<Placement>, Vec<KeepClear>), runs: usize) -> Duration {
    validate_keeping(shared_library(), placements, keep);
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let started = Instant::now();
            validate_keeping(shared_library(), placements, keep);
            started.elapsed()
        })
        .collect();
    times.sort();
    times[runs / 2]
}

#[test]
#[ignore]
fn validator_bench() {
    let release = !cfg!(debug_assertions);
    let mut table = String::from("| Parts | Budget | Median of 5, warm | |\n|---|---|---|---|\n");
    let mut over = Vec::new();
    for (n, budget) in [(500, 300), (2000, 1500)] {
        let placements = scene(n);
        let t = time(&placements, 5).as_millis();
        let (placements, _) = placements;
        let ok = t <= budget;
        table += &format!(
            "| {} | ≤ {budget} ms | {t} ms | {} |\n",
            placements.len(),
            if ok { "ok" } else { "over" }
        );
        if !ok {
            over.push(format!("{} parts: {t} ms, budget {budget} ms", placements.len()));
        }
    }
    let os = std::env::consts::OS;
    let build = if release {
        "release"
    } else {
        "debug (not the budget's build)"
    };
    let report = format!("## Validator bench ({os}, {build})\n\n{table}");
    println!("{report}");
    let out = repo().join("tests/e2e/out");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join(format!("bench-validator-{os}.md")), &report).unwrap();
    if release {
        assert!(over.is_empty(), "over the validator's budget:\n{}", over.join("\n"));
    }
}
