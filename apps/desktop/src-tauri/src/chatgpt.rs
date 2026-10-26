// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sign in with ChatGPT in the app (1E, ADR 0011): the account picker's commands, the sign-in in the system browser,
//! sign-out, which way Codex pays, the one-time plan notice, and Manage usage. The window never sees a token: it learns
//! only the accounts' labels and states, and the host opens the only URLs it opens itself.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use emmet_agent::chatgpt::{AccountView, ChatGpt, ChatGptState, Choice, SignInResult, SignOut, USAGE_URL};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app::{AppState, Problem};

/// What Settings → Agents shows for Codex.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct CodexBilling {
    pub accounts: Vec<AccountView>,
    pub choice: Option<Choice>,
    /// An OpenAI API key is in the keychain.
    pub key_saved: bool,
}

fn chatgpt(app: &AppHandle) -> ChatGpt {
    let state = app.state::<AppState>();
    ChatGpt::new(state.data.root(), state.chat.env())
}

fn problem(e: emmet_agent::AgentError) -> Box<Problem> {
    Problem::internal(e.message)
}

/// Codex's sign-in changed: detect it again.
fn changed(app: &AppHandle) {
    app.state::<AppState>().chat.forget_agents();
}

/// OpenAI's sign-out starts by stopping requests: the live chat ends if it bills `account`.
fn stop_plan_chat(app: &AppHandle, account: &str) {
    let chat = &app.state::<AppState>().chat;
    if chat.live_plan().as_deref() == Some(account) {
        chat.close();
    }
}

#[tauri::command]
#[specta::specta]
pub fn chatgpt_state(app: AppHandle) -> Result<CodexBilling, Box<Problem>> {
    let ChatGptState { accounts, codex } = chatgpt(&app).state().map_err(problem)?;
    Ok(CodexBilling {
        accounts,
        choice: codex,
        key_saved: emmet_agent::keychain::has(app.state::<AppState>().chat.env()).map_err(problem)?,
    })
}

/// Continue with ChatGPT: a new account (`account` null), or the saved one again; `consent` asks OpenAI for the
/// permission screen again, to enable plan use after a decline. Waits for the browser; `chatgpt_cancel` stops it.
#[tauri::command]
#[specta::specta]
pub async fn chatgpt_sign_in(
    app: AppHandle,
    account: Option<String>,
    consent: bool,
) -> Result<SignInResult, Box<Problem>> {
    let cancel = Arc::new(AtomicBool::new(false));
    if let Some(old) = app
        .state::<AppState>()
        .chat
        .sign_in
        .lock()
        .expect("sign-in")
        .replace(cancel.clone())
    {
        old.store(true, Ordering::SeqCst);
    }
    let handle = app.clone();
    let result =
        tauri::async_runtime::spawn_blocking(move || chatgpt(&handle).sign_in(account.as_deref(), consent, &cancel))
            .await
            .map_err(|e| Problem::internal(e.to_string()))?
            .map_err(problem);
    changed(&app);
    result
}

/// Stops a sign-in that's waiting for the browser.
#[tauri::command]
#[specta::specta]
pub fn chatgpt_cancel(app: AppHandle) {
    if let Some(c) = app.state::<AppState>().chat.sign_in.lock().expect("sign-in").take() {
        c.store(true, Ordering::SeqCst);
    }
}

/// Signs the account out: its chat stops, OpenAI revokes the session, and its tokens are cleared.
#[tauri::command]
#[specta::specta]
pub async fn chatgpt_sign_out(app: AppHandle, account: String) -> Result<SignOut, Box<Problem>> {
    stop_plan_chat(&app, &account);
    let handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || chatgpt(&handle).sign_out(&account))
        .await
        .map_err(|e| Problem::internal(e.to_string()))?
        .map_err(problem);
    changed(&app);
    result
}

/// Codex runs on the ChatGPT account `account`, or on the API key (null). A running Codex chat keeps what it started
/// with; the next one uses the choice.
#[tauri::command]
#[specta::specta]
pub fn chatgpt_choose(app: AppHandle, account: Option<String>) -> Result<(), Box<Problem>> {
    chatgpt(&app).choose(account.as_deref()).map_err(problem)?;
    changed(&app);
    Ok(())
}

/// "You're using your ChatGPT plan" was shown for `account`; it isn't shown again.
#[tauri::command]
#[specta::specta]
pub fn chatgpt_notice_seen(app: AppHandle, account: String) -> Result<(), Box<Problem>> {
    chatgpt(&app).notice_seen(&account).map_err(problem)
}

/// Manage usage: ChatGPT Settings → Usage in the system browser.
#[tauri::command]
#[specta::specta]
pub fn open_chatgpt_usage(app: AppHandle) -> Result<(), Box<Problem>> {
    emmet_agent::chatgpt::open_url(app.state::<AppState>().chat.env(), USAGE_URL).map_err(problem)
}
