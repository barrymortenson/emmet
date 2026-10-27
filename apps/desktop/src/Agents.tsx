// SPDX-License-Identifier: AGPL-3.0-or-later
// Agent setup without a terminal (D-41, session 1F; 4A restyles it): for each agent, which plans work, with the
// provider's page; Install, which runs the provider's own installer in the host, with its progress, Cancel, and one
// Try again; and Sign in: Claude Code's own sign-in in the browser, or Settings → Agents for Codex (1E's Continue with
// ChatGPT and API key live there, once, so the window never has two copies of them). The
// provider notice (D-37) comes once, before the first install or sign-in. Antigravity works in terminal mode (D-42):
// one click adds Emmet's tools to the project, and the user runs agy there.
import { useEffect, useRef, useState } from "react";
import { type AgentKind, type AgentView, type AntigravityView, commands, events, type SetupError } from "./bindings";

const windows = navigator.userAgent.includes("Windows");
const terminal = windows ? "PowerShell" : "Terminal";

/** Which plans work, from each provider's page (docs/terms/quotes.json, checked 2026-10-26). No prices. */
const PLANS: Record<AgentKind, string> = {
  claude:
    "Claude Code needs a Claude Pro, Max, Team, or Enterprise plan, or a Claude Console account. The free Claude plan doesn't include it.",
  codex: "Codex comes with every ChatGPT plan, Free included, or runs on an OpenAI API key.",
};

/** Asks for the provider notice first, once; then runs `action`. */
type Gate = (action: () => void) => void;

/** Every agent's setup card; `only` limits it to the agents that aren't ready yet. */
export function AgentSetup({
  agents,
  onChanged,
  onSettings,
  only,
}: {
  agents: AgentView[] | null;
  onChanged: () => void;
  onSettings: () => void;
  only?: "unready";
}) {
  const [pending, setPending] = useState<(() => void) | null>(null);

  // Asked at each click, not remembered: the start screen and the picker each have these cards.
  const gate: Gate = (action) => {
    void commands.providerNotice().then((seen) => (seen ? action() : setPending(() => action)));
  };
  const shown = (agents ?? []).filter((a) => only !== "unready" || !(a.detection.path != null && a.detection.signed_in));
  if (!agents) return <p className="muted">Looking for Claude Code and Codex…</p>;
  return (
    <div className="agent-setup" role="group" aria-label="Agent setup">
      {shown.map((a) => (
        <AgentCard key={a.kind} view={a} gate={gate} onChanged={onChanged} onSettings={onSettings} />
      ))}
      {pending && (
        <ProviderNotice
          onContinue={async () => {
            await commands.providerNoticeSeen();
            const go = pending;
            setPending(null);
            go();
          }}
          onCancel={() => setPending(null)}
        />
      )}
    </div>
  );
}

/** D-37's notice, in plain words, shown once before the first install or sign-in. */
function ProviderNotice({ onContinue, onCancel }: { onContinue: () => void; onCancel: () => void }) {
  const dialog = useRef<HTMLDialogElement | null>(null);
  useEffect(() => dialog.current?.showModal(), []);
  return (
    <dialog ref={dialog} className="settings" id="provider-notice" aria-labelledby="provider-notice-title" onCancel={onCancel}>
      <h2 id="provider-notice-title">Before you set up an agent</h2>
      <p>
        Emmet runs your own Claude Code or Codex on your own account. Installing runs the provider's own installer, and
        you sign in on the provider's own page: Emmet never sees your password or your sign-in.
      </p>
      <p>
        Emmet follows each provider's policies as we read them, but Anthropic, OpenAI, and Google set and change their
        own terms and can act on accounts in ways Emmet doesn't control, so use is at your own risk. How your agent
        handles your data is between you and its provider.
      </p>
      <button type="button" className="primary" id="provider-notice-ok" onClick={onContinue}>
        Continue
      </button>{" "}
      <button type="button" onClick={onCancel}>
        Not now
      </button>
    </dialog>
  );
}

type Job = { what: "install" | "sign-in"; console: boolean; started: number; line: string | null };

