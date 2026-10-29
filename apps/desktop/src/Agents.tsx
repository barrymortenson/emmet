// SPDX-License-Identifier: AGPL-3.0-or-later
// Agent setup without a terminal (D-41, session 1F; 4A restyled it to the frozen setup-card): for each agent, which plans work, with the
// provider's page; Install, which runs the provider's own installer in the host, with its progress, Cancel, and one
// Try again; and Sign in: Claude Code's own sign-in in the browser, or Settings → Agents for Codex (1E's Continue with
// ChatGPT and API key live there, so the window never has two copies of them). The provider notice (D-37) comes before
// an install or sign-in until the user clicks Continue.
import { Dialog } from "radix-ui";
import { useEffect, useState } from "react";
import { type AgentKind, type AgentView, commands, events, type SetupError } from "./bindings";
import { ContinueButton } from "./ChatGpt";
import { Badge, Button, SetupCard } from "./ui";

const windows = navigator.userAgent.includes("Windows");
const terminal = windows ? "PowerShell" : "Terminal";

/** Which plans work, from each provider's page (docs/terms/quotes.json, checked 2026-10-26). No prices. */
const PLANS: Record<AgentKind, string> = {
  claude:
    "Claude Code needs a Claude Pro, Max, Team, or Enterprise plan, or a Claude Console account. The free Claude plan doesn't include it.",
  codex: "Codex comes with every ChatGPT plan, Free included, or runs on an OpenAI API key.",
};

/** Asks the host whether the provider notice was accepted; if not, shows it first, and runs `action` after Continue. */
type Gate = (action: () => void) => void;

/** A host call's failure in words, for the cards' small notices. */
const said = (e: { message: string; hint?: string }) => `${e.message} ${e.hint ?? ""}`.trim();

/** Every agent's setup card; `only` limits it to the agents that aren't ready yet. */
export function AgentSetup({
  agents,
  onChanged,
  onSettings,
  only,
  cards,
}: {
  agents: AgentView[] | null;
  onChanged: () => void;
  onSettings: () => void;
  only?: "unready";
  /** First run: Claude's plans once under the cards, as the frozen screen says them. */
  cards?: boolean;
}) {
  const [pending, setPending] = useState<(() => void) | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Asked at each click, not remembered: the start screen and the picker each have these cards.
  const gate: Gate = (action) => {
    commands
      .providerNotice()
      .then((seen) => (seen ? action() : setPending(() => action)))
      .catch((e: unknown) => setError(String(e)));
  };
  const shown = (agents ?? []).filter((a) => only !== "unready" || !(a.detection.path != null && a.detection.signed_in));
  if (!agents) return <p className="muted">Looking for Claude Code and Codex…</p>;
  return (
    <div className="agent-setup" role="group" aria-label="Agent setup" id={cards ? "agents" : undefined}>
      {shown.map((a) => (
        <AgentCard key={a.kind} view={a} gate={gate} onChanged={onChanged} onSettings={onSettings} />
      ))}
      {cards && <p className="small">{PLANS.claude}</p>}
      {pending && (
        <ProviderNotice
          onContinue={async () => {
            const r = await commands
              .providerNoticeSeen()
              .catch((e: unknown) => ({ status: "error" as const, error: { message: String(e) } }));
            const go = pending;
            setPending(null);
            if (r.status === "error") setError(`Emmet couldn't save that you read the notice: ${said(r.error)}`);
            else go();
          }}
          onCancel={() => setPending(null)}
        />
      )}
      {error && (
        <p className="notice" role="alert" id="setup-error">
          {error}
        </p>
      )}
    </div>
  );
}

