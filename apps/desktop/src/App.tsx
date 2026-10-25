// SPDX-License-Identifier: AGPL-3.0-or-later
// The functional UI (plan section 14, before Phase 2 designs it): a project tree, the viewer with a params drawer,
// a run panel with the error and its traceback, and a status line. The last good model stays on screen while a run
// is going or after one fails.
import { createViewer, type View, type Viewer } from "@emmet/viewer";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { type AppInfo, commands, events, type Problem, type Scene, type UiState } from "./bindings";
import { base64, json, library, placements } from "./library";
import { Params } from "./Params";

type Theme = "light" | "dark";
const VIEWS: View[] = ["iso", "front", "back", "left", "right", "top"];

/** The system appearance, unless a test overrides it with data-theme. */
function useTheme(): [Theme, (t: Theme | null) => void] {
  const query = useMemo(() => window.matchMedia("(prefers-color-scheme: dark)"), []);
  const [system, setSystem] = useState<Theme>(query.matches ? "dark" : "light");
  const [forced, setForced] = useState<Theme | null>(null);
  useEffect(() => {
    const on = () => setSystem(query.matches ? "dark" : "light");
    query.addEventListener("change", on);
    return () => query.removeEventListener("change", on);
  }, [query]);
  return [forced ?? system, setForced];
}