function AgentCard({
  view,
  gate,
  onChanged,
  onSettings,
}: {
  view: AgentView;
  gate: Gate;
  onChanged: () => void;
  onSettings: () => void;
}) {
  const { kind, name, detection } = view;
  const [job, setJob] = useState<Job | null>(null);
  const [failed, setFailed] = useState<{ job: Job; error: SetupError } | null>(null);
  const [, tick] = useState(0);

  useEffect(() => {
    const off = events.setupEvent.listen((e) => {
      if (e.payload.kind === kind) setJob((j) => j && { ...j, line: e.payload.line });
    });
    return () => void off.then((f) => f());
  }, [kind]);
  useEffect(() => {
    if (!job) return;
    const t = setInterval(() => tick((n) => n + 1), 1000);
    return () => clearInterval(t);
  }, [job]);

  const run = (what: Job["what"], console = false) =>
    gate(async () => {
      const j: Job = { what, console, started: Date.now(), line: null };
      setFailed(null);
      setJob(j);
      const r = what === "install" ? await commands.agentInstall(kind) : await commands.claudeSignIn(console);
      setJob(null);
      if (r.status === "error") setFailed({ job: j, error: r.error });
      onChanged();
    });

  const installed = detection.path != null;
  const ready = installed && detection.signed_in;
  const seconds = job ? Math.floor((Date.now() - job.started) / 1000) : 0;
  const elapsed = `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;

  return (
    <section className="agent-card" id={`setup-${kind}`} aria-labelledby={`setup-${kind}-title`}>
      <h3 id={`setup-${kind}-title`}>{name}</h3>
      {ready && (
        <p className="ready" id={`ready-${kind}`}>
          Ready · {view.badge}
        </p>
      )}
      {!ready && !job && (
        <p className="muted plans">
          {PLANS[kind]}{" "}
          <button type="button" className="link" id={`plans-${kind}`} onClick={() => void commands.openPlans(kind)}>
            Which plans work
          </button>
        </p>
      )}
      {!installed && !job && kind === "claude" && windows && (
        <p className="muted">On Windows, Claude Code runs its commands in PowerShell, so nothing else is needed.</p>
      )}

      {job && (
        <div className="job" role="status" id={`job-${kind}`}>
          {job.what === "install" ? (
            <p>
              Installing {name} with {kind === "claude" ? "Anthropic" : "OpenAI"}'s installer… {elapsed}
            </p>
          ) : (
            <p>Finish signing in in your browser. Emmet is waiting for Claude Code… {elapsed}</p>
          )}
          {job.line && <p className="muted job-line">{job.line}</p>}
          <button type="button" id={`cancel-${kind}`} onClick={() => void commands.agentCancel(kind)}>
            Cancel
          </button>
          {job.what === "sign-in" && <TerminalSignIn />}
        </div>
      )}

      {failed && !job && (
        <div className="failed" role="alert" id={`failed-${kind}`}>
          <p>{failed.error.message}</p>
          {failed.error.detail && (
            <details>
              <summary>What {failed.job.what === "install" ? "the installer" : "Claude Code"} said</summary>
              <pre>{failed.error.detail}</pre>
            </details>
          )}
          <button type="button" className="primary" id={`retry-${kind}`} onClick={() => run(failed.job.what, failed.job.console)}>
            Try again
          </button>
          {failed.job.what === "sign-in" && <TerminalSignIn />}
        </div>
      )}

      {!installed && !job && !failed && (
        <button type="button" className="primary" id={`install-${kind}`} onClick={() => run("install")}>
          Install {name}
        </button>
      )}

      {installed && !ready && !job && !failed && kind === "claude" && (
        <>
          <button type="button" className="primary" id="sign-in-claude" onClick={() => run("sign-in")}>
            Sign in to Claude
          </button>{" "}
          <button type="button" className="link" id="sign-in-console" onClick={() => run("sign-in", true)}>
            Use a Claude Console account instead
          </button>
        </>
      )}
      {installed && !ready && !job && !failed && kind === "codex" && (
        <>
          <button type="button" className="primary" id="sign-in-codex" onClick={() => gate(onSettings)}>
            Sign in to Codex
          </button>
          <p className="muted">Settings opens with Continue with ChatGPT, or a place for an OpenAI API key.</p>
        </>
      )}
      {detection.note && <p className="muted">{detection.note}</p>}
    </section>
  );
}

/** The fallback when the browser shows a code to paste: Emmet never takes it (docs/decisions.md, 2026-10-26). */
function TerminalSignIn() {
  return (
    <div className="terminal-sign-in" id="terminal-sign-in">
      <p className="muted">
        Does your browser show a code to paste? Emmet can't take it. Cancel here, then sign in from {terminal}: type{" "}
        <code>claude</code> and press Enter, then type <code>/login</code> and follow the steps.
      </p>
      <button type="button" id="open-terminal" onClick={() => void commands.openTerminal(false)}>
        Open {terminal}
      </button>
    </div>
  );
}

/** Antigravity in terminal mode (D-42): agy runs in the user's terminal, and Emmet shows the model live. `root` is the
 * open project, whose `.agents/mcp_config.json` the card reports on. */
export function AntigravityCard({ root }: { root: string }) {
  const [view, setView] = useState<AntigravityView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const load = () => void commands.antigravity().then(setView);
  useEffect(load, [root]);
  if (!view) return null;
  const { path, version } = view.found;
  return (
    <details className="agent-card antigravity" id="antigravity">
      <summary>Antigravity (terminal mode)</summary>
      {!path ? (
        <>
          <p className="muted">
            Emmet didn't find Antigravity's agy. To use it, install the Antigravity CLI from Google's Antigravity site,
            run agy once to sign in with your Google account, then look again.
          </p>
          <button type="button" id="antigravity-look" onClick={load}>
            Look again
          </button>
        </>
      ) : (
        <>
          <p>
            Antigravity{version ? ` ${version}` : ""} works in terminal mode: it runs in {terminal}, on your own Google
            sign-in, and Emmet shows the model live as it builds.
          </p>
          {view.configured ? (
            <p className="muted" id="antigravity-ready">
              Emmet's tools are in this project's .agents/mcp_config.json, not in your Antigravity settings.
            </p>
          ) : (
            <button
              type="button"
              className="primary"
              id="antigravity-set-up"
              onClick={async () => {
                const r = await commands.antigravitySetUp();
                if (r.status === "error") setError(r.error.message);
                else {
                  setError(null);
                  load();
                }
              }}
            >
              Add Emmet's tools to this project
            </button>
          )}
          {error && (
            <p className="notice" role="alert">
              {error}
            </p>
          )}
          <p className="muted">
            Then open {terminal} in this project and type <code>agy</code>. If agy asks whether to trust the folder, say
            yes, so it loads Emmet's tools.
          </p>
          <button type="button" id="antigravity-terminal" onClick={() => void commands.openTerminal(true)}>
            Open {terminal} here
          </button>
        </>
      )}
    </details>
  );
}
