// SPDX-License-Identifier: AGPL-3.0-or-later
//! OpenAI's ID token: an RS256 JWT, checked as the sign-in page asks (signature against the published JWKS, issuer,
//! audience = the issued client id, expiry, and the attempt's nonce). Its `sub` is the account's identity.

use emmet_core::base64;
use ring::signature::{RSA_PKCS1_2048_8192_SHA256, RsaPublicKeyComponents};
use serde_json::Value;

/// Clock skew allowed on `exp`, in seconds.
const LEEWAY: i64 = 60;

/// What the token must say.
pub struct Expect<'a> {
    pub issuer: &'a str,
    pub audience: &'a str,
    pub nonce: &'a str,
    /// Unix seconds.
    pub now: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Claims {
    pub sub: String,
    pub email: Option<String>,
}

fn part(s: &str) -> Result<Value, String> {
    let bytes = base64::decode(s).ok_or("it isn't base64url")?;
    serde_json::from_slice(&bytes).map_err(|_| "it isn't JSON".to_string())
}

/// Checks `token` against `jwks` (the JWKS document) and `want`. The error says what failed, never the token.
pub fn verify(token: &str, jwks: &Value, want: &Expect) -> Result<Claims, String> {
    let mut parts = token.split('.');
    let (Some(h), Some(p), Some(s), None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return Err("the ID token isn't a JWT".into());
    };
    let header = part(h).map_err(|e| format!("the ID token's header: {e}"))?;
    // Only RS256: OpenAI's discovery lists no other, and accepting the header's choice is the classic JWT hole.
    if header["alg"] != "RS256" {
        return Err(format!("the ID token is signed with {}, not RS256", header["alg"]));
    }
    let keys = jwks["keys"].as_array().ok_or("the JWKS has no keys")?;
    let kid = header["kid"].as_str();
    let key = keys
        .iter()
        .filter(|k| k["kty"] == "RSA")
        .find(|k| kid.is_none() || k["kid"].as_str() == kid)
        .ok_or("no key in OpenAI's JWKS matches the ID token")?;
    let n = key["n"]
        .as_str()
        .and_then(base64::decode)
        .ok_or("the JWKS key has no modulus")?;
    let e = key["e"]
        .as_str()
        .and_then(base64::decode)
        .ok_or("the JWKS key has no exponent")?;
    let sig = base64::decode(s).ok_or("the ID token's signature isn't base64url")?;
    let signed = &token[..h.len() + 1 + p.len()];
    RsaPublicKeyComponents { n: &n, e: &e }
        .verify(&RSA_PKCS1_2048_8192_SHA256, signed.as_bytes(), &sig)
        .map_err(|_| "the ID token's signature doesn't match OpenAI's key")?;

    let c = part(p).map_err(|e| format!("the ID token's claims: {e}"))?;
    if c["iss"] != want.issuer {
        return Err(format!("the ID token's issuer is {}, not {}", c["iss"], want.issuer));
    }
    let aud_ok = match &c["aud"] {
        Value::String(a) => a == want.audience,
        Value::Array(a) => a.iter().any(|a| a == want.audience),
        _ => false,
    };
    if !aud_ok {
        return Err("the ID token is for another client".into());
    }
    match c["exp"].as_i64() {
        Some(exp) if exp + LEEWAY > want.now => {}
        _ => return Err("the ID token has expired".into()),
    }
    if c["nonce"] != want.nonce {
        return Err("the ID token's nonce isn't this sign-in's".into());
    }
    let sub = c["sub"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("the ID token has no subject")?;
    Ok(Claims {
        sub: sub.to_string(),
        email: c["email"].as_str().map(String::from),
    })
}
