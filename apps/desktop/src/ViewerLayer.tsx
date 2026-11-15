// SPDX-License-Identifier: AGPL-3.0-or-later
// What the viewer carries over the model (plan section 14, docs/design/components.md's viewer): the hovered part's
// name and color, ⌥-drag's box, a finding's pin at its parts, a step's parts box (and any sub-assembly's box, with
// its leader to its first copy), a sub-assembly shown on its own in its frame with its bar (session 4H), the
// cut-away's bar, and Skip animation while the build animation runs. The pins and
// the leader are placed from the viewer's own projection and follow the camera; nothing here moves it (law 1).
import type { Rect, Viewer } from "@emmet/viewer";
import { useEffect, useMemo, useRef, useState } from "react";
import type { Finding, Scene, StepView, SubmodelView } from "./bindings";
import { pinWords } from "./Checks";
import { assemblyPicture, partPicture } from "./pictures";
import { Button, Icon, IconButton } from "./ui";

/** Redraws when the viewer's camera or canvas changes, at most once a frame. */
function useCamera(viewer: Viewer | null): number {
  const [tick, setTick] = useState(0);
  useEffect(() => {
    if (!viewer) return;
    let queued = false;
    return viewer.on("camera", () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(() => {
        queued = false;
        setTick((t) => t + 1);
      });
    });
  }, [viewer]);
  return tick;
}

function Picture({ load, alt }: { load: () => Promise<string | null>; alt: string }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    void load().then((s) => live && setSrc(s));
    return () => {
      live = false;
    };
    // `load` is a new closure each render; the parent's `key` names what it loads, so a new picture remounts.
  }, []);
  return src ? <img src={src} alt={alt} /> : <span className="parts-box-pic" aria-hidden="true" />;
}

/** A part's name and color, from the parts list. */
function useNames(scene: Scene | null): (id: string) => string | null {
  return useMemo(() => {
    const byId = new Map(scene?.placements.map((p) => [p.id, p]) ?? []);
    const names = new Map(scene?.bom?.lines.map((l) => [`${l.part}|${l.color}`, l]) ?? []);
    return (id: string) => {
      const p = byId.get(id);
      const l = p && names.get(`${p.part}|${p.color}`);
      return l ? `${l.name} · ${l.color_name}` : p ? p.part : null;
    };
  }, [scene]);
}

export function ViewerLayer({
  viewer,
  scene,
  step,
  apart,
  cut,
  layers,
  animating,
  findings,
  showPins,
  onPin,
  onCut,
  onApart,
  onSkip,
}: {
  viewer: Viewer | null;
  scene: Scene | null;
  /** The step shown, or null for the whole model. */
  step: number | null;
  /** The sub-assembly shown on its own: its file, the copy shown, and the sub-step it's built up to. */
  apart: { file: string; copy: string; n: number } | null;
  cut: number | null;
  layers: number;
  animating: boolean;
  findings: Finding[];
  showPins: boolean;
  onPin: (f: Finding) => void;
  onCut: (layer: number | null) => void;
  /** Another of the sub-assembly's steps, or null for the model again. */
  onApart: (n: number | null) => void;
  onSkip: () => void;
}) {
  const tick = useCamera(viewer);
  const name = useNames(scene);
  const [hover, setHover] = useState<{ id: string; x: number; y: number } | null>(null);
  const [box, setBox] = useState<Rect | null>(null);
  const pins = useMemo(() => pinned(findings), [findings]);
  useEffect(() => {
    if (!viewer) return;
    const off = [viewer.on("hover", setHover), viewer.on("boxing", setBox)];
    return () => off.forEach((f) => f());
  }, [viewer]);

  const tree = scene?.tree;
  const alone: SubmodelView | undefined = apart ? tree?.submodels.find((m) => m.file === apart.file) : undefined;
  const subStep = apart ? alone?.steps.find((s) => s.n === apart.n) : undefined;
  // A sub-step's own sub-assemblies are named from inside it: in the scene they're this copy's.
  const shown: StepView | undefined =
    apart && subStep
      ? { ...subStep, subs: subStep.subs.map((s) => ({ ...s, ids: s.ids.map((id) => `${apart.copy}/${id}`) })) }
      : step != null && !apart
        ? tree?.steps.find((s) => s.n === step)
        : undefined;
  const label = hover && name(hover.id);
  return (
    <>
      {label && hover && (
        <div className="tooltip viewer-hover" role="tooltip" id="viewer-hover" style={{ left: hover.x, top: hover.y }}>
          {label}
        </div>
      )}
      {box && <div className="viewer-box" id="viewer-box" aria-hidden="true" style={{ left: box.x, top: box.y, width: box.width, height: box.height }} />}
      {showPins && viewer && pins.map((p) => <Pin key={p.rule} viewer={viewer} pin={p} tick={tick} onPin={onPin} />)}
      {apart && alone && <div className="apart-frame" id="apart-frame" aria-hidden="true" />}
      {shown && scene && <StepBox key={`${scene.generation}:${apart?.copy ?? ""}:${shown.n}`} viewer={viewer} scene={scene} step={shown} tick={tick} underBar={!!apart} />}
      {apart && alone && (
        <div className="viewer-bar" role="status" id="apart-bar">
          <Icon name="submodel" />
          <span>
            {alone.name}
            {alone.count > 1 ? ` (${alone.count}x)` : ""}
          </span>
          <b id="apart-at">
            {apart.n} of {alone.steps.length}
          </b>
          <IconButton icon="chevron-left" label="Previous step ([)" size="sm" id="apart-prev" disabled={apart.n <= 1} onClick={() => onApart(apart.n - 1)} />
          <IconButton icon="chevron-right" label="Next step (])" size="sm" id="apart-next" disabled={apart.n >= alone.steps.length} onClick={() => onApart(apart.n + 1)} />
          <Button label="Back" variant="quiet" size="sm" id="apart-off" title="Show the model again (Esc)" onClick={() => onApart(null)} />
        </div>
      )}
      {cut != null && !apart && (
        <div className="viewer-bar" role="status" id="cut-bar">
          <Icon name="layers" />
          <span>
            Showing layers 1–{cut} of {layers}
          </span>
          <IconButton icon="chevron-down" label="Lower the cut-away (⌥↓)" size="sm" id="cut-down" disabled={cut <= 1} onClick={() => onCut(cut - 1)} />
          <IconButton icon="chevron-up" label="Raise the cut-away (⌥↑)" size="sm" id="cut-up" onClick={() => onCut(cut + 1 > layers ? null : cut + 1)} />
          <Button label="Show all" variant="quiet" size="sm" id="cut-off" onClick={() => onCut(null)} />
        </div>
      )}
      {animating && (
        <div className="viewer-tools">
          <Button label="Skip animation" size="sm" className="viewer-skip" id="skip-animation" onClick={onSkip} />
        </div>
      )}
    </>
  );
}

