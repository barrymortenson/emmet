// SPDX-License-Identifier: AGPL-3.0-or-later
// The Checks tab (plan section 14, docs/design/frozen): each finding in plain words, one click from its parts (law 2):
// what's wrong, a tag, and why, with Ask agent to fix and It's fine. Its ids, source line, and hint go to the agent,
// never to the screen. Allowed findings go quiet, with Check it again.
import type { AllowedView, Checks as ChecksView, Finding } from "./bindings";
import { Badge, Button, Icon } from "./ui";

/** Each rule's tag, and its headline for `n` parts. */
const RULES: Record<string, [tag: string, headline: (n: number) => string]> = {
  floating: ["Floating", (n) => (n === 1 ? "1 part isn't held by anything" : `${n} parts aren't held by anything`)],
  collision: ["Collision", (n) => `${n} parts overlap`],
  disconnected_submodel: ["Loose sub-assembly", () => "A sub-assembly isn't attached to the model"],
  step_order: ["Step order", () => "A step builds on parts that come later"],
  step_size: ["Step size", () => "A step adds more parts than the age allows"],
  weak_connection: ["Weak connection", (n) => (n === 1 ? "1 part hangs on one stud" : `${n} parts hang on one stud`)],
  off_grid: ["Off the grid", (n) => (n === 1 ? "1 part is off the grid" : `${n} parts are off the grid`)],
  unknown_part: ["Unknown part", (n) => (n === 1 ? "1 part isn't in the library" : `${n} parts aren't in the library`)],
  part_unavailable: ["Not made", (n) => (n === 1 ? "1 part was never made" : `${n} parts were never made`)],
  color_unavailable: ["Color not made", (n) => (n === 1 ? "1 part was never made in its color" : `${n} parts were never made in their color`)],
  obsolete_part: ["Obsolete", (n) => (n === 1 ? "1 part is obsolete" : `${n} parts are obsolete`)],
  alias_part: ["Renamed", (n) => (n === 1 ? "1 part has a newer number" : `${n} parts have newer numbers`)],
};

const rule = (r: string): [string, (n: number) => string] => RULES[r] ?? [r.replaceAll("_", " "), (n) => `${n} parts need a look`];

/** The validator's sentence for the screen: a part named "wall/a (Brick 2 x 4)" reads "Brick 2 x 4" (no ids, law 2). */
export function plain(message: string): string {
  return message.replace(/\b[\w.:-]+(?:\/[\w.:-]+)* \(([^()]+)\)/g, "$1");
}

export function checksCount(checks: ChecksView | null | undefined): number {
  return checks?.findings.length ?? 0;
}

export function Checks({
  checks,
  selected,
  readonly,
  onPick,
  onAsk,
  onAllow,
  onUnallow,
}: {
  checks: ChecksView | null | undefined;
  selected: string | null;
  readonly: boolean;
  onPick: (f: Finding) => void;
  onAsk: (f: Finding) => void;
  onAllow: (f: Finding) => void;
  onUnallow: (a: AllowedView) => void;
}) {
  if (!checks) return <p className="muted">Checks run after each build.</p>;
  if (checks.problem)
    return (
      <p className="notice" role="alert">
        The checks couldn't run: {checks.problem}
      </p>
    );
  return (
    <>
      {checks.findings.length === 0 ? (
        <div className="checks-clean" data-c="finding-row" id="checks-clean">
          <Icon name="circle-check" size="lg" />
          <div>
            <b>No problems</b>
            <p className="muted">Nothing collides, nothing floats, and every part is current.</p>
          </div>
        </div>
      ) : (
        checks.findings.map((f) => {
          const [tag, headline] = rule(f.rule);
          return (
            <div
              key={f.id}
              className="finding"
              data-c="finding-row"
              data-finding={f.id}
              aria-selected={f.id === selected}
              role="button"
              tabIndex={0}
              onClick={() => onPick(f)}
              onKeyDown={(e) => {
                if (e.target === e.currentTarget && (e.key === "Enter" || e.key === " ")) {
                  e.preventDefault();
                  onPick(f);
                }
              }}
            >
              <div className="finding-top">
                <Icon name={f.rule === "floating" ? "floating" : "triangle-alert"} />
                <span>{headline(f.parts.length)}</span>
                <Badge text={tag} />
              </div>
              <p>{plain(f.message)}</p>
              {!readonly && (
                <div className="row">
                  <Button label="Ask agent to fix" variant="primary" onClick={(e) => (e.stopPropagation(), onAsk(f))} id={`fix-${f.id}`} />
                  <Button label="It's fine" variant="quiet" title="Allow it: the check won't flag these parts again" onClick={(e) => (e.stopPropagation(), onAllow(f))} id={`allow-${f.id}`} />
                </div>
              )}
            </div>
          );
        })
      )}
      {(checks.allowed ?? []).map((a) => {
        const [, headline] = rule(a.rule);
        const key = `${a.rule}:${a.parts.join(",")}`;
        return (
          <div key={key} className="finding" data-c="finding-row" data-state="allowed" data-allowed={a.rule}>
            <div className="finding-top">
              <Icon name={a.rule === "floating" ? "floating" : "triangle-alert"} />
              <span>{headline(a.parts.length)}</span>
              <Badge text="Allowed" />
            </div>
            {!readonly && (
              <div className="row">
                <Button label="Check it again" size="sm" variant="quiet" onClick={() => onUnallow(a)} />
              </div>
            )}
          </div>
        );
      })}
    </>
  );
}

/** What the chat shows for "Ask agent to fix": the finding in plain words, as Checks says it. */
export function findingWords(f: Finding): string {
  return `${rule(f.rule)[1](f.parts.length)}. ${plain(f.message)}`.trim();
}

/** What "Ask agent to fix" gives the agent as the turn's details (the chat shows findingWords): the finding as validate
 * reports it, and what to do. Its words are the validator's own and the library's part names; ids can't hold spaces
 * (the DSL refuses them), and a long list of parts or lines is cut short. */
export function fixRequest(f: Finding): string {
  const few = (list: string[]) => (list.length > 8 ? `${list.slice(0, 8).join(", ")} and ${list.length - 8} more` : list.join(", "));
  const at = few(f.source.map((s) => `${s.file}:${s.line}`));
  return `Fix ${f.id} (${f.rule}, parts ${few(f.parts)}${at ? ` at ${at}` : ""}): ${f.message} ${f.hint} Then run validate again.`;
}
