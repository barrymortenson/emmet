// SPDX-License-Identifier: AGPL-3.0-or-later
//! fake-openai: OpenAI's Sign in with ChatGPT endpoints for the tests (1E): discovery, the JWKS, authorize, the token
//! endpoint (code exchange and refresh), and revocation, at the paths OpenAI documents. It checks every parameter the
//! docs require and answers 400 with the reason when one is wrong, so a test fails on the request Emmet got wrong.
//!
//!   fake-openai <dir>
//!
//! It prints its base URL, then serves until killed. Each request is recorded in `<dir>/openai.jsonl`, and each token
//! it issues in `<dir>/issued.txt`, so a leak test knows what to look for. How it answers comes from
//! `<dir>/openai.json`, read on every request so a test can change it mid-run:
//!
//! - `account` `{sub, email}`: who signs in (default `user-1`, `builder@example.com`)
//! - `client_id`: the id issued to a new registration (default `oaiapp_test1`)
//! - `decline`: authorize answers `error=access_denied`
//! - `grant_plan` (default true): whether `chatgpt.tokens.use.direct` is granted
//! - `callback_client_id`: the `client_id` in the callback ("" leaves it out); by default a new registration gets
//!   its issued id and a returning one none
//! - `bad_state`: the callback carries another state (a stale tab, or a forgery)
//! - `tamper`: an ID token that fails one check (`sig`, `alg`, `iss`, `aud`, `exp`, `nonce`)
//! - `exchange_error`, `refresh_error`: the OAuth error code to answer with (a number is an HTTP status instead)
//! - `revoke_status` (default 200), `expires_in` (default 3600), `refresh_delay_ms`
//! - `earliest_refresh_in`: seconds from now for the token response's `earliest_refresh_at`
//! - `refresh_without_plan`: a refresh grants the scopes without `chatgpt.tokens.use.direct` (plan use turned off)
//!
//! Its JWKS lists a decoy key (kid `old`) before the signing key (kid `test`), as a rotation would.
//!
//! The ID tokens are signed with tests/fixtures/chatgpt/test-signing-key.der, a test-only key.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use emmet_core::base64::encode_url as b64;
use ring::rand::SystemRandom;
use ring::signature::{RSA_PKCS1_SHA256, RsaKeyPair, RsaPublicKeyComponents};
use serde_json::{Value, json};

const SCOPES: &str = "openid profile email offline_access resource.invoke chatgpt.tokens.use.direct";
const RESOURCE: &str = "https://api.openai.com/v1";
const KEY: &[u8] = include_bytes!("../../../fixtures/chatgpt/test-signing-key.der");

struct Code {
    client_id: String,
    challenge: String,
    redirect_uri: String,
    nonce: String,
    scope: String,
}

#[derive(Default)]
struct State {
    codes: HashMap<String, Code>,
    /// Live refresh tokens → (client id, scope).
    refresh: HashMap<String, (String, String)>,
    used: HashSet<String>,
    n: u64,
}

struct Fake {
    dir: PathBuf,
    base: String,
    key: RsaKeyPair,
    state: Mutex<State>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let hex = |j: usize| b.get(j).and_then(|c| (*c as char).to_digit(16));
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match (b[i], hex(i + 1), hex(i + 2)) {
            (b'+', ..) => out.push(b' '),
            (b'%', Some(h), Some(l)) => {
                out.push((h * 16 + l) as u8);
                i += 2;
            }
            (c, ..) => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => (b as char).to_string(),
            b => format!("%{b:02X}"),
        })
        .collect()
}

fn form(s: &str) -> HashMap<String, String> {
    s.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (decode(k), decode(v))
        })
        .collect()
}

struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

fn json_reply(status: u16, v: Value) -> Reply {
    Reply {
        status,
        headers: vec![("Content-Type".into(), "application/json".into())],
        body: v.to_string(),
    }
}

fn bad(why: impl Into<String>) -> Reply {
    json_reply(
        400,
        json!({ "error": "invalid_request", "error_description": why.into() }),
    )
}

impl Fake {
    fn config(&self) -> Value {
        std::fs::read_to_string(self.dir.join("openai.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| json!({}))
    }

    fn record(&self, kind: &str, params: &HashMap<String, String>) {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("openai.jsonl"))
            .expect("the request log");
        let _ = writeln!(f, "{}", json!({ "kind": kind, "params": params }));
    }

    fn issued(&self, token: &str) {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.dir.join("issued.txt"))
            .expect("the issued log");
        let _ = writeln!(f, "{token}");
    }

