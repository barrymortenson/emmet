// SPDX-License-Identifier: AGPL-3.0-or-later
// The window, to the frozen design (plan section 14, docs/design/frozen; session 4A): the viewer on the left with the
// title row over it and the chat docked along its bottom, the options panel on the right behind a resizable separator,
// banners across the viewer's top, and first run as one card over everything until a project is open. The chat's
// internals are 1C's until 4B, and the viewer is 1B's until 4C. Every action (keys, macOS's menu bar, Windows' title-row
// menus, the project chip's menu) goes through `run`.
import { createViewer, type Theme, type View, type Viewer } from "@emmet/viewer";
import { Toast, Tooltip } from "radix-ui";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  type AgentKind,
  type AgentView,
  type AllowedView,
  type AppInfo,
  type BomLine,
  type BuildSettings,
  commands,
  events,
  type ExportFormat,
  type Finding,
  type NewKind,
  type PrefsView,
  type Problem,
  type Scene,
  type UiState,
  type UsageSnapshot,
  type VersionView,
} from "./bindings";
import { ChatPane } from "./Chat";
import { fixRequest } from "./Checks";
import { About, Settings, Shortcuts } from "./Dialogs";
import { FirstRun, type Step, useSetup } from "./FirstRun";
import { type Action, actionFor, windows } from "./keys";
import { base64, json, library, placements } from "./library";
import { rebuildRequest } from "./Options";
import { Panel, type Tab } from "./Panel";
import { type Pop, Titlebar } from "./Titlebar";
import { limitWarning, UsageBar } from "./Usage";
import { AlertCard, Banner, Button, Icon, Resizable, when } from "./ui";

type Dialog = "settings" | "about" | "shortcuts" | null;
const PANEL = { min: 320, max: 520, start: 384 };
/** A menu click and its key press can both arrive for one action: the second within this long is the same one. */
const SAME_ACTION_MS = 250;

/** The system appearance, unless Settings pins one or a test forces one. */
function useTheme(pinned: string | undefined): [Theme, (t: Theme | null) => void] {
  const query = useMemo(() => window.matchMedia("(prefers-color-scheme: dark)"), []);
  const [system, setSystem] = useState<Theme>(query.matches ? "dark" : "light");
  const [forced, setForced] = useState<Theme | null>(null);
  useEffect(() => {
    const on = () => setSystem(query.matches ? "dark" : "light");
    query.addEventListener("change", on);
    return () => query.removeEventListener("change", on);
  }, [query]);
  const settings = pinned === "light" || pinned === "dark" ? pinned : null;
  return [forced ?? settings ?? system, setForced];
}

/** The ids of the parts a sub-assembly's copies hold: every placement whose submodel path names its file. */
const copiesOf = (scene: Scene, file: string) => scene.placements.filter((p) => p.submodel.includes(file)).map((p) => p.id);