/** D-37's notice, in plain words, before an install or sign-in until the user clicks Continue (Not now records nothing). */
function ProviderNotice({ onContinue, onCancel }: { onContinue: () => void; onCancel: () => void }) {
  return (
    <Dialog.Root open onOpenChange={(o) => !o && onCancel()}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" data-c="dialog">
          <Dialog.Content className="dialog" id="provider-notice" aria-describedby={undefined}>
            <Dialog.Title className="dialog-title">Before you set up an agent</Dialog.Title>
            <p>
              Emmet runs your own Claude Code or Codex on your own account. Installing runs the provider's own installer, and
              you sign in on the provider's own page, so Emmet never sees your password. Claude Code keeps its own sign-in;
              for Codex, Emmet keeps your ChatGPT sign-in or API key only in your system's keychain.
            </p>
            <p>
              Emmet follows each provider's policies as we read them, but Anthropic, OpenAI, and Google set and change their
              own terms and can act on accounts in ways Emmet doesn't control, so use is at your own risk. How your agent
              handles your data is between you and its provider.
            </p>
            <div className="dialog-actions">
              <Button label="Not now" variant="quiet" size="sm" onClick={onCancel} />
              <Button label="Continue" variant="primary" size="sm" id="provider-notice-ok" onClick={onContinue} />
            </div>
          </Dialog.Content>
        </Dialog.Overlay>
      </Dialog.Portal>
    </Dialog.Root>
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
  const [note, setNote] = useState<string | null>(null);
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
      try {
        const r = what === "install" ? await commands.agentInstall(kind) : await commands.claudeSignIn(console);
        if (r.status === "error") setFailed({ job: j, error: r.error });
      } catch (e) {
        const message = `Emmet couldn't reach its host: ${String(e)}. Try again.`;
        setFailed({ job: j, error: { failure: "failed", message, detail: null } });
      } finally {
        setJob(null);
        onChanged();
      }
    });
  /** Runs a host call that opens something, and says so if it couldn't. */
  type Opened = { status: "ok" } | { status: "error"; error: { message: string; hint?: string } };
  const open = (call: () => Promise<Opened>) =>
    void call()
      .then((r) => setNote(r.status === "error" ? said(r.error) : null))
      .catch((e: unknown) => setNote(String(e)));

  const installed = detection.path != null;
  const ready = installed && detection.signed_in;
  const seconds = job ? Math.floor((Date.now() - job.started) / 1000) : 0;
  const elapsed = `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;

  const plan = view.badge.split(" · ")[1];
  const text = ready
    ? `Signed in${plan ? ` · ${plan}` : ""}`
    : !installed
      ? PLANS[kind]
      : kind === "codex"
        ? "Installed · not signed in. Sign in with ChatGPT to use your plan, or use an OpenAI API key."
        : `Installed · not signed in. ${PLANS.claude}`;
  const idle = !job && !failed;
  let action: React.ReactNode = null;
  if (ready) action = <Badge text="Ready" id={`ready-${kind}`} />;
  else if (!installed && idle) action = <Button label={`Install ${name}`} variant="primary" id={`install-${kind}`} onClick={() => run("install")} />;
  else if (installed && idle && kind === "claude") action = <Button label="Sign in to Claude" variant="primary" id="sign-in-claude" onClick={() => run("sign-in")} />;
  else if (installed && idle && kind === "codex")
    // Codex's sign-in is 1E's, in Settings: these open it there (Continue with ChatGPT, or a place for an API key).
    action = (
      <span className="row">
        <ContinueButton id="sign-in-codex" onClick={() => gate(onSettings)} />
        <Button label="Use an API key" variant="quiet" size="sm" id="codex-api-key" onClick={() => gate(onSettings)} />
      </span>
    );
  else if (job)
    action = <Button label={job.what === "install" ? "Installing" : "Signing in"} busy disabled />;

  return (
    <SetupCard
      id={`setup-${kind}`}
      ready={ready}
      icon={ready ? "circle-check" : installed ? "log-in" : "download"}
      title={installed && detection.version ? `${name} ${detection.version}` : name}
      text={text}
      action={action}
    >
      {!ready && idle && (
        <p>
          <button type="button" className="link-button" id={`plans-${kind}`} onClick={() => open(() => commands.openPlans(kind))}>
            Which plans work
          </button>
          {installed && kind === "claude" && (
            <>
              {" · "}
              <button type="button" className="link-button" id="sign-in-console" onClick={() => run("sign-in", true)}>
                Use a Claude Console account instead
              </button>
            </>
          )}
        </p>
      )}
      {!installed && idle && kind === "claude" && windows && <p>On Windows, Claude Code runs its commands in PowerShell, so nothing else is needed.</p>}

      {job && (
        <div className="job" role="status" id={`job-${kind}`}>
          {job.what === "install" ? (
            <p>
              Installing {name} with {kind === "claude" ? "Anthropic" : "OpenAI"}'s installer… {elapsed}
            </p>
          ) : (
            <p>Finish signing in in your browser. Emmet is waiting for Claude Code… {elapsed}</p>
          )}
          {job.line && <p className="job-line">{job.line}</p>}
          <Button label="Cancel" size="sm" id={`cancel-${kind}`} onClick={() => void commands.agentCancel(kind)} />
          {job.what === "sign-in" && <TerminalSignIn open={open} />}
        </div>
      )}

      {failed && !job && !ready && (
        <div className="failed" role="alert" id={`failed-${kind}`}>
          <p>{failed.error.message}</p>
          {failed.error.detail && (
            <details>
              <summary>What {failed.job.what === "install" ? "the installer" : "Claude Code"} said</summary>
              <pre>{failed.error.detail}</pre>
            </details>
          )}
          {failed.error.failure === "busy" ? (
            <>
              <Button label="Cancel it" size="sm" id={`cancel-${kind}`} onClick={() => void commands.agentCancel(kind)} />{" "}
              <Button
                label="Check again"
                size="sm"
                id={`retry-${kind}`}
                onClick={() => {
                  setFailed(null);
                  onChanged();
                }}
              />
            </>
          ) : (
            <Button label="Try again" variant="primary" size="sm" id={`retry-${kind}`} onClick={() => run(failed.job.what, failed.job.console)} />
          )}
          {failed.job.what === "sign-in" && <TerminalSignIn open={open} />}
        </div>
      )}
      {detection.note && <p>{detection.note}</p>}
      {note && (
        <p className="notice" role="alert">
          {note}
        </p>
      )}
    </SetupCard>
  );
}

/** The fallback when the browser shows a code to paste: Emmet never takes it (docs/decisions.md, 2026-10-26). */
function TerminalSignIn({ open }: { open: (call: () => ReturnType<typeof commands.openTerminal>) => void }) {
  return (
    <div className="terminal-sign-in" id="terminal-sign-in">
      <p>
        Does your browser show a code to paste? Emmet can't take it. Cancel here, then sign in from {terminal}: type{" "}
        <code>claude</code> and press Enter, then type <code>/login</code> and follow the steps.
      </p>
      <Button label={`Open ${terminal}`} size="sm" icon="terminal" id="open-terminal" onClick={() => open(() => commands.openTerminal())} />
    </div>
  );
}
