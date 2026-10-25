// SPDX-License-Identifier: AGPL-3.0-or-later
//! How `emmet run` treats the model process (plan sections 08 and 15): the environment it gets, its output, processes
//! it leaves behind, timeouts, and the passes that fetch part geometry. Each test pins a fix from the PR review.

use std::path::{Path, PathBuf};
use std::time::Instant;

use emmet_it::{emmet, emmet_in, envelope, shared_data, temp_dir};
use serde_json::Value;

const HEAD: &str = "# SPDX-License-Identifier: AGPL-3.0-or-later\n";

/// A trusted project from the empty template, with `model` as its model.py and `toml_extra` appended to emmet.toml.
fn project(name: &str, model: &str, toml_extra: &str) -> PathBuf {
    let dir = temp_dir(name).join(name);
    let out = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["new", "--template", "empty"])
        .arg(&dir)
        .output();
    assert!(out.unwrap().status.success());
    std::fs::write(dir.join("model.py"), format!("{HEAD}{model}")).unwrap();
    if !toml_extra.is_empty() {
        let mut t = std::fs::read_to_string(dir.join("emmet.toml")).unwrap();
        t.push_str(toml_extra);
        std::fs::write(dir.join("emmet.toml"), t).unwrap();
    }
    dir
}

fn log(dir: &Path) -> String {
    String::from_utf8_lossy(&std::fs::read(dir.join("build/run.log")).unwrap_or_default()).into_owned()
}

#[test]
fn a_process_the_model_leaves_running_doesnt_hold_up_the_run() {
    // It inherits Python's output pipes; emmet used to wait for them to close, so the run hung.
    let dir = project(
        "leak",
        "import subprocess, sys\nfrom emmet_dsl import Model, model\n\n\n@model\ndef leak():\n    \"\"\"Leaves a child running.\"\"\"\n    subprocess.Popen([sys.executable, \"-c\", \"import time; time.sleep(60)\"])\n    return Model()\n",
        "\n[run]\ntimeout = 20\n",
    );
    let started = Instant::now();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    assert!(started.elapsed().as_secs() < 10, "took {:?}", started.elapsed());
}

