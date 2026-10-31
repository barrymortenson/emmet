// SPDX-License-Identifier: AGPL-3.0-or-later
// Plan usage on the plan badge at the head of the chat (D-36, plan sections 10 and 14, session 4B): Claude's plan and
// how much of it is used, warn from 90%, and a popover with each window's meter and reset, usage credits when the
// report has them, when it was updated, and Codex's billing state with Manage usage. Fetched only when the user acts
// (opening the popover, Refresh, the end of a turn they started; the host caps it at once a minute); between fetches,
// Claude's live rate_limit_events update it. An API key, unavailable, and unknown each have their own words, and a
// meter is never 0% for want of data. Codex on the ChatGPT plan says "Using ChatGPT plan" with Manage usage, as
// OpenAI's UI rules ask (1E).
import { Popover } from "radix-ui";
import { useCallback, useEffect, useState } from "react";
import { type AgentKind, commands, type UsageSnapshot, type UsageWindow } from "./bindings";
import { ManageUsage } from "./ChatGpt";
import { Badge, IconButton } from "./ui";

/** A wait as hours and minutes (docs/design/content.md): "40m", "2h 14m", "3d 4h". */
export function until(resetsAt: number | null, now = Date.now()): string {
  if (resetsAt == null) return "";
  const m = Math.max(0, Math.round((resetsAt * 1000 - now) / 60_000));
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ${m % 60}m`;
  return `${Math.floor(h / 24)}d ${h % 24}h`;
}

function ago(ms: number, now: number): string {
  const m = Math.round((now - ms) / 60_000);
  if (m < 1) return "Updated just now.";
  if (m < 60) return `Updated ${m}m ago.`;
  return `Updated ${Math.floor(m / 60)}h ago.`;
}

/** The line the chat shows before the next turn when a Claude window is at 90% or more (plan section 10). */
export function limitWarning(s: UsageSnapshot | null, now = Date.now()): string | null {
  const w = s?.windows.find((w) => (w.percent ?? 0) >= 90 && (w.percent ?? 0) < 100);
  if (!w) return null;
  const left = until(w.resets_at, now);
  return `Claude's ${w.label.toLowerCase()} limit is at ${Math.round(w.percent!)}%${left ? `, resets in ${left}` : ""}.`;
}

/** The fullest window: what the badge's percent is. */
const fullest = (s: UsageSnapshot | null) => s?.windows.reduce<UsageWindow | null>((a, w) => (w.percent != null && (a?.percent == null || w.percent > a.percent) ? w : a), null) ?? null;