    fn jwt(&self, claims: Value, tamper: &str) -> String {
        let alg = if tamper == "alg" { "HS256" } else { "RS256" };
        let head = b64(json!({ "alg": alg, "kid": "test", "typ": "JWT" })
            .to_string()
            .as_bytes());
        let body = b64(claims.to_string().as_bytes());
        let signed = format!("{head}.{body}");
        let mut sig = vec![0; self.key.public().modulus_len()];
        self.key
            .sign(&RSA_PKCS1_SHA256, &SystemRandom::new(), signed.as_bytes(), &mut sig)
            .expect("signs");
        if tamper == "sig" {
            sig[10] ^= 1;
        }
        format!("{signed}.{}", b64(&sig))
    }

    /// A token response for `client_id`: access, refresh, and (on a code exchange) ID tokens.
    fn tokens(&self, client_id: &str, scope: &str, nonce: Option<&str>) -> Value {
        let cfg = self.config();
        let mut st = self.state.lock().expect("state");
        st.n += 1;
        let n = st.n;
        let salt = format!("{:x}", now() ^ (n << 20) ^ u64::from(std::process::id()));
        let access = format!("fake-access-{n}-{salt}");
        let refresh = format!("fake-refresh-{n}-{salt}");
        st.refresh.insert(refresh.clone(), (client_id.into(), scope.into()));
        drop(st);
        self.issued(&access);
        self.issued(&refresh);
        let expires_in = cfg["expires_in"].as_i64().unwrap_or(3600);
        let mut v = json!({
            "access_token": access,
            "refresh_token": refresh,
            "token_type": "Bearer",
            "expires_in": expires_in,
            "scope": scope,
        });
        if let Some(secs) = cfg["earliest_refresh_in"].as_i64() {
            v["earliest_refresh_at"] = json!(now() as i64 + secs);
        }
        if let Some(nonce) = nonce {
            let tamper = cfg["tamper"].as_str().unwrap_or("");
            let account = &cfg["account"];
            let t = now() as i64;
            let claims = json!({
                "iss": if tamper == "iss" { "https://evil.example".to_string() } else { self.base.clone() },
                "aud": if tamper == "aud" { "oaiapp_someone_else" } else { client_id },
                "sub": account["sub"].as_str().unwrap_or("user-1"),
                "email": account["email"].as_str().unwrap_or("builder@example.com"),
                "iat": t,
                "exp": if tamper == "exp" { t - 3600 } else { t + 3600 },
                "nonce": if tamper == "nonce" { "not-the-nonce" } else { nonce },
            });
            let id = self.jwt(claims, tamper);
            self.issued(&id);
            v["id_token"] = json!(id);
        }
        v
    }

    fn handle(&self, method: &str, path: &str, query: &str, body: &str) -> Reply {
        let cfg = self.config();
        match (method, path) {
            ("GET", "/.well-known/openid-configuration") => json_reply(
                200,
                json!({
                    "issuer": self.base,
                    "authorization_endpoint": format!("{}/api/accounts/authorize", self.base),
                    "token_endpoint": format!("{}/api/accounts/oauth/token", self.base),
                    "revocation_endpoint": format!("{}/api/accounts/oauth/revoke", self.base),
                    "jwks_uri": format!("{}/.well-known/jwks.json", self.base),
                }),
            ),
            ("GET", "/.well-known/jwks.json") => {
                let c = RsaPublicKeyComponents::<Vec<u8>>::from(self.key.public());
                // The decoy is the same key with its modulus changed: a verifier that ignores `kid` fails on it.
                let mut decoy = c.n.clone();
                decoy[100] ^= 0xff;
                json_reply(
                    200,
                    json!({ "keys": [
                        { "kty": "RSA", "alg": "RS256", "use": "sig", "kid": "old", "n": b64(&decoy), "e": b64(&c.e) },
                        { "kty": "RSA", "alg": "RS256", "use": "sig", "kid": "test", "n": b64(&c.n), "e": b64(&c.e) },
                    ] }),
                )
            }
            ("GET", "/api/accounts/authorize") => self.authorize(&form(query), &cfg),
            ("POST", "/api/accounts/oauth/token") => self.token(&form(body), &cfg),
            ("POST", "/api/accounts/oauth/revoke") => {
                let p = form(body);
                self.record("revoke", &p);
                if p.get("token").is_none_or(|t| t.is_empty())
                    || p.get("token_type_hint").map(String::as_str) != Some("refresh_token")
                    || !p.contains_key("client_id")
                {
                    return bad("revocation needs token, token_type_hint=refresh_token, and client_id");
                }
                let status = cfg["revoke_status"].as_u64().unwrap_or(200) as u16;
                if status == 200 {
                    self.state.lock().expect("state").refresh.remove(&p["token"]);
                }
                Reply {
                    status,
                    headers: vec![],
                    body: String::new(),
                }
            }
            _ => Reply {
                status: 404,
                headers: vec![],
                body: format!("fake-openai has no {method} {path}"),
            },
        }
    }