#[test]
fn the_model_gets_no_inherited_environment() {
    let dir = project(
        "env",
        "import os, sys\nfrom emmet_dsl import Model, model\n\n\n@model\ndef env():\n    \"\"\"Prints its environment.\"\"\"\n    print(\"ENV\", sorted(os.environ), file=sys.stderr)\n    return Model()\n",
        "",
    );
    let hook = temp_dir("env-hook");
    let marker = hook.join("ran.txt");
    std::fs::write(
        hook.join("sitecustomize.py"),
        format!("open({:?}, 'w').write('ran')\n", marker.display().to_string()),
    )
    .unwrap();
    let out = emmet()
        .env("ANTHROPIC_API_KEY", "canary-anthropic")
        .env("OPENAI_API_KEY", "canary-openai")
        .env("GITHUB_TOKEN", "canary-github")
        .env("PYTHONPATH", &hook)
        .env("PYTHONSTARTUP", hook.join("sitecustomize.py"))
        .arg("--json")
        .arg("--data-dir")
        .arg(shared_data())
        .args(["run", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    let (v, code) = envelope(&out);
    assert_eq!(code, 0, "{v}");
    let log = log(&dir);
    let line = log
        .lines()
        .find(|l| l.starts_with("ENV"))
        .expect("the model printed its environment");
    for leak in ["API_KEY", "TOKEN", "PYTHONPATH", "PYTHONSTARTUP", "HOME", "PATH'"] {
        assert!(!line.contains(leak), "{leak} reached the model: {line}");
    }
    assert!(!marker.exists(), "a sitecustomize.py on PYTHONPATH ran");
}

#[test]
fn output_past_10_mb_is_cut_and_drained() {
    // The pipe must keep draining past the cap, or the model blocks and looks like a timeout.
    let dir = project(
        "chatty",
        "import sys\nfrom emmet_dsl import Model, model\n\n\n@model\ndef chatty():\n    \"\"\"Prints 12 MB.\"\"\"\n    for _ in range(12):\n        sys.stderr.write(\"x\" * (1 << 20))\n    return Model()\n",
        "\n[run]\ntimeout = 20\n",
    );
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let bytes = std::fs::read(dir.join("build/run.log")).unwrap();
    assert!(
        bytes.len() >= 10 << 20 && bytes.len() < (10 << 20) + 100,
        "run.log is {} bytes",
        bytes.len()
    );
    assert!(String::from_utf8_lossy(&bytes[bytes.len() - 40..]).contains("[run.log stopped at 10 MB]"));
}

#[test]
fn a_model_that_measures_parts_builds_from_a_clean_checkout() {
    // On the first pass emmet hasn't sent any sizes yet; the model divided by a zero width and that error won.
    let dir = project(
        "measure",
        "from emmet_dsl import Model, model\n\n\n@model\ndef measure():\n    \"\"\"Lays bricks by their own width, then picks a part by a size.\"\"\"\n    m = Model()\n    h = m.add(\"3001\", \"Red\", at=(0, 0, 0))\n    width = h.max[0] - h.min[0]\n    for i in range(1, int(8 // width)):\n        m.add(\"3001\", \"Red\", at=(i * width, 0, 0))\n    # A part chosen from another part's size: a third pass, after 3001's size is known.\n    m.add(\"3004\" if width == 2 else \"3005\", \"Blue\", at=(0, 0, 3))\n    return m\n",
        "",
    );
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["parts"], 5);
}

#[test]
fn float_steps_allow_for_float_error() {
    let dir = project(
        "steps",
        "from typing import Annotated\n\nfrom emmet_dsl import Model, Range, Step, model\n\n\n@model\ndef steps(gap: Annotated[float, Range(0, 1), Step(0.1)] = 0.5):\n    \"\"\"A float slider.\"\"\"\n    m = Model()\n    m.add(\"3001\", \"Red\", at=(gap, 0, 0))\n    return m\n",
        "",
    );
    for ok in ["0.3", "0.7", "1"] {
        let (v, code) = emmet_in(&dir, &["run", "--params", &format!("gap={ok}")]);
        assert_eq!(code, 0, "gap={ok}: {v}");
    }
    let (v, code) = emmet_in(&dir, &["run", "--params", "gap=0.25"]);
    assert_eq!((code, v["error"]["code"].as_str()), (3, Some("PARAM_INVALID")), "{v}");
}

#[test]
fn default_ids_stay_unique_across_steps_with_one_title() {
    let dir = project(
        "ids",
        "from emmet_dsl import Model, model\n\n\n@model\ndef ids():\n    \"\"\"Repeated step titles and loose parts.\"\"\"\n    m = Model()\n    m.add(\"3005\", \"Red\", at=(0, 0, 0))\n    for x in (2, 4):\n        with m.step(\"Row\"):\n            m.add(\"3005\", \"Red\", at=(x, 0, 0))\n    m.add(\"3005\", \"Red\", at=(6, 0, 0))\n    return m\n",
        "",
    );
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let placements: Value = serde_json::from_slice(&std::fs::read(dir.join("build/placements.json")).unwrap()).unwrap();
    let ids: Vec<&str> = placements["parts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["part/1", "row/1", "row/2", "part/2"]);
}

#[test]
fn errors_point_into_the_project_never_at_python_internals() {
    // A model saved in another encoding: its own syntax error, not a crash while reporting it.
    let dir = project("latin1", "", "");
    let mut bytes = HEAD.as_bytes().to_vec();
    bytes.extend_from_slice(b"# caf\xe9\nfrom emmet_dsl import Model, model\n");
    std::fs::write(dir.join("model.py"), bytes).unwrap();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(
        (
            code,
            v["error"]["code"].as_str(),
            v["error"]["details"]["file"].as_str()
        ),
        (3, Some("MODEL_EXEC_FAILED"), Some("model.py")),
        "{v}"
    );

    // No model.py at all.
    std::fs::remove_file(dir.join("model.py")).unwrap();
    let (v, _) = emmet_in(&dir, &["run"]);
    assert_eq!(v["error"]["message"], "The project has no model.py.", "{v}");

    // An annotation naming something undefined: the @model line, not <string> line 1.
    let src = "from __future__ import annotations\n\nfrom emmet_dsl import Model, model\n\n\n@model\ndef ann(width: Annotated[int, 3] = 4):\n    \"\"\"Forgot an import.\"\"\"\n    return Model()\n";
    std::fs::write(dir.join("model.py"), format!("{HEAD}{src}")).unwrap();
    let (v, _) = emmet_in(&dir, &["run"]);
    assert_eq!(
        (
            v["error"]["details"]["file"].as_str(),
            v["error"]["details"]["line"].as_u64()
        ),
        (Some("model.py"), Some(7)),
        "{v}"
    );

    // exit() keeps the line it was called on.
    let src = "import sys\n\nfrom emmet_dsl import Model, model\n\n\n@model\ndef bye():\n    \"\"\"Exits.\"\"\"\n    sys.exit(2)\n";
    std::fs::write(dir.join("model.py"), format!("{HEAD}{src}")).unwrap();
    let (v, _) = emmet_in(&dir, &["run"]);
    assert_eq!(
        (v["error"]["message"].as_str(), v["error"]["details"]["line"].as_u64()),
        (Some("The model called exit()."), Some(10)),
        "{v}"
    );
}

#[test]
fn an_error_naming_a_file_outside_the_project_reads_nothing_from_it() {
    let outside = temp_dir("outside").join("secret.txt");
    std::fs::write(&outside, "the secret line\n").unwrap();
    let dir = project("excerpt", "", "");
    let rel = format!(
        "../{}",
        outside
            .strip_prefix(dir.parent().unwrap().parent().unwrap())
            .unwrap()
            .display()
    )
    .replace('\\', "/");
    for path in [rel, outside.display().to_string().replace('\\', "/")] {
        let src = format!(
            "from emmet_dsl import EmmetError, model\n\n\n@model\ndef sneaky():\n    \"\"\"Raises an error naming another file.\"\"\"\n    raise EmmetError(\"UNKNOWN_PART\", \"m\", \"h\", {path:?}, 1)\n"
        );
        std::fs::write(dir.join("model.py"), format!("{HEAD}{src}")).unwrap();
        let (v, _) = emmet_in(&dir, &["run"]);
        assert_eq!(v["error"]["code"], "UNKNOWN_PART", "{v}");
        assert!(!v.to_string().contains("the secret line"), "{path}: {v}");
    }
}

#[test]
fn an_unusable_timeout_is_an_emmet_toml_error() {
    // inf used to panic when emmet turned it into a deadline, after Python had started.
    for bad in ["inf", "1e20", "0", "-3", "nan"] {
        let dir = project(
            "bad-timeout",
            "from emmet_dsl import Model, model\n\n\n@model\ndef m():\n    \"\"\"Empty.\"\"\"\n    return Model()\n",
            &format!("\n[run]\ntimeout = {bad}\n"),
        );
        let (v, code) = emmet_in(&dir, &["run"]);
        assert_eq!(
            (code, v["error"]["code"].as_str()),
            (3, Some("IO_ERROR")),
            "timeout = {bad}: {v}"
        );
        let lines = std::fs::read_to_string(dir.join("emmet.toml")).unwrap();
        let want = lines.lines().position(|l| l.starts_with("timeout")).unwrap() as u64 + 1;
        assert_eq!(v["error"]["details"]["line"].as_u64(), Some(want), "{v}");
    }
}

#[test]
fn the_kill_backstop_ends_a_model_that_stops_its_own_watchdog() {
    let dir = project(
        "backstop",
        "import faulthandler, time\nfrom emmet_dsl import Model, model\n\n\n@model\ndef stubborn():\n    \"\"\"Cancels the timeout dump and sleeps.\"\"\"\n    faulthandler.cancel_dump_traceback_later()\n    time.sleep(60)\n    return Model()\n",
        "\n[run]\ntimeout = 1\n",
    );
    let started = Instant::now();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!((code, v["error"]["code"].as_str()), (3, Some("MODEL_TIMEOUT")), "{v}");
    assert!(started.elapsed().as_secs() < 15, "took {:?}", started.elapsed());
}

#[cfg(unix)]
#[test]
fn a_link_loop_in_models_doesnt_hang_the_inputs_hash() {
    let dir = project(
        "links",
        "from emmet_dsl import Model, model\n\n\n@model\ndef m():\n    \"\"\"Empty.\"\"\"\n    return Model()\n",
        "",
    );
    std::fs::create_dir_all(dir.join("models")).unwrap();
    std::os::unix::fs::symlink("..", dir.join("models/a")).unwrap();
    std::os::unix::fs::symlink("..", dir.join("models/b")).unwrap();
    std::os::unix::fs::symlink("/", dir.join("models/root")).unwrap();
    let started = Instant::now();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    assert!(started.elapsed().as_secs() < 10);
}

#[test]
fn untrusted_projects_run_no_code_for_params_either() {
    let dir = temp_dir("untrusted-params").join("p");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("emmet.toml"),
        "schema = 1\nname = \"P\"\ncreated_by = \"m:0000000000000000\"\nlibrary = \"x\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("params.json"), "{}\n").unwrap();
    std::fs::write(
        dir.join("model.py"),
        format!("{HEAD}open(\"pwned.txt\", \"w\").write(\"ran\")\n"),
    )
    .unwrap();
    for args in [&["params"][..], &["params", "--set", "width=6"], &["run"]] {
        let (v, code) = emmet_in(&dir, args);
        assert_eq!(
            (code, v["error"]["code"].as_str()),
            (3, Some("PROJECT_UNTRUSTED")),
            "{args:?}: {v}"
        );
    }
    assert!(!dir.join("pwned.txt").exists(), "model.py ran in an untrusted project");
    assert_eq!(std::fs::read_to_string(dir.join("params.json")).unwrap(), "{}\n");

    // A project trusted with one install's secret isn't trusted by another.
    let trusted = emmet()
        .arg("--data-dir")
        .arg(shared_data())
        .args(["trust", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(trusted.status.success());
    std::fs::write(dir.join("model.py"), format!("{HEAD}from emmet_dsl import Model, model\n\n\n@model\ndef m():\n    \"\"\"Empty.\"\"\"\n    return Model()\n")).unwrap();
    let other = temp_dir("other-install");
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(&other)
        .args(["run", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(envelope(&out).0["error"]["code"], "PROJECT_UNTRUSTED");
}

#[test]
fn an_emmet_toml_error_names_its_own_line() {
    // The line used to be off by one when the error started a line.
    let dir = project("toml-line", "", "");
    let toml = std::fs::read_to_string(dir.join("emmet.toml")).unwrap();
    let lines: Vec<&str> = toml.lines().collect();
    let name_line = lines.iter().position(|l| l.starts_with("name")).unwrap();
    let mut broken = lines.clone();
    broken.insert(name_line + 1, "name = \"Again\"");
    std::fs::write(dir.join("emmet.toml"), broken.join("\n") + "\n").unwrap();
    let (v, code) = emmet_in(&dir, &["run"]);
    assert_eq!((code, v["error"]["code"].as_str()), (3, Some("IO_ERROR")), "{v}");
    assert_eq!(
        v["error"]["details"]["line"].as_u64(),
        Some(name_line as u64 + 2),
        "{v}"
    );
}

#[cfg(unix)]
#[test]
fn a_part_with_no_geometry_is_an_error_at_its_call() {
    // Placed at a zero box it would have landed in the wrong spot without a word. The library here has one part
    // that is only a header; the shared data dir's Python is linked in so no download is needed.
    let data = temp_dir("no-geometry").join("data");
    let zip = emmet_it::library_zip(&[("ldraw/parts/ghost.dat", b"0 Ghost Part\r\n0 Name: ghost.dat\r\n")]);
    let file = data.parent().unwrap().join("complete.zip");
    std::fs::write(&file, &zip).unwrap();
    let out = emmet()
        .arg("--data-dir")
        .arg(&data)
        .args([
            "library",
            "install",
            "--release",
            "x",
            "--sha256",
            &emmet_it::sha256(&zip),
            "--zip",
        ])
        .arg(&file)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", emmet_it::text(&out));
    std::os::unix::fs::symlink(shared_data().join("python"), data.join("python")).unwrap();
    let dir = project(
        "ghost",
        "from emmet_dsl import Model, model\n\n\n@model\ndef ghost():\n    \"\"\"Uses a part with no geometry.\"\"\"\n    m = Model()\n    m.add(\"ghost\", \"Red\", at=(0, 0, 0))\n    return m\n",
        "",
    );
    let trust = emmet()
        .arg("--data-dir")
        .arg(&data)
        .args(["trust", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(trust.status.success());
    let out = emmet()
        .arg("--json")
        .arg("--data-dir")
        .arg(&data)
        .args(["run", "--project"])
        .arg(&dir)
        .output()
        .unwrap();
    let (v, code) = envelope(&out);
    assert_eq!((code, v["error"]["code"].as_str()), (3, Some("UNKNOWN_PART")), "{v}");
    assert!(v["error"]["message"].as_str().unwrap().contains("no geometry"), "{v}");
    assert_eq!(v["error"]["details"]["line"], 9, "{v}");
}
