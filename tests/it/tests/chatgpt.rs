// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sign in with ChatGPT (1E, ADR 0011), against fake-openai: each test is one of OpenAI's documented requirements
//! (docs/spec/sessions/1E.md maps them), proved through the same `ChatGpt` the app and the Codex adapter use. The fake
//! checks every request's parameters and answers 400 when one is wrong, so a test fails on Emmet's mistake.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::time::Duration;

use emmet_agent::chatgpt::{CodexUse, SignInResult};
use emmet_agent::codex::RENEW_BEFORE;
use emmet_agent::{AgentErrorCode, AgentEvent, AgentKind, UserTurn};
use emmet_it::agents::Rig;
use emmet_it::chatgpt::Plan;
use emmet_it::shared_data;
use serde_json::{Value, json};

const ID: &str = "oaiapp_test1";

fn now() -> i64 {
    (emmet_agent::time::now_ms() / 1000) as i64
}

fn param<'a>(p: &'a Value, k: &str) -> &'a str {
    p[k].as_str().unwrap_or_else(|| panic!("no {k} in {p}"))
}

#[test]
fn host_id_is_made_once_before_the_first_sign_in() {
    let plan = Plan::new("host");
    plan.signed_in(None);
    let host = plan.chatgpt().host_id().unwrap();
    assert!(host.starts_with("urn:uuid:"), "{host}");
    plan.signed_in(Some(ID));
    let sent: Vec<Value> = plan
        .openai
        .requests("authorize")
        .iter()
        .map(|p| p["ext_agent_host_id"].clone())
        .collect();
    assert_eq!(sent, [json!(host), json!(host)], "every attempt sends this host's id");
    assert_eq!(plan.record(ID).unwrap()["ext_agent_host_id"], host.as_str());
}

#[test]
fn first_sign_in_registers_emmet() {
    let plan = Plan::new("first");
    let SignInResult::SignedIn { account, notice } = plan.sign_in(None, false).unwrap() else {
        panic!("not signed in")
    };
    assert_eq!(
        (
            account.id.as_str(),
            account.label.as_str(),
            account.active,
            account.signed_in,
            account.plan,
            notice
        ),
        (ID, "builder@example.com", true, true, true, true)
    );
    let auth = &plan.openai.requests("authorize")[0];
    assert_eq!(param(auth, "client_id"), "dynamic_agent_client");
    assert_eq!(param(auth, "agent_name_hint"), "Emmet");
    assert_eq!(
        param(auth, "scope"),
        "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct"
    );
    assert_eq!(param(auth, "resource"), "https://api.openai.com/v1");
    assert!(auth.get("id_token_hint").is_none() && auth.get("login_hint").is_none() && auth.get("prompt").is_none());
    let redirect = param(auth, "redirect_uri");
    assert!(
        redirect.starts_with("http://127.0.0.1:") && redirect.ends_with("/auth/callback"),
        "{redirect}"
    );
    let token = &plan.openai.requests("token")[0];
    assert_eq!(param(token, "grant_type"), "authorization_code");
    assert_eq!(param(token, "client_id"), ID, "the exchange uses the issued client id");
    assert_eq!(param(token, "redirect_uri"), redirect);
    // OpenAI's credential record, in the keychain.
    let r = plan.record(ID).expect("a keychain record");
    let keys: BTreeSet<&str> = r.as_object().unwrap().keys().map(String::as_str).collect();
    for k in [
        "email",
        "issuer",
        "subject",
        "client_id",
        "ext_agent_host_id",
        "id_token",
        "access_token",
        "refresh_token",
        "token_type",
        "expires_in",
        "scopes",
        "saved_at",
    ] {
        assert!(keys.contains(k), "the record has no {k}: {keys:?}");
    }
    assert_eq!(r["subject"], "user-1");
    assert_eq!(r["client_id"], ID);
    // The files hold no token, and the browser log has no authorize query (a returning one carries id_token_hint).
    let files = std::fs::read_to_string(plan.data.join("chatgpt/accounts.json")).unwrap()
        + &std::fs::read_to_string(plan.data.join("chatgpt/host.json")).unwrap();
    assert!(!files.contains("fake-"), "{files}");
    assert!(plan.opened().iter().all(|u| !u.contains('?')), "{:?}", plan.opened());
}

#[test]
fn each_attempt_is_fresh() {
    let plan = Plan::new("fresh");
    plan.signed_in(None);
    plan.signed_in(Some(ID));
    let a = plan.openai.requests("authorize");
    let t = plan.openai.requests("token");
    for k in ["state", "nonce", "code_challenge"] {
        assert_ne!(a[0][k], a[1][k], "{k} is new each attempt");
    }
    for i in 0..2 {
        assert_eq!(
            a[i]["redirect_uri"], t[i]["redirect_uri"],
            "attempt {i} sends one redirect URI"
        );
    }
}

