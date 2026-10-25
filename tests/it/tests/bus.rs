// SPDX-License-Identifier: AGPL-3.0-or-later
//! emmet-bus v0 (docs/spec/bus.md): a message reaches the app's handler with the right token, and every other case
//! (no app, a stale token file, a wrong token) is `delivered: false` with a reason, never an error.

use std::time::Duration;

use emmet_bus::{Message, Reply, Server, info_path, send};
use emmet_core::data::DataDir;

fn data() -> DataDir {
    DataDir::at(emmet_it::temp_dir("bus"))
}

fn highlight() -> Message {
    Message::Highlight {
        project: "/p".into(),
        ids: vec!["roof".into()],
    }
}

#[test]
fn with_no_app_the_reply_is_undelivered() {
    let r = send(&data(), highlight(), Duration::from_secs(2));
    assert!(!r.delivered);
    assert_eq!(r.reason.as_deref(), Some("Emmet isn't running"));
}

#[test]
fn a_message_reaches_the_handler_and_its_reply_comes_back() {
    let data = data();
    let _server = Server::start(&data, |msg| match msg {
        Message::Highlight { ids, .. } => Reply {
            delivered: true,
            highlighted: ids,
            ..Default::default()
        },
        _ => Reply::undelivered("no window"),
    })
    .unwrap();
    let r = send(&data, highlight(), Duration::from_secs(5));
    assert!(r.delivered, "{r:?}");
    assert_eq!(r.highlighted, ["roof"]);
    let r = send(
        &data,
        Message::Render {
            project: "/p".into(),
            view: "iso".into(),
            size: 64,
            highlight: vec![],
        },
        Duration::from_secs(5),
    );
    assert_eq!(r.reason.as_deref(), Some("no window"));
}

#[test]
fn a_wrong_token_is_refused() {
    let data = data();
    let _server = Server::start(&data, |_| Reply {
        delivered: true,
        ..Default::default()
    })
    .unwrap();
    let path = info_path(&data);
    let text = std::fs::read_to_string(&path).unwrap();
    let mut info: serde_json::Value = serde_json::from_str(&text).unwrap();
    info["token"] = "0".repeat(64).into();
    std::fs::write(&path, info.to_string()).unwrap();
    let r = send(&data, highlight(), Duration::from_secs(5));
    assert!(!r.delivered);
    assert_eq!(r.reason.as_deref(), Some("wrong token"));
    // The server hangs up after a wrong token: a second request on the same stream gets no answer.
    use interprocess::local_socket::prelude::*;
    use std::io::{BufRead, BufReader, Write};
    let mut conn = interprocess::local_socket::Stream::connect(socket(info["name"].as_str().unwrap())).unwrap();
    let req = |id: u64| {
        format!(
            "{{\"v\":1,\"nonce\":\"n\",\"auth\":\"{}\",\"id\":{id},\"type\":\"highlight\",\"project\":\"/p\",\"ids\":[]}}\n",
            "0".repeat(64)
        )
    };
    conn.write_all(req(1).as_bytes()).unwrap();
    let mut reader = BufReader::new(conn);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    assert!(line.contains("wrong token"), "{line}");
    let _ = reader.get_mut().write_all(req(2).as_bytes());
    line.clear();
    assert_eq!(
        reader.read_line(&mut line).unwrap_or(0),
        0,
        "the connection should be closed: {line}"
    );
}

/// The socket's name, as emmet-bus makes it: a file in /tmp on macOS, a named pipe on Windows.
fn socket(name: &str) -> interprocess::local_socket::Name<'static> {
    use interprocess::local_socket::prelude::*;
    #[cfg(unix)]
    {
        emmet_bus::socket_path(name)
            .to_fs_name::<interprocess::local_socket::GenericFilePath>()
            .unwrap()
            .into_owned()
    }
    #[cfg(windows)]
    {
        name.to_string()
            .to_ns_name::<interprocess::local_socket::GenericNamespaced>()
            .unwrap()
    }
}

/// A process squatting on the socket's name (a crashed app left bus.json behind) doesn't know the token, so its
/// answers aren't believed: an "allow" from it is undelivered.
#[test]
fn an_answer_from_a_process_that_doesnt_know_the_token_is_refused() {
    use interprocess::local_socket::ListenerOptions;
    use interprocess::local_socket::prelude::*;
    use std::io::{BufRead, BufReader, Write};
    let data = data();
    let name = format!("emmet-squat-{}.sock", std::process::id());
    let listener = ListenerOptions::new()
        .name(socket(&name))
        .try_overwrite(true)
        .create_sync()
        .unwrap();
    std::fs::write(
        info_path(&data),
        format!(r#"{{"v":1,"name":"{name}","token":"{}","pid":1}}"#, "a".repeat(64)),
    )
    .unwrap();
    let squatter = std::thread::spawn(move || {
        let conn = listener.incoming().next().unwrap().unwrap();
        let mut reader = BufReader::new(conn);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        assert!(!line.contains(&"a".repeat(64)), "the token crossed the socket: {line}");
        let forged = r#"{"id":1,"delivered":true,"answer":{"behavior":"allow"},"auth":"forged"}"#;
        reader.get_mut().write_all(format!("{forged}\n").as_bytes()).unwrap();
    });
    let r = send(
        &data,
        Message::Approve {
            session: "s".into(),
            request: serde_json::json!({ "tool_name": "Bash" }),
        },
        Duration::from_secs(5),
    );
    squatter.join().unwrap();
    assert!(!r.delivered);
    assert_eq!(r.answer, None);
    assert_eq!(
        r.reason.as_deref(),
        Some("the answer on the bus didn't come from Emmet")
    );
}

/// Only this user may open the socket (plan section 15).
#[cfg(unix)]
#[test]
fn the_socket_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let data = data();
    let _server = Server::start(&data, |_| Reply::default()).unwrap();
    let info: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(info_path(&data)).unwrap()).unwrap();
    let path = emmet_bus::socket_path(info["name"].as_str().unwrap());
    assert_eq!(std::fs::metadata(path).unwrap().permissions().mode() & 0o777, 0o600);
}

#[test]
fn the_token_file_is_private_and_goes_with_the_app() {
    let data = data();
    let server = Server::start(&data, |_| Reply::default()).unwrap();
    let path = info_path(&data);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    }
    drop(server);
    assert!(!path.exists(), "bus.json outlived the app");
    let r = send(&data, highlight(), Duration::from_secs(2));
    assert!(!r.delivered);
}

#[test]
fn a_token_file_left_by_a_crashed_app_is_undelivered() {
    let data = data();
    std::fs::write(
        info_path(&data),
        r#"{"v":1,"name":"emmet-ffffffffffffffff.sock","token":"00","pid":1}"#,
    )
    .unwrap();
    let started = std::time::Instant::now();
    let r = send(&data, highlight(), Duration::from_secs(5));
    assert!(!r.delivered);
    assert_eq!(r.reason.as_deref(), Some("Emmet isn't running"));
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "a dead socket should fail fast"
    );
}
