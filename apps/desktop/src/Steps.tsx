// SPDX-License-Identifier: AGPL-3.0-or-later
// The Steps tab (Barry 2026-10-27, docs/design/frozen): the model's tree. Built apart first, each sub-assembly with how
// many times it's used and its own sub-steps; then the steps, grouped by the words before a title's ": " (else the step's group). Clicking a step
// shows the build up to it (later steps are hidden); clicking a sub-assembly highlights every copy; clicking one of its
// sub-steps shows it on its own in the viewer, built up to that sub-step (session 4H), until a main step, Escape, or
// its own row shows the model again. The WAI-ARIA tree pattern: up and down move, right and left open and close, Enter
// picks.
import { useEffect, useRef, useState } from "react";
import type { StepsTree, StepView } from "./bindings";
import { Badge, Icon } from "./ui";

/** A step's group and its own name: "Walls, row 1: right wall" is "right wall" under "Walls, row 1". */
function split(s: StepView, first: boolean): [string, string] {
  const at = s.title.indexOf(": ");
  if (at >= 0) return [s.title.slice(0, at), s.title.slice(at + 2)];
  return [s.group || (first ? "Start" : "Steps"), s.title || `Step ${s.n}`];
}

const parts = (n: number) => `${n} part${n === 1 ? "" : "s"}`;

export function Steps({
  tree,
  step,
  sub,
  apart,
  onStep,
  onSub,
  onApart,
}: {
  tree: StepsTree | null | undefined;
  /** The step shown (later ones hidden), or null for the whole model. */
  step: number | null;
  /** The sub-assembly selected (its file), or null. */
  sub: string | null;
  /** The sub-assembly shown on its own (its file), built up to its sub-step `n`, or null. */
  apart: { file: string; n: number } | null;
  onStep: (n: number) => void;
  onSub: (file: string | null) => void;
  onApart: (file: string, n: number) => void;
}) {
  const [open, setOpen] = useState<Set<string>>(new Set());
  // The sub-step shown is in view: its sub-assembly is open (the step keys can get to it with the tree closed).
  useEffect(() => {
    if (apart) setOpen((o) => (o.has(apart.file) ? o : new Set(o).add(apart.file)));
  }, [apart?.file]);
  const list = useRef<HTMLUListElement | null>(null);
  if (!tree || tree.steps.length === 0) return <p className="muted">The steps are made after each build.</p>;
  const total = tree.steps.length;
  const cur = step ?? total;
  const toggle = (file: string, to?: boolean) =>
    setOpen((o) => {
      const n = new Set(o);
      if (to ?? !n.has(file)) n.add(file);
      else n.delete(file);
      return n;
    });

  const onKeyDown = (e: React.KeyboardEvent<HTMLUListElement>) => {
    const items = [...(list.current?.querySelectorAll<HTMLElement>('[role="treeitem"]') ?? [])];
    const i = items.indexOf(document.activeElement as HTMLElement);
    if (i < 0) return;
    const item = items[i]!;
    const move = (j: number) => items[Math.max(0, Math.min(items.length - 1, j))]?.focus();
    const file = item.dataset.sub;
    if (e.key === "ArrowDown") move(i + 1);
    else if (e.key === "ArrowUp") move(i - 1);
    else if (e.key === "Home") move(0);
    else if (e.key === "End") move(items.length - 1);
    else if (e.key === "ArrowRight" && file) toggle(file, true);
    else if (e.key === "ArrowLeft" && file) toggle(file, false);
    else if (e.key === "Enter" || e.key === " ") item.click();
    else return;
    e.preventDefault();
  };

  let head: string | null = null;
  // One item is in the Tab order: the selected one, else the current step.
  const focusable = (key: string) => (apart ? key === `apart:${apart.file}/${apart.n}` : sub ? key === `sub:${sub}` : key === `step:${cur}`);
  const alone = apart && tree.submodels.find((m) => m.file === apart.file);
  return (
    <>
      <div className="row">
        <b id="steps-head">{alone ? `${alone.name}: step ${apart.n} of ${alone.steps.length}` : `Step ${cur} of ${total}`}</b>
        {alone ? <span className="small">Built apart, shown on its own</span> : cur < total && <span className="small">Later steps are hidden</span>}
      </div>
      <ul className="tree" data-c="tree" role="tree" aria-label="Sub-assemblies and steps" ref={list} onKeyDown={onKeyDown} id="steps-tree">
        {tree.submodels.length > 0 && (
          <li className="tree-group" role="presentation">
            Built apart
          </li>
        )}
        {tree.submodels.map((m) => {
          const expanded = open.has(m.file);
          return [
            <li
              key={m.file}
              className="tree-item"
              role="treeitem"
              aria-level={1}
              aria-expanded={expanded}
              aria-selected={sub === m.file}
              data-sub={m.file}
              tabIndex={focusable(`sub:${m.file}`) ? 0 : -1}
              onClick={() => {
                onSub(sub === m.file ? null : m.file);
                toggle(m.file, true);
              }}
            >
              <span
                className="tree-twisty"
                onClick={(e) => {
                  e.stopPropagation();
                  toggle(m.file);
                }}
              >
                <Icon name="chevron-down" size="sm" />
              </span>
              <Icon name="submodel" />
              <span className="tree-name">{m.name}</span>
              <Badge text={`× ${m.count}`} tone="count" />
              <span className="small">{parts(m.parts)}</span>
            </li>,
            ...(expanded
              ? m.steps.map((s) => {
                  const shown = apart?.file === m.file ? apart.n : null;
                  return (
                    <li
                      key={`${m.file}/${s.n}`}
                      className="tree-item"
                      role="treeitem"
                      aria-level={2}
                      aria-selected={shown === s.n}
                      data-step={shown == null ? undefined : s.n < shown ? "past" : s.n === shown ? "current" : "later"}
                      data-sub-step={`${m.file}/${s.n}`}
                      tabIndex={focusable(`apart:${m.file}/${s.n}`) ? 0 : -1}
                      onClick={() => onApart(m.file, s.n)}
                    >
                      <span className="tree-n">{s.n}</span>
                      <span className="tree-name">{split(s, false)[1]}</span>
                      <span className="small">{parts(s.parts)}</span>
                    </li>
                  );
                })
              : []),
          ];
        })}
        {tree.steps.map((s, i) => {
          const [group, name] = split(s, i === 0);
          const top =
            group !== head ? (
              <li key={`g${s.n}`} className="tree-group" role="presentation">
                {group}
              </li>
            ) : null;
          head = group;
          const state = s.n < cur ? "past" : s.n === cur ? "current" : "later";
          return [
            top,
            <li
              key={s.n}
              className="tree-item"
              role="treeitem"
              aria-level={1}
              aria-selected={state === "current" && !sub && !apart}
              data-step={apart ? undefined : state}
              data-n={s.n}
              tabIndex={focusable(`step:${s.n}`) ? 0 : -1}
              onClick={() => onStep(s.n)}
            >
              <span className="tree-n">{s.n}</span>
              <span className="tree-name">{name}</span>
              <span className="small">{parts(s.parts)}</span>
            </li>,
          ];
        })}
      </ul>
    </>
  );
}