export function App() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [prefs, setPrefs] = useState<PrefsView | null>(null);
  const [ui, setUi] = useState<UiState>({ project: null, running: false, scene: null, error: null, notice: null });
  const [problem, setProblem] = useState<Problem | null>(null);
  const [viewerError, setViewerError] = useState<string | null>(null);
  const [theme, forceTheme] = useTheme(prefs?.prefs.appearance);
  const viewer = useRef<Viewer | null>(null);
  const shown = useRef(-1);
  const framed = useRef<string | null>(null);
  const root = useRef<string | null>(null);
  /** When the last scene finished updating in the viewer (performance.now()), for the bench. */
  const drawnAt = useRef(0);
  const canvas = useRef<HTMLCanvasElement | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("options");
  const [step, setStep] = useState<number | null>(null);
  const [sub, setSub] = useState<string | null>(null);
  const [view, setView] = useState<"iso" | "front" | "top" | null>("iso");
  const [panelWidth, setPanelWidth] = useState(() => Number(localStorage.getItem("emmet.panel")) || PANEL.start);
  const [dialog, setDialog] = useState<Dialog>(null);
  const [pop, setPop] = useState<Pop>(null);
  const [exportOpen, setExportOpen] = useState(false);
  const [versions, setVersions] = useState<VersionView[] | null>(null);
  const [compare, setCompare] = useState<{ v: VersionView; scene: Scene } | null>(null);
  const [toast, setToast] = useState<{ text: string; action?: { label: string; run: () => void }; n: number } | null>(null);
  const [online, setOnline] = useState(navigator.onLine);
  const [libraryLater, setLibraryLater] = useState(false);
  const [firstRun, setFirstRun] = useState<Step | null>(null);
  const [creating, setCreating] = useState<NewKind | null>(null);
  const [startupMs, setStartupMs] = useState<number | null>(null);
  // The chat (1C's, until 4B): which agent, its live limits, the turns that ended (each fetches usage).
  const [chatKind, setChatKind] = useState<AgentKind | null>(null);
  const [limits, setLimits] = useState<UsageSnapshot | null>(null);
  const [usage, setUsage] = useState<UsageSnapshot | null>(null);
  const [turns, setTurns] = useState(0);
  const [agentsChanged, setAgentsChanged] = useState(0);
  const [agents, setAgents] = useState<AgentView[] | null>(null);
  /** A message for the chat ("Ask agent to fix", Rebuild, Make it parametric); `n` counts them, so one can go twice. */
  const [ask, setAsk] = useState<{ text: string; n: number } | null>(null);
  /** A knob wrote params.json: the run it causes shows at once, without the build animation (plan section 13). */
  const knobRun = useRef(false);
  const onTurnEnd = useCallback(() => setTurns((n) => n + 1), []);
  const refreshInfo = useCallback(() => void commands.appInfo().then(setInfo), []);
  const { setup, start: startSetup } = useSetup(refreshInfo);
  const ready = !!info?.library && !!info.python;
  const say = (text: string, action?: { label: string; run: () => void }) => setToast((t) => ({ text, action, n: (t?.n ?? 0) + 1 }));
  const sendToAgent = (text: string) => setAsk((a) => ({ text, n: (a?.n ?? 0) + 1 }));

  useEffect(() => {
    void commands.agents(agentsChanged > 0).then(setAgents);
  }, [agentsChanged]);

  // The viewer lives as long as the window.
  useEffect(() => {
    if (!canvas.current) return;
    const v = createViewer(canvas.current, { partSource: library, theme, quality: prefs?.prefs.viewer_quality === "fast" ? "low" : "auto" });
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
        offline: (off: boolean | null) => setOnline(off == null ? navigator.onLine : !off),
      };
    }
    return () => {
      v.dispose();
      viewer.current = null;
    };
    // Created once; the theme effect below keeps it current.
  }, []);
  useEffect(() => {
    if (__EMMET_TEST_HOOKS__) (window as unknown as { __emmet: Record<string, unknown> }).__emmet.startup = () => startupMs;
  }, [startupMs]);

  useEffect(() => {
    viewer.current?.setTheme(theme);
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  /** Shows a scene if it's newer than the one on screen (the generation check), framing a project's first one. A
   * project's first scene and a knob's runs appear at once; other runs (the agent's, an edit) build up part by part
   * (D-34). */
  const show = useCallback(async (scene: Scene | null, at: string | null, animate: "build" | "none" = "none") => {
    const v = viewer.current;
    if (!v || !scene || scene.generation < shown.current) return;
    shown.current = scene.generation;
    const first = at != null && framed.current !== at;
    try {
      await v.update(placements(scene), { animate: first ? "none" : animate });
      drawnAt.current = performance.now();
      const failed = Object.entries(v.stats().failed);
      setViewerError(failed.length ? `Some parts can't be shown: ${failed.map(([id, why]) => `${id} (${why})`).join("; ")}` : null);
    } catch (e) {
      setViewerError(`The viewer couldn't show the model: ${e instanceof Error ? e.message : String(e)}`);
      return;
    }
    if (at && framed.current !== at) {
      framed.current = at;
      v.setView("iso");
      setView("iso");
    }
  }, []);

  useEffect(() => {
    const off: Promise<() => void>[] = [
      events.projectOpened.listen(({ payload }) => {
        shown.current = -1;
        framed.current = null;
        root.current = payload.project.root;
        setSelected(null);
        setStep(null);
        setSub(null);
        setCompare(null);
        setVersions(null);
        setTab("options");
        setFirstRun(null);
        setCreating(null);
        viewer.current?.highlight([]);
        setUi((u) => ({ ...u, project: payload.project, running: false, scene: payload.scene ?? null, error: null }));
        void show(payload.scene ?? null, payload.project.root);
        if (!payload.scene) void viewer.current?.update([]);
        refreshInfo();
      }),
      events.runStarted.listen(() => setUi((u) => ({ ...u, running: true }))),
      events.runFinished.listen(({ payload }) => {
        // A stale run, or a scene older than the one on screen (a bus render may have shown a newer build), changes
        // nothing but the status.
        const scene = payload.scene && payload.scene.generation >= shown.current ? payload.scene : null;
        const animate = knobRun.current ? "none" : "build";
        if (scene) {
          knobRun.current = false;
          setStep(null);
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
      events.menuAction.listen(({ payload }) => run(payload.id as Action, "menu")),
    ];
    void commands.getPrefs().then(setPrefs);
    void Promise.all([commands.appInfo(), commands.uiState()]).then(([i, s]) => {
      setInfo(i);
      root.current = s.project?.root ?? null;
      setUi(s);
      void show(s.scene ?? null, root.current);
      // With no project open: first run, or Start once the parts and Python are set up; else the last project.
      if (!s.project) {
        if (i.library && i.python && i.recent[0]) void commands.openRecent(i.recent[0].path).then((r) => r.status === "error" && setFirstRun("start"));
        else setFirstRun(i.library && i.python ? "start" : "welcome");
      }
      // Interactive: the first frame after this one has drawn with its handlers in place.
      requestAnimationFrame(() => setTimeout(() => void commands.appInfo().then((n) => setStartupMs(n.uptime_ms)), 0));
    });
    const net = () => setOnline(navigator.onLine);
    window.addEventListener("online", net);
    window.addEventListener("offline", net);
    return () => {
      off.forEach((p) => void p.then((f) => f()));
      window.removeEventListener("online", net);
      window.removeEventListener("offline", net);
    };
    // `run` reads the latest state through a ref.
  }, [show, refreshInfo]);

  const act = async <T,>(r: Promise<{ status: "ok"; data: T } | { status: "error"; error: Problem }>): Promise<T | null> => {
    const out = await r;
    setProblem(out.status === "error" ? out.error : null);
    return out.status === "ok" ? out.data : null;
  };

  const highlight = (key: string | null, ids: string[], mode: "issue" | "select" = "select") => {
    setSelected(key);
    viewer.current?.highlight(key ? ids : [], { mode });
  };
  const toggle = (key: string, ids: string[], mode: "issue" | "select" = "select") => highlight(selected === key ? null : key, ids, mode);

  const scene = ui.scene;
  const project = ui.project;
  // A finding that a new build no longer has stops being highlighted.
  const findings = scene?.checks?.findings;
  useEffect(() => {
    if (selected?.startsWith("finding:") && !findings?.some((f) => `finding:${f.id}` === selected)) highlight(null, []);
  }, [findings, selected]);

  // What the viewer shows: a version opened to compare, the build up to a step, or the whole model.
  useEffect(() => {
    const v = viewer.current;
    if (!v || !scene) return;
    if (compare) void v.update(placements(compare.scene), { animate: "none" });
    else if (step != null) void v.update(placements(scene).filter((p) => p.step <= step), { animate: "none" });
    else void v.update(placements(scene), { animate: "none" });
  }, [compare, step]);

  const openVersions = async () => {
    // After a menu that chose it has closed, so the popover isn't dismissed as the menu goes.
    setTimeout(() => setPop("versions"), 0);
    setVersions(await act(commands.listVersions()));
  };
  const saveVersion = async (name: string) => {
    let thumb: string | null = null;
    try {
      const v = viewer.current;
      if (v && scene) thumb = base64((await v.render({ view: "iso", size: 128 })).png);
    } catch {
      thumb = null;
    }
    if (await act(commands.saveVersion(name, thumb))) await openVersions();
  };
  const openVersion = async (v: VersionView) => {
    const s = await act(commands.versionScene(v.n));
    if (s) {
      setCompare({ v, scene: s });
      setPop(null);
    }
  };
  const exportAs = async (f: ExportFormat) => {
    const e = await act(commands.export(f));
    if (e) say(`Saved “${e.file}” to ${e.folder}`, { label: "Show in folder", run: () => void commands.revealExport() });
  };

  const can = (a: Action): boolean => {
    if (["help:guide", "agent:switch", "agent:model", "undo-turn"].includes(a)) return false;
    if (["save", "versions", "export", "frame", "step:prev", "step:next"].includes(a) || a.startsWith("view:") || a.startsWith("tab:")) return !!project;
    return true;
  };
  const lastAction = useRef<{ a: Action; at: number } | null>(null);
  const latest = useRef<(a: Action, from: "key" | "menu") => void>(() => {});
  latest.current = (a, from) => {
    const now = performance.now();
    const same = lastAction.current && lastAction.current.a === a && now - lastAction.current.at < SAME_ACTION_MS;
    lastAction.current = { a, at: now };
    if (same && from === "menu") return;
    if (!can(a)) return;
    const stepTo = (d: number) => {
      const total = scene?.tree?.steps.length ?? 0;
      if (!total) return;
      setTab("steps");
      setSub(null);
      setStep((s) => Math.max(1, Math.min(total, (s ?? total) + d)));
    };
    if (a.startsWith("view:")) {
      const v = a.slice(5) as View;
      viewer.current?.setView(v);
      setView(v === "iso" || v === "front" || v === "top" ? v : null);
    } else if (a.startsWith("tab:")) setTab(a.slice(4) as Tab);
    else
      switch (a) {
        case "new":
          setFirstRun("start");
          break;
        case "open":
          void act(commands.openDialog());
          break;
        case "save":
          setTimeout(() => setPop("save"), 0);
          break;
        case "versions":
          void openVersions();
          break;
        case "export":
          setTab("parts");
          setExportOpen(true);
          break;
        case "settings":
        case "updates":
        case "cli":
          setDialog("settings");
          break;
        case "about":
          setDialog("about");
          break;
        case "shortcuts":
          setDialog("shortcuts");
          break;
        case "diagnostics":
          void commands.diagnostics().then((t) => navigator.clipboard.writeText(t).then(() => say("Copied the diagnostics")));
          break;
        case "help:report":
          void commands.reportProblem();
          break;
        case "frame":
          viewer.current?.frame(selected ? viewer.current.highlighted() : undefined);
          break;
        case "step:prev":
          stepTo(-1);
          break;
        case "step:next":
          stepTo(1);
          break;
        case "undo-knob":
        case "redo-knob":
          window.dispatchEvent(new Event(`emmet:${a}`));
          break;
        case "agent:message":
          document.getElementById("chat-input")?.focus();
          break;
        case "agent:send":
          document.getElementById("chat-send")?.click();
          break;
        case "agent:stop":
          void commands.chatStop();
          break;
        case "agent:fresh":
          document.getElementById("chat-new")?.click();
          break;
        case "agent:usage":
          document.getElementById("usage-button")?.click();
          break;
        case "history":
          document.getElementById("chat-log")?.scrollIntoView({ block: "end" });
          break;
        case "region:next": {
          const regions = ["viewer", "dock", "panel"].map((id) => document.getElementById(id)).filter((r): r is HTMLElement => !!r);
          const i = regions.findIndex((r) => r.contains(document.activeElement));
          regions[(i + 1) % regions.length]?.focus();
          break;
        }
        case "escape":
          // Radix closes its own menus, popovers, and dialogs first; with nothing open, Esc stops the agent.
          if (compare) setCompare(null);
          else if (firstRun === "start" && project) setFirstRun(null);
          else void commands.chatStop();
          break;
      }
  };
  const run = useCallback((a: Action, from: "key" | "menu" = "key") => latest.current(a, from), []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // A Radix menu, popover, or dialog that's open handles its own keys (Esc, arrows) first.
      if (e.defaultPrevented) return;
      const a = actionFor(e);
      if (!a) return;
      if (a === "escape" && document.querySelector('[role="dialog"], [role="menu"], [role="listbox"]')) return;
      if (a !== "escape") e.preventDefault();
      run(a, "key");
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [run]);

  const name = project?.name ?? "Emmet";
  const readOnly = project != null && !project.trusted;
  const banner = (() => {
    if (!project) return null;
    if (readOnly)
      return (
        <Banner
          kind="untrusted"
          icon="lock"
          title={`Read-only: ${project.name} was made on another computer, or moved, so Emmet won't build it until you trust it.`}
          actions={<Button label="Trust this project" variant="primary" size="sm" id="trust" title="A project can run code on your computer. Trust it only if you know where it came from." onClick={() => void act(commands.trustProject())} />}
        />
      );
    if (!online)
      return <Banner kind="offline" icon="wifi-off" title="You're offline." actions={<Button label="Try again" variant="quiet" size="sm" onClick={() => setOnline(navigator.onLine)} />} />;
    if (info?.library_old && !libraryLater)
      return (
        <Banner
          kind="old-library"
          icon="download"
          title={`Your parts library is ${info.library}. This Emmet expects ${info.pinned}.`}
          text="Some parts may be missing or renamed until you update (about 150 MB, a minute or two)."
          actions={
            <>
              <Button label="Later" variant="quiet" size="sm" onClick={() => setLibraryLater(true)} />
              <Button label={setup.at === "library" || setup.at === "python" ? "Updating" : "Update the library"} variant="primary" size="sm" id="update-library" busy={setup.at === "library" || setup.at === "python"} onClick={() => void startSetup()} />
            </>
          }
        />
      );
    if (project.imported && scene)
      return (
        <Banner
          kind="imported"
          icon="file-up"
          title={`Opened ${project.imported}: ${count(scene.parts, "part")} in ${count(scene.tree?.steps.length ?? scene.steps, "step")}.`}
          actions={<Button label="Make it parametric" variant="primary" size="sm" id="make-parametric" onClick={() => sendToAgent(`Make this model parametric: rebuild ${project.imported} in model.py with emmet_dsl, keeping its parts, colors, and steps, with two to five knobs that change its character.`)} />}
        />
      );
    return null;
  })();
  const titlebar = <Titlebar name={name} hasProject={!!project} view={view} onView={(v) => run(`view:${v}`)} run={run} can={can} pop={pop} onPop={setPop} versions={versions} onSaveVersion={saveVersion} onOpenVersion={openVersion} />;

  return (
    <Tooltip.Provider delayDuration={500}>
      <Toast.Provider duration={5000} swipeDirection="down">
        <div className="app" data-c="app" style={{ "--panel-width": `${panelWidth}px` } as React.CSSProperties}>
          {windows && titlebar}
          <div className="app-main">
            {!windows && titlebar}
            {banner}
            <section className="viewer" data-c="viewer" aria-label={`3D view of ${name}`} id="viewer" tabIndex={-1}>
              <canvas ref={canvas} aria-label="3D view of the model; the Steps and Parts tabs list its parts" />
              {compare && (
                <div className="viewer-bar" role="status" id="version-bar">
                  <Icon name="rotate-ccw-clock" />
                  <span>
                    <b>
                      Version {compare.v.n}: {compare.v.name}
                    </b>{" "}
                    · {when(compare.v.time)}
                  </span>
                  <Button
                    label="Restore"
                    variant="primary"
                    size="sm"
                    icon="undo-2"
                    id="version-restore"
                    title="Saves the current model as a version first"
                    onClick={async () => {
                      const saved = await act(commands.restoreVersion(compare.v.n));
                      if (saved) {
                        say(`Restored version ${compare.v.n}. The model before is version ${saved.n}.`);
                        setCompare(null);
                      }
                    }}
                  />
                  <Button label="Open as new project" size="sm" id="version-open" onClick={() => void act(commands.openVersionAsNew(compare.v.n))} />
                  <Button label="Back" variant="quiet" size="sm" id="version-back" onClick={() => setCompare(null)} />
                </div>
              )}
              {(problem || viewerError || ui.notice || (project && !project.watching)) && (
                <div className="viewer-notes">
                  {problem && (
                    <p className="notice" role="alert" id="app-problem">
                      {problem.message} {problem.hint}
                    </p>
                  )}
                  {ui.notice && (
                    <p className="notice" role="alert" id="app-notice">
                      {ui.notice}
                    </p>
                  )}
                  {project && !project.watching && (
                    <p className="notice" role="alert">
                      Emmet can't watch this folder, so edits won't rebuild on their own.
                    </p>
                  )}
                  {viewerError && (
                    <p className="notice" role="alert" id="viewer-error">
                      {viewerError}
                    </p>
                  )}
                </div>
              )}
              {toast && (
                <Toast.Root key={toast.n} className="toast" data-c="toast" id="toast" onOpenChange={(o) => !o && setToast(null)}>
                  <Icon name="circle-check" />
                  <Toast.Title className="toast-text">{toast.text}</Toast.Title>
                  {toast.action && (
                    <Toast.Action altText={toast.action.label} asChild>
                      <Button label={toast.action.label} size="sm" variant="quiet" onClick={toast.action.run} id="toast-action" />
                    </Toast.Action>
                  )}
                  <Toast.Close className="icon-button" data-c="icon-button" data-size="sm" aria-label="Dismiss" title="Dismiss">
                    <Icon name="x" />
                  </Toast.Close>
                </Toast.Root>
              )}
              <Toast.Viewport className="toast-viewport" aria-label="Notifications" />
            </section>
            {project && (
              <section className="dock" data-c="dock" aria-label="Chat" id="dock" tabIndex={-1}>
                {ui.error &&
                  (ui.error.code.startsWith("MODEL_") || ui.error.code === "PARAM_INVALID" ? (
                    <AlertCard
                      tone="bad"
                      icon="octagon-alert"
                      id="build-failed"
                      title="The model didn't build"
                      text="The agent's last change has a mistake in it, so the new model couldn't be built."
                      foot={scene ? "The last good model is still shown." : undefined}
                      actions={<Button label="Ask agent to fix" variant="primary" id="ask-fix-build" onClick={() => sendToAgent(buildFixRequest(ui.error!))} />}
                    />
                  ) : (
                    // Emmet's own trouble, not the model's: what happened, and Rebuild.
                    <AlertCard
                      tone="bad"
                      icon="octagon-alert"
                      id="build-failed"
                      title="The build stopped"
                      text={`${ui.error.message} ${ui.error.hint}`.trim()}
                      foot={scene ? "The last good model is still shown." : undefined}
                      actions={<Button label="Rebuild" variant="primary" icon="refresh-cw" id="rebuild" onClick={() => void act(commands.rebuild())} />}
                    />
                  ))}
                <ChatPane project={project} ask={ask} agentsChanged={agentsChanged} warning={limitWarning(limits ?? usage)} onTurnEnd={onTurnEnd} onLimits={setLimits} onStatus={noop} onAgent={setChatKind} onSettings={() => setDialog("settings")} />
                <UsageBar kind={chatKind} live={limits} turns={turns} onSnapshot={setUsage} />
              </section>
            )}
          </div>
          <Resizable
            orientation="vertical"
            label="Resize the options panel"
            value={panelWidth}
            min={PANEL.min}
            max={PANEL.max}
            invert
            onChange={(w) => {
              setPanelWidth(w);
              localStorage.setItem("emmet.panel", String(w));
            }}
          />
          {project ? (
            <Panel
              project={project}
              scene={scene}
              running={ui.running}
              error={ui.error}
              theme={theme}
              tab={tab}
              onTab={setTab}
              selected={selected}
              step={step}
              sub={sub}
              exportOpen={exportOpen}
              onExportOpen={setExportOpen}
              onExport={(f) => void exportAs(f)}
              onPickFinding={(f: Finding) => toggle(`finding:${f.id}`, f.parts, "issue")}
              onAsk={(f: Finding) => sendToAgent(fixRequest(f))}
              onAllow={(f: Finding) => void act(commands.allowFinding(f.rule, f.parts))}
              onUnallow={(a: AllowedView) => void act(commands.unallowFinding(a.rule, a.parts))}
              onPickPart={(l: BomLine) => scene && toggle(`part:${l.part}|${l.color}`, scene.placements.filter((p) => p.part === l.part && p.color === l.color).map((p) => p.id))}
              onStep={(n) => {
                setSub(null);
                highlight(null, []);
                setStep((s) => (s === n && n === (scene?.tree?.steps.length ?? 0) ? null : n));
              }}
              onSub={(file) => {
                setSub(file);
                setStep(null);
                if (scene) highlight(file ? `sub:${file}` : null, file ? copiesOf(scene, file) : []);
              }}
              onSaveBuild={async (s: BuildSettings) => {
                const saved = await act(commands.setBuildSettings(s));
                if (saved) setUi((u) => (u.project ? { ...u, project: { ...u.project, build: saved } } : u));
                return saved != null;
              }}
              onRebuild={(before, after) => sendToAgent(rebuildRequest(before, after))}
              onKnobWrite={() => (knobRun.current = true)}
            />
          ) : (
            <aside className="panel" data-c="panel" aria-label="Options" id="panel" tabIndex={-1} />
          )}
          {firstRun && (
            <div className="first-run-layer">
              <FirstRun
                step={firstRun}
                onStep={setFirstRun}
                setup={setup}
                onSetup={() => void startSetup()}
                ready={ready}
                theme={theme}
                projectsFolder={prefs?.projects_folder ?? "Documents/Emmet"}
                agents={agents}
                onAgentsChanged={() => setAgentsChanged((n) => n + 1)}
                onSettings={() => setDialog("settings")}
                onNew={async (kind) => {
                  setCreating(kind);
                  if (!(await act(commands.newProject(kind)))) setCreating(null);
                  else if (kind === "empty") setTimeout(() => document.getElementById("chat-input")?.focus(), 0);
                }}
                onOpen={() => run("open")}
                creating={creating}
              />
              {problem && (
                <p className="notice first-run-problem" role="alert" id="first-run-problem">
                  {problem.message} {problem.hint}
                </p>
              )}
            </div>
          )}
          {dialog === "settings" && <Settings info={info} prefs={prefs} onPrefs={setPrefs} onClose={() => setDialog(null)} onAgentsChanged={() => setAgentsChanged((n) => n + 1)} onCopied={() => say("Copied the diagnostics")} />}
          {dialog === "about" && <About info={info} theme={theme} onClose={() => setDialog(null)} />}
          {dialog === "shortcuts" && <Shortcuts onClose={() => setDialog(null)} />}
        </div>
      </Toast.Provider>
    </Tooltip.Provider>
  );
}

const noop = () => {};
/** A count with its noun: "1 step", "31 steps". */
const count = (n: number, noun: string) => `${n.toLocaleString("en-US")} ${noun}${n === 1 ? "" : "s"}`;

/** What the failed build's "Ask agent to fix" sends: the error as emmet reported it, with its file and line (the agent
 * sees them; the screen doesn't, law 2). */
function buildFixRequest(e: Problem): string {
  const at = e.file ? ` at ${e.file}${e.line ? `:${e.line}` : ""}` : "";
  return `The model didn't build (${e.code}${at}): ${e.message} ${e.hint} Fix it, then run the model again.`.replace(/\s+/g, " ");
}
