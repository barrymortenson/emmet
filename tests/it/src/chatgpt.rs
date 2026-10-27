// SPDX-License-Identifier: AGPL-3.0-or-later
//! The Sign in with ChatGPT rig (1E): `fake-openai` standing in for OpenAI's sign-in endpoints, a data dir of the
//! test's own (the host id and the accounts), a file keychain, and a fake browser that follows the authorize redirect
//! to Emmet's loopback callback, as a person clicking Continue would.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::AtomicBool;

use emmet_agent::chatgpt::{ChatGpt, SignInResult};
use serde_json::Value;

use crate::agents::fake_bin;
use crate::temp_dir;

/// One `fake-openai`, killed when dropped.
pub struct OpenAi {
    child: Child,
    pub base: String,
    pub dir: PathBuf,
}

impl OpenAi {
    pub fn start(dir: &Path) -> Self {
        let mut child = Command::new(fake_bin("openai"))
            .arg(dir)
            .stdout(Stdio::piped())
            .spawn()
            .expect("fake-openai starts");
        let mut line = String::new();
        BufReader::new(child.stdout.take().expect("stdout"))
            .read_line(&mut line)
            .expect("fake-openai's URL");
        Self {
            child,
            base: line.trim().to_string(),
            dir: dir.to_path_buf(),
        }
    }

    /// How it answers from now on (see fake-openai's docs).
    pub fn set(&self, cfg: Value) {
        std::fs::write(self.dir.join("openai.json"), cfg.to_string()).expect("openai.json");
    }

    /// The parameters of each request of `kind` ("authorize", "token", "revoke") it got, in order.
    pub fn requests(&self, kind: &str) -> Vec<Value> {
        std::fs::read_to_string(self.dir.join("openai.jsonl"))
            .unwrap_or_default()
            .lines()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter(|l| l["kind"] == kind)
            .map(|l| l["params"].clone())
            .collect()
    }
}

impl Drop for OpenAi {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub struct Plan {
    pub openai: OpenAi,
    /// The app data dir: `chatgpt/host.json` and `chatgpt/accounts.json`.
    pub data: PathBuf,
    /// The fake keychain's API-key file; each other entry is `<file>.<entry>`.
    pub keychain: PathBuf,
    /// Where the fake browser notes the URLs it opened (`opened.txt`).
    pub browser: PathBuf,
    pub env: Vec<(String, String)>,
}

impl Plan {
    pub fn new(name: &str) -> Self {
        let dir = temp_dir(&format!("chatgpt-{name}"));
        let (data, fake) = (dir.join("data"), dir.join("fake"));
        std::fs::create_dir_all(&data).expect("data dir");
        std::fs::create_dir_all(&fake).expect("fake dir");
        let openai = OpenAi::start(&dir.join("openai"));
        let keychain = fake.join("keychain");
        let env = vec![
            ("EMMET_FAKE_OPENAI".to_string(), openai.base.clone()),
            ("EMMET_FAKE_BROWSER".into(), fake.display().to_string()),
            ("EMMET_FAKE_KEYCHAIN".into(), keychain.display().to_string()),
        ];
        Self {
            openai,
            data,
            keychain,
            browser: fake,
            env,
        }
    }

    pub fn chatgpt(&self) -> ChatGpt {
        ChatGpt::new(&self.data, &self.env)
    }

    /// Signs in through the fake browser.
    pub fn sign_in(&self, account: Option<&str>, consent: bool) -> emmet_agent::Result<SignInResult> {
        self.chatgpt().sign_in(account, consent, &AtomicBool::new(false))
    }

    /// The account `id`, signed in: panics on anything else.
    pub fn signed_in(&self, account: Option<&str>) -> emmet_agent::chatgpt::AccountView {
        match self.sign_in(account, false) {
            Ok(SignInResult::SignedIn { account, .. }) => account,
            other => panic!("expected a sign-in, got {other:?}"),
        }
    }

    /// The keychain record for registration `id`, as JSON.
    pub fn record(&self, id: &str) -> Option<Value> {
        let text = emmet_agent::keychain::get_entry(&self.env, &format!("chatgpt:{id}")).ok()??;
        serde_json::from_str(&text).ok()
    }

    /// Changes registration `id`'s keychain record, as time passing would.
    pub fn edit_record(&self, id: &str, f: impl FnOnce(&mut Value)) {
        let mut r = self.record(id).expect("a keychain record");
        f(&mut r);
        emmet_agent::keychain::set_entry(&self.env, &format!("chatgpt:{id}"), &r.to_string()).expect("record");
    }

    pub fn accounts(&self) -> Value {
        serde_json::from_str(&std::fs::read_to_string(self.data.join("chatgpt/accounts.json")).expect("accounts.json"))
            .expect("accounts JSON")
    }

    /// The URLs the fake browser opened.
    pub fn opened(&self) -> Vec<String> {
        std::fs::read_to_string(self.browser.join("opened.txt"))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }

    /// The HTTP status of each callback the fake browser delivered.
    pub fn callbacks(&self) -> Vec<String> {
        std::fs::read_to_string(self.browser.join("callbacks.txt"))
            .unwrap_or_default()
            .lines()
            .map(String::from)
            .collect()
    }
}
