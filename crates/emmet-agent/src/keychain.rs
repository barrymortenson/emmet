// SPDX-License-Identifier: AGPL-3.0-or-later
//! Secrets in the OS keychain (the macOS Keychain, the Windows Credential Manager) under the service "Emmet", never in
//! a file or a log: the user's OpenAI API key for Codex (D-35, plan section 10), which Settings → Agents sets and
//! clears and the Codex adapter passes to the app-server through the provider's `env_key`; and one credential record
//! per Sign in with ChatGPT registration (`chatgpt:<client id>`, 1E).
//!
//! The Windows Credential Manager holds at most 2,560 bytes a credential, which the store writes as UTF-16 (1,280
//! units), and a Sign in with ChatGPT record (three tokens) is longer. On Windows (and in the fake keychain), a secret
//! that doesn't fit is split into `<entry>#1`, `<entry>#2`, …, with `emmet-parts:<n>` in the entry itself; one that
//! fits is stored as it is. The macOS Keychain has no such limit and stores every secret whole.
//!
//! Tests never touch the real keychain: in a debug build, `EMMET_FAKE_KEYCHAIN=<file>` in the agents' environment
//! keeps the API key in that file instead, and each other entry in `<file>.<entry>`, with any character but ASCII
//! letters, digits, `_`, and `-` made `-` (so `chatgpt:<id>` is `<file>.chatgpt-<id>`). The fake splits as Windows
//! does, and refuses any one file over Windows' limit on every OS, as the Credential Manager does.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Once;

use crate::{AgentError, AgentErrorCode, Result};

const SERVICE: &str = "Emmet";
/// The account name for the OpenAI API key.
pub const OPENAI: &str = "openai-api-key";

fn fake(env: &[(String, String)], account: &str) -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    let file = PathBuf::from(crate::env::get(env, "EMMET_FAKE_KEYCHAIN")?);
    if account == OPENAI {
        return Some(file);
    }
    let name = account.replace(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-', "-");
    let mut f = file.into_os_string();
    f.push(format!(".{name}"));
    Some(f.into())
}

fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        #[cfg(target_os = "macos")]
        if let Ok(s) = apple_native_keyring_store::keychain::Store::new() {
            keyring_core::set_default_store(s);
        }
        #[cfg(windows)]
        if let Ok(s) = windows_native_keyring_store::Store::new() {
            keyring_core::set_default_store(s);
        }
    });
}

fn entry(account: &str) -> Result<keyring_core::Entry> {
    init();
    keyring_core::Entry::new(SERVICE, account).map_err(failed)
}

fn failed(e: keyring_core::Error) -> AgentError {
    AgentError::new(AgentErrorCode::Io, format!("the keychain: {e}"))
}

/// Whether `account` is stored, without reading it. On macOS, reading the secret asks for the login password whenever
/// the app's signature differs from the one that saved it (every rebuild of a dev build), so detection only looks the
/// item up by its attributes; the secret itself is read when a session starts.
pub fn has_entry(env: &[(String, String)], account: &str) -> Result<bool> {
    if cfg!(target_os = "macos") && fake(env, account).is_none() {
        init();
        let spec = HashMap::from([("service", SERVICE), ("user", account)]);
        return Ok(!keyring_core::Entry::search(&spec).map_err(failed)?.is_empty());
    }
    Ok(get_entry(env, account)?.is_some())
}

/// The longest secret one credential holds, in UTF-16 units (Windows: 2,560 bytes).
const LIMIT: usize = 1280;
/// How long each part of a split secret is.
const PART: usize = 1200;
/// What the entry holds when its secret is split.
const PARTS: &str = "emmet-parts:";

fn part(account: &str, i: usize) -> String {
    format!("{account}#{i}")
}

/// One credential as the store has it.
fn raw_get(env: &[(String, String)], account: &str) -> Result<Option<String>> {
    if let Some(file) = fake(env, account) {
        return Ok(std::fs::read_to_string(file).ok().filter(|s| !s.is_empty()));
    }
    match entry(account)?.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => Err(failed(e)),
    }
}

fn raw_set(env: &[(String, String)], account: &str, secret: &str) -> Result<()> {
    if let Some(file) = fake(env, account) {
        if secret.encode_utf16().count() > LIMIT {
            return Err(AgentError::new(
                AgentErrorCode::Io,
                "the keychain: the secret is longer than Windows' limit of 2560 bytes",
            ));
        }
        return std::fs::write(file, secret).map_err(|e| AgentError::new(AgentErrorCode::Io, e.to_string()));
    }
    entry(account)?.set_password(secret).map_err(failed)
}

fn raw_clear(env: &[(String, String)], account: &str) -> Result<()> {
    if let Some(file) = fake(env, account) {
        let _ = std::fs::remove_file(file);
        return Ok(());
    }
    match entry(account)?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(e) => Err(failed(e)),
    }
}

/// Whether `account`'s secrets are split when long: on Windows, and in the fake on every OS so the tests see it. The
/// macOS Keychain has no such limit, and reading an item there first could ask for the login password.
fn splits(env: &[(String, String)], account: &str) -> bool {
    cfg!(windows) || fake(env, account).is_some()
}

