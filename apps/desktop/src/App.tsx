// SPDX-License-Identifier: AGPL-3.0-or-later
// The window, to the frozen design (plan section 14, docs/design/frozen; session 4A): the viewer on the left with the
// title row over it and the chat docked along its bottom, the options panel on the right behind a resizable separator,
// banners across the viewer's top, and first run as one card over everything until a project is open. What the viewer
// carries over the model (the hovered part, a step's parts box, pins, the cut-away's bar) is ViewerLayer's (4C). Every
// action (keys, macOS's menu bar, Windows' title-row menus, the project chip's menu) goes through `run`.
import { createViewer, type Pace, type Preset, type Quality, type Theme, type View, type Viewer } from "@emmet/viewer";
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
  type TurnContext,
  type UiState,
  type VersionView,
} from "./bindings";
import { type Ask, Dock, DockBoundary } from "./Chat";
import { Compare } from "./Compare";
import { findingWords, fixRequest } from "./Checks";
import { About, NewModel, Settings, Shortcuts } from "./Dialogs";
import { FirstRun, libraryFailure, type Step, useSetup } from "./FirstRun";
import { drawBook } from "./instructions";
import { type Action, actionFor, windows } from "./keys";
import { base64, json, library, placements } from "./library";
import { rebuildRequest, rebuildWords } from "./Options";
import { Panel, type Tab, type TipId } from "./Panel";
import { type Pop, Titlebar } from "./Titlebar";
import { Banner, Button, Icon, Resizable, when } from "./ui";
import { partPicture } from "./pictures";
import { ViewerLayer } from "./ViewerLayer";

