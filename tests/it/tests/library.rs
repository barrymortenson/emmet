// SPDX-License-Identifier: AGPL-3.0-or-later
//! `emmet library install|status|verify` against a local HTTP server serving a zip built from ldraw-mini: the
//! checksum, the zip-slip guard, and resuming an interrupted download (plan sections 12 and 15).

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use emmet_it::{emmet, envelope, library_zip, sha256, temp_dir};
use serde_json::Value;

/// How the server answers each request, in order.
#[derive(Clone, Copy)]
enum Reply {
    /// The whole body, or the rest of it when the request has a Range header.
    Full,
    /// Promise the whole body, send the first `n` bytes, and drop the connection.
    CutAfter(usize),
}

/// Serves `body` at `http://127.0.0.1:<port>/complete.zip` for each reply in `replies`, recording the Range headers.
fn serve(body: Vec<u8>, replies: Vec<Reply>) -> (String, Arc<Mutex<Vec<Option<String>>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!(
        "http://127.0.0.1:{}/complete.zip",
        listener.local_addr().unwrap().port()
    );
    let ranges = Arc::new(Mutex::new(Vec::new()));
    let seen = ranges.clone();
    std::thread::spawn(move || {
        for reply in replies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut range = None;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line.trim().is_empty() {
                    break;
                }
                if let Some(v) = line.to_lowercase().strip_prefix("range: bytes=") {
                    range = Some(v.trim().trim_end_matches('-').to_string());
                }
            }
            seen.lock().unwrap().push(range.clone());
            let start: usize = range.as_deref().map_or(0, |r| r.parse().unwrap());
            let status = if start > 0 { "206 Partial Content" } else { "200 OK" };
            let rest = &body[start..];
            let head = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                rest.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            match reply {
                Reply::Full => stream.write_all(rest).unwrap(),
                Reply::CutAfter(n) => stream.write_all(&rest[..n]).unwrap(),
            }
            stream.flush().unwrap();
        }
    });
    (url, ranges)
}

fn install(data: &Path, url: &str, sha: &str) -> (Value, i32) {
    let out = emmet()
        .args(["--json", "--data-dir"])
        .arg(data)
        .args([
            "library",
            "install",
            "--release",
            "ldraw-mini",
            "--url",
            url,
            "--sha256",
            sha,
        ])
        .output()
        .unwrap();
    envelope(&out)
}

fn status(data: &Path) -> Value {
    let out = emmet()
        .args(["--json", "--data-dir"])
        .arg(data)
        .args(["library", "status"])
        .output()
        .unwrap();
    envelope(&out).0
}

#[test]
fn install_downloads_checks_and_indexes() {
    let zip = library_zip(&[]);
    let (url, _) = serve(zip.clone(), vec![Reply::Full]);
    let data = temp_dir("install");
    let (v, code) = install(&data, &url, &sha256(&zip));
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["data"]["installed"]["release"], "ldraw-mini");
    assert_eq!(v["data"]["installed"]["sha256"], sha256(&zip));
    assert!(v["data"]["indexed_parts"].as_u64().unwrap() > 150);
    assert!(data.join("ldraw/parts/3001.dat").is_file());
    assert!(
        !data.join("downloads/complete-ldraw-mini.zip").exists(),
        "the download is removed after install"
    );
    assert_eq!(status(&data)["data"]["installed"]["release"], "ldraw-mini");
}

#[test]
fn install_checks_the_sha256() {
    let zip = library_zip(&[]);
    let (url, _) = serve(zip.clone(), vec![Reply::Full]);
    let data = temp_dir("sha");
    let wrong = sha256(b"something else");
    let (v, code) = install(&data, &url, &wrong);
    assert_eq!(code, 4, "{v}");
    assert_eq!(v["error"]["code"], "LIBRARY_CORRUPT");
    assert!(
        v["error"]["message"].as_str().unwrap().contains(&sha256(&zip)),
        "names the actual checksum: {v}"
    );
    assert!(v["error"]["hint"].as_str().unwrap().contains("emmet library install"));
    assert!(!data.join("ldraw").exists(), "nothing installed");
    assert!(
        !data.join("downloads/complete-ldraw-mini.zip").exists(),
        "the bad download is removed"
    );
    assert_eq!(status(&data)["data"]["installed"], Value::Null);
}