    fn authorize(&self, p: &HashMap<String, String>, cfg: &Value) -> Reply {
        self.record("authorize", p);
        let get = |k: &str| p.get(k).map(String::as_str).unwrap_or("");
        let redirect = get("redirect_uri");
        let port_ok = redirect
            .strip_prefix("http://127.0.0.1:")
            .and_then(|r| r.strip_suffix("/auth/callback"))
            .is_some_and(|port| port.parse::<u16>().is_ok());
        let checks = [
            (get("response_type") == "code", "response_type=code"),
            (get("scope") == SCOPES, "the documented scopes"),
            (get("resource") == RESOURCE, "resource=https://api.openai.com/v1"),
            (get("code_challenge_method") == "S256", "code_challenge_method=S256"),
            (get("code_challenge").len() == 43, "a base64url SHA-256 code_challenge"),
            (
                get("state").len() >= 32 && get("nonce").len() >= 32,
                "a random state and nonce",
            ),
            (port_ok, "a 127.0.0.1 loopback redirect_uri ending /auth/callback"),
            (
                get("ext_agent_host_id").starts_with("urn:uuid:"),
                "an ext_agent_host_id",
            ),
        ];
        if let Some((_, what)) = checks.iter().find(|(ok, _)| !ok) {
            return bad(format!("authorize needs {what}"));
        }
        let new = get("client_id") == "dynamic_agent_client";
        if new && get("agent_name_hint").is_empty() {
            return bad("a new registration needs agent_name_hint");
        }
        if !new && p.contains_key("agent_name_hint") {
            return bad("agent_name_hint is only for a new registration");
        }
        let state = if cfg["bad_state"] == true {
            "not-the-state".to_string()
        } else {
            encode(get("state"))
        };
        let to = |q: String| Reply {
            status: 302,
            headers: vec![("Location".into(), format!("{redirect}?{q}"))],
            body: String::new(),
        };
        if cfg["decline"] == true {
            return to(format!("error=access_denied&state={state}"));
        }
        let client_id = if new {
            cfg["client_id"].as_str().unwrap_or("oaiapp_test1").to_string()
        } else {
            get("client_id").to_string()
        };
        let scope = if cfg["grant_plan"] == false {
            "openid profile email offline_access".to_string()
        } else {
            SCOPES.to_string()
        };
        let mut st = self.state.lock().expect("state");
        st.n += 1;
        let code = format!("code-{}", st.n);
        st.codes.insert(
            code.clone(),
            Code {
                client_id: client_id.clone(),
                challenge: get("code_challenge").into(),
                redirect_uri: redirect.into(),
                nonce: get("nonce").into(),
                scope: scope.clone(),
            },
        );
        let callback_client = match cfg["callback_client_id"].as_str() {
            Some("") => None,
            Some(c) => Some(c.to_string()),
            None => new.then_some(client_id),
        };
        let mut q = format!("code={code}&scope={}&state={state}", encode(&scope));
        if let Some(c) = callback_client {
            q.push_str(&format!("&client_id={}", encode(&c)));
        }
        to(q)
    }