const SEVERITY: Record<string, number> = { error: 0, warning: 1 };

/** One pin for each rule about where parts are, over all its findings' parts: five floating findings make one "12
 * parts float", not five pins stacked over the model. A click opens the rule's first finding in Checks. */
type PinGroup = { rule: string; first: Finding; parts: string[]; words: string };

/** The pins: errors first, then warnings, in Checks' order. */
function pinned(findings: Finding[]): PinGroup[] {
  const groups = new Map<string, { first: Finding; worst: number; at: number; parts: Set<string> }>();
  findings.forEach((f, i) => {
    if (!f.parts.length || f.severity === "info" || pinWords(f.rule, 1) == null) return;
    const g = groups.get(f.rule) ?? { first: f, worst: 3, at: i, parts: new Set<string>() };
    g.worst = Math.min(g.worst, SEVERITY[f.severity] ?? 3);
    for (const id of f.parts) g.parts.add(id);
    groups.set(f.rule, g);
  });
  return [...groups]
    .sort(([, a], [, b]) => a.worst - b.worst || a.at - b.at)
    .map(([rule, g]) => ({ rule, first: g.first, parts: [...g.parts], words: pinWords(rule, g.parts.size) ?? "" }));
}

/** A rule's pin, just above its parts, with its glyph and a few words; it opens the rule's first finding in Checks. */
function Pin({ viewer, pin, tick, onPin }: { viewer: Viewer; pin: PinGroup; tick: number; onPin: (f: Finding) => void }) {
  const at = useMemo(() => viewer.screenPoint(pin.parts), [viewer, pin.parts, tick]);
  if (!at?.visible) return null;
  const glyph = pin.rule === "floating" ? "floating" : pin.rule === "collision" ? "collision" : "triangle-alert";
  return (
    <button type="button" className="pin viewer-pin" data-c="pin" data-pin={pin.rule} style={{ left: at.x, top: at.y }} onClick={() => onPin(pin.first)}>
      <span className="pin-dot">
        <Icon name={glyph} size="sm" />
      </span>
      {pin.words}
    </button>
  );
}

/** A step as the instruction books draw it (plan section 08): the step number over its parts box with each new
 * part's picture and count, and each sub-assembly it puts on in its own box, with its parts and numbered sub-steps and
 * a leader to the first copy it can reach. */
