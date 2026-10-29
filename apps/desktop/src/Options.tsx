// SPDX-License-Identifier: AGPL-3.0-or-later
// The Options tab (plan section 14, docs/design/frozen): the build settings, which steer the agent (each Any until
// set; a change waits in a bar with Undo and Rebuild, and Rebuild asks the agent to rework the model), then the model's
// knobs (Tweaks: the two to five parameters the model names with @model(knobs=...)), which change it at once: a moved knob is a draft (params.json) until Keep writes it into model.py as
// its default, and Reset drops it. ⌘Z and ⇧⌘Z undo and redo a draft.
import { Popover, Slider, Switch, ToggleGroup } from "radix-ui";
import { useEffect, useRef, useState } from "react";
import { type ColorView, commands, type ParamSpec, type ParamsView, type Problem, type Value } from "./bindings";
import type { BuildSettings } from "./bindings";
import { Badge, Button, Icon, Tip } from "./ui";

// ---- Build settings ------------------------------------------------------------------------------------------------

type Key = keyof BuildSettings;
type Choice = { id: string; name?: string; sub?: string };

/** The settings, their meanings, and their choices, as the frozen design names them (docs/design/shared/data.js). */
const BRIEF: Record<Key, { label: string; help: string; options: Choice[] }> = {
  base: { label: "Base", help: "What it stands on", options: ["None", "16 × 16", "32 × 32", "48 × 48"].map((id) => ({ id })) },
  scale: {
    label: "Scale",
    help: "What the details are sized for; the whole can be shortened to stay buildable",
    options: [
      { id: "Micro", sub: "1:100 and smaller, like the Titanic at 1:200" },
      { id: "Micropolis", sub: "The community micro city standard: 16 × 16 modules, two-stud roads, a brick about 9 feet tall" },
      { id: "Midi", sub: "Between micro and figure scale, like the Endurance" },
      { id: "Figure scale", sub: "Doors, seats, and rooms fit a 4-brick-tall figure; the whole is shortened, like most official sets" },
      { id: "Large", sub: "Bigger than figure scale, like the large display cars" },
      { id: "Car 1:8", sub: "A large car with gears, panels, and working steering" },
    ],
  },
  age: {
    label: "Age",
    help: "Who builds it; sets the usual detail and pieces for that age",
    options: [
      { id: "4+", sub: "Big pieces, few steps" },
      { id: "6+", sub: "More pieces, simple techniques" },
      { id: "9+", sub: "More complex techniques and steps" },
      { id: "13+", sub: "Advanced techniques, fine detail" },
      { id: "18+", sub: "Display models for experienced builders" },
    ],
  },
  detail: {
    label: "Detail",
    help: "How much the agent adds beyond the shape",
    options: [
      { id: "Low", sub: "Bricks, plates, and slopes: quick to build" },
      { id: "Medium", sub: "Plus tiles, trim, and window frames" },
      { id: "High", sub: "Plus textures, sideways building, and small details" },
    ],
  },
  pieces: {
    label: "Pieces",
    help: "How many pieces the model has",
    options: [
      { id: "Small", name: "50–250" },
      { id: "Medium", name: "250–1,000" },
      { id: "Large", name: "1,000–2,500" },
      { id: "Huge", name: "2,500+" },
    ],
  },
  style: {
    label: "Style",
    help: "The look the agent builds in",
    options: [
      { id: "Town", sub: "Houses, shops, streets" },
      { id: "Medieval", sub: "Stone, timber, castles" },
      { id: "Blocky", sub: "Voxel art, one-stud cubes" },
      { id: "Dinosaur park", sub: "Jungle, fences, big animals" },
      { id: "Space", sub: "Bases, rovers, gray and blue" },
      { id: "Pirate", sub: "Ships, docks, islands" },
      { id: "Modern", sub: "Glass, clean lines" },
      { id: "Fantasy", sub: "Towers, magic, color" },
    ],
  },
};

/** Picking an age sets Detail and Pieces to what sets for that age usually have (Barry 2026-10-27). */
const AGE_PRESET: Record<string, { detail: string; pieces: string }> = {
  "4+": { detail: "Low", pieces: "Small" },
  "6+": { detail: "Low", pieces: "Small" },
  "9+": { detail: "Medium", pieces: "Medium" },
  "13+": { detail: "High", pieces: "Large" },
  "18+": { detail: "High", pieces: "Huge" },
};

const ORDER: Key[] = ["base", "scale", "age", "detail", "pieces", "style"];
const ANY = "Any";

