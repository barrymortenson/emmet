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
        Message::Render { .. } => Reply::undelivered("no window"),
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