#[test]
fn zip_slip_entry_is_refused() {
    for evil in [
        "ldraw/../../evil.dat",
        "/tmp/emmet-evil.dat",
        "ldraw\\..\\..\\evil.dat",
        "C:/evil.dat",
    ] {
        let zip = library_zip(&[(evil, b"0 evil\n")]);
        let data = temp_dir("slip").join("data");
        let file = data.parent().unwrap().join("complete.zip");
        std::fs::write(&file, &zip).unwrap();
        let out = emmet()
            .args(["--json", "--data-dir"])
            .arg(&data)
            .args([
                "library",
                "install",
                "--release",
                "x",
                "--sha256",
                &sha256(&zip),
                "--zip",
            ])
            .arg(&file)
            .output()
            .unwrap();
        let (v, code) = envelope(&out);
        assert_eq!(code, 4, "{evil}: {v}");
        assert_eq!(v["error"]["code"], "LIBRARY_CORRUPT", "{evil}");
        assert!(
            v["error"]["message"].as_str().unwrap().contains("escapes"),
            "{evil}: {v}"
        );
        assert!(!data.join("ldraw").exists(), "{evil}: nothing installed");
        assert!(!data.parent().unwrap().join("evil.dat").exists() && !Path::new("/tmp/emmet-evil.dat").exists());
    }
}

#[test]
fn interrupted_download_resumes() {
    let zip = library_zip(&[]);
    let half = zip.len() / 2;
    let (url, ranges) = serve(zip.clone(), vec![Reply::CutAfter(half), Reply::Full]);
    let data = temp_dir("resume");
    let (v, code) = install(&data, &url, &sha256(&zip));
    assert_eq!(code, 4, "{v}");
    assert_eq!(v["error"]["code"], "LIBRARY_MISSING");
    assert!(v["error"]["message"].as_str().unwrap().contains("stopped after"), "{v}");
    assert!(
        v["error"]["hint"]
            .as_str()
            .unwrap()
            .contains("picks up where it stopped")
    );
    let part = data.join("downloads/complete-ldraw-mini.zip.part");
    assert_eq!(std::fs::metadata(&part).unwrap().len() as usize, half);

    let (v, code) = install(&data, &url, &sha256(&zip));
    assert_eq!(code, 0, "{v}");
    assert_eq!(
        *ranges.lock().unwrap(),
        vec![None, Some(half.to_string())],
        "the second request resumes"
    );
    assert_eq!(v["data"]["installed"]["sha256"], sha256(&zip));
}

#[test]
fn plain_http_is_refused_except_to_this_machine() {
    let data = temp_dir("http");
    let (v, code) = install(&data, "http://example.com/complete.zip", &sha256(b""));
    assert_eq!(code, 4);
    assert!(v["error"]["message"].as_str().unwrap().contains("plain HTTP"), "{v}");
}

#[test]
fn verify_detects_a_changed_file_and_reindexes() {
    let zip = library_zip(&[]);
    let (url, _) = serve(zip.clone(), vec![Reply::Full]);
    let data = temp_dir("verify");
    assert_eq!(install(&data, &url, &sha256(&zip)).1, 0);
    let verify = |args: &[&str]| {
        let out = emmet()
            .args(["--json", "--data-dir"])
            .arg(&data)
            .args(["library", "verify"])
            .args(args)
            .output();
        envelope(&out.unwrap())
    };
    let (v, code) = verify(&[]);
    assert_eq!(code, 0, "{v}");
    assert!(v["data"]["files"].as_u64().unwrap() > 400);

    let started = Instant::now();
    let (v, code) = verify(&["--reindex"]);
    let secs = started.elapsed().as_secs_f64();
    assert_eq!(code, 0, "{v}");
    assert!(v["data"]["reindexed_parts"].as_u64().unwrap() > 150);
    // What good looks like: an index build from ldraw-mini in 2 s or less (the hash check above is included).
    assert!(secs <= 2.0, "verify --reindex took {secs:.2} s");
    println!("ldraw-mini verify --reindex: {secs:.3} s");

    std::fs::write(data.join("ldraw/parts/3001.dat"), "0 tampered\n").unwrap();
    let (v, code) = verify(&[]);
    assert_eq!(code, 4, "{v}");
    assert_eq!(v["error"]["code"], "LIBRARY_CORRUPT");
    assert!(
        v["error"]["message"].as_str().unwrap().contains("parts/3001.dat"),
        "{v}"
    );
}