const said = (v: string | null | undefined) => v ?? ANY;
const words = (k: Key, v: string | null | undefined) => (v == null ? null : k === "pieces" ? `${BRIEF.pieces.options.find((o) => o.id === v)?.name ?? v} pieces` : k === "age" ? `Ages ${v}` : k === "detail" ? `${v} detail` : v);

/** The message Rebuild sends to the agent: what changed, in words. */
export function rebuildRequest(before: BuildSettings, after: BuildSettings): string {
  const changes = ORDER.filter((k) => (before[k] ?? null) !== (after[k] ?? null)).map((k) => `${BRIEF[k].label}: ${said(after[k])}`);
  const all = ORDER.map((k) => `${BRIEF[k].label} ${said(after[k])}`).join(", ");
  return `Rework the model for the new build settings (${changes.join("; ")}). All of them now: ${all}. They're in emmet.toml's [build].`;
}

/** One build setting or knob: its name (its meaning is the name's tooltip), "was …" when changed, then its control. */
function Setting({ name, help, was, moved, inline, children, id }: { name: string; help: string; was?: string; moved?: boolean; inline?: boolean; children: React.ReactNode; id?: string }) {
  return (
    <div className="setting" data-c="setting" data-moved={moved ? "true" : undefined} data-inline={inline || undefined} id={id}>
      <div className="setting-head">
        <Tip text={help}>
          <span className="setting-name" tabIndex={0}>
            {name}
          </span>
        </Tip>
        {was !== undefined && <Badge text={`was ${was}`} tone="accent" />}
      </div>
      {children}
    </div>
  );
}

function Segmented({ label, items, value, onChange }: { label: string; items: Choice[]; value: string; onChange: (v: string) => void }) {
  return (
    <ToggleGroup.Root type="single" className="segmented" data-c="segmented" aria-label={label} value={value} onValueChange={(v) => v && onChange(v)}>
      {items.map((it) => (
        <ToggleGroup.Item key={it.id} value={it.id} className="segmented-item" title={it.sub}>
          {it.name ?? it.id}
        </ToggleGroup.Item>
      ))}
    </ToggleGroup.Root>
  );
}

function Chips({ label, items, value, onChange, own, onRemove }: { label: string; items: Choice[]; value: string | null; onChange: (v: string) => void; own?: string; onRemove?: () => void }) {
  return (
    <ToggleGroup.Root type="single" className="chips" data-c="chips" aria-label={label} value={value ?? ""} onValueChange={(v) => v && onChange(v)}>
      {items.map((it) => (
        <ToggleGroup.Item key={it.id} value={it.id} className="chip" title={it.sub}>
          {it.name ?? it.id}
        </ToggleGroup.Item>
      ))}
      {own && (
        <ToggleGroup.Item value={own} className="chip" title="Your own style">
          {own}
          <span className="chip-remove" role="button" aria-label={`Remove ${own}`} onClick={(e) => (e.stopPropagation(), onRemove?.())}>
            <Icon name="x" size="sm" />
          </span>
        </ToggleGroup.Item>
      )}
    </ToggleGroup.Root>
  );
}