function StepBox({ viewer, scene, step, tick, underBar }: { viewer: Viewer | null; scene: Scene; step: StepView; tick: number; underBar?: boolean }) {
  const ref = useRef<HTMLDivElement | null>(null);
  const subs = step.subs
    .map((s) => ({ s, model: scene.tree?.submodels.find((m) => m.file === s.file) }))
    .filter((x): x is { s: (typeof step.subs)[number]; model: NonNullable<typeof x.model> } => !!x.model);
  const [leader, setLeader] = useState<string | null>(null);
  // The box grows as its pictures draw: the leader starts from where its foot is now.
  const [layout, setLayout] = useState(0);
  useEffect(() => {
    const el = ref.current;
    if (!el || typeof ResizeObserver === "undefined") return;
    // On the next frame, never inside the observer's own callback: laying out again there is reported as an uncaught
    // "ResizeObserver loop completed with undelivered notifications" (WebView2, 5B; virtual.tsx has the same for WebKit).
    let frame = 0;
    const o = new ResizeObserver(() => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => setLayout((n) => n + 1));
    });
    o.observe(el);
    return () => {
      cancelAnimationFrame(frame);
      o.disconnect();
    };
  }, []);
  useEffect(() => {
    const el = ref.current?.querySelector(".sub-assembly");
    const host = ref.current?.parentElement;
    const first = subs[0];
    if (!el || !host || !viewer || !first) return setLeader(null);
    const hr = host.getBoundingClientRect();
    const sr = el.getBoundingClientRect();
    // The first copy the leader can reach: a copy behind the box itself has no line to it (in a small window the
    // street lamps' first copy sat under the box, and the step had no leader at all, though seven others showed).
    let path: string | null = null;
    for (const id of first.s.ids) {
      const to = viewer.screenPoint([id]);
      if (to?.visible && (path = route(to, sr, hr))) break;
    }
    setLeader(path);
  }, [tick, layout, viewer, step]);
  return (
    <>
      <div className="step-view" data-c="parts-box" data-under-bar={underBar || undefined} aria-label={`Step ${step.n}`} id="step-view" ref={ref}>
        <span className="step-number">{step.n}</span>
        {step.items.length > 0 && (
          <div className="parts-box" id="parts-box">
            {step.items.map((i) => (
              <span className="parts-box-item" key={`${i.part}|${i.color}`} data-part={i.part} data-color={i.color}>
                <Picture load={() => partPicture(i.part, i.color)} alt="" />
                {i.count}x
              </span>
            ))}
          </div>
        )}
        {subs.map(({ s, model }) => (
          <SubAssembly key={s.file} scene={scene} file={s.file} copy={s.ids[0] ?? ""} name={model.name} steps={model.steps} count={s.count} />
        ))}
      </div>
      {leader && (
        <svg className="leader" aria-hidden="true" id="step-leader">
          <path d={leader} />
        </svg>
      )}
    </>
  );
}

/** The leader from a sub-assembly's box (`box`, in the page's coordinates like `host`) to a copy at `to`, in the
 * host's coordinates: down from the box's foot, then across to the copy; or, for a copy no lower than the box, across
 * from the box's right edge, then up or down to it. The arrowhead points the way the last run goes. Null when neither
 * route reaches it (the copy is beside or behind the box, to its left). */
function route(to: { x: number; y: number }, box: DOMRect, host: DOMRect): string | null {
  const below = to.y > box.bottom - host.top;
  const x0 = below ? box.left - host.left + 24 : box.right - host.left;
  const y0 = below ? box.bottom - host.top : (box.top + box.bottom) / 2 - host.top;
  if (below) {
    const d = to.x >= x0 ? 1 : -1;
    return `M${x0} ${y0}V${to.y}H${to.x - 4 * d}M${to.x - 12 * d} ${to.y - 5}L${to.x - 3 * d} ${to.y}L${to.x - 12 * d} ${to.y + 5}`;
  }
  if (to.x <= x0) return null;
  const d = to.y >= y0 ? 1 : -1;
  return `M${x0} ${y0}H${to.x}V${to.y - 4 * d}M${to.x - 5} ${to.y - 12 * d}L${to.x} ${to.y - 3 * d}L${to.x + 5} ${to.y - 12 * d}`;
}

function SubAssembly({ scene, file, copy, name, steps, count }: { scene: Scene; file: string; copy: string; name: string; steps: StepView[]; count: number }) {
  // One copy's parts, each with its step inside the copy: its own, or for a part of a copy nested in this one, the
  // step that places that copy.
  const parts = scene.placements.flatMap((p) => {
    const depth = p.copies?.findIndex((c) => c.id === copy) ?? -1;
    return depth < 0 ? [] : [{ ...p, step: p.copies![depth + 1]?.step ?? p.step }];
  });
  const items = new Map<string, { part: string; color: number; count: number }>();
  for (const p of parts) {
    const k = `${p.part}|${p.color}`;
    const i = items.get(k);
    if (i) i.count++;
    else items.set(k, { part: p.part, color: p.color, count: 1 });
  }
  return (
    <div className="sub-assembly" aria-label={`Sub-assembly: ${name}${count > 1 ? `, ${count} times` : ""}`} data-file={file}>
      <div className="parts-box">
        {[...items.values()].map((i) => (
          <span className="parts-box-item" key={`${i.part}|${i.color}`}>
            <Picture load={() => partPicture(i.part, i.color)} alt="" />
            {i.count}x
          </span>
        ))}
      </div>
      <ol className="sub-steps">
        {steps.map((s) => (
          <li key={s.n}>
            <span className="sub-step-n">{s.n}</span>
            <Picture load={() => assemblyPicture(`${scene.generation}:${copy}:${s.n}`, parts.filter((p) => p.step <= s.n))} alt={`Sub-step ${s.n}`} />
          </li>
        ))}
      </ol>
    </div>
  );
}