type Dialog = "settings" | "about" | "shortcuts" | "new-model" | null;
const PANEL = { min: 320, max: 520, start: 384 };
/** A menu click and its key press can both arrive for one action: the second within this long is the same one. */
const SAME_ACTION_MS = 250;
/** What works while first run covers the window: its own ways out, and the dialogs over it. */
const COVERED_OK: Action[] = ["new", "open", "settings", "updates", "cli", "about", "shortcuts", "diagnostics", "help:report", "escape"];

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
/** The parts an agent's ids light, as the viewer lights them: an id, or the id of a copy that holds the part. */
const litBy = (scene: Scene, ids: string[]) => {
  const want = new Set(ids);
  const lit = (id: string): boolean => want.has(id) || (id.lastIndexOf("/") > 0 && lit(id.slice(0, id.lastIndexOf("/"))));
  return scene.placements.filter((p) => lit(p.id)).map((p) => p.id);
};
/** Who an agent's highlight came from, by the name its MCP client gives ("claude-code"). */
const pointer = (client: string | null) => (/claude/i.test(client ?? "") ? "Claude" : /codex/i.test(client ?? "") ? "Codex" : "Your agent");

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
  /** The project opened on an empty model, which isn't framed yet: its first parts build up and get framed. */
  const openedEmpty = useRef(false);
  /** A version is open to compare (show() leaves it on screen). */
  const comparing = useRef(false);
  const root = useRef<string | null>(null);
  /** When the last scene finished updating in the viewer (performance.now()), for the bench. */
  const drawnAt = useRef(0);
  // The viewer's last error, for the E2E hooks (the state is for the window).
  const lastViewerError = useRef<string | null>(null);
  const viewerBox = useRef<HTMLElement | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  // An agent's highlight (the bus's), named in the bar over the viewer until it's cleared or the user picks something.
  const [pointing, setPointing] = useState<{ who: string; ids: string[] } | null>(null);
  const [tab, setTab] = useState<Tab>("options");
  const [step, setStep] = useState<number | null>(null);
  const [sub, setSub] = useState<string | null>(null);
  const [view, setView] = useState<"iso" | "front" | "top" | null>("iso");
  /** The cut-away's top layer (null when it's off). */
  const [cut, setCut] = useState<number | null>(null);
  const [animating, setAnimating] = useState(false);
  /** What "auto" quality picked for this GPU, for Settings to show. */
  const [autoQuality, setAutoQuality] = useState<Preset | null>(null);
  const [panelWidth, setPanelWidth] = useState(() => Number(localStorage.getItem("emmet.panel")) || PANEL.start);
  const [dialog, setDialog] = useState<Dialog>(null);
  const [pop, setPop] = useState<Pop>(null);
  const [exportOpen, setExportOpen] = useState(false);
  const [versions, setVersions] = useState<VersionView[] | null>(null);
  const [compare, setCompare] = useState<{ v: VersionView; scene: Scene } | null>(null);
  /** The photo beside the model (4D's compare view), by its path in refs/. */
  const [photo, setPhoto] = useState<string | null>(null);
  const closePhoto = useCallback(() => setPhoto(null), []);
  useEffect(() => setPhoto(null), [ui.project?.root]);
  comparing.current = compare != null;
  const [toast, setToast] = useState<{ text: string; action?: { label: string; run: () => void }; n: number } | null>(null);
  const [online, setOnline] = useState(navigator.onLine);
  const [libraryLater, setLibraryLater] = useState(false);
  const [firstRun, setFirstRun] = useState<Step | null>(null);
  const [creating, setCreating] = useState<NewKind | null>(null);
  const [startupMs, setStartupMs] = useState<number | null>(null);
  // The chat: which agent it's with, and what the user sees, which goes with each message (4B).
  const [, setChatKind] = useState<AgentKind | null>(null);
  /** The agent is working: a tip waits until it's done. */
  const [agentBusy, setAgentBusy] = useState(false);
  const [agentsChanged, setAgentsChanged] = useState(0);
  const [agents, setAgents] = useState<AgentView[] | null>(null);
  /** A message for the chat ("Ask agent to fix", Rebuild, Make it parametric); `n` counts them, so one can go twice. */
  const [ask, setAsk] = useState<Ask | null>(null);
  /** The ids the user picked (a part, a sub-assembly, a finding) and the view: the agent sees them (the turn's context,
   * and the bus's selection and camera). */
  const picked = useRef<string[]>([]);
  const viewNow = useRef<string>("iso");
  /** A knob wrote params.json: the run it causes shows at once, without the build animation (plan section 13). */
  const knobRun = useRef(false);
  /** The tips this user has seen (one at a time, each once: plan section 14). Null until the host says. */
  const [tipsSeen, setTipsSeen] = useState<string[] | null>(null);
  const tipSeen = (t: TipId) => {
    if (tipsSeen && !tipsSeen.includes(t)) void commands.tipSeen(t).then((r) => r.status === "ok" && setTipsSeen(r.data));
  };
  const refreshInfo = useCallback(() => void commands.appInfo().then(setInfo), []);
  const { setup, start: startSetup } = useSetup(refreshInfo);
  const ready = !!info?.library && !!info.python;
  const say = (text: string, action?: { label: string; run: () => void }) => setToast((t) => ({ text, action, n: (t?.n ?? 0) + 1 }));
  const sendToAgent = (from: Ask["from"], text: string, detail: string) => setAsk((a) => ({ text, detail, from, n: (a?.n ?? 0) + 1 }));
  const context = useCallback((): TurnContext => ({ selection: picked.current, view: viewNow.current, detail: null }), []);
  const tellHost = () => void commands.viewerState(picked.current, viewNow.current);

  useEffect(() => {
    void commands.agents(agentsChanged > 0).then(setAgents);
  }, [agentsChanged]);
  useEffect(() => {
    void commands.tipsSeen().then(setTipsSeen);
  }, []);

  // The viewer lives as long as the window, on a canvas of its own: dispose() gives up the canvas's WebGL context, so
  // an effect run twice (React's StrictMode in dev builds) must not draw on the same one again.
  useEffect(() => {
    const box = viewerBox.current;
    if (!box) return;
    const canvas = document.createElement("canvas");
    canvas.setAttribute("aria-label", "3D view of the model; the Steps and Parts tabs list its parts");
    box.prepend(canvas);
    const v = createViewer(canvas, { partSource: library, theme });
    viewer.current = v;
    setAutoQuality(v.stats().quality);
    const offAnimation = v.on("animation", setAnimating);
    if (__EMMET_TEST_HOOKS__) {
      // Merged: the chat dock adds its own hooks, and a child's effects run first.
      const w = window as unknown as { __emmet?: Record<string, unknown> };
      w.__emmet = {
        ...(w.__emmet ?? {}),
        viewer: () => viewer.current,
        shown: () => shown.current,
        project: () => root.current,
        drawnAt: () => drawnAt.current,
        viewerError: () => lastViewerError.current,
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
        // A part's picture, as the Parts list and the parts box ask for one.
        partPicture,
        offline: (off: boolean | null) => setOnline(off == null ? navigator.onLine : !off),
        // First run's step, for a spec to come back to one the rail has passed.
        firstRun: (step: Step) => setFirstRun(step),
      };
    }
    return () => {
      offAnimation();
      v.dispose();
      canvas.remove();
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
  // First run covers the window: Tab and clicks can't reach what's behind it (the title row, viewer, chat, and panel).
  useEffect(() => {
    for (const el of document.querySelectorAll(".app > .app-main, .app > .resizable, .app > .panel, .app > .titlebar")) el.toggleAttribute("inert", firstRun != null);
  }, [firstRun, ui.project]);
  const quality = prefs?.prefs.viewer_quality;
  useEffect(() => {
    if (quality) viewer.current?.setQuality(quality as Quality);
  }, [quality]);
  const pace = prefs?.prefs.build_animation;
  useEffect(() => {
    if (pace) viewer.current?.setPace(pace as Pace);
  }, [pace]);

  /** Puts placements in the viewer and says over it when parts can't be shown, or the viewer failed (ADR 0005: never
   * a model with holes, unremarked). False when the update failed. */
  const showIn = useCallback(async (v: Viewer, list: ReturnType<typeof placements>, animate: "build" | "none"): Promise<boolean> => {
    try {
      await v.update(list, { animate });
      const failed = Object.entries(v.stats().failed);
      lastViewerError.current = failed.length ? `Some parts can't be shown: ${failed.map(([id, why]) => `${id} (${why})`).join("; ")}` : null;
      setViewerError(lastViewerError.current);
      return true;
    } catch (e) {
      lastViewerError.current = `The viewer couldn't show the model: ${e instanceof Error ? e.message : String(e)}`;
      setViewerError(lastViewerError.current);
      return false;
    }
  }, []);

  /** Shows a scene if it's newer than the one on screen (the generation check), framing a project's first one. A
   * project's first scene and a knob's runs appear at once; other runs (the agent's, an edit) build up part by part
   * (D-34). */
  const show = useCallback(async (scene: Scene | null, at: string | null, animate: "build" | "none" = "none") => {
    const v = viewer.current;
    if (!v || !scene || scene.generation < shown.current) return;
    shown.current = scene.generation;
    // A version open to compare stays on screen; the newest build shows when it closes.
    if (comparing.current) return;
    const first = at != null && framed.current !== at;
    // The opening scene appears at once; the first parts of a model that opened empty build up as any run's do.
    if (!(await showIn(v, placements(scene), first && !openedEmpty.current ? "none" : animate))) return;
    drawnAt.current = performance.now();
    if (at && first) {
      // A model that starts empty (Base None, Barry 2026-11-10) is framed when its first parts come, not before: the
      // camera never moves on its own after that, so framing the empty table left the model off to one side.
      if (scene.placements.length > 0) {
        framed.current = at;
        openedEmpty.current = false;
      } else openedEmpty.current = true;
      v.setView("iso");
      setView("iso");
      viewNow.current = "iso";
      tellHost();
    }
  }, [showIn]);

  useEffect(() => {
    const off: Promise<() => void>[] = [
      events.projectOpened.listen(({ payload }) => {
        shown.current = -1;
        framed.current = null;
        openedEmpty.current = false;
        comparing.current = false;
        root.current = payload.project.root;
        setViewerError(null);
        setSelected(null);
        setPointing(null);
        // Nothing is picked in the new project: the agent's `selection` and the next turn's context say so.
        picked.current = [];
        tellHost();
        setStep(null);
        setSub(null);
        setCut(null);
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
      events.busHighlight.listen(({ payload }) => {
        // The agent's pointing replaces what the user picked, so `selection` doesn't read a pick no longer shown.
        setSelected(null);
        picked.current = [];
        tellHost();
        viewer.current?.highlight(payload.ids, { mode: "agent" });
        setPointing(payload.ids.length ? { who: pointer(payload.from), ids: payload.ids } : null);
      }),
      events.busCamera.listen(({ payload }) => {
        const v = payload.view as View;
        viewer.current?.setView(v);
        viewNow.current = v;
        setView(v === "iso" || v === "front" || v === "top" ? v : null);
      }),
      events.busRender.listen(async ({ payload }) => {
        const v = viewer.current;
        try {
          if (!v) throw new Error("the viewer isn't ready");
          if (payload.scene && payload.scene.generation > shown.current) {
            const scene = payload.scene;
            setUi((u) => ({ ...u, scene, error: null }));
            // As a run's new build does, the agent's shows whole.
            setStep(null);
            await show(scene, null, "build");
          }
          // Only the ids the request names are highlighted in the render, whatever the window shows.
          const { png, camera } = await v.render({ view: payload.view as View, size: payload.size, highlight: payload.highlight });
          await commands.busReply(payload.id, base64(png), json(camera), null);
        } catch (e) {
          await commands.busReply(payload.id, null, null, e instanceof Error ? e.message : String(e));
        }
      }),
      events.menuAction.listen(({ payload }) => {
        // Open Recent's items name their place in the list; a project that can't open says why.
        const recent = /^recent:(\d+)$/.exec(payload.id);
        if (!recent) return run(payload.id as Action, "menu");
        void commands.appInfo().then((i) => {
          const r = i.recent[Number(recent[1])];
          if (r) void act(commands.openRecent(r.path));
        });
      }),
    ];
    void commands.getPrefs().then(setPrefs);
    // Only once every listener is in place: opening the last project sends ProjectOpened, which a window still
    // registering its listeners would miss, and stay empty (a dev build's restart showed it). A dev build mounts twice,
    // so only the mount still in place goes on.
    let live = true;
    void Promise.all(off)
      .then(() => Promise.all([commands.appInfo(), commands.uiState()] as const))
      .then(([i, s]) => {
        if (!live) return;
        setInfo(i);
        root.current = s.project?.root ?? null;
        setUi(s);
        void show(s.scene ?? null, root.current);
        // With no project open: first run, or Start once the parts and Python are set up; else the last project.
        if (!s.project) {
          // A last project that can't open says why over Start.
          if (i.library && i.python && i.recent[0]) void act(commands.openRecent(i.recent[0].path)).then((p) => p == null && setFirstRun("start"));
          else setFirstRun(i.library && i.python ? "start" : "welcome");
        }
        // Interactive: the first frame after this one has drawn with its handlers in place.
        requestAnimationFrame(() => setTimeout(() => void commands.appInfo().then((n) => setStartupMs(n.uptime_ms)), 0));
      });
    const net = () => setOnline(navigator.onLine);
    window.addEventListener("online", net);
    window.addEventListener("offline", net);
    return () => {
      live = false;
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

  // A click in the viewer picks a part (shift, ⌘, or Ctrl adds it to the pick, or takes it away), a click on nothing
  // clears what's
  // picked, and ⌥-drag's box picks the parts in it: the agent sees them as it sees a row picked in the panel.
  useEffect(() => {
    const v = viewer.current;
    if (!v) return;
    const off = [
      v.on("pick", (id, { add }) => {
        const now = latestPick.current;
        if (!id) return now.key && now.highlight(null, []);
        const ids = add && now.key === "pick" ? (now.ids.includes(id) ? now.ids.filter((x) => x !== id) : [...now.ids, id]) : [id];
        now.highlight(ids.length ? "pick" : null, ids);
      }),
      v.on("box", (ids) => latestPick.current.highlight(ids.length ? "pick" : null, ids)),
    ];
    return () => off.forEach((f) => f());
  }, []);

  const highlight = (key: string | null, ids: string[], mode: "issue" | "select" = "select") => {
    setSelected(key);
    setPointing(null);
    viewer.current?.highlight(key ? ids : [], { mode });
    picked.current = key ? ids : [];
    tellHost();
  };
  const toggle = (key: string, ids: string[], mode: "issue" | "select" = "select") => highlight(selected === key ? null : key, ids, mode);
  const latestPick = useRef({ key: selected, ids: picked.current, highlight });
  latestPick.current = { key: selected, ids: picked.current, highlight };

  const scene = ui.scene;
  const project = ui.project;
  const pointed = pointing && scene ? litBy(scene, pointing.ids) : [];
  // The one tip to teach now: on the tab where it's useful, the first time it is, and never two at once.
  const tip: TipId | null = (() => {
    if (!tipsSeen || !project || !scene || firstRun || agentBusy) return null;
    const unseen = (t: TipId) => !tipsSeen.includes(t);
    if (tab === "checks" && unseen("finding") && (scene.checks?.findings.length ?? 0) > 0) return "finding";
    if (tab === "options" && unseen("knob") && scene.parts > 1 && !project.imported) return "knob";
    if (tab === "parts" && unseen("export") && scene.parts > 1) return "export";
    return null;
  })();
  // A finding that a new build no longer has stops being highlighted.
  const findings = scene?.checks?.findings;
  useEffect(() => {
    if (selected?.startsWith("finding:") && !findings?.some((f) => `finding:${f.id}` === selected)) highlight(null, []);
  }, [findings, selected]);

  // What the viewer shows: a version opened to compare, or the model, whole or built up to a step (the viewer hides the
  // later steps, so a sub-assembly's parts show at the step that places the copy).
  useEffect(() => {
    const v = viewer.current;
    if (!v || !scene) return;
    if (compare) {
      v.showSteps(null);
      void showIn(v, placements(compare.scene), "none");
    } else {
      void showIn(v, placements(scene), "none");
      v.showSteps(step);
    }
  }, [compare, step]);
  useEffect(() => viewer.current?.cutAway(cut), [cut]);
  const layers = viewer.current && scene ? viewer.current.layers() : 1;

  /** Screenshot: a picture of what the viewer shows, onto the clipboard. The clipboard item is made inside the click,
   * as WebKit asks, with the picture still being drawn. */
  const copyShot = () => {
    const v = viewer.current;
    if (!v) return;
    const png = v.snapshot(2);
    const save = { label: "Save to file", run: () => void saveShot(png) };
    // The clipboard refusing offers Save; the picture itself failing doesn't, since Save would fail the same way.
    const failed = (e: unknown) =>
      png.then(
        () => say("Emmet couldn't copy the picture. You can save it to a file instead.", save),
        () => say(`Emmet couldn't take the picture: ${e instanceof Error ? e.message : String(e)}`),
      );
    try {
      const item = new ClipboardItem({ "image/png": png.then((b) => new Blob([b as BlobPart], { type: "image/png" })) });
      navigator.clipboard.write([item]).then(() => say("Copied a picture of the model", save), failed);
    } catch (e) {
      void failed(e);
    }
  };
  const saveShot = async (png?: Promise<Uint8Array>) => {
    const v = viewer.current;
    if (!v) return;
    let bytes: Uint8Array;
    try {
      bytes = await (png ?? v.snapshot(2));
    } catch (e) {
      say(`Emmet couldn't take the picture: ${e instanceof Error ? e.message : String(e)}`);
      return;
    }
    const e = await act(commands.saveScreenshot(base64(bytes)));
    if (e) say(`Saved “${e.file}” to ${e.folder}`, { label: "Show in folder", run: () => void act(commands.revealExport()) });
  };

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
    tipSeen("export");
    const e = await act(commands.export(f));
    if (e) say(`Saved “${e.file}” to ${e.folder}`, { label: "Show in folder", run: () => void act(commands.revealExport()) });
  };

  // Export → Instructions → PDF (4E): the host says what to draw, a hidden viewer draws it here, and the host lays
  // the book out and writes it into Downloads, on the paper this locale uses.
  const drawing = useRef(false);
  const exportInstructions = async () => {
    // Say why a click does nothing.
    if (drawing.current) return say("Emmet is already drawing the instructions.");
    if (!scene) return say("Build the model first: the instructions are drawn from it.");
    drawing.current = true;
    tipSeen("export");
    try {
      const jobs = await act(commands.instructionsJobs());
      if (!jobs) return;
      say("Drawing the instructions");
      let shown = 0;
      let drawn;
      try {
        drawn = await drawBook(placements(scene), jobs, (done, total) => {
          // A new toast every tenth, not every picture.
          if (done - shown >= total / 10) say(`Drawing the instructions: ${Math.round((100 * (shown = done)) / total)} %`);
        });
      } catch (e) {
        say(`Emmet couldn't draw the instructions: ${e instanceof Error ? e.message : String(e)}`);
        return;
      }
      const e = await act(commands.exportInstructions(navigator.language, jobs, drawn));
      if (e) say(`Saved “${e.file}” to ${e.folder}`, { label: "Show in folder", run: () => void act(commands.revealExport()) });
    } finally {
      drawing.current = false;
    }
  };

  const can = (a: Action): boolean => {
    if (["help:guide"].includes(a)) return false;
    if (["agent:switch", "agent:model", "undo-turn", "agent:usage", "agent:fresh", "history"].includes(a)) return !!project && !firstRun;
    // First run covers the window: what's behind it (the viewer, the panel, the chat) takes no keys.
    if (firstRun && !COVERED_OK.includes(a)) return false;
    if (a === "replay") return !!project && !!scene && pace !== "off";
    if (a === "cut:up" || a === "cut:off") return !!project && cut != null;
    if (["save", "versions", "export", "frame", "step:prev", "step:next", "cut:down", "screenshot:copy", "screenshot:save"].includes(a) || a.startsWith("view:") || a.startsWith("tab:") || a.startsWith("orbit:") || a.startsWith("zoom:")) return !!project;
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
      viewNow.current = v;
      tellHost();
    } else if (a.startsWith("tab:")) setTab(a.slice(4) as Tab);
    else
      switch (a) {
        case "new":
          // New is a new model from a prompt: its dialog, not the start screen again (Barry 2026-11-09). Until the parts
          // and Python are set up, the start screen says they're getting ready.
          if (ready) setDialog("new-model");
          else setFirstRun("start");
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
          void commands
            .diagnostics()
            .then((t) => navigator.clipboard.writeText(t))
            .then(
              () => say("Copied the diagnostics"),
              () => say("Emmet couldn't copy the diagnostics. Try Copy diagnostics in Settings."),
            );
          break;
        case "help:report":
          void commands.reportProblem();
          break;
        case "frame":
          viewer.current?.frame(selected ? viewer.current.highlighted() : undefined);
          break;
        case "orbit:left":
        case "orbit:right":
        case "orbit:up":
        case "orbit:down": {
          const turn = Math.PI / 12;
          const d = { "orbit:left": [-turn, 0], "orbit:right": [turn, 0], "orbit:up": [0, turn], "orbit:down": [0, -turn] }[a] as [number, number];
          viewer.current?.orbit(d[0], d[1]);
          break;
        }
        case "zoom:in":
          viewer.current?.zoom(0.8);
          break;
        case "zoom:out":
          viewer.current?.zoom(1.25);
          break;
        case "cut:down":
          setCut((c) => Math.max(1, (c ?? viewer.current?.layers() ?? 1) - 1));
          break;
        case "cut:up":
          setCut((c) => (c == null || c + 1 > (viewer.current?.layers() ?? 1) ? null : c + 1));
          break;
        case "cut:off":
          setCut(null);
          break;
        case "replay":
          // The whole model plays: the steps and the cut-away go first, in the viewer at once (their effects come
          // after this render, and would end the replay).
          setStep(null);
          setCut(null);
          viewer.current?.showSteps(null);
          viewer.current?.cutAway(null);
          viewer.current?.replayBuild();
          break;
        case "screenshot:copy":
          copyShot();
          break;
        case "screenshot:save":
          void saveShot();
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
          document.getElementById("start-fresh")?.click();
          break;
        case "agent:usage":
          document.getElementById("plan-badge")?.click();
          break;
        case "agent:switch":
          document.getElementById("agent-picker")?.dispatchEvent(new PointerEvent("pointerdown", { bubbles: true }));
          break;
        case "agent:model":
          document.getElementById("model-chip")?.click();
          break;
        case "undo-turn":
          document.getElementById("undo-turn")?.click();
          break;
        case "history":
          document.getElementById("history-toggle")?.click();
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
      // Orbit and zoom are the viewer's keys: elsewhere the arrows and minus are a control's own (the tree, a slider).
      if ((a.startsWith("orbit:") || a.startsWith("zoom:")) && !document.getElementById("viewer")?.contains(document.activeElement)) return;
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
    if (info?.library_old && !libraryLater) {
      const updating = setup.at === "library" || setup.at === "python";
      // An update that stops says why, and Try again picks up where it stopped.
      const stopped = setup.at === "failed" ? setup : null;
      const why = stopped && (stopped.python ? `${stopped.problem.message} ${stopped.problem.hint}`.trim() : libraryFailure(stopped.problem, stopped.done, stopped.total));
      return (
        <Banner
          kind="old-library"
          icon="download"
          title={`Your parts library is ${info.library}. This Emmet expects ${info.pinned}.`}
          text={why ?? "Some parts may be missing or renamed until you update (about 150 MB, a minute or two)."}
          actions={
            <>
              <Button label="Later" variant="quiet" size="sm" onClick={() => setLibraryLater(true)} />
              <Button label={updating ? "Updating" : stopped ? "Try again" : "Update the library"} variant="primary" size="sm" id="update-library" busy={updating} onClick={() => void startSetup()} />
            </>
          }
        />
      );
    }
    if (project.imported && scene)
      return (
        <Banner
          kind="imported"
          icon="file-up"
          title={`Opened ${project.imported}: ${count(scene.parts, "part")} in ${count(scene.tree?.steps.length ?? scene.steps, "step")}.`}
          actions={<Button label="Make it parametric" variant="primary" size="sm" id="make-parametric" onClick={() => sendToAgent("import", "Make this model parametric, with a few knobs that change its character.", `Rebuild ${project.imported} in model.py with emmet_dsl, keeping its parts, colors, and steps, with two to five knobs that change its character.`)} />}
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
            <section className="viewer" data-c="viewer" aria-label={`3D view of ${name}`} id="viewer" tabIndex={-1} ref={viewerBox} data-photo={photo ? "" : undefined}>
              {photo && <Compare path={photo} onClose={closePhoto} />}
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
                        // The version's build settings, so the panel doesn't write the ones before over them.
                        const now = await commands.uiState();
                        setUi((u) => ({ ...u, project: now.project }));
                      }
                    }}
                  />
                  <Button label="Open as new project" size="sm" id="version-open" onClick={() => void act(commands.openVersionAsNew(compare.v.n))} />
                  <Button label="Back" variant="quiet" size="sm" id="version-back" onClick={() => setCompare(null)} />
                </div>
              )}
              {pointing && !compare && (
                <div className="viewer-bar" role="status" id="pointing-bar">
                  <Icon name="eye" />
                  <span>
                    {pointed.length ? (
                      <>
                        <b>{pointing.who}</b> is pointing at {pointed.length} {pointed.length === 1 ? "part" : "parts"}
                      </>
                    ) : (
                      <>
                        <b>{pointing.who}</b> pointed at parts this build doesn't have
                      </>
                    )}
                  </span>
                  {pointed.length > 0 && <Button label="Zoom to them" size="sm" icon="scan" id="pointing-zoom" onClick={() => viewer.current?.frame(pointed)} />}
                  <Button label="Clear" variant="quiet" size="sm" id="pointing-clear" onClick={() => highlight(null, [])} />
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
              <ViewerLayer
                viewer={viewer.current}
                scene={compare ? null : scene}
                step={compare ? null : step}
                cut={cut}
                layers={layers}
                animating={animating}
                findings={findings ?? []}
                showPins={!compare && !photo && step == null}
                onPin={(f) => {
                  tipSeen("finding");
                  setTab("checks");
                  highlight(`finding:${f.id}`, f.parts, "issue");
                }}
                onCut={setCut}
                onSkip={() => viewer.current?.skipAnimation()}
              />
              <Toast.Viewport className="toast-viewport" aria-label="Notifications" />
            </section>
            {project && (
              <DockBoundary>
              <Dock
                project={project}
                ask={ask}
                buildError={ui.error ? { problem: ui.error, agent: ui.error.code.startsWith("MODEL_") || ui.error.code === "PARAM_INVALID", shown: true } : null}
                onAskBuild={() => sendToAgent("build", "The model didn't build. Fix the mistake in the last change, then run the model again.", buildFixRequest(ui.error!))}
                onRebuild={() => void act(commands.rebuild())}
                agentsChanged={agentsChanged}
                context={context}
                onSettings={() => setDialog("settings")}
                onAgent={setChatKind}
                onBusy={setAgentBusy}
                onUndone={(saved, who) => say(`Undid ${who}'s last turn. The model before is version ${saved.n}.`)}
                onPhoto={setPhoto}
              />
              </DockBoundary>
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
              tab={tab}
              onTab={setTab}
              selected={selected}
              step={step}
              sub={sub}
              exportOpen={exportOpen}
              onExportOpen={setExportOpen}
              onExport={(f) => void exportAs(f)}
              onInstructions={() => void exportInstructions()}
              onPickFinding={(f: Finding) => {
                tipSeen("finding");
                toggle(`finding:${f.id}`, f.parts, "issue");
              }}
              onAsk={(f: Finding) => {
                tipSeen("finding");
                sendToAgent("checks", `Fix this: ${findingWords(f)}`, fixRequest(f));
              }}
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
              onRebuild={(before, after) => sendToAgent("settings", rebuildWords(before, after), rebuildRequest(before, after))}
              onKnobWrite={() => {
                knobRun.current = true;
                tipSeen("knob");
              }}
              tip={tip}
              onTip={tipSeen}
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
                  // A new model from a prompt asks for its name and build settings first (Barry 2026-11-09).
                  if (kind === "empty") return setDialog("new-model");
                  setCreating(kind);
                  if (!(await act(commands.newProject(kind, null, null)))) setCreating(null);
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
          {dialog === "settings" && <Settings info={info} prefs={prefs} autoQuality={autoQuality} onPrefs={setPrefs} onClose={() => setDialog(null)} onAgentsChanged={() => setAgentsChanged((n) => n + 1)} onCopied={(copied) => say(copied ? "Copied the diagnostics" : "Emmet couldn't copy the diagnostics to the clipboard.")} />}
          {dialog === "about" && <About info={info} theme={theme} onClose={() => setDialog(null)} />}
          {dialog === "shortcuts" && <Shortcuts onClose={() => setDialog(null)} />}
          {dialog === "new-model" && (
            <NewModel
              onClose={() => setDialog(null)}
              onCreated={() => {
                setDialog(null);
                setTimeout(() => document.getElementById("chat-input")?.focus(), 0);
              }}
            />
          )}
        </div>
      </Toast.Provider>
    </Tooltip.Provider>
  );
}

/** A count with its noun: "1 step", "31 steps". */
const count = (n: number, noun: string) => `${n.toLocaleString("en-US")} ${noun}${n === 1 ? "" : "s"}`;

/** What the failed build's "Ask agent to fix" sends: the error as emmet reported it, with its file and line (the agent
 * sees them; the screen doesn't, law 2). */
function buildFixRequest(e: Problem): string {
  const at = e.file ? ` at ${e.file}${e.line ? `:${e.line}` : ""}` : "";
  return `The model didn't build (${e.code}${at}): ${e.message} ${e.hint} Fix it, then run the model again.`.replace(/\s+/g, " ");
}