export function BuildSettingsSection({
  saved,
  fresh,
  disabled,
  onSave,
  onRebuild,
}: {
  saved: BuildSettings;
  /** An empty model: every setting open on Any. */
  fresh: boolean;
  disabled: boolean;
  onSave: (s: BuildSettings) => Promise<boolean>;
  onRebuild: (before: BuildSettings, after: BuildSettings) => void;
}) {
  const [draft, setDraft] = useState<BuildSettings>(saved);
  const [open, setOpen] = useState(false);
  const [style, setStyle] = useState("");
  useEffect(() => setDraft(saved), [saved]);
  const changed = ORDER.filter((k) => (draft[k] ?? null) !== (saved[k] ?? null));
  const full = fresh || open || changed.length > 0;
  const set = (k: Key, v: string | null) =>
    setDraft((d) => {
      const next = { ...d, [k]: v === ANY ? null : v };
      if (k === "age" && v && AGE_PRESET[v]) Object.assign(next, AGE_PRESET[v]);
      return next;
    });
  const moved = (k: Key) => (changed.includes(k) ? { moved: true, was: said(saved[k]) } : {});
  const ownStyle = draft.style && !BRIEF.style.options.some((o) => o.id === draft.style) ? draft.style : undefined;
  const bar = changed.length > 0 && (
    <div className="pending-bar" id="settings-bar">
      <span>{changed.length} changed</span>
      <Button label="Undo" variant="quiet" size="sm" onClick={() => setDraft(saved)} id="settings-undo" />
      <Button
        label="Rebuild"
        variant="primary"
        size="sm"
        icon="refresh-cw"
        id="settings-rebuild"
        disabled={disabled}
        onClick={async () => {
          if (await onSave(draft)) onRebuild(saved, draft);
        }}
      />
    </div>
  );
  return (
    <section className="panel-section" aria-labelledby="build-settings-title">
      <div className="panel-section-head">
        <Tip text="Together they set how the agent designs the model">
          <span id="build-settings-title" tabIndex={0}>
            Build settings
          </span>
        </Tip>
        {open && !fresh && !changed.length && <Button label="Done" variant="quiet" size="sm" onClick={() => setOpen(false)} />}
      </div>
      {bar}
      {!full ? (
        <div className="summary-row" id="settings-summary">
          <span>{ORDER.map((k) => words(k, saved[k])).filter(Boolean).join(" · ") || ANY}</span>
          <Button label="Change" size="sm" onClick={() => setOpen(true)} disabled={disabled} id="settings-change" />
        </div>
      ) : (
        <div className="settings-list" id="settings-list">
          <Setting name="Base" help={BRIEF.base.help} {...moved("base")}>
            <Segmented label="Base" items={[{ id: ANY }, ...BRIEF.base.options]} value={said(draft.base)} onChange={(v) => set("base", v)} />
          </Setting>
          <Setting name="Scale" help={BRIEF.scale.help} {...moved("scale")}>
            <Chips label="Scale" items={[{ id: ANY, sub: "The prompt decides" }, ...BRIEF.scale.options]} value={said(draft.scale)} onChange={(v) => set("scale", v)} />
          </Setting>
          <Setting name="Age" help={BRIEF.age.help} {...moved("age")}>
            <Segmented label="Age" items={[{ id: ANY }, ...BRIEF.age.options]} value={said(draft.age)} onChange={(v) => set("age", v)} />
          </Setting>
          <Setting name="Detail" help={BRIEF.detail.help} {...moved("detail")}>
            <Segmented label="Detail" items={[{ id: ANY }, ...BRIEF.detail.options]} value={said(draft.detail)} onChange={(v) => set("detail", v)} />
          </Setting>
          <Setting name="Pieces" help={BRIEF.pieces.help} {...moved("pieces")}>
            <Chips label="Pieces" items={[{ id: ANY }, ...BRIEF.pieces.options]} value={said(draft.pieces)} onChange={(v) => set("pieces", v)} />
          </Setting>
          <Setting name="Style" help={BRIEF.style.help} {...moved("style")}>
            <Chips label="Style" items={BRIEF.style.options} value={draft.style ?? null} onChange={(v) => set("style", v)} own={ownStyle} onRemove={() => set("style", null)} />
            <input
              className="text-field"
              data-c="text-field"
              type="text"
              placeholder="Your own, like steampunk or zen garden"
              aria-label="Your own style"
              value={style}
              maxLength={60}
              onChange={(e) => setStyle(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && style.trim()) {
                  set("style", style.trim());
                  setStyle("");
                }
              }}
            />
          </Setting>
        </div>
      )}
    </section>
  );
}

// ---- Knobs ---------------------------------------------------------------------------------------------------------

/** How long a knob waits after the last change before writing params.json. */
const SETTLE_MS = 60;

const label = (name: string) => {
  const s = name.replaceAll("_", " ");
  return s[0]!.toUpperCase() + s.slice(1);
};
const help = (p: ParamSpec) => (p.help ? p.help[0]!.toUpperCase() + p.help.slice(1) : label(p.name));
/** A value with its unit: "20%" for a percent knob (its help says "in percent"). */
const withUnit = (p: ParamSpec, v: Value) => (/\bpercent\b/.test(p.help) ? `${String(v)}%` : String(v));
const same = (a: Value | undefined, b: Value | undefined) => JSON.stringify(a) === JSON.stringify(b);
/** "seed": another arrangement of the same design, which Shuffle picks. */
const isSeed = (p: ParamSpec) => p.type === "int" && p.name === "seed";

