// SPDX-License-Identifier: AGPL-3.0-or-later
//! Plan usage v0 (D-36, plan section 10): one shape for every agent. Claude's meters come only from Claude Code's
//! documented `/usage` report (`usage_report.rate_limits`) and `rate_limit_event`; Codex reports its billing state
//! only, because OpenAI documents no usage source for an API key. Rows keep the order the CLI reports them, so a new
//! meter appears without an Emmet release, and a meter is never 0 % for want of data: those states have their own
//! status.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{AgentKind, Billing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum UsageStatus {
    /// Meters from the CLI.
    Ok,
    /// A limit is reached: the CLI is rejecting requests until a reset.
    Limited,
    /// Billed to an API key, which has no plan meters.
    ApiKey,
    /// The CLI didn't report usage (see `failure`).
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "snake_case")]
pub enum UsageSource {
    UsageReport,
    RateLimitEvent,
    Billing,
}

/// One meter: "Session 4 %, resets in 3h", "Week · Fable 0 %".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct UsageWindow {
    /// The CLI's own name for it (`session`, `weekly_all`, `five_hour`).
    pub kind: String,
    pub group: String,
    pub label: String,
    /// 0–100, or None when the CLI gave no number.
    pub percent: Option<f64>,
    /// Unix seconds.
    pub resets_at: Option<i64>,
    /// What it's limited to (a model, "Fable"), if anything.
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct Credits {
    pub enabled: bool,
    pub used: f64,
    pub limit: Option<f64>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub struct UsageSnapshot {
    pub agent: AgentKind,
    pub billing: Billing,
    pub windows: Vec<UsageWindow>,
    pub credits: Option<Credits>,
    pub source: UsageSource,
    /// Unix milliseconds.
    pub updated_at: u64,
    pub status: UsageStatus,
    /// Why there are no meters, when `status` is unavailable.
    pub failure: Option<String>,
}

impl UsageSnapshot {
    /// A snapshot with no meters: an API key, or usage that's unavailable for `failure`.
    pub fn without_meters(agent: AgentKind, billing: Billing, failure: Option<String>) -> Self {
        let status = match (&billing, &failure) {
            (Billing::ApiKey { .. }, _) => UsageStatus::ApiKey,
            (_, Some(_)) => UsageStatus::Unavailable,
            _ => UsageStatus::Unknown,
        };
        Self {
            agent,
            billing,
            windows: vec![],
            credits: None,
            source: UsageSource::Billing,
            updated_at: crate::time::now_ms(),
            status,
            failure,
        }
    }

    /// Claude Code's `/usage` report: `rate_limits.limits[]` and `extra_usage`.
    pub fn from_usage_report(report: &Value, billing: Billing) -> Self {
        let windows: Vec<UsageWindow> = report["rate_limits"]["limits"]
            .as_array()
            .map(|limits| {
                limits
                    .iter()
                    .map(|l| {
                        let kind = l["kind"].as_str().unwrap_or("limit").to_string();
                        let scope = l["scope"]["model"]["display_name"].as_str().map(String::from);
                        UsageWindow {
                            label: label(&kind, scope.as_deref()),
                            group: l["group"].as_str().unwrap_or("").to_string(),
                            percent: l["percent"].as_f64(),
                            resets_at: l["resets_at"].as_str().and_then(crate::time::parse_rfc3339),
                            scope,
                            kind,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        let extra = &report["rate_limits"]["extra_usage"];
        let credits = extra.is_object().then(|| Credits {
            enabled: extra["is_enabled"].as_bool().unwrap_or(false),
            used: extra["used_credits"].as_f64().unwrap_or(0.0),
            limit: extra["monthly_limit"].as_f64(),
            currency: extra["currency"].as_str().map(String::from),
        });
        let mut s = Self::without_meters(AgentKind::Claude, billing, None);
        s.status = if windows.is_empty() {
            UsageStatus::Unavailable
        } else {
            UsageStatus::Ok
        };
        if windows.is_empty() {
            s.failure = Some("Claude Code's /usage report had no meters".into());
        }
        s.windows = windows;
        s.credits = credits;
        s.source = UsageSource::UsageReport;
        s
    }

    /// A `rate_limit_event`'s `rate_limit_info`: `unifiedWindows` with `utilization` 0–1, and `status`.
    pub fn from_rate_limit(info: &Value) -> Self {
        let windows = info["unifiedWindows"]
            .as_object()
            .map(|w| {
                w.iter()
                    .map(|(kind, v)| UsageWindow {
                        label: label(kind, None),
                        group: kind.clone(),
                        percent: v["utilization"].as_f64().map(|u| (u * 1000.0).round() / 10.0),
                        resets_at: v["resetsAt"].as_i64(),
                        scope: None,
                        kind: kind.clone(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut s = Self::without_meters(AgentKind::Claude, Billing::Plan { name: None }, None);
        s.windows = windows;
        s.source = UsageSource::RateLimitEvent;
        s.status = match info["status"].as_str() {
            Some("rejected") => UsageStatus::Limited,
            _ => UsageStatus::Ok,
        };
        s
    }
}

/// A meter's label from the CLI's kind: "Session", "Week", "Week · Fable", "5-hour".
fn label(kind: &str, scope: Option<&str>) -> String {
    let base = match kind {
        "session" => "Session".to_string(),
        "weekly_all" | "weekly_scoped" | "seven_day" => "Week".to_string(),
        "five_hour" => "5-hour".to_string(),
        other => {
            let words = other.replace('_', " ");
            crate::capitalize(&words)
        }
    };
    match scope {
        Some(s) => format!("{base} · {s}"),
        None => base,
    }
}