export function PlanBadge({
  kind,
  live,
  turns,
  plan,
  tokens,
  named,
  onSnapshot,
}: {
  kind: AgentKind;
  /** The newest rate_limit_event, between fetches. */
  live: UsageSnapshot | null;
  /** Counts the turns that ended: each end fetches. */
  turns: number;
  /** Codex runs on the ChatGPT plan. */
  plan: boolean;
  /** What this conversation used, where the CLI reports it. */
  tokens: { input: number; output: number };
  /** Who pays, as detection says it ("Max plan"), until a report says more. */
  named: string | null;
  onSnapshot: (s: UsageSnapshot) => void;
}) {
  const [snap, setSnap] = useState<UsageSnapshot | null>(null);
  const [checking, setChecking] = useState(false);
  const now = Date.now();

  const fetch = useCallback(async () => {
    setChecking(true);
    const s = await commands.usage(kind).finally(() => setChecking(false));
    if (s) {
      setSnap(s);
      onSnapshot(s);
    }
  }, [kind, onSnapshot]);

  useEffect(() => {
    if (turns > 0 && kind === "claude") void fetch();
  }, [turns, fetch, kind]);
  useEffect(() => {
    setSnap(null);
    // Codex's billing state is local (no meters, no request), so it's shown at once.
    if (kind === "codex") void fetch();
  }, [kind, fetch]);

  const onPlan = kind === "codex" && (plan || snap?.billing.kind === "plan");
  if (onPlan)
    return (
      <span className="row" id="plan-label">
        <Badge text="Using ChatGPT plan" />
        <ManageUsage id="plan-manage-usage" />
      </span>
    );
  if (kind === "codex") return <Badge text="API key" id="plan-badge" title="Codex bills your OpenAI API key" />;

  // The newest of the fetched report and the live events.
  const shown = live && (!snap || live.updated_at > snap.updated_at) ? live : snap;
  const top = fullest(shown);
  const name = shown?.billing.kind === "plan" && shown.billing.name ? `${shown.billing.name[0]!.toUpperCase()}${shown.billing.name.slice(1)} plan` : shown?.status === "api_key" ? "API key" : (named ?? "Plan");
  const text = top?.percent != null ? `${name} · ${Math.round(top.percent)}%` : name;
  const warn = (top?.percent ?? 0) >= 90;
  return (
    <Popover.Root onOpenChange={(o) => o && void fetch()}>
      <Popover.Trigger asChild>
        <button type="button" className="badge" data-c="badge" data-tone={warn ? "warn" : undefined} id="plan-badge" title="Plan usage">
          {text}
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content className="popover" data-c="popover" aria-label="Plan usage" side="top" align="start" sideOffset={6} id="usage-popover">
          <span className="popover-title">Claude · {name}</span>
          {shown?.windows.length ? (
            shown.windows.map((w) => (
              <div className="meter-row" key={`${w.kind}-${w.label}`}>
                <span>{w.label}</span>
                {w.percent == null ? (
                  <span className="small">No number reported</span>
                ) : (
                  <span className="meter" data-c="meter" role="meter" aria-label={w.label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={Math.round(w.percent)} data-tone={w.percent >= 90 ? "warn" : undefined}>
                    <i className="meter-fill" style={{ width: `${Math.max(w.percent, 2)}%` }} />
                  </span>
                )}
                <span className="num">{w.percent == null ? "" : `${Math.round(w.percent)}%`}</span>
                <span className="small">{w.resets_at != null && `resets in ${until(w.resets_at, now)}`}</span>
              </div>
            ))
          ) : (
            <p className="small" id="usage-none">
              {shown?.status === "api_key" ? "Claude Code bills an API key, which has no plan meters." : shown?.status === "unavailable" ? `Usage unavailable: ${shown.failure ?? "Claude Code didn't report it."}` : shown ? "Usage unknown: Claude Code reported no meters." : checking ? "Checking…" : "No usage report yet."}
            </p>
          )}
          {shown?.credits?.enabled && (
            <p className="small" id="usage-credits">
              Usage credits: {shown.credits.used ?? "?"} of {shown.credits.limit ?? "no limit"}
              {shown.credits.currency ? ` ${shown.credits.currency}` : ""}
            </p>
          )}
          {tokens.input + tokens.output > 0 && (
            <p className="small" id="usage-tokens">
              This conversation: {tokens.input.toLocaleString("en-US")} tokens in, {tokens.output.toLocaleString("en-US")} out.
            </p>
          )}
          <div className="popover-head">
            <span className="small">{shown ? ago(shown.updated_at, now) : ""}</span>
            <IconButton icon="refresh-cw" label="Refresh" size="sm" id="usage-refresh" onClick={() => void fetch()} />
          </div>
          <CodexLine />
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}

/** Codex's billing state under Claude's meters, as the frozen popover draws it. */
function CodexLine() {
  const [codex, setCodex] = useState<UsageSnapshot | null>(null);
  useEffect(() => {
    void commands.usage("codex").then(setCodex);
  }, []);
  if (!codex) return null;
  return (
    <div className="popover-head" id="usage-codex">
      <span className="small">Codex · {codex.billing.kind === "plan" ? "Using ChatGPT plan" : codex.billing.kind === "api_key" ? "API key" : "not set up"}</span>
      {codex.billing.kind === "plan" && <ManageUsage />}
    </div>
  );
}