#[test]
fn declined_consent_stops_without_an_exchange() {
    let plan = Plan::new("declined");
    plan.openai.set(json!({ "decline": true }));
    assert_eq!(plan.sign_in(None, false).unwrap(), SignInResult::Declined);
    assert!(plan.openai.requests("token").is_empty(), "no code is exchanged");
    assert!(plan.chatgpt().state().accounts.is_empty());
}

#[test]
fn callback_problems_are_refused() {
    // Another state: answered and ignored; the attempt waits on, and nothing is exchanged.
    let plan = Plan::new("callback");
    plan.openai.set(json!({ "bad_state": true }));
    let cancel = Arc::new(AtomicBool::new(false));
    let (c, chatgpt) = (cancel.clone(), plan.chatgpt());
    let t = std::thread::spawn(move || chatgpt.sign_in(None, false, &c));
    while plan.openai.requests("authorize").is_empty() {
        std::thread::sleep(Duration::from_millis(20));
    }
    std::thread::sleep(Duration::from_millis(500));
    cancel.store(true, Ordering::SeqCst);
    assert_eq!(t.join().unwrap().unwrap(), SignInResult::Canceled);
    assert!(plan.openai.requests("token").is_empty());

    // A registration callback with no issued client id is incomplete.
    plan.openai.set(json!({ "callback_client_id": "" }));
    let e = plan.sign_in(None, false).unwrap_err();
    assert!(e.message.contains("didn't finish registering"), "{e}");
    assert!(plan.openai.requests("token").is_empty());

    // A returning sign-in's callback naming another client is rejected; the account is untouched.
    plan.openai.set(json!({}));
    plan.signed_in(None);
    let before = plan.record(ID).unwrap();
    plan.openai.set(json!({ "callback_client_id": "oaiapp_other" }));
    let e = plan.sign_in(Some(ID), false).unwrap_err();
    assert!(e.message.contains("different registration"), "{e}");
    assert_eq!(plan.record(ID).unwrap(), before);
}

#[test]
fn returning_sign_in_reuses_the_registration() {
    let plan = Plan::new("returning");
    plan.signed_in(None);
    let first = plan.record(ID).unwrap();
    plan.signed_in(Some(ID));
    let again = &plan.openai.requests("authorize")[1];
    assert_eq!(param(again, "client_id"), ID);
    assert!(
        again.get("agent_name_hint").is_none(),
        "no name hint on reauthorization"
    );
    assert_eq!(again["id_token_hint"], first["id_token"]);
    assert_eq!(param(again, "login_hint"), "builder@example.com");
    let second = plan.record(ID).unwrap();
    assert_ne!(
        second["access_token"], first["access_token"],
        "the new token set replaced the old"
    );

    // The browser signed in as someone else: refused, and the saved account is untouched.
    plan.openai
        .set(json!({ "account": { "sub": "user-2", "email": "other@example.com" } }));
    let e = plan.sign_in(Some(ID), false).unwrap_err();
    assert!(e.message.contains("different ChatGPT account"), "{e}");
    assert_eq!(plan.record(ID).unwrap(), second);
}

#[test]
fn invalid_grant_on_exchange_keeps_the_client() {
    let plan = Plan::new("invalid-grant");
    plan.openai.set(json!({ "exchange_error": "invalid_grant" }));
    let e = plan.sign_in(None, false).unwrap_err();
    assert!(e.message.contains("try again"), "{e}");
    assert!(plan.chatgpt().state().accounts.is_empty(), "nothing to pick yet");
    plan.openai.set(json!({}));
    plan.signed_in(None);
    let a = plan.openai.requests("authorize");
    assert_eq!(param(&a[1], "client_id"), ID, "the retry reuses the issued client id");
    assert!(a[1].get("agent_name_hint").is_none());
}

#[test]
fn bad_id_tokens_are_refused() {
    let plan = Plan::new("id-token");
    plan.signed_in(None);
    for tamper in ["sig", "alg", "iss", "aud", "exp", "nonce"] {
        plan.openai
            .set(json!({ "tamper": tamper, "client_id": format!("oaiapp_{tamper}") }));
        let e = plan.sign_in(None, false).expect_err(tamper);
        assert!(e.message.contains("couldn't verify"), "{tamper}: {e}");
        assert!(
            plan.record(&format!("oaiapp_{tamper}")).is_none(),
            "{tamper}: nothing stored"
        );
        let state = plan.chatgpt().state();
        assert_eq!(state.accounts.len(), 1, "{tamper}");
        assert!(state.accounts[0].active, "{tamper}: the active account stays");
    }
}