    fn token(&self, p: &HashMap<String, String>, cfg: &Value) -> Reply {
        self.record("token", p);
        let get = |k: &str| p.get(k).map(String::as_str).unwrap_or("");
        if p.contains_key("client_secret") {
            return bad("an open-source client sends no client_secret");
        }
        if get("resource") != RESOURCE {
            return bad("the token request needs resource=https://api.openai.com/v1");
        }
        let fail = |e: &Value| match e {
            Value::Number(n) => Some(json_reply(
                n.as_u64().unwrap_or(500) as u16,
                json!({ "detail": "unavailable" }),
            )),
            Value::String(code) => Some(json_reply(
                if code == "invalid_client" { 401 } else { 400 },
                json!({ "error": code }),
            )),
            _ => None,
        };
        match get("grant_type") {
            "authorization_code" => {
                let Some(c) = self.state.lock().expect("state").codes.remove(get("code")) else {
                    return json_reply(400, json!({ "error": "invalid_grant" }));
                };
                if let Some(r) = fail(&cfg["exchange_error"]) {
                    return r;
                }
                if get("client_id") != c.client_id {
                    return bad("the exchange must use the issued client_id");
                }
                if get("redirect_uri") != c.redirect_uri {
                    return bad("the exchange must send the authorize request's exact redirect_uri");
                }
                let verifier = get("code_verifier");
                let challenge = b64(ring::digest::digest(&ring::digest::SHA256, verifier.as_bytes()).as_ref());
                if challenge != c.challenge {
                    return bad("the code_verifier doesn't match the code_challenge");
                }
                json_reply(200, self.tokens(&c.client_id, &c.scope, Some(&c.nonce)))
            }
            "refresh_token" => {
                if p.contains_key("scope") {
                    return bad("a refresh omits scope to keep the grant");
                }
                if let Some(ms) = cfg["refresh_delay_ms"].as_u64() {
                    std::thread::sleep(Duration::from_millis(ms));
                }
                if let Some(r) = fail(&cfg["refresh_error"]) {
                    return r;
                }
                let rt = get("refresh_token");
                let mut st = self.state.lock().expect("state");
                if st.used.contains(rt) {
                    return json_reply(400, json!({ "error": "refresh_token_reused" }));
                }
                let Some((client, scope)) = st.refresh.remove(rt) else {
                    return json_reply(400, json!({ "error": "invalid_grant" }));
                };
                st.used.insert(rt.to_string());
                drop(st);
                if get("client_id") != client {
                    return bad("a refresh must use the token's issued client_id");
                }
                let scope = if cfg["refresh_without_plan"] == true {
                    "openid profile email offline_access".to_string()
                } else {
                    scope
                };
                json_reply(200, self.tokens(&client, &scope, None))
            }
            g => bad(format!("unknown grant_type {g}")),
        }
    }
}

fn serve(fake: &Fake, mut s: TcpStream) {
    let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
    let mut r = BufReader::new(s.try_clone().expect("clone"));
    let mut line = String::new();
    if r.read_line(&mut line).is_err() {
        return;
    }
    let mut len = 0;
    loop {
        let mut h = String::new();
        if r.read_line(&mut h).is_err() || h.trim().is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':')
            && k.eq_ignore_ascii_case("content-length")
        {
            len = v.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0; len];
    let _ = r.read_exact(&mut body);
    let mut parts = line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let reply = fake.handle(method, path, query, &String::from_utf8_lossy(&body));
    let mut out = format!(
        "HTTP/1.1 {} X\r\nConnection: close\r\nContent-Length: {}\r\n",
        reply.status,
        reply.body.len()
    );
    for (k, v) in &reply.headers {
        out.push_str(&format!("{k}: {v}\r\n"));
    }
    out.push_str("\r\n");
    out.push_str(&reply.body);
    let _ = s.write_all(out.as_bytes());
}

fn main() {
    let dir = PathBuf::from(std::env::args().nth(1).expect("usage: fake-openai <dir>"));
    std::fs::create_dir_all(&dir).expect("the dir");
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("a port");
    let base = format!("http://127.0.0.1:{}", listener.local_addr().expect("addr").port());
    let fake = Arc::new(Fake {
        dir: dir.clone(),
        base: base.clone(),
        key: RsaKeyPair::from_pkcs8(KEY).expect("the test key"),
        state: Mutex::new(State::default()),
    });
    println!("{base}");
    let _ = std::io::stdout().flush();
    for s in listener.incoming().flatten() {
        let fake = fake.clone();
        std::thread::spawn(move || serve(&fake, s));
    }
}