export function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [ui, setUi] = useState<UiState>({ project: null, running: false, scene: null, error: null });
  const [problem, setProblem] = useState<Problem | null>(null);
  const [viewerError, setViewerError] = useState<string | null>(null);
  const [theme, forceTheme] = useTheme();
  const viewer = useRef<Viewer | null>(null);
  const shown = useRef(-1);
  const framed = useRef<string | null>(null);
  const root = useRef<string | null>(null);
  const canvas = useRef<HTMLCanvasElement | null>(null);
  const [selected, setSelected] = useState<string | null>(null);

  // The viewer lives as long as the window.
  useEffect(() => {
    if (!canvas.current) return;
    const v = createViewer(canvas.current, { partSource: library, theme });
    viewer.current = v;
    if (__EMMET_TEST_HOOKS__) {
      (window as unknown as { __emmet: unknown }).__emmet = {
        viewer: () => viewer.current,
        shown: () => shown.current,
        project: () => root.current,
        theme: (t: Theme | null) => forceTheme(t),
      };
    }
    return () => {
      v.dispose();
      viewer.current = null;
    };
    // Created once; the theme effect below keeps it current.
  }, []);

  useEffect(() => {
    viewer.current?.setTheme(theme);
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  /** Shows a scene if it's newer than the one on screen (the generation check), framing a project's first one. */
  const show = useCallback(async (scene: Scene | null, root: string | null) => {
    const v = viewer.current;
    if (!v || !scene || scene.generation < shown.current) return;
    shown.current = scene.generation;
    try {
      await v.update(placements(scene), { animate: "none" });
      const failed = Object.entries(v.stats().failed);
      setViewerError(failed.length ? `Some parts can't be shown: ${failed.map(([id, why]) => `${id} (${why})`).join("; ")}` : null);
    } catch (e) {
      setViewerError(`The viewer couldn't show the model: ${e instanceof Error ? e.message : String(e)}`);
      return;
    }
    if (root && framed.current !== root) {
      framed.current = root;
      v.setView("iso");
    }
  }, []);

  useEffect(() => {
    const off: Promise<() => void>[] = [
      events.projectOpened.listen(({ payload }) => {
        shown.current = -1;
        framed.current = null;
        root.current = payload.project.root;
        setSelected(null);
        viewer.current?.highlight([]);
        setUi({ project: payload.project, running: false, scene: payload.scene ?? null, error: null });
        void show(payload.scene ?? null, payload.project.root);
        if (!payload.scene) void viewer.current?.update([]);
      }),
      events.runStarted.listen(() => setUi((u) => ({ ...u, running: true }))),
      events.runFinished.listen(({ payload }) => {
        if (payload.scene) void show(payload.scene, root.current);
        setUi((u) => ({
          ...u,
          running: false,
          scene: payload.scene ?? u.scene,
          error: payload.scene ? null : (payload.error ?? null),
        }));
      }),
      events.busHighlight.listen(({ payload }) => viewer.current?.highlight(payload.ids)),
      events.busRender.listen(async ({ payload }) => {
        const v = viewer.current;
        try {
          if (!v) throw new Error("the viewer isn't ready");
          if (payload.scene && payload.scene.generation > shown.current) {
            const scene = payload.scene;
            setUi((u) => ({ ...u, scene, error: null }));
            await show(scene, null);
          }
          const { png, camera } = await v.render({
            view: payload.view as View,
            size: payload.size,
            highlight: payload.highlight.length ? payload.highlight : undefined,
          });
          await commands.busReply(payload.id, base64(png), json(camera), null);
        } catch (e) {
          await commands.busReply(payload.id, null, null, e instanceof Error ? e.message : String(e));
        }
      }),
    ];
    void commands.appInfo().then(setInfo);
    void commands.uiState().then((s) => {
      root.current = s.project?.root ?? null;
      setUi(s);
      void show(s.scene ?? null, root.current);
    });
    return () => off.forEach((p) => void p.then((f) => f()));
  }, [show]);

  const act = async (f: () => Promise<{ status: "ok" } | { status: "error"; error: Problem }>) => {
    const r = await f();
    setProblem(r.status === "error" ? r.error : null);
    void commands.appInfo().then(setInfo);
  };

  const highlight = (key: string, ids: string[]) => {
    const next = selected === key ? null : key;
    setSelected(next);
    viewer.current?.highlight(next ? ids : []);
  };

  const scene = ui.scene;
  return (
    <div className="app">
      <header className="titlebar">
        <h1>{ui.project?.name ?? "Emmet"}</h1>
        <div className="actions">
          <button type="button" onClick={() => act(commands.openDialog)}>
            Open…
          </button>
          <button type="button" onClick={() => act(commands.newProjectDialog)}>
            New…
          </button>
          {ui.project && (
            <button type="button" onClick={() => act(commands.rebuild)}>
              Rebuild
            </button>
          )}
        </div>
      </header>
      {problem && (
        <p className="notice" role="alert">
          {problem.message} {problem.hint}
        </p>
      )}
      {viewerError && (
        <p className="notice" role="alert" id="viewer-error">
          {viewerError}
        </p>
      )}
      <div className="panes">
        <nav className="tree" aria-label="Project">
          {ui.project ? (
            <Tree scene={scene ?? null} selected={selected} onPick={highlight} />
          ) : (
            <Recents info={info} onOpen={(path) => act(() => commands.openRecent(path))} />
          )}
        </nav>
        <main className="center">
          <div className="viewer">
            <canvas ref={canvas} aria-label="3D view of the model; the project tree lists its parts" />
            {!ui.project && <p className="empty">Open a project folder, or make a new one, to see its model here.</p>}
            {ui.project && (
              <div className="toolbar" role="toolbar" aria-label="View">
                {VIEWS.map((v) => (
                  <button key={v} type="button" onClick={() => viewer.current?.setView(v)}>
                    {v[0]!.toUpperCase() + v.slice(1)}
                  </button>
                ))}
                <button type="button" onClick={() => viewer.current?.frame()}>
                  Frame
                </button>
              </div>
            )}
          </div>
          {ui.project && <Params key={ui.project.root} running={ui.running} lastGeneration={scene?.generation ?? -1} />}
        </main>
        <aside className="side" aria-label="Run">
          <RunPanel ui={ui} />
        </aside>
      </div>
      <footer className="status">
        <span role="status" id="status">
          {statusText(ui)}
        </span>
        <span className="versions">
          {info && `Emmet ${info.version} · LDraw library ${info.library ?? "not installed"}`}
        </span>
      </footer>
    </div>
  );
}

function where(p: Problem): string {
  return p.file ? ` at ${p.file}${p.line ? `:${p.line}` : ""}` : "";
}

function statusText(ui: UiState): string {
  if (!ui.project) return "No project open";
  if (ui.running) return ui.scene ? "Out of date: rebuilding…" : "Building…";
  if (ui.error) return `Build failed: ${ui.error.code}${where(ui.error)}${ui.scene ? "; showing the last good model" : ""}`;
  const s = ui.scene;
  if (!s) return "Not built yet";
  const time = s.ms != null ? ` in ${(s.ms / 1000).toFixed(2)} s` : "";
  return `Built ${s.parts} parts in ${s.steps} steps${time}`;
}

function RunPanel({ ui }: { ui: UiState }) {
  const e = ui.error;
  if (e) {
    return (
      <section className="error" aria-labelledby="error-title" id="error-panel">
        <h2 id="error-title">Build failed</h2>
        <p className="code">{e.code}</p>
        <p className="message">{e.message}</p>
        {e.file && (
          <p className="location" id="error-location">
            {e.file}
            {e.line ? `:${e.line}` : ""}
          </p>
        )}
        {e.excerpt && (
          <pre className="excerpt">
            {e.line ? `${e.line} | ` : ""}
            {e.excerpt}
          </pre>
        )}
        {e.hint && <p className="hint">{e.hint}</p>}
        {ui.scene && <p className="kept">The last good model is still shown.</p>}
      </section>
    );
  }
  if (!ui.scene) return <p className="muted">{ui.project ? "Waiting for the first build." : ""}</p>;
  return (
    <section aria-labelledby="run-title">
      <h2 id="run-title">Last build</h2>
      <dl className="facts">
        <dt>Parts</dt>
        <dd id="part-count">{ui.scene.parts}</dd>
        <dt>Steps</dt>
        <dd>{ui.scene.steps}</dd>
        <dt>Run</dt>
        <dd>#{ui.scene.generation}</dd>
      </dl>
    </section>
  );
}

function Tree({
  scene,
  selected,
  onPick,
}: {
  scene: Scene | null;
  selected: string | null;
  onPick: (key: string, ids: string[]) => void;
}) {
  const { steps, parts } = useMemo(() => {
    const steps = new Map<number, string[]>();
    const parts = new Map<string, string[]>();
    for (const p of scene?.placements ?? []) {
      steps.set(p.step, [...(steps.get(p.step) ?? []), p.id]);
      parts.set(p.part, [...(parts.get(p.part) ?? []), p.id]);
    }
    return {
      steps: [...steps].sort((a, b) => a[0] - b[0]),
      parts: [...parts].sort((a, b) => b[1].length - a[1].length || a[0].localeCompare(b[0])),
    };
  }, [scene]);
  if (!scene) return <p className="muted">No model yet.</p>;
  const item = (key: string, label: string, ids: string[]) => (
    <li key={key}>
      <button type="button" aria-pressed={selected === key} onClick={() => onPick(key, ids)}>
        {label}
      </button>
    </li>
  );
  return (
    <>
      <h2>Model</h2>
      <p className="model-name">{scene.model}</p>
      <h2>Steps · {steps.length}</h2>
      <ul>{steps.map(([n, ids]) => item(`step:${n}`, `Step ${n} · ${ids.length} ${ids.length === 1 ? "part" : "parts"}`, ids))}</ul>
      <h2>Parts · {scene.parts}</h2>
      <ul>{parts.map(([part, ids]) => item(`part:${part}`, `${part} × ${ids.length}`, ids))}</ul>
    </>
  );
}

function Recents({ info, onOpen }: { info: AppInfo | null; onOpen: (path: string) => void }) {
  if (!info?.recent.length) return <p className="muted">No recent projects.</p>;
  return (
    <>
      <h2>Recent</h2>
      <ul>
        {info.recent.map((r) => (
          <li key={r.path}>
            <button type="button" title={r.path} onClick={() => onOpen(r.path)}>
              {r.name}
            </button>
          </li>
        ))}
      </ul>
    </>
  );
}
