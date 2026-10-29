// SPDX-License-Identifier: AGPL-3.0-or-later
// First run (plan section 14's key flow, docs/design/frozen): one card over the model with the wordmark and a rail of
// its four steps. Welcome; Parts and Python (the library download with its progress, then Python; Continue while it
// downloads; a stopped download says where it stopped, and Try again picks up there); Your agent (1F's setup, no
// terminal); Start (the Cottage example or a new model from a prompt, into the projects folder). With the parts and
// Python already set up, and no project open, the window starts at Start.
import { useEffect, useRef, useState } from "react";
import { AgentSetup } from "./Agents";
import { type AgentView, commands, events, type NewKind, type Problem } from "./bindings";
import { windows } from "./keys";
import { CaptionButtons } from "./Titlebar";
import { AlertCard, Button, Icon, mb, Progress } from "./ui";
import antDark from "./assets/brand/wordmark-emmet-builder-ant-dark.svg";
import antLight from "./assets/brand/wordmark-emmet-builder-ant-light.svg";
import cottageDark from "./assets/renders/dark/cottage.webp";
import emptyDark from "./assets/renders/dark/empty.webp";
import stage1Dark from "./assets/renders/dark/stage1.webp";
import stage2Dark from "./assets/renders/dark/stage2.webp";
import cottageLight from "./assets/renders/light/cottage.webp";
import emptyLight from "./assets/renders/light/empty.webp";
import stage1Light from "./assets/renders/light/stage1.webp";
import stage2Light from "./assets/renders/light/stage2.webp";

export type Step = "welcome" | "library" | "agents" | "start";
const STEPS: [Step, string][] = [
  ["welcome", "Welcome"],
  ["library", "Parts and Python"],
  ["agents", "Your agent"],
  ["start", "Start"],
];
const RENDERS = {
  light: { cottage: cottageLight, stage1: stage1Light, stage2: stage2Light, empty: emptyLight },
  dark: { cottage: cottageDark, stage1: stage1Dark, stage2: stage2Dark, empty: emptyDark },
};
const BACKDROP: Record<Step, keyof (typeof RENDERS)["light"]> = { welcome: "cottage", library: "stage1", agents: "stage2", start: "empty" };
/** The pinned library's size, for the lede before the download knows it. */
const LIBRARY_MB = 150;

/** What setup is doing: not started, the library (bytes so far), Python, done, or where it stopped. */
type Setup =
  | { at: "idle" }
  | { at: "library"; done: number; total: number | null }
  | { at: "python" }
  | { at: "done" }
  | { at: "failed"; done: number; total: number | null; problem: Problem; python: boolean };

/** The setup's progress, kept for the window's life: first run and the old library's banner share it. */
export function useSetup(onDone: () => void) {
  const [setup, setSetup] = useState<Setup>({ at: "idle" });
  const last = useRef<{ done: number; total: number | null }>({ done: 0, total: null });
  useEffect(() => {
    const off = events.partsProgress.listen(({ payload }) => {
      if (payload.step === "library") {
        last.current = { done: payload.done, total: payload.total };
        setSetup({ at: "library", done: payload.done, total: payload.total });
      } else setSetup({ at: "python" });
    });
    return () => void off.then((f) => f());
  }, []);
  const start = async () => {
    setSetup((s) => (s.at === "failed" ? { at: "library", ...last.current } : { at: "library", done: 0, total: null }));
    const r = await commands.setup().catch((e: unknown) => ({ status: "error" as const, error: { code: "INTERNAL", message: String(e), hint: "", file: null, line: null, excerpt: null } }));
    if (r.status === "ok") {
      setSetup({ at: "done" });
      onDone();
    } else {
      setSetup((s) => ({ at: "failed", ...last.current, problem: r.error, python: s.at === "python" }));
    }
  };
  return { setup, start };
}

