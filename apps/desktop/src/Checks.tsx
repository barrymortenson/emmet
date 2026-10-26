// SPDX-License-Identifier: AGPL-3.0-or-later
// The Checks drawer (plan section 14): the validator's findings for the model on screen. A click highlights the
// finding's parts and shows the model.py line that placed them (design law 2); "Ask agent to fix" sends it to the chat.
import type { Checks as ChecksView, Finding } from "./bindings";

const SEVERITY: Record<string, string> = { error: "Error", warning: "Warning", info: "Note" };

export function checksLabel(checks: ChecksView | null | undefined): string {
  if (!checks) return "Checks";
  const n = checks.findings.length;
  if (!n) return "Checks · clean";
  if (checks.errors) return `Checks · ${checks.errors} ${checks.errors === 1 ? "error" : "errors"}`;
  return `Checks · ${n}`;
}

export function Checks({
  checks,
  selected,
  onPick,
  onAsk,
}: {
  checks: ChecksView | null | undefined;
  selected: string | null;
  onPick: (f: Finding) => void;
  onAsk: (f: Finding) => void;
}) {
  if (!checks) return <p className="muted">Checks run after each build.</p>;
  if (checks.problem) {
    return (
      <p className="notice" role="alert">
        The checks couldn't run: {checks.problem}
      </p>
    );
  }
  if (!checks.findings.length) {
    return <p className="muted" id="checks-clean">No problems: nothing collides, nothing floats, and every part is current.</p>;
  }
  const open = checks.findings.find((f) => f.id === selected);
  return (
    <div className="checks">
      <ul aria-label="Findings">
        {checks.findings.map((f) => (
          <li key={f.id}>
            <button type="button" className="finding" data-finding={f.id} aria-pressed={f.id === selected} onClick={() => onPick(f)}>
              <span className={`severity ${f.severity}`}>{SEVERITY[f.severity] ?? f.severity}</span>
              <span className="what">{f.message}</span>
              {f.source[0] && (
                <span className="where">
                  {f.source[0].file}:{f.source[0].line}
                </span>
              )}
            </button>
          </li>
        ))}
      </ul>
      {open && (
        <section className="finding-detail" aria-label={`Finding ${open.id}`} id="finding-detail">
          {open.source.map((s) => (
            <figure key={`${s.file}:${s.line}`} className="source">
              <figcaption className="location">
                {s.file}:{s.line}
              </figcaption>
              {s.text != null && (
                <pre className="excerpt">
                  {s.line} | {s.text}
                </pre>
              )}
            </figure>
          ))}
          <p className="hint">{open.hint}</p>
          <button type="button" id="ask-fix" onClick={() => onAsk(open)}>
            Ask agent to fix
          </button>
        </section>
      )}
    </div>
  );
}

/** What "Ask agent to fix" sends: the finding as the agent's tools would report it, and what to do. */
export function fixRequest(f: Finding): string {
  const at = f.source.map((s) => `${s.file}:${s.line}`).join(", ");
  return `Fix ${f.id} (${f.rule}, parts ${f.parts.join(", ")}${at ? ` at ${at}` : ""}): ${f.message} ${f.hint} Then run validate again.`;
}