#[test]
fn plan_permission_declined_keeps_the_sign_in() {
    let plan = Plan::new("no-plan");
    plan.openai.set(json!({ "grant_plan": false }));
    let SignInResult::SignedIn { account, notice } = plan.sign_in(None, false).unwrap() else {
        panic!("not signed in")
    };
    assert!(account.signed_in && !account.plan && !notice, "{account:?}");
    let CodexUse::PlanOff { fix, .. } = plan.chatgpt().codex_use() else {
        panic!("plan use should be off")
    };
    assert!(fix.contains("isn't enabled") && fix.contains("API key"), "{fix}");
    // Codex doesn't start on the plan, and doesn't fall back to a saved API key either.
    let rig = Rig::new("chatgpt-no-plan", "codex", "plain-turn").with_plan(&plan);
    rig.key("sk-test-not-a-real-key");
    let codex = rig.adapter(AgentKind::Codex);
    let e = codex.start(rig.cfg.clone()).err().expect("no session");
    assert_eq!(e.code, AgentErrorCode::NotSignedIn);
    assert!(!rig.fake.join("codex-0.json").exists(), "Codex never started");
    let d = codex.detect();
    assert!(!d.signed_in && d.fix.as_deref() == Some(fix.as_str()), "{d:?}");
}

#[test]
fn enable_plan_use_asks_for_consent_again() {
    let plan = Plan::new("enable");
    plan.openai.set(json!({ "grant_plan": false }));
    plan.signed_in(None);
    plan.openai.set(json!({}));
    let SignInResult::SignedIn { account, notice } = plan.sign_in(Some(ID), true).unwrap() else {
        panic!("not signed in")
    };
    assert!(account.plan && notice, "now enabled, with the one-time notice");
    let a = &plan.openai.requests("authorize")[1];
    assert_eq!(param(a, "prompt"), "consent");
    assert_eq!(param(a, "client_id"), ID);
    assert!(param(a, "scope").contains("chatgpt.tokens.use.direct"));
    assert!(matches!(plan.chatgpt().codex_use(), CodexUse::Plan { .. }));
    // Once seen, never again on a later sign-in.
    plan.chatgpt().notice_seen(ID).unwrap();
    let SignInResult::SignedIn { notice, .. } = plan.sign_in(Some(ID), false).unwrap() else {
        panic!()
    };
    assert!(!notice);
}

#[test]
fn two_accounts_stay_apart() {
    let plan = Plan::new("two");
    plan.signed_in(None);
    // The same email in another workspace: its own registration, label, and record.
    plan.openai
        .set(json!({ "client_id": "oaiapp_test2", "account": { "sub": "user-1b", "email": "builder@example.com" } }));
    let second = plan.signed_in(None);
    assert_eq!(
        (second.id.as_str(), second.label.as_str()),
        ("oaiapp_test2", "builder@example.com (2)")
    );
    assert_ne!(
        plan.record(ID).unwrap()["access_token"],
        plan.record("oaiapp_test2").unwrap()["access_token"]
    );
    let active = |plan: &Plan| -> Vec<(String, bool)> {
        plan.chatgpt()
            .state()
            .accounts
            .into_iter()
            .map(|a| (a.id, a.active))
            .collect()
    };
    assert_eq!(active(&plan), [(ID.into(), false), ("oaiapp_test2".into(), true)]);
    // Switching doesn't sign the other out, and a sign-in to the first uses its own client id.
    plan.chatgpt().choose(Some(ID)).unwrap();
    assert_eq!(active(&plan), [(ID.into(), true), ("oaiapp_test2".into(), false)]);
    assert!(plan.chatgpt().state().accounts.iter().all(|a| a.signed_in));
    plan.openai
        .set(json!({ "account": { "sub": "user-1", "email": "builder@example.com" } }));
    let first = plan.signed_in(Some(ID));
    assert_eq!(first.label, "builder@example.com", "labels are stable");
    assert_eq!(
        param(plan.openai.requests("authorize").last().unwrap(), "client_id"),
        ID
    );
}

