// SPDX-License-Identifier: AGPL-3.0-or-later
//! The LDraw layer against the real files in ldraw-mini: parse, write, measure, flatten.

use emmet_core::ldraw::geometry::{LibFiles, Measurer, is_stud};
use emmet_core::ldraw::placements::placements;
use emmet_core::ldraw::{self, Line, Meta, normalize_ref};
use emmet_it::fixture;

fn mini() -> LibFiles {
    LibFiles::scan(&fixture("ldraw-mini")).expect("scan ldraw-mini")
}

#[test]
fn every_ldraw_mini_file_parses_and_round_trips() {
    let lib = mini();
    let mut count = 0;
    for dir in ["parts", "p"] {
        let mut stack = vec![fixture("ldraw-mini").join(dir)];
        while let Some(d) = stack.pop() {
            for entry in std::fs::read_dir(&d).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap()).into_owned();
                let doc = ldraw::parse(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                let again = ldraw::parse(&ldraw::write(&doc)).unwrap();
                for (a, b) in doc
                    .files
                    .iter()
                    .flat_map(|f| &f.lines)
                    .zip(again.files.iter().flat_map(|f| &f.lines))
                {
                    assert_eq!(a, b, "{} doesn't round-trip", path.display());
                }
                assert_eq!(doc, again, "{} doesn't round-trip", path.display());
                count += 1;
            }
        }
    }
    assert!(count > 400, "only {count} files");
    assert!(lib.parts().len() > 150);
}

#[test]
fn body_boxes_leave_out_studs() {
    let lib = mini();
    let mut m = Measurer::new(&lib);
    let cases: &[(&str, [f64; 6])] = &[
        ("3001", [-40.0, 0.0, -20.0, 40.0, 24.0, 20.0]),    // Brick 2 x 4
        ("3024", [-10.0, 0.0, -10.0, 10.0, 8.0, 10.0]),     // Plate 1 x 1
        ("3811", [-320.0, 0.0, -320.0, 320.0, 4.0, 320.0]), // Baseplate 32 x 32: half a plate thick
        ("3039", [-20.0, 0.0, -30.0, 20.0, 24.0, 10.0]),    // Slope 45 2 x 2: the slope faces -Z
        ("3062b", [-10.0, 0.0, -10.0, 10.0, 24.0, 10.0]),   // Brick 1 x 1 round, hollow stud
        ("3070b", [-10.0, 0.0, -10.0, 10.0, 8.0, 10.0]),    // Tile 1 x 1: no studs to leave out
    ];
    for (part, want) in cases {
        let got = m
            .body_box(&format!("{part}.dat"))
            .unwrap_or_else(|| panic!("{part} has no box"))
            .to_array();
        assert_eq!(&got, want, "{part}");
    }
    assert_eq!(m.body_box("nope.dat"), None);
}

#[test]
fn references_are_case_insensitive_with_either_separator() {
    assert_eq!(normalize_ref(" S\\3001S01.DAT "), "s/3001s01.dat");
    let lib = mini();
    let a = lib.path("s\\3001s01.dat").expect("backslash");
    assert_eq!(Some(a), lib.path("S/3001S01.DAT"));
    assert!(lib.path("48\\1-4CYLI.dat").is_some(), "p/48 primitives resolve");
    assert!(is_stud("STUD.DAT") && is_stud("48\\stud2.dat") && is_stud("stug-2x2.dat") && !is_stud("box5.dat"));
}

