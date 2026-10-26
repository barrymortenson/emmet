// SPDX-License-Identifier: AGPL-3.0-or-later
//! Secrets in the OS keychain (the macOS Keychain, the Windows Credential Manager) under the service "Emmet", never in
//! a file or a log: the user's OpenAI API key for Codex (D-35, plan section 10), which Settings → Agents sets and
//! clears and the Codex adapter passes to the app-server through the provider's `env_key`; and one credential record
//! per Sign in with ChatGPT registration (`chatgpt:<client id>`, 1E).
//!
//! Tests never touch the real keychain: in a debug build, `EMMET_FAKE_KEYCHAIN=<file>` in the agents' environment
//! keeps the API key in that file instead, and each other entry in `<file>.<entry>`, with any character but ASCII
//! letters, digits, `_`, and `-` made `-` (so `chatgpt:<id>` is `<file>.chatgpt-<id>`).

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

/// The secret stored for `account`, or None.
pub fn get_entry(env: &[(String, String)], account: &str) -> Result<Option<String>> {
    if let Some(file) = fake(env, account) {
        return Ok(std::fs::read_to_string(file).ok().filter(|s| !s.is_empty()));
    }
    match entry(account)?.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => Err(failed(e)),
    }
}

pub fn set_entry(env: &[(String, String)], account: &str, secret: &str) -> Result<()> {
    if let Some(file) = fake(env, account) {
        return std::fs::write(file, secret).map_err(|e| AgentError::new(AgentErrorCode::Io, e.to_string()));
    }
    entry(account)?.set_password(secret).map_err(failed)
}

pub fn clear_entry(env: &[(String, String)], account: &str) -> Result<()> {
    if let Some(file) = fake(env, account) {
        let _ = std::fs::remove_file(file);
        return Ok(());
    }
    match entry(account)?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(e) => Err(failed(e)),
    }
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