#[test]
fn sign_out_revokes_and_keeps_the_registration() {
    let plan = Plan::new("sign-out");
    plan.signed_in(None);
    let host = std::fs::read_to_string(plan.data.join("chatgpt/host.json")).unwrap();
    let refresh = plan.record(ID).unwrap()["refresh_token"].clone();
    assert!(plan.chatgpt().sign_out(ID).unwrap().confirmed);
    let r = &plan.openai.requests("revoke")[0];
    assert_eq!(r["token"], refresh);
    assert_eq!(
        (param(r, "token_type_hint"), param(r, "client_id")),
        ("refresh_token", ID)
    );
    assert!(plan.record(ID).is_none(), "the tokens are cleared");
    let a = &plan.chatgpt().state().accounts[0];
    assert!(a.id == ID && !a.signed_in, "the registration stays: {a:?}");
    assert_eq!(
        std::fs::read_to_string(plan.data.join("chatgpt/host.json")).unwrap(),
        host
    );
    assert!(matches!(plan.chatgpt().codex_use(), CodexUse::PlanOff { .. }));
    // Signing in again reuses the registration, without the cleared ID token as a hint.
    plan.signed_in(Some(ID));
    let again = &plan.openai.requests("authorize")[1];
    assert_eq!(param(again, "client_id"), ID);
    assert!(again.get("id_token_hint").is_none());
}

#[test]
fn sign_out_without_confirmed_revocation() {
    let plan = Plan::new("sign-out-503");
    plan.signed_in(None);
    plan.openai.set(json!({ "revoke_status": 503 }));
    assert!(!plan.chatgpt().sign_out(ID).unwrap().confirmed);
    assert!(plan.openai.requests("revoke").len() > 1, "a 5xx is retried");
    assert!(plan.record(ID).is_none(), "cleared locally anyway");
}

/// Makes the stored access token expire in `secs`.
fn expire_in(plan: &Plan, secs: i64) {
    plan.edit_record(ID, |r| r["expires_at"] = json!(now() + secs));
}

#[test]
fn refresh_replaces_the_token_set() {
    let plan = Plan::new("refresh");
    plan.signed_in(None);
    let before = plan.record(ID).unwrap();
    let chatgpt = plan.chatgpt();
    assert_eq!(
        chatgpt.access_token(ID, None, RENEW_BEFORE).unwrap(),
        before["access_token"].as_str().unwrap(),
        "an hour-old token is good"
    );
    assert!(plan.openai.requests("token").len() == 1, "no refresh yet");
    expire_in(&plan, 600);
    let renewed = chatgpt.access_token(ID, None, RENEW_BEFORE).unwrap();
    let r = &plan.openai.requests("token")[1];
    assert_eq!(param(r, "grant_type"), "refresh_token");
    assert_eq!(
        (param(r, "client_id"), r["refresh_token"].clone()),
        (ID, before["refresh_token"].clone())
    );
    assert!(r.get("scope").is_none());
    let after = plan.record(ID).unwrap();
    assert_eq!(after["access_token"], renewed.as_str());
    assert_ne!(after["access_token"], before["access_token"]);
    assert_ne!(
        after["refresh_token"], before["refresh_token"],
        "the rotating refresh token is replaced too"
    );
    assert!(after["expires_at"].as_i64().unwrap() > now() + 3000);
    assert_eq!(chatgpt.access_token(ID, None, RENEW_BEFORE).unwrap(), renewed);
    // A refused token is renewed even though it hasn't expired; one someone else already replaced isn't.
    let again = chatgpt.access_token(ID, Some(&renewed), RENEW_BEFORE).unwrap();
    assert_ne!(again, renewed);
    assert_eq!(chatgpt.access_token(ID, Some(&renewed), RENEW_BEFORE).unwrap(), again);
    assert_eq!(plan.openai.requests("token").len(), 3);
}

#[test]
fn refreshes_are_serialized() {
    let plan = Arc::new(Plan::new("serialized"));
    plan.signed_in(None);
    expire_in(&plan, 600);
    plan.openai.set(json!({ "refresh_delay_ms": 300 }));
    let go = Arc::new(Barrier::new(2));
    let threads: Vec<_> = (0..2)
        .map(|_| {
            let (plan, go) = (plan.clone(), go.clone());
            std::thread::spawn(move || {
                go.wait();
                plan.chatgpt().access_token(ID, None, RENEW_BEFORE)
            })
        })
        .collect();
    let got: Vec<String> = threads.into_iter().map(|t| t.join().unwrap().unwrap()).collect();
    assert_eq!(got[0], got[1], "both callers get the one renewed token");
    let refreshes = plan
        .openai
        .requests("token")
        .iter()
        .filter(|r| r["grant_type"] == "refresh_token")
        .count();
    assert_eq!(refreshes, 1);
}