#[test]
fn mpd_files_steps_and_emmet_meta() {
    let text = "0 FILE main.ldr\r\n0 Main\n0 !EMMET ID base SRC model.py:4\n1 2 0 0 0 1 0 0 0 1 0 0 0 1 3811.dat\n\
                0 STEP\n0 !EMMET ID roof/1 SRC model.py:9\n1 16 0 -24 0 1 0 0 0 1 0 0 0 1 Sub Model.ldr\n\
                0 NOFILE\n0 FILE sub model.ldr\n1 4 20 0 0 0 0 1 0 1 0 -1 0 0 3001.DAT\n0 NOFILE\n";
    let doc = ldraw::parse(text).unwrap();
    assert_eq!(doc.files.len(), 2);
    assert_eq!(
        doc.files[0].lines[1],
        Line::Meta(Meta::EmmetPart {
            id: "base".into(),
            src: "model.py:4".into()
        })
    );
    assert!(
        doc.file("SUB MODEL.LDR").is_some(),
        "file names match case-insensitively"
    );
    // A file name with spaces survives the round trip.
    assert_eq!(ldraw::parse(&ldraw::write(&doc)).unwrap(), doc);

    let p = placements(&doc);
    assert_eq!(p.len(), 2);
    assert_eq!(
        (p[0].id.as_str(), p[0].part.as_str(), p[0].color, p[0].step),
        ("base", "3811", 2, 1)
    );
    // A part inside a submodel takes the reference's transform, keeps its own color 4 (16 would inherit), and counts
    // steps within its submodel. The !EMMET id before the reference names the reference, not the parts inside.
    assert_eq!(
        (p[1].id.as_str(), p[1].part.as_str(), p[1].color, p[1].step),
        ("part/2", "3001", 4, 1)
    );
    assert_eq!(p[1].submodel, vec!["sub model.ldr".to_string()]);
    assert_eq!(&p[1].matrix[12..15], &[20.0, -24.0, 0.0]);
    // The copy is named by its !EMMET id and placed in the main model's step 2.
    assert_eq!(
        serde_json::to_value(&p[1].copies).unwrap(),
        serde_json::json!([{ "id": "roof/1", "step": 2 }])
    );
}

/// Two copies of a submodel are two copies, and a part embedded in the file (an OMR model's unofficial part) is one
/// placed part, not a submodel of primitives: it used to flatten into `box5`, `4-4cyli`, … each "unknown".
#[test]
fn submodel_copies_and_embedded_parts() {
    let text = "0 FILE main.ldr\n1 16 0 0 0 1 0 0 0 1 0 0 0 1 wing.ldr\n0 STEP\n1 16 40 0 0 1 0 0 0 1 0 0 0 1 wing.ldr\n\
                0 NOFILE\n0 FILE wing.ldr\n1 4 0 0 0 1 0 0 0 1 0 0 0 1 3001.dat\n1 4 0 -24 0 1 0 0 0 1 0 0 0 1 x99.dat\n\
                0 NOFILE\n0 FILE x99.dat\n0 Brick 1 x 1 Special\n0 !LDRAW_ORG Unofficial_Part\n\
                1 16 0 0 0 1 0 0 0 1 0 0 0 1 box5.dat\n0 NOFILE\n";
    let p = placements(&ldraw::parse(text).unwrap());
    let rows: Vec<(String, Vec<(String, u32)>)> = p
        .iter()
        .map(|q| {
            (
                q.part.clone(),
                q.copies.iter().map(|c| (c.id.clone(), c.step)).collect(),
            )
        })
        .collect();
    let copy = |id: &str, step| vec![(id.to_string(), step)];
    assert_eq!(
        rows,
        vec![
            ("3001".into(), copy("main/1", 1)),
            ("x99".into(), copy("main/1", 1)),
            ("3001".into(), copy("main/3", 2)),
            ("x99".into(), copy("main/3", 2)),
        ]
    );
}

#[test]
fn numbers_are_written_compactly() {
    assert_eq!(ldraw::num(40.0), "40");
    assert_eq!(ldraw::num(-0.0), "0");
    assert_eq!(ldraw::num(1.767767), "1.767767");
    assert_eq!(ldraw::num(0.0980171), "0.0980171");
    assert_eq!(ldraw::num(0.1 + 0.2), "0.3");
    assert_eq!(
        ldraw::write_line(&ldraw::parse_line("1 0x2FF0000 1 2 3 1 0 0 0 1 0 0 0 1 a.dat").unwrap()),
        "1 0x2FF0000 1 2 3 1 0 0 0 1 0 0 0 1 a.dat"
    );
    assert!(ldraw::parse_line("1 16 0 0").is_err());
    assert!(ldraw::parse_line("7 1 2 3").is_err());
}

#[test]
fn a_stray_nofile_doesnt_add_an_empty_first_file() {
    // It used to push an empty file, so placements() of the real model found nothing.
    let doc = ldraw::parse("0 NOFILE\n0 FILE main.ldr\n1 4 0 0 0 1 0 0 0 1 0 0 0 1 3001.dat\n0 NOFILE\n").unwrap();
    assert_eq!(doc.files.len(), 1);
    assert_eq!(placements(&doc).len(), 1);
}
