// SPDX-License-Identifier: AGPL-3.0-or-later
// Plan usage in the status bar (D-36, plan section 10): Claude's meters from its own /usage report and live
// rate_limit_events, fetched only when the user acts (opening the popover, ⟳, the end of a turn they started; the host
// caps it at once a minute). An API key, unavailable, and unknown each have their own label, and a meter is never 0 %
// for want of data. Codex shows its billing state only: on the ChatGPT plan, OpenAI's "Using ChatGPT plan" with Manage
// usage (1E).
import { useCallback, useEffect, useState } from "react";
import { type AgentKind, commands, type UsageSnapshot, type UsageWindow } from "./bindings";
import { ManageUsage } from "./ChatGpt";

function until(resetsAt: number | null, now: number): string {
  if (resetsAt == null) return "";
  const m = Math.max(0, Math.round((resetsAt * 1000 - now) / 60_000));
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h ${m % 60}m`;
  return `${Math.floor(h / 24)}d ${h % 24}h`;
}

function ago(ms: number, now: number): string {
  const m = Math.round((now - ms) / 60_000);
  return m < 1 ? "just now" : `${m}m ago`;
}

const pct = (w: UsageWindow) => (w.percent == null ? "—" : `${Math.round(w.percent)}%`);

/** The line the chat shows before the next turn when a Claude window is at 90 % or more (plan section 10). */
export function limitWarning(s: UsageSnapshot | null, now = Date.now()): string | null {
  const w = s?.windows.find((w) => (w.percent ?? 0) >= 90);
  if (!w) return null;
  const left = until(w.resets_at, now);
  return `Claude's ${w.label.toLowerCase()} limit is at ${Math.round(w.percent!)}%${left ? `, resets in ${left}` : ""}.`;
}

export function UsageBar({
  kind,
  live,
  turns,
  onSnapshot,
}: {
  kind: AgentKind | null;
  /** The newest rate_limit_event, between fetches. */
  live: UsageSnapshot | null;
  /** Counts the turns the user started that have ended: each end fetches. */
  turns: number;
  onSnapshot: (s: UsageSnapshot) => void;
}) {
  const [snap, setSnap] = useState<UsageSnapshot | null>(null);
  const [open, setOpen] = useState(false);
  const now = Date.now();

  const fetch = useCallback(async () => {
    if (!kind) return;
    const s = await commands.usage(kind);
    if (s) {
      setSnap(s);
      onSnapshot(s);
    }
  }, [kind, onSnapshot]);

  useEffect(() => {
    if (turns > 0) void fetch();
  }, [turns, fetch]);
  useEffect(() => {
    setSnap(null);
    // Codex's billing state is local (no meters, no request), so it's shown as soon as its chat starts.
    if (kind === "codex") void fetch();
  }, [kind, fetch]);

  if (!kind) return null;
  if (kind === "codex" && snap?.billing.kind === "plan")
    return (
      <span className="usage" id="usage-plan">
        Codex · Using ChatGPT plan <ManageUsage id="usage-manage" />
      </span>
    );
  // The newest of the fetched report and the live events.
  const shown = live && kind === "claude" && (!snap || live.updated_at > snap.updated_at) ? live : snap;
  const who = kind === "claude" ? "Claude" : "Codex";
  let line: string;
  if (!shown) line = `${who} usage`;
  else if (shown.status === "api_key") line = `${who} · API key`;
  else if (shown.status === "unavailable") line = `${who} usage unavailable`;
  else if (shown.status === "unknown" || !shown.windows.length) line = `${who} usage unknown`;
  else line = `Used · ${who} ${shown.windows.map((w) => `${pct(w)} ${until(w.resets_at, now)}`.trim()).join(" · ")}`;

  return (
    <span className="usage">
      <button
        type="button"
        id="usage-button"
        aria-expanded={open}
        onClick={() => {
          setOpen(!open);
          if (!open) void fetch();
        }}
      >
        {line}
      </button>
      {open && (
        <div className="popover" role="dialog" aria-label="Plan usage" id="usage-popover">
          {shown?.windows.length ? (
            <table>
              <tbody>
                {shown.windows.map((w) => (
                  <tr key={`${w.kind}-${w.label}`}>
                    <th scope="row">{w.label}</th>
                    <td>
                      <meter min={0} max={100} value={w.percent ?? 0} aria-label={`${w.label} used`} />
                    </td>
                    <td>{pct(w)}</td>
                    <td className="muted">{w.resets_at != null && `resets in ${until(w.resets_at, now)}`}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p className="muted">
              {shown?.status === "api_key"
                ? `${who} bills an API key, which has no plan meters.`
                : (shown?.failure ?? "No usage report yet.")}
            </p>
          )}
          {shown?.credits?.enabled && (
            <p className="muted">
              Extra usage: {shown.credits.used} of {shown.credits.limit ?? "no limit"} {shown.credits.currency ?? ""}
            </p>
          )}
          <p className="muted">
            {shown && `Updated ${ago(shown.updated_at, now)} `}
            <button type="button" aria-label="Refresh usage" onClick={() => void fetch()}>
              ⟳
            </button>
          </p>
        </div>
      )}
    </span>
  );
}