/** The knobs. `onWrite` is called as a change goes to params.json, so the next run shows without the build animation. */
export function Knobs({ lastGeneration, onWrite, problemOut }: { lastGeneration: number; onWrite?: () => void; problemOut?: (p: Problem | null) => void }) {
  const [params, setParams] = useState<ParamsView | null>(null);
  const [values, setValues] = useState<Record<string, Value>>({});
  const [colors, setColors] = useState<ColorView[]>([]);
  const [problem, setProblem] = useState<Problem | null>(null);
  const pending = useRef<Record<string, Value>>({});
  const timer = useRef<number | undefined>(undefined);
  const writes = useRef(0);
  const undo = useRef<{ name: string; from: Value; to: Value }[]>([]);
  const redo = useRef<{ name: string; from: Value; to: Value }[]>([]);
  useEffect(() => problemOut?.(problem), [problem, problemOut]);

  const reload = async () => {
    const r = await commands.getParams();
    if (r.status === "ok") {
      setParams(r.data);
      setValues(r.data.values as Record<string, Value>);
      setProblem(null);
    } else setProblem(r.error);
  };
  // The schema can change with model.py, so it's read again after each build, unless a write is on its way.
  useEffect(() => {
    if (writes.current > 0 || Object.keys(pending.current).length) return;
    void reload();
  }, [lastGeneration]);
  useEffect(() => {
    // A color knob without its own choices offers these; with none, it says why.
    void commands.listColors().then((r) => (r.status === "ok" ? setColors(r.data) : setProblem(r.error)));
  }, []);

  const write = (name: string, value: Value) => {
    setValues((v) => ({ ...v, [name]: value }));
    pending.current[name] = value;
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(async () => {
      const batch = pending.current;
      pending.current = {};
      writes.current++;
      onWrite?.();
      const r = await commands.setParams(batch);
      writes.current--;
      setProblem(r.status === "ok" ? null : r.error);
      if (writes.current === 0 && !Object.keys(pending.current).length) {
        const now = r.status === "ok" ? r : await commands.getParams();
        if (now.status === "ok" && writes.current === 0 && !Object.keys(pending.current).length) setValues(now.data.values as Record<string, Value>);
      }
    }, SETTLE_MS);
  };
  const change = (name: string, value: Value) => {
    undo.current.push({ name, from: values[name]!, to: value });
    redo.current = [];
    write(name, value);
  };

  // ⌘Z and ⇧⌘Z (App sends them): undo and redo a draft, one change at a time.
  useEffect(() => {
    const step = (from: typeof undo, to: typeof redo, back: boolean) => () => {
      const last = from.current.pop();
      if (!last) return;
      to.current.push(last);
      write(last.name, back ? last.from : last.to);
    };
    const u = step(undo, redo, true);
    const r = step(redo, undo, false);
    window.addEventListener("emmet:undo-knob", u);
    window.addEventListener("emmet:redo-knob", r);
    return () => {
      window.removeEventListener("emmet:undo-knob", u);
      window.removeEventListener("emmet:redo-knob", r);
    };
  });

  if (!params) return <p className="muted">{problem ? `The knobs aren't available: ${problem.message}` : "Reading the model's knobs…"}</p>;
  // The panel shows the model's knobs only (@model(knobs=...)): its other parameters are for the CLI and the agent.
  const knobs = params.params.filter((p) => p.knob);
  const drafts = knobs.filter((p) => !same(values[p.name], p.default)).map((p) => p.name);
  const finish = async (call: (names: string[]) => ReturnType<typeof commands.keepParams>) => {
    window.clearTimeout(timer.current);
    pending.current = {};
    undo.current = [];
    redo.current = [];
    writes.current++;
    onWrite?.();
    const r = await call(drafts);
    writes.current--;
    if (r.status === "ok") {
      setParams(r.data);
      setValues(r.data.values as Record<string, Value>);
      setProblem(null);
    } else setProblem(r.error);
  };
  return (
    <>
      {drafts.length > 0 && (
        <div className="pending-bar" id="knobs-bar">
          <span>
            {drafts.length} change{drafts.length > 1 ? "s" : ""}
          </span>
          <Button label="Reset" variant="quiet" size="sm" id="knobs-reset" onClick={() => void finish(commands.resetParams)} />
          <Button label="Keep" variant="primary" size="sm" id="knobs-keep" onClick={() => void finish(commands.keepParams)} />
        </div>
      )}
      {problem && (
        <p className="notice" role="alert">
          {problem.message} {problem.hint}
        </p>
      )}
      {knobs.length === 0 && <p className="muted">This model has no knobs yet.</p>}
      {knobs.map((p) => (
        <Knob key={p.name} spec={p} value={values[p.name] ?? p.default} colors={colors} onChange={change} />
      ))}
    </>
  );
}