/// How many parts the entry's secret is split into (0 when it isn't, or there's none).
fn parts(account: &str, head: Option<&str>) -> Result<usize> {
    match head.and_then(|h| h.strip_prefix(PARTS)) {
        None => Ok(0),
        Some(n) => n.parse().map_err(|_| {
            AgentError::new(
                AgentErrorCode::Io,
                format!("the keychain: {account} names no part count ({n})"),
            )
        }),
    }
}

/// `secret` in pieces of at most [`PART`] UTF-16 units each.
fn split(secret: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut units = 0;
    for c in secret.chars() {
        if units + c.len_utf16() > PART {
            out.push(String::new());
            units = 0;
        }
        units += c.len_utf16();
        out.last_mut().expect("a part").push(c);
    }
    out
}

/// The secret stored for `account`, or None.
pub fn get_entry(env: &[(String, String)], account: &str) -> Result<Option<String>> {
    let Some(head) = raw_get(env, account)? else {
        return Ok(None);
    };
    let n = parts(account, Some(&head))?;
    if n == 0 {
        return Ok(Some(head));
    }
    let mut secret = String::new();
    for i in 1..=n {
        secret += &raw_get(env, &part(account, i))?.ok_or_else(|| {
            AgentError::new(
                AgentErrorCode::Io,
                format!("the keychain: part {i} of {account} is missing"),
            )
        })?;
    }
    Ok(Some(secret))
}

pub fn set_entry(env: &[(String, String)], account: &str, secret: &str) -> Result<()> {
    if !splits(env, account) {
        return raw_set(env, account, secret);
    }
    let old = parts(account, raw_get(env, account)?.as_deref()).unwrap_or(0);
    let new = if secret.encode_utf16().count() <= LIMIT {
        raw_set(env, account, secret)?;
        0
    } else {
        // The entry goes first and comes back last, so a write that fails partway leaves no record, never one that
        // mixes new parts with old.
        raw_clear(env, account)?;
        let pieces = split(secret);
        for (i, p) in pieces.iter().enumerate() {
            raw_set(env, &part(account, i + 1), p)?;
        }
        raw_set(env, account, &format!("{PARTS}{}", pieces.len()))?;
        pieces.len()
    };
    for i in new + 1..=old {
        raw_clear(env, &part(account, i))?;
    }
    Ok(())
}

pub fn clear_entry(env: &[(String, String)], account: &str) -> Result<()> {
    if !splits(env, account) {
        return raw_clear(env, account);
    }
    let n = parts(account, raw_get(env, account)?.as_deref()).unwrap_or(0);
    raw_clear(env, account)?;
    for i in 1..=n {
        raw_clear(env, &part(account, i))?;
    }
    Ok(())
}

/// Whether the OpenAI API key is stored (see [`has_entry`]).
pub fn has(env: &[(String, String)]) -> Result<bool> {
    has_entry(env, OPENAI)
}

/// The stored OpenAI API key, or None.
pub fn get(env: &[(String, String)]) -> Result<Option<String>> {
    get_entry(env, OPENAI)
}

pub fn set(env: &[(String, String)], key: &str) -> Result<()> {
    set_entry(env, OPENAI, key)
}

pub fn clear(env: &[(String, String)]) -> Result<()> {
    clear_entry(env, OPENAI)
}

/// A live-eval API key (H-21, D-37): the generic password Barry stored for this user under `service`
/// (`emmet-eval-anthropic`, `emmet-eval-openai`) with `security add-generic-password -s <service> -a "$USER" -w`.
/// Only `emmet eval` reads these; the app never does.
pub fn eval_key(service: &str) -> Result<Option<String>> {
    init();
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_default();
    let entry = keyring_core::Entry::new(service, &user).map_err(failed)?;
    match entry.get_password() {
        Ok(k) => Ok(Some(k.trim().to_string()).filter(|k| !k.is_empty())),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => Err(failed(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_secret_is_split_and_no_part_outlives_it() {
        let dir = std::env::temp_dir().join(format!("emmet-keychain-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let env = vec![("EMMET_FAKE_KEYCHAIN".to_string(), dir.join("kc").display().to_string())];
        let files = || std::fs::read_dir(&dir).unwrap().count();
        // Non-ASCII counts in UTF-16 units, as Windows does.
        let long = format!("{}{}", "é".repeat(1000), "x".repeat(2000));
        set_entry(&env, "chatgpt:a", &long).unwrap();
        assert_eq!(get_entry(&env, "chatgpt:a").unwrap().as_deref(), Some(long.as_str()));
        assert_eq!(files(), 4, "the entry and three parts");
        set_entry(&env, "chatgpt:a", "short").unwrap();
        assert_eq!(get_entry(&env, "chatgpt:a").unwrap().as_deref(), Some("short"));
        assert_eq!(files(), 1, "the old parts went");
        set_entry(&env, "chatgpt:a", &long).unwrap();
        clear_entry(&env, "chatgpt:a").unwrap();
        assert_eq!((get_entry(&env, "chatgpt:a").unwrap(), files()), (None, 0));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