export function FirstRun({
  step,
  onStep,
  setup,
  onSetup,
  ready,
  theme,
  projectsFolder,
  agents,
  onAgentsChanged,
  onSettings,
  onNew,
  onOpen,
  creating,
}: {
  step: Step;
  onStep: (s: Step) => void;
  setup: Setup;
  onSetup: () => void;
  /** The library and Python are set up. */
  ready: boolean;
  theme: "light" | "dark";
  projectsFolder: string;
  agents: AgentView[] | null;
  onAgentsChanged: () => void;
  onSettings: () => void;
  onNew: (kind: NewKind) => void;
  onOpen: () => void;
  creating: NewKind | null;
}) {
  const at = STEPS.findIndex(([id]) => id === step);
  const r = RENDERS[theme];
  let body: React.ReactNode;
  if (step === "welcome") {
    body = (
      <>
        <h1>Build brick models by describing them</h1>
        <p className="lede">
          Emmet Builder works with your own Claude Code or Codex. You say what to build; your agent writes a small Python
          model from real LDraw parts, and Emmet shows it, checks it, and gives you a parts list and building steps.
        </p>
        <ul className="feature-list">
          <li>
            <Icon name="brick" size="lg" />
            Real parts, in colors they're actually made in
          </li>
          <li>
            <Icon name="circle-check" size="lg" />
            Checks for parts that collide or float
          </li>
          <li>
            <Icon name="sliders-horizontal" size="lg" />
            Pick a style, scale, and size instead of placing 400 bricks
          </li>
        </ul>
        <div className="row">
          <Button
            label="Get started"
            variant="primary"
            size="lg"
            id="get-started"
            onClick={() => {
              onStep("library");
              if (!ready && setup.at === "idle") onSetup();
            }}
          />
        </div>
        <p className="small">Your models and chats stay on this computer. Emmet has no accounts and no tracking.</p>
      </>
    );
  } else if (step === "library") {
    const failed = setup.at === "failed" ? setup : null;
    const lib = setup.at === "library" || failed ? (setup as { done: number; total: number | null }) : null;
    const total = lib?.total ?? LIBRARY_MB * 1_000_000;
    const libDone = ready || setup.at === "python" || setup.at === "done" || (failed?.python ?? false);
    const pct = libDone ? 100 : lib ? Math.min(100, Math.round((lib.done / total) * 100)) : 0;
    const amount = libDone ? "Done" : lib ? `${failed ? "Stopped at " : ""}${mb(lib.done)} of ${mb(total)} MB` : "Waiting";
    const python = ready || setup.at === "done" ? 100 : 0;
    const pythonWords = ready || setup.at === "done" ? "Done" : setup.at === "python" ? "Setting up" : failed?.python ? "Stopped" : "Next";
    body = (
      <>
        <h1>Getting the parts ready</h1>
        <p className="lede">
          Downloading the LDraw parts library (about {LIBRARY_MB} MB) and setting up Python. This takes a minute or two, and
          only happens once.
        </p>
        <Progress label="LDraw parts library" pct={pct} amount={amount} error={!!failed && !failed.python} id="library-progress" />
        <Progress label="Python for models" pct={python} amount={pythonWords} error={!!failed?.python} id="python-progress" />
        {failed && (
          <AlertCard
            tone="bad"
            icon="wifi-off"
            id="setup-failed"
            title={failed.python ? "Python didn't set up" : "The download stopped"}
            text={
              failed.python
                ? `${failed.problem.message} The parts library is ready, and Try again sets Python up.`
                : `The connection dropped at ${mb(failed.done)} of ${mb(failed.total ?? LIBRARY_MB * 1_000_000)} MB. Emmet picks up where it stopped.`
            }
            actions={<Button label="Try again" variant="primary" icon="refresh-cw" id="setup-retry" onClick={onSetup} />}
          />
        )}
        {!failed && (
          <>
            <p className="small">While you wait: set up your agent on the next screen. Everything keeps downloading.</p>
            <div className="row">
              <Button label="Continue" variant="primary" size="lg" id="library-continue" onClick={() => onStep("agents")} />
            </div>
          </>
        )}
      </>
    );
  } else if (step === "agents") {
    body = (
      <>
        <h1>Set up your agent</h1>
        <p className="lede">Emmet builds with your own Claude Code or Codex. Set up either one here, no terminal needed.</p>
        <AgentSetup agents={agents} onChanged={onAgentsChanged} onSettings={onSettings} cards />
        <div className="notice">
          <Icon name="info" />
          <p>
            Claude Code and Codex are made by Anthropic and OpenAI, not by Emmet. Your plan and its limits are between you and
            them, and you use them with Emmet at your own risk.
          </p>
        </div>
        <div className="row">
          <Button label="Check agents again" variant="quiet" icon="refresh-cw" id="first-run-agents-again" onClick={onAgentsChanged} />
          <span className="grow" />
          <Button label="Continue" variant="primary" size="lg" id="agents-continue" onClick={() => onStep("start")} />
        </div>
      </>
    );
  } else {
    const folder = windows ? projectsFolder.replaceAll("/", "\\") : projectsFolder;
    const waiting = !ready && setup.at !== "done";
    body = (
      <>
        <h1>What do you want to build?</h1>
        <p className="lede">New projects go in {folder}. You can change that in Settings.</p>
        <div className="start-cards">
          <button type="button" className="start-card" id="start-cottage" disabled={waiting || creating != null} onClick={() => onNew("cottage")}>
            <span className="viewer" data-c="viewer">
              <img className="viewer-render" src={r.cottage} alt="" />
            </span>
            <b>Start with the Cottage example</b>
            <span className="small">165 parts: a real door, glass windows, and a room inside</span>
          </button>
          <button type="button" className="start-card" id="start-prompt" disabled={waiting || creating != null} onClick={() => onNew("empty")}>
            <span className="viewer" data-c="viewer">
              <img className="viewer-render" src={r.empty} alt="" />
            </span>
            <b>New model from a prompt</b>
            <span className="small">An empty baseplate and your agent</span>
          </button>
        </div>
        {waiting ? (
          <p className="small" role="status">
            The parts are still getting ready. These start as soon as they are.
          </p>
        ) : (
          <p className="small">
            Or{" "}
            <button type="button" className="link-button" id="start-open" onClick={onOpen}>
              open a folder, or an LDraw file
            </button>{" "}
            from Studio, LDCad, or LeoCAD.
          </p>
        )}
      </>
    );
  }
  return (
    <div className="first-run" data-c="first-run" id="first-run">
      {/* The window moves by its top edge and the model behind the card, as by the title row it covers. */}
      <section className="viewer" data-c="viewer" aria-hidden="true" data-tauri-drag-region>
        <img className="viewer-render" src={r[BACKDROP[step]]} alt="" />
      </section>
      <div className="first-run-drag" data-tauri-drag-region />
      {windows && <CaptionButtons />}
      <section className="first-run-card" aria-label="Set up Emmet">
        <img className="wordmark" src={theme === "dark" ? antDark : antLight} alt="Emmet Builder" />
        <ol className="rail" aria-label="Setup">
          {STEPS.map(([id, label], i) => (
            <li key={id} aria-current={i === at ? "step" : undefined} data-state={i < at ? "done" : i === at ? "current" : "later"}>
              {i < at ? <Icon name="check" size="sm" /> : <b>{i + 1}</b>}
              {label}
            </li>
          ))}
        </ol>
        {body}
      </section>
    </div>
  );
}
