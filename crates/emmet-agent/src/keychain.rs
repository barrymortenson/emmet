// SPDX-License-Identifier: AGPL-3.0-or-later
//! The user's OpenAI API key for Codex (D-35, plan section 10): kept in the OS keychain (the macOS Keychain, the
//! Windows Credential Manager) under the service "Emmet", never in a file or a log. Settings → Agents sets and clears
//! it; the Codex adapter passes it to the app-server through the provider's `env_key`.
//!
//! Tests never touch the real keychain: in a debug build, `EMMET_FAKE_KEYCHAIN=<file>` in the agents' environment
//! keeps the secret in that file instead.

use std::path::PathBuf;
use std::sync::Once;

use crate::{AgentError, AgentErrorCode, Result};

const SERVICE: &str = "Emmet";
/// The account name for the OpenAI API key.
pub const OPENAI: &str = "openai-api-key";

fn fake(env: &[(String, String)]) -> Option<PathBuf> {
    if !cfg!(debug_assertions) {
        return None;
    }
    crate::env::get(env, "EMMET_FAKE_KEYCHAIN").map(PathBuf::from)
}

fn store() -> Result<keyring_core::Entry> {
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
    keyring_core::Entry::new(SERVICE, OPENAI).map_err(failed)
}

fn failed(e: keyring_core::Error) -> AgentError {
    AgentError::new(AgentErrorCode::Io, format!("the keychain: {e}"))
}

/// The stored key, or None.
pub fn get(env: &[(String, String)]) -> Result<Option<String>> {
    if let Some(file) = fake(env) {
        return Ok(std::fs::read_to_string(file).ok().filter(|s| !s.is_empty()));
    }
    match store()?.get_password() {
        Ok(k) => Ok(Some(k)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => Err(failed(e)),
    }
}

pub fn set(env: &[(String, String)], key: &str) -> Result<()> {
    if let Some(file) = fake(env) {
        return std::fs::write(file, key).map_err(|e| AgentError::new(AgentErrorCode::Io, e.to_string()));
    }
    store()?.set_password(key).map_err(failed)
}

pub fn clear(env: &[(String, String)]) -> Result<()> {
    if let Some(file) = fake(env) {
        let _ = std::fs::remove_file(file);
        return Ok(());
    }
    match store()?.delete_credential() {
        Ok(()) | Err(keyring_core::Error::NoEntry) => Ok(()),
        Err(e) => Err(failed(e)),
    }
}