function Knob({ spec, value, colors, onChange }: { spec: ParamSpec; value: Value; colors: ColorView[]; onChange: (name: string, v: Value) => void }) {
  const id = `param-${spec.name}`;
  const moved = !same(value, spec.default);
  const props = { name: label(spec.name), help: help(spec), moved, id };
  if (isSeed(spec)) {
    const [min, max] = [spec.min ?? 1, spec.max ?? 99];
    const shuffle = () => {
      let next = Number(value);
      while (next === Number(value) && max > min) next = min + Math.floor(Math.random() * (max - min + 1));
      onChange(spec.name, next);
    };
    return (
      <Setting {...props} name="Arrangement" inline>
        <Button label="Shuffle" size="sm" icon="shuffle" onClick={shuffle} id={`shuffle-${spec.name}`} />
      </Setting>
    );
  }
  if (spec.type === "bool") {
    const on = value === true;
    return (
      <Setting {...props} inline>
        <Switch.Root className="switch" data-c="switch" aria-label={label(spec.name)} checked={on} onCheckedChange={(c) => onChange(spec.name, c)} id={`${id}-switch`}>
          <Switch.Thumb className="switch-thumb" />
        </Switch.Root>
      </Setting>
    );
  }
  if (spec.type === "color") return <ColorKnob {...props} spec={spec} value={String(value)} colors={colors} onChange={onChange} />;
  if (spec.type === "choice") {
    const items = (spec.choices ?? []).map((c) => ({ id: String(c) }));
    return (
      <Setting {...props} was={moved ? String(spec.default) : undefined}>
        <Segmented label={label(spec.name)} items={items} value={String(value)} onChange={(v) => onChange(spec.name, spec.choices?.find((c) => String(c) === v) ?? v)} />
      </Setting>
    );
  }
  const n = Number(value);
  const int = spec.type === "int";
  const [min, max] = [spec.min ?? 0, spec.max ?? Math.max(n * 2, 10)];
  const step = spec.step ?? (int ? 1 : (max - min) / 100);
  return (
    <Setting {...props} was={moved ? withUnit(spec, spec.default) : undefined}>
      <div className="slider" data-c="slider">
        <Slider.Root className="slider-root" min={min} max={max} step={step} value={[n]} onValueChange={([v]) => v != null && onChange(spec.name, int ? Math.round(v) : v)}>
          <Slider.Track className="slider-track">
            <Slider.Range className="slider-range" />
          </Slider.Track>
          <Slider.Thumb className="slider-thumb" aria-label={label(spec.name)} aria-valuetext={withUnit(spec, n)} id={`${id}-thumb`} />
        </Slider.Root>
        <output htmlFor={`${id}-thumb`}>{withUnit(spec, n)}</output>
      </div>
    </Setting>
  );
}

function ColorKnob({ spec, value, colors, onChange, ...props }: { name: string; help: string; moved: boolean; id: string; spec: ParamSpec; value: string; colors: ColorView[]; onChange: (name: string, v: Value) => void }) {
  const choices = spec.choices?.map(String) ?? colors.map((c) => c.name);
  const colorOf = (name: string) => colors.find((c) => c.name === name)?.rgb;
  return (
    <Setting {...props} inline was={props.moved ? String(spec.default) : undefined}>
      <Popover.Root>
        <span className="color-picker" data-c="color-picker">
          <Popover.Trigger className="select-trigger" id={`${props.id}-color`}>
            <i className="swatch" aria-hidden="true" style={{ background: colorOf(value) }} />
            <span>{value}</span>
            <Icon name="chevron-down" size="sm" />
          </Popover.Trigger>
        </span>
        <Popover.Portal>
          <Popover.Content className="popover color-popover" data-c="popover" aria-label={props.name} align="end" sideOffset={6}>
            <span className="popover-title">{props.name}</span>
            <div className="swatch-grid" role="radiogroup" aria-label={props.name}>
              {choices.map((c) => (
                <Popover.Close asChild key={c}>
                  <button type="button" className="swatch-item" role="radio" aria-checked={c === value} data-state={c === value ? "checked" : "unchecked"} title={c} onClick={() => onChange(spec.name, c)}>
                    <i className="swatch" aria-hidden="true" style={{ background: colorOf(c) }} />
                  </button>
                </Popover.Close>
              ))}
            </div>
            <p className="small">{spec.choices ? "Only colors this part is made in." : "Every LDraw color."}</p>
          </Popover.Content>
        </Popover.Portal>
      </Popover.Root>
    </Setting>
  );
}
