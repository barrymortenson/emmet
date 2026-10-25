// SPDX-License-Identifier: AGPL-3.0-or-later
// The params drawer: a control for each parameter in the model's schema. A change writes params.json through the host
// (checked against the schema), and the watcher runs the model; the viewer updates in place.
import { useEffect, useRef, useState } from "react";
import { type ColorView, commands, type ParamSpec, type ParamsView, type Problem, type Value } from "./bindings";

/** How long a slider waits after the last change before writing params.json. */
const SETTLE_MS = 60;

export function Params({ lastGeneration }: { running: boolean; lastGeneration: number }) {
  const [params, setParams] = useState<ParamsView | null>(null);
  const [values, setValues] = useState<Record<string, Value>>({});
  const [colors, setColors] = useState<ColorView[]>([]);
  const [problem, setProblem] = useState<Problem | null>(null);
  const pending = useRef<Record<string, Value>>({});
  const timer = useRef<number | undefined>(undefined);
  const writes = useRef(0);

  // The schema can change with model.py, so it's read again after each build, unless a write is on its way.
  useEffect(() => {
    if (writes.current > 0 || Object.keys(pending.current).length) return;
    let live = true;
    void commands.getParams().then((r) => {
      if (!live || writes.current > 0) return;
      if (r.status === "ok") {
        setParams(r.data);
        setValues(r.data.values as Record<string, Value>);
        setProblem(null);
      } else setProblem(r.error);
    });
    return () => {
      live = false;
    };
  }, [lastGeneration]);

  const [colorsProblem, setColorsProblem] = useState<Problem | null>(null);
  useEffect(() => {
    void commands.listColors().then((r) => {
      if (r.status === "ok") setColors(r.data);
      else setColorsProblem(r.error);
    });
  }, []);

  const change = (name: string, value: Value) => {
    setValues((v) => ({ ...v, [name]: value }));
    pending.current[name] = value;
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(async () => {
      const batch = pending.current;
      pending.current = {};
      writes.current++;
      const r = await commands.setParams(batch);
      writes.current--;
      setProblem(r.status === "ok" ? null : r.error);
      // Show what params.json holds now, unless another change is already on its way: a rejected value goes back.
      if (writes.current === 0 && !Object.keys(pending.current).length) {
        const now = r.status === "ok" ? r : await commands.getParams();
        if (now.status === "ok" && writes.current === 0 && !Object.keys(pending.current).length) {
          setValues(now.data.values as Record<string, Value>);
        }
      }
    }, SETTLE_MS);
  };

  if (!params) {
    return (
      <section className="drawer" aria-label="Parameters">
        <h2>Parameters</h2>
        <p className="muted">{problem ? `Parameters unavailable: ${problem.message}` : "Reading the model's parameters…"}</p>
      </section>
    );
  }
  return (
    <section className="drawer" aria-label="Parameters">
      <h2>Parameters</h2>
      {problem && (
        <p className="notice" role="alert">
          {problem.message} {problem.hint}
        </p>
      )}
      {colorsProblem && (
        <p className="notice" role="alert">
          The color list isn't available: {colorsProblem.message} {colorsProblem.hint}
        </p>
      )}
      {params.params.length === 0 && <p className="muted">This model has no parameters.</p>}
      <div className="params">
        {params.params.map((p) => (
          <Control key={p.name} spec={p} value={values[p.name] ?? p.default} colors={colors} onChange={change} />
        ))}
      </div>
    </section>
  );
}

function label(name: string): string {
  const s = name.replaceAll("_", " ");
  return s[0]!.toUpperCase() + s.slice(1);
}

function Control({
  spec,
  value,
  colors,
  onChange,
}: {
  spec: ParamSpec;
  value: Value;
  colors: ColorView[];
  onChange: (name: string, value: Value) => void;
}) {
  const id = `param-${spec.name}`;
  const help = spec.help ? <span className="help">{spec.help}</span> : null;
  if (spec.type === "bool") {
    return (
      <div className="param">
        <label htmlFor={id}>
          <input id={id} type="checkbox" checked={value === true} onChange={(e) => onChange(spec.name, e.target.checked)} />{" "}
          {label(spec.name)}
        </label>
        {help}
      </div>
    );
  }
  if (spec.type === "choice" || spec.type === "color") {
    const options = spec.type === "choice" ? (spec.choices ?? []).map(String) : colors.map((c) => c.name);
    const current = String(value);
    if (!options.includes(current)) options.unshift(current);
    return (
      <div className="param">
        <label htmlFor={id}>{label(spec.name)}</label>
        <select id={id} value={current} onChange={(e) => onChange(spec.name, e.target.value)}>
          {options.map((o) => (
            <option key={o}>{o}</option>
          ))}
        </select>
        {help}
      </div>
    );
  }
  const number = Number(value);
  const int = spec.type === "int";
  const parse = (s: string) => (int ? Math.round(Number(s)) : Number(s));
  if (spec.min != null && spec.max != null) {
    return (
      <div className="param">
        <label htmlFor={id}>
          {label(spec.name)} <output htmlFor={id}>{number}</output>
        </label>
        <input
          id={id}
          type="range"
          min={spec.min}
          max={spec.max}
          step={spec.step ?? (int ? 1 : (spec.max - spec.min) / 100)}
          value={number}
          onChange={(e) => onChange(spec.name, parse(e.target.value))}
        />
        {help}
      </div>
    );
  }
  return (
    <div className="param">
      <label htmlFor={id}>{label(spec.name)}</label>
      <input
        id={id}
        type="number"
        step={spec.step ?? (int ? 1 : "any")}
        value={number}
        onChange={(e) => e.target.value !== "" && onChange(spec.name, parse(e.target.value))}
      />
      {help}
    </div>
  );
}
