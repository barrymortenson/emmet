// SPDX-License-Identifier: AGPL-3.0-or-later
//! The parts data pack (plan section 12, ADR 0004): built from rebrickable-mini, installed only past its signature and
//! sha256 with the installed pack untouched otherwise, the bundled pack, rollback, ADR 0004's four mapping rules, and
//! honest availability: unknown is never unavailable.

use std::path::Path;

use emmet_core::data::DataDir;
use emmet_core::index::Index;
use emmet_core::parts_data::{MappingRule, PartsData, Status};
use emmet_it::{
    MINI_PACK_DATE, build_mini_pack, emmet, emmet_in, emmet_in_data, envelope, fixture, project, project_for,
    shared_data, shared_data_with_parts, temp_dir,
};
use serde_json::{Value, json};

/// `emmet --json --data-dir <data> parts-data <args>`, with the bundled pack at `bundle` when given.
fn parts_data(data: &Path, args: &[&str], bundle: Option<&Path>) -> (Value, i32) {
    let mut cmd = emmet();
    cmd.arg("--json")
        .arg("--data-dir")
        .arg(data)
        .arg("parts-data")
        .args(args);
    if let Some(b) = bundle {
        cmd.env(emmet_core::parts_data::BUNDLE_ENV, b);
    }
    envelope(&cmd.output().expect("run emmet parts-data"))
}

fn install(data: &Path, pack: &Path) -> (Value, i32) {
    parts_data(data, &["install", pack.to_str().unwrap()], None)
}

fn installed_version(data: &Path) -> Value {
    parts_data(data, &["status"], None).0["data"]["version"].clone()
}

/// A copy of the pack in `from`, to tamper with.
fn copy_pack(from: &Path, name: &str) -> std::path::PathBuf {
    let to = temp_dir(name);
    emmet_it::copy_tree(from, &to);
    to
}

#[test]
fn builds_from_rebrickable_mini_and_installs_with_a_good_signature() {
    let pack = temp_dir("pack-good");
    let manifest = build_mini_pack(&pack, MINI_PACK_DATE);
    // Every table, with img_url, MOC data, minifigs, and themes left out; the inventories become one row per part ×
    // color.
    assert_eq!(
        manifest["rows"],
        json!({
            "color_mappings": 438, "colors": 275, "elements": 9822, "part_categories": 76, "part_mappings": 394,
            "part_relationships": 25, "parts": 185, "set_parts": 212
        })
    );
    assert_eq!(
        manifest["credit"],
        "Part, color, and set data from Rebrickable (rebrickable.com)"
    );
    let sqlite = pack.join(manifest["file"].as_str().unwrap());
    assert_eq!(manifest["sha256"], emmet_it::sha256(&std::fs::read(&sqlite).unwrap()));
    // The build is reproducible: the same catalog and date make the same bytes.
    let again = build_mini_pack(&temp_dir("pack-again"), MINI_PACK_DATE);
    assert_eq!(again["sha256"], manifest["sha256"]);

    let data = temp_dir("pack-data");
    assert_eq!(
        parts_data(&data, &["status"], None).0["data"],
        Value::Null,
        "no pack yet"
    );
    let (v, code) = install(&data, &pack);
    assert_eq!(code, 0, "{v}");
    assert_eq!(installed_version(&data), MINI_PACK_DATE);
}

#[test]
fn a_bad_pack_is_refused_and_the_installed_one_stays() {
    let good = temp_dir("pack-installed");
    build_mini_pack(&good, MINI_PACK_DATE);
    let data = temp_dir("pack-refuse");
    assert_eq!(install(&data, &good).1, 0);
    let pointer = std::fs::read(data.join("parts-data/current.json")).unwrap();
    let newer = temp_dir("pack-newer");
    build_mini_pack(&newer, "2026-10-31");

    // A manifest changed after it was signed.
    let bad_sig = copy_pack(&newer, "pack-bad-sig");
    let m = bad_sig.join("parts-data.json");
    let text = std::fs::read_to_string(&m).unwrap().replace("2026-10-31", "2026-11-01");
    std::fs::write(&m, text).unwrap();
    // The SQLite file changed, so its sha256 isn't the signed manifest's.
    let tampered = copy_pack(&newer, "pack-tampered");
    let sqlite = tampered.join("parts-data-2026-10-31.sqlite");
    let mut bytes = std::fs::read(&sqlite).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    std::fs::write(&sqlite, bytes).unwrap();
    // A good pack signed by a key Emmet doesn't trust.
    let other = temp_dir("pack-other-key");
    let out = emmet()
        .args(["parts-data", "build"])
        .arg(fixture("rebrickable-mini"))
        .arg("--out")
        .arg(&other)
        .args(["--date", "2026-10-31", "--key"])
        .arg(fixture("parts-data/other-signing-key.der"))
        .output()
        .unwrap();
    assert!(out.status.success());

    for (pack, why) in [
        (&bad_sig, "isn't signed by a key Emmet trusts"),
        (&tampered, "but its manifest says"),
        (&other, "isn't signed by a key Emmet trusts"),
    ] {
        let (v, code) = install(&data, pack);
        assert_eq!((code, v["error"]["code"].clone()), (4, json!("LIBRARY_CORRUPT")), "{v}");
        assert!(v["error"]["message"].as_str().unwrap().contains(why), "{v}");
        assert_eq!(std::fs::read(data.join("parts-data/current.json")).unwrap(), pointer);
        assert_eq!(installed_version(&data), MINI_PACK_DATE);
        let leftovers: Vec<String> = std::fs::read_dir(data.join("parts-data"))
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains("2026-10-31") || n.ends_with(".tmp"))
            .collect();
        assert_eq!(leftovers, Vec::<String>::new(), "nothing of the refused pack stays");
    }
}

