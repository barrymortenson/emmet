// SPDX-License-Identifier: AGPL-3.0-or-later
//! The app's command line: `Emmet [project]` and render mode's options, refused when they don't make sense.

use emmet_desktop::parse_args;

fn args(list: &[&str]) -> Result<emmet_desktop::Args, String> {
    parse_args(list.iter().map(|s| s.to_string()))
}

#[test]
fn render_mode_needs_a_known_view_and_an_out_file() {
    let a = args(&[
        "--render",
        "p",
        "--view",
        "top",
        "--size",
        "64",
        "--out",
        "x.png",
        "--highlight",
        "a,b",
    ])
    .unwrap();
    let r = a.render.unwrap();
    assert_eq!((r.view.as_str(), r.size, r.highlight.len()), ("top", 64, 2));
    assert!(
        args(&["--render", "p", "--view", "diagonal", "--out", "x.png"])
            .unwrap_err()
            .contains("no view")
    );
    assert!(args(&["--render", "p"]).unwrap_err().contains("--out"));
    assert!(args(&["--frobnicate"]).is_err());
}

#[test]
fn a_project_to_open_and_finders_launch_argument() {
    let a = args(&["-psn_0_12345", "some/project"]).unwrap();
    assert_eq!(a.open.as_deref(), Some(std::path::Path::new("some/project")));
    assert!(a.render.is_none());
}
