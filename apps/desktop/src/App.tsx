// SPDX-License-Identifier: AGPL-3.0-or-later
// The functional UI (plan section 14, before Phase 2 designs it): a project tree, the viewer with the Parameters,
// Checks, and Parts drawers, a run panel with the error and its traceback, the chat pane, and a status line with plan
// usage. The last good model stays on screen while a run is going or after one fails.
import { createViewer, type View, type Viewer } from "@emmet/viewer";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  type AgentKind,
  type AppInfo,
  commands,
  events,
  type Finding,
  type Problem,
  type Scene,
  type UiState,
  type UsageSnapshot,
} from "./bindings";
import { ChatPane } from "./Chat";
import { Checks, checksLabel, fixRequest } from "./Checks";
import { base64, json, library, placements } from "./library";
import { Params } from "./Params";
import { Parts, partsLabel } from "./Parts";
import { Settings } from "./Settings";
import { limitWarning, UsageBar } from "./Usage";

type Theme = "light" | "dark";
type Drawer = "params" | "checks" | "parts";
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
  const [ui, setUi] = useState<UiState>({ project: null, running: false, scene: null, error: null, notice: null });
  const [problem, setProblem] = useState<Problem | null>(null);
  const [viewerError, setViewerError] = useState<string | null>(null);
  const [theme, forceTheme] = useTheme();
  const viewer = useRef<Viewer | null>(null);
  const shown = useRef(-1);
  const framed = useRef<string | null>(null);
  const root = useRef<string | null>(null);
  /** When the last scene finished updating in the viewer (performance.now()), for the bench. */
  const drawnAt = useRef(0);
  const canvas = useRef<HTMLCanvasElement | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  // The chat: which agent, its live limits, the turns that ended (each fetches usage), and its status line.
  const [chatKind, setChatKind] = useState<AgentKind | null>(null);
  const [limits, setLimits] = useState<UsageSnapshot | null>(null);
  const [usage, setUsage] = useState<UsageSnapshot | null>(null);
  const [turns, setTurns] = useState(0);
  const [chatStatus, setChatStatus] = useState<string | null>(null);
  const [settings, setSettings] = useState(false);
  const [agentsChanged, setAgentsChanged] = useState(0);
  const [drawer, setDrawer] = useState<Drawer>("params");
  /** First run: "idle", "running" (the library download and Python), or why it failed. */
  const [setup, setSetup] = useState<"idle" | "running" | string>("idle");
  /** A finding to send to the chat ("Ask agent to fix"); `n` counts the asks, so the same finding can go twice. */
  const [ask, setAsk] = useState<{ text: string; n: number } | null>(null);
  /** A slider wrote params.json: the run it causes shows at once, without the build animation (plan section 13). */
  const sliderRun = useRef(false);
  const onTurnEnd = useCallback(() => setTurns((n) => n + 1), []);

  // The viewer lives as long as the window.
  useEffect(() => {
    if (!canvas.current) return;
    const v = createViewer(canvas.current, { partSource: library, theme });
    viewer.current = v;
    if (__EMMET_TEST_HOOKS__) {
      // Merged: the chat pane adds its own hooks, and a child's effects run first.
      const w = window as unknown as { __emmet?: Record<string, unknown> };
      w.__emmet = {
        ...(w.__emmet ?? {}),
        viewer: () => viewer.current,
        shown: () => shown.current,
        project: () => root.current,
        drawnAt: () => drawnAt.current,
        // The app's part source, and a second viewer with an empty part cache (the bench's cold first loads; specs
        // that wrap the part source).
        library,
        newViewer: (partSource = library) => {
          const c = document.createElement("canvas");
          c.style.cssText = "position:fixed;left:0;top:0;width:800px;height:600px;visibility:hidden";
          document.body.append(c);
          return createViewer(c, { partSource, theme });
        },
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

  /** Shows a scene if it's newer than the one on screen (the generation check), framing a project's first one. A
   * project's first scene and a slider's runs appear at once; other runs (the agent's, an edit) build up part by part
   * (D-34). */
  const show = useCallback(async (scene: Scene | null, root: string | null, animate: "build" | "none" = "none") => {
    const v = viewer.current;
    if (!v || !scene || scene.generation < shown.current) return;
    shown.current = scene.generation;
    const first = root != null && framed.current !== root;
    try {
      await v.update(placements(scene), { animate: first ? "none" : animate });
      drawnAt.current = performance.now();
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
        setDrawer("params");
        viewer.current?.highlight([]);
        setUi((u) => ({ ...u, project: payload.project, running: false, scene: payload.scene ?? null, error: null }));
        void show(payload.scene ?? null, payload.project.root);
        if (!payload.scene) void viewer.current?.update([]);
      }),
      events.runStarted.listen(() => setUi((u) => ({ ...u, running: true }))),
      events.runFinished.listen(({ payload }) => {
        // A stale run, or a scene older than the one on screen (a bus render may have shown a newer build), changes
        // nothing but the status.
        const scene = payload.scene && payload.scene.generation >= shown.current ? payload.scene : null;
        const animate = sliderRun.current ? "none" : "build";
        if (scene) {
          sliderRun.current = false;
          void show(scene, root.current, animate);
        }
        setUi((u) => {
          if (scene) return { ...u, running: false, scene, error: null };
          if (payload.error) return { ...u, running: false, error: payload.error };
          return { ...u, running: false };
        });
      }),
      events.busHighlight.listen(({ payload }) => viewer.current?.highlight(payload.ids)),
      events.busRender.listen(async ({ payload }) => {
        const v = viewer.current;
        try {
          if (!v) throw new Error("the viewer isn't ready");
          if (payload.scene && payload.scene.generation > shown.current) {
            const scene = payload.scene;
            setUi((u) => ({ ...u, scene, error: null }));
            await show(scene, null, "build");
          }
          // Only the ids the request names are highlighted in the render, whatever the window shows.
          const { png, camera } = await v.render({ view: payload.view as View, size: payload.size, highlight: payload.highlight });
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

  const highlight = (key: string, ids: string[], mode: "issue" | "select" = "select") => {
    const next = selected === key ? null : key;
    setSelected(next);
    viewer.current?.highlight(next ? ids : [], { mode });
  };

  const scene = ui.scene;
  // A finding that a new build no longer has stops being highlighted.
  const findings = scene?.checks?.findings;
  useEffect(() => {
    if (selected?.startsWith("finding:") && !findings?.some((f) => `finding:${f.id}` === selected)) {
      setSelected(null);
      viewer.current?.highlight([]);
    }
  }, [findings, selected]);
  const pickFinding = (f: Finding) => highlight(`finding:${f.id}`, f.parts, "issue");
  const askToFix = (f: Finding) => setAsk((a) => ({ text: fixRequest(f), n: (a?.n ?? 0) + 1 }));
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
          <button type="button" id="settings-button" onClick={() => setSettings(true)}>
            Settings
          </button>
        </div>
      </header>
      {problem && (
        <p className="notice" role="alert">
          {problem.message} {problem.hint}
        </p>
      )}
      {info && (!info.library || !info.python) && (
        <div className="setup" role="region" aria-label="Set up" id="setup">
          {setup === "running" ? (
            <p>Setting up: downloading the LDraw parts library (about 150 MB) and Python. This takes a minute or two.</p>
          ) : (
            <>
              <p>
                Emmet needs the LDraw parts library and Python before it can build a model. Setting them up downloads about
                150 MB once.
              </p>
              <button
                type="button"
                id="setup-button"
                onClick={async () => {
                  setSetup("running");
                  const r = await commands.setup();
                  setSetup(r.status === "ok" ? "idle" : `${r.error.message} ${r.error.hint}`.trim());
                  void commands.appInfo().then(setInfo);
                }}
              >
                Set up
              </button>
              {setup !== "idle" && <p className="notice">{setup}</p>}
            </>
          )}
        </div>
      )}
      {ui.notice && (
        <p className="notice" role="alert" id="app-notice">
          {ui.notice}
        </p>
      )}
      {ui.project && !ui.project.watching && (
        <p className="notice" role="alert">
          Emmet can't watch this folder, so edits won't rebuild on their own. Use Rebuild after a change.
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
          {ui.project && (
            <section className="drawers" aria-label="Model">
              <div className="tabs" role="tablist">
                {(
                  [
                    ["params", "Parameters"],
                    ["checks", checksLabel(scene?.checks)],
                    ["parts", partsLabel(scene?.bom)],
                  ] as const
                ).map(([key, label]) => (
                  <button
                    key={key}
                    type="button"
                    role="tab"
                    id={`tab-${key}`}
                    aria-selected={drawer === key}
                    className={key === "checks" && scene?.checks?.errors ? "has-errors" : undefined}
                    onClick={() => setDrawer(key)}
                  >
                    {label}
                  </button>
                ))}
              </div>
              <div className="drawer" role="tabpanel" hidden={drawer !== "params"}>
                <Params
                  key={ui.project.root}
                  running={ui.running}
                  lastGeneration={scene?.generation ?? -1}
                  onWrite={() => (sliderRun.current = true)}
                />
              </div>
              {drawer === "checks" && (
                <div className="drawer" role="tabpanel" id="checks">
                  <Checks
                    checks={scene?.checks}
                    selected={selected?.startsWith("finding:") ? selected.slice(8) : null}
                    onPick={pickFinding}
                    onAsk={askToFix}
                  />
                </div>
              )}
              {drawer === "parts" && (
                <div className="drawer" role="tabpanel" id="parts">
                  <Parts bom={scene?.bom} />
                </div>
              )}
            </section>
          )}
        </main>
        <aside className="side" aria-label="Run and chat">
          <RunPanel ui={ui} />
          <ChatPane
            project={ui.project ?? null}
            ask={ask}
            agentsChanged={agentsChanged}
            warning={limitWarning(limits ?? usage)}
            onTurnEnd={onTurnEnd}
            onLimits={setLimits}
            onStatus={setChatStatus}
            onAgent={setChatKind}
          />
        </aside>
      </div>
      {settings && <Settings onClose={() => setSettings(false)} onChanged={() => setAgentsChanged((n) => n + 1)} />}
      <footer className="status">
        <span role="status" id="status">
          {chatStatus ?? statusText(ui)}
        </span>
        <UsageBar kind={chatKind} live={limits} turns={turns} onSnapshot={setUsage} />
        <span className="versions">
          Emmet {__EMMET_VERSION__}
          {info && ` · LDraw library ${info.library ?? "not installed"}`}
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