#[test]
fn refresh_errors() {
    // A refresh token that can't be used again: signed out, ID token kept for the hint.
    let plan = Plan::new("refresh-errors");
    plan.signed_in(None);
    expire_in(&plan, 600);
    plan.openai.set(json!({ "refresh_error": "invalid_grant" }));
    let e = plan.chatgpt().access_token(ID, None, RENEW_BEFORE).unwrap_err();
    assert_eq!(e.code, AgentErrorCode::NotSignedIn);
    let r = plan.record(ID).unwrap();
    assert!(
        r["access_token"].is_null() && r["refresh_token"].is_null() && r["id_token"].is_string(),
        "{r}"
    );
    assert!(!plan.chatgpt().state().accounts[0].signed_in);

    // A 503 never erases credentials: a token that's still good is used; an expired one is an error to retry.
    let plan = Plan::new("refresh-503");
    plan.signed_in(None);
    expire_in(&plan, 600);
    let before = plan.record(ID).unwrap();
    plan.openai.set(json!({ "refresh_error": 503 }));
    assert_eq!(
        plan.chatgpt().access_token(ID, None, RENEW_BEFORE).unwrap(),
        before["access_token"].as_str().unwrap()
    );
    expire_in(&plan, -10);
    let e = plan.chatgpt().access_token(ID, None, RENEW_BEFORE).unwrap_err();
    assert!(e.message.contains("Try again"), "{e}");
    assert_eq!(plan.record(ID).unwrap()["refresh_token"], before["refresh_token"]);
    assert!(plan.chatgpt().state().accounts[0].signed_in);

    // invalid_client: a configuration error, credentials kept.
    plan.openai.set(json!({ "refresh_error": "invalid_client" }));
    let e = plan.chatgpt().access_token(ID, None, RENEW_BEFORE).unwrap_err();
    assert!(e.message.contains("invalid_client"), "{e}");
    assert_eq!(plan.record(ID).unwrap()["refresh_token"], before["refresh_token"]);
}

/// Every file under `dir`, as text.
fn files(dir: &Path, out: &mut Vec<(String, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else if let Ok(bytes) = std::fs::read(&p) {
            out.push((p.display().to_string(), String::from_utf8_lossy(&bytes).into_owned()));
        }
    }
}

/// What good looks like #1: a session whose CLI echoes its token on stderr, in an error, and in its text; a renewal;
/// and a sign-out. Then every token issued is looked for in every output Emmet wrote: the project (transcript), the
/// app data (accounts, host id, the session dir), and every event. The keychain is the one place they may be.
#[test]
fn no_token_reaches_any_output() {
    let plan = Plan::new("leak");
    plan.signed_in(None);
    let rig = Rig::new("chatgpt-leak", "codex", "derived-plan-echo-token").with_plan(&plan);
    let (mut s, mut ev) = rig.start(AgentKind::Codex);
    s.send(UserTurn {
        text: "Reply with exactly the word: ready".into(),
    })
    .unwrap();
    let mut events = ev.turn();
    let first = plan.record(ID).unwrap();
    expire_in(&plan, 600);
    // The restarted app-server resumes the thread: resume.jsonl's second process.
    rig.scenario("codex", "", "resume");
    s.send(UserTurn {
        text: "Reply with exactly the word: ready".into(),
    })
    .unwrap();
    events.extend(ev.turn());
    let second = plan.record(ID).unwrap();
    s.close();
    events.extend(ev.exit());
    assert!(plan.chatgpt().sign_out(ID).unwrap().confirmed);

    let tokens: Vec<String> = [&first, &second]
        .iter()
        .flat_map(|r| ["access_token", "refresh_token", "id_token"].map(|k| r[k].as_str().unwrap().to_string()))
        .collect();
    assert_ne!(tokens[0], tokens[3], "the session renewed its token");
    // The CLI did echo it: the transcript shows where, redacted.
    let transcript =
        std::fs::read_to_string(rig.project.join(format!(".emmet/transcripts/{}.jsonl", rig.cfg.id))).unwrap();
    assert!(transcript.matches("<redacted>").count() >= 3, "{transcript}");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, AgentEvent::Text { delta, .. } if delta.contains("<redacted>")))
    );

    let mut out = vec![("events".to_string(), serde_json::to_string(&events).unwrap())];
    files(&rig.project, &mut out);
    files(&plan.data, &mut out);
    files(&shared_data().join("sessions").join(&rig.cfg.id), &mut out);
    assert!(out.len() > 3, "{:?}", out.iter().map(|(p, _)| p).collect::<Vec<_>>());
    for (path, text) in &out {
        for t in &tokens {
            assert!(!text.contains(t.as_str()), "a token reached {path}");
        }
    }
}