#[test]
fn rollback_goes_back_and_forth() {
    let data = temp_dir("pack-rollback");
    let (a, b) = (temp_dir("pack-a"), temp_dir("pack-b"));
    build_mini_pack(&a, MINI_PACK_DATE);
    build_mini_pack(&b, "2026-10-31");
    let (v, code) = parts_data(&data, &["rollback"], None);
    assert_eq!((code, v["error"]["code"].clone()), (4, json!("LIBRARY_MISSING")), "{v}");
    assert_eq!(install(&data, &a).1, 0);
    assert_eq!(install(&data, &b).1, 0);
    assert_eq!(installed_version(&data), "2026-10-31");
    assert_eq!(
        parts_data(&data, &["rollback"], None).0["data"]["version"],
        MINI_PACK_DATE
    );
    assert_eq!(installed_version(&data), MINI_PACK_DATE);
    assert_eq!(
        parts_data(&data, &["rollback"], None).0["data"]["version"],
        "2026-10-31"
    );
}

/// The pack an installer bundles installs on first use, checked by sha256 only, and replaces only an older pack.
#[test]
fn the_bundled_pack_installs_on_first_use_when_newer() {
    let bundled = temp_dir("pack-bundled");
    build_mini_pack(&bundled, MINI_PACK_DATE);
    // It needs no signature.
    std::fs::remove_file(bundled.join("parts-data.json.sig")).unwrap();
    let data = temp_dir("pack-bundled-data");
    let (v, _) = parts_data(&data, &["status"], Some(&bundled));
    assert_eq!(v["data"]["version"], MINI_PACK_DATE, "{v}");

    // A newer installed pack stays.
    let newer = temp_dir("pack-bundled-newer");
    build_mini_pack(&newer, "2026-10-31");
    assert_eq!(install(&data, &newer).1, 0);
    let (v, _) = parts_data(&data, &["status"], Some(&bundled));
    assert_eq!(v["data"]["version"], "2026-10-31", "{v}");

    // A damaged bundled pack is refused by its sha256 and the data dir keeps what it had.
    let fresh = temp_dir("pack-bundled-bad");
    let bad = copy_pack(&bundled, "pack-bundled-tampered");
    let sqlite = bad.join(format!("parts-data-{MINI_PACK_DATE}.sqlite"));
    let mut bytes = std::fs::read(&sqlite).unwrap();
    bytes[100] ^= 0xff;
    std::fs::write(&sqlite, bytes).unwrap();
    let (v, code) = parts_data(&fresh, &["status"], Some(&bad));
    assert_eq!((code, v["error"]["code"].clone()), (4, json!("LIBRARY_CORRUPT")), "{v}");
    assert_eq!(parts_data(&fresh, &["status"], None).0["data"], Value::Null);
}

fn open_pack() -> (Index, PartsData) {
    let data = DataDir::at(shared_data_with_parts());
    let index = Index::open(&data).expect("index");
    let pack = PartsData::open(&data).expect("open").expect("the pack is installed");
    (index, pack)
}

fn rule(pack: &PartsData, index: &Index, id: &str) -> Option<(String, MappingRule)> {
    pack.mapping(index, id).unwrap().map(|m| (m.rebrickable_id, m.rule))
}

#[test]
fn rule_1_an_ld_row() {
    let (index, pack) = open_pack();
    assert_eq!(rule(&pack, &index, "3001"), Some(("3001".into(), MappingRule::LdRow)));
    // Rebrickable's 3046a is LDraw's 3046.
    assert_eq!(rule(&pack, &index, "3046"), Some(("3046a".into(), MappingRule::LdRow)));
}

/// Rebrickable's LD row for the 9V switch still names 2859c01, which LDraw moved to 75541-f1.
#[test]
fn rule_2_an_ld_row_for_a_moved_id() {
    let (index, pack) = open_pack();
    assert_eq!(
        rule(&pack, &index, "75541-f1"),
        Some(("75541".into(), MappingRule::MovedLdRow))
    );
    assert_eq!(
        rule(&pack, &index, "2859c01"),
        Some(("75541".into(), MappingRule::LdRow))
    );
}

/// 3023 (moved to 3023b) has no LD row of its own, but Rebrickable has a part 3023.
#[test]
fn rule_3_an_identical_id() {
    let (index, pack) = open_pack();
    assert_eq!(rule(&pack, &index, "3023"), Some(("3023".into(), MappingRule::SameId)));
}

/// 75542-f1 has no LD row and ldraw-mini has no old id moved to it: its base, 75542, is Rebrickable's id.
#[test]
fn rule_4_the_base_of_a_formed_variant() {
    let (index, pack) = open_pack();
    assert_eq!(
        rule(&pack, &index, "75542-f1"),
        Some(("75542".into(), MappingRule::FormedVariant))
    );
}

#[test]
fn availability_is_honest() {
    let (index, pack) = open_pack();
    let status = |part: &str, color: u32| pack.availability(&index, part, color).unwrap();
    let red = status("3001", 4);
    assert_eq!(red.status, Status::Available);
    assert!(red.sets.unwrap() > 0 && red.years.is_some(), "{red:?}");
    assert_eq!(status("3001", 110).status, Status::ColorUnavailable, "Violet");
    // 3853 (a window) isn't in rebrickable-mini: unmapped, so unknown, never unavailable.
    assert_eq!(pack.mapping(&index, "3853").unwrap(), None);
    assert_eq!(status("3853", 4).status, Status::Unknown);
    // Main color 16, and a direct color, have no Rebrickable color.
    assert_eq!(status("3001", 16).status, Status::Unknown);
    assert_eq!(status("3001", 0x2FF_0000).status, Status::Unknown);
    // 3645 (Baseplate 24 x 40) maps, but rebrickable-mini has no element or set with it.
    assert_eq!(status("3645", 2).status, Status::PartUnavailable);
}

/// part_info, list_colors --part, and search's colors_count read the pack; without it they're null (or every color),
/// never an empty list.
#[test]
fn part_info_colors_and_search_use_the_pack() {
    let with = shared_data_with_parts();
    let run = |data: &Path, args: &[&str]| {
        envelope(
            &emmet()
                .arg("--json")
                .arg("--data-dir")
                .arg(data)
                .args(args)
                .output()
                .unwrap(),
        )
        .0
    };
    let info = run(with, &["part", "3001"]);
    let made = info["data"]["produced_colors"].as_array().unwrap();
    assert!(made.contains(&json!(4)) && !made.contains(&json!(110)), "{info}");
    assert_eq!(run(with, &["part", "3065"])["data"]["alternates"], json!(["3004"]));
    let colors = run(with, &["colors", "--part", "3001"]);
    assert_eq!(
        colors["data"]["colors"].as_array().unwrap().len(),
        made.len(),
        "{colors}"
    );
    let hits = run(with, &["search", "brick 2 x 4"]);
    assert_eq!(hits["data"]["parts"][0]["id"], "3001");
    assert_eq!(hits["data"]["parts"][0]["colors_count"], made.len());

    let without = shared_data();
    let info = run(without, &["part", "3001"]);
    assert_eq!(
        (&info["data"]["produced_colors"], &info["data"]["alternates"]),
        (&Value::Null, &Value::Null)
    );
    assert_eq!(
        run(without, &["search", "brick 2 x 4"])["data"]["parts"][0]["colors_count"],
        Value::Null
    );
    let all = run(without, &["colors"])["data"]["colors"].as_array().unwrap().len();
    assert_eq!(
        run(without, &["colors", "--part", "3001"])["data"]["colors"]
            .as_array()
            .unwrap()
            .len(),
        all
    );
}

/// The run manifest names the pack, the inputs hash changes with it, and build/colors.json says which colors were
/// ever made.
#[test]
fn the_run_records_the_pack() {
    let dir = project_for("broken-color-unavailable", shared_data_with_parts());
    let (v, code) = emmet_in_data(shared_data_with_parts(), &dir, &["run"]);
    assert_eq!(code, 0, "{v}");
    let manifest: Value = serde_json::from_slice(&std::fs::read(dir.join("build/manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["parts_data"], MINI_PACK_DATE, "{manifest}");
    let with_hash = v["meta"]["inputs_hash"].clone();
    let colors: Value = serde_json::from_slice(&std::fs::read(dir.join("build/colors.json")).unwrap()).unwrap();
    let produced = |code: u64| {
        colors["colors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["code"] == code)
            .unwrap()["produced"]
            .clone()
    };
    assert_eq!((produced(4), produced(16)), (json!(true), json!(false)));

    let plain = project("broken-color-unavailable");
    let (v, _) = emmet_in(&plain, &["run"]);
    assert_ne!(v["meta"]["inputs_hash"], with_hash, "the pack is an input");
    let manifest: Value = serde_json::from_slice(&std::fs::read(plain.join("build/manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["parts_data"], Value::Null);
    let colors = std::fs::read_to_string(plain.join("build/colors.json")).unwrap();
    assert!(!colors.contains("produced"), "no pack, no produced flags");
}
