// SPDX-License-Identifier: AGPL-3.0-or-later
// Sign in with ChatGPT's UI (1E), in OpenAI's words from its UI/UX guidelines, in the frozen design (4B) but for the
// button, which keeps OpenAI's own look: the "Continue with ChatGPT" button in OpenAI's approved black format (the only place
// the ChatGPT logo appears), the one-time "You're using your ChatGPT plan" notice, "Using ChatGPT plan · Manage usage"
// near the composer, the usage-limit card with Manage usage first, the banner inviting API-key users, and Settings'
// account picker. The host opens ChatGPT's usage page; the window never sees a token.
import { Dialog } from "radix-ui";
import { useCallback, useEffect, useState } from "react";
import logo from "./assets/openai/chatgpt-logo-white.svg";
import { type AccountView, type CodexBilling, commands } from "./bindings";
import { AlertCard, Button } from "./ui";

/** OpenAI's approved "Continue with ChatGPT" button (the continue-black format). */
export function ContinueButton({ onClick, disabled, id }: { onClick: () => void; disabled?: boolean; id?: string }) {
  return (
    <button type="button" className="continue-chatgpt" id={id} onClick={onClick} disabled={disabled}>
      <img src={logo} alt="" width={21} height={21} />
      <span>Continue with ChatGPT</span>
    </button>
  );
}

/** Manage usage: ChatGPT Settings → Usage, in the browser. */
export function ManageUsage({ primary, id }: { primary?: boolean; id?: string }) {
  return (
    <Button
      label="Manage usage"
      variant={primary ? "primary" : "quiet"}
      size="sm"
      icon="external-link"
      id={id}
      title="Manage usage: opens ChatGPT's usage settings in your browser"
      onClick={() => void commands.openChatgptUsage()}
    />
  );
}

/** OpenAI's usage-limit message: the ChatGPT identity visible and Manage usage the primary action. Emmet sells no
 * credits, so there's no second action, and no reset time: OpenAI's error doesn't say when the limit ends, or whose
 * limit it is. */
export function LimitCard() {
  return <AlertCard icon="gauge" id="limit-card" title="ChatGPT · Usage limit reached" text="Review your plan or this app's limit in ChatGPT settings." actions={<ManageUsage primary id="limit-manage-usage" />} />;
}

/** The welcome notice, shown once per account: the first sign-in with plan use on. */
export function PlanNotice({ account, onDone }: { account: string; onDone: () => void }) {
  const done = () => {
    void commands.chatgptNoticeSeen(account);
    onDone();
  };
  // OpenAI's words, in the frozen dialog (docs/design/frozen's chatgpt-notice), over Settings when it opens there.
  return (
    <Dialog.Root open onOpenChange={(o) => !o && done()}>
      <Dialog.Portal>
        <Dialog.Overlay className="dialog-overlay" data-c="dialog">
          <Dialog.Content className="dialog plan-notice" id="plan-notice" aria-describedby={undefined}>
            <Dialog.Title className="dialog-title">You're using your ChatGPT plan</Dialog.Title>
            <p>Eligible usage in this app uses your ChatGPT plan. Manage usage in your ChatGPT settings.</p>
            <div className="dialog-actions">
              <Dialog.Close asChild>
                <button type="button" className="button" data-c="button" data-variant="primary" data-size="sm" id="plan-notice-ok">
                  <span>Got it</span>
                </button>
              </Dialog.Close>
            </div>
          </Dialog.Content>
        </Dialog.Overlay>
      </Dialog.Portal>
    </Dialog.Root>
  );
}

function status(a: AccountView): string {
  if (!a.signed_in) return "Signed out";
  return a.plan ? "Signed in" : "Signed in, ChatGPT plan use not enabled";
}

/** Settings → Agents → Codex: which way Codex pays, the ChatGPT accounts, signing in and out. */
export function CodexAccounts({ onChanged }: { onChanged: () => void }) {
  const [billing, setBilling] = useState<CodexBilling | null>(null);
  const [waiting, setWaiting] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  const load = useCallback(async () => {
    const r = await commands.chatgptState();
    if (r.status === "ok") setBilling(r.data);
    else setMessage(`${r.error.message} ${r.error.hint}`.trim());
  }, []);
  useEffect(() => {
    void load();
  }, [load]);

  const after = async () => {
    await load();
    onChanged();
  };

  const signIn = async (account: string | null, consent = false) => {
    setMessage(null);
    setWaiting(true);
    const r = await commands.chatgptSignIn(account, consent);
    setWaiting(false);
    if (r.status === "error") setMessage(`${r.error.message} ${r.error.hint}`.trim());
    else if (r.data.kind === "declined")
      setMessage("ChatGPT plan use isn't enabled. Enable ChatGPT plan usage, or use an OpenAI API key.");
    else if (r.data.kind === "timed_out")
      setMessage("Emmet didn't hear back from your browser in 10 minutes. Press Continue with ChatGPT to try again.");
    else if (r.data.kind === "signed_in") {
      if (r.data.notice) setNotice(r.data.account.id);
      if (!r.data.account.plan)
        setMessage(
          `ChatGPT plan use isn't enabled for ${r.data.account.label}. Enable ChatGPT plan usage, or use an OpenAI API key.`,
        );
    }
    await after();
  };

  const signOut = async (a: AccountView) => {
    setMessage(null);
    const r = await commands.chatgptSignOut(a.id);
    if (r.status === "error") setMessage(`${r.error.message} ${r.error.hint}`.trim());
    else if (!r.data.confirmed)
      setMessage(
        `Signed out of ${a.label} here, but OpenAI didn't confirm it ended the session. You can disconnect Emmet in ChatGPT Settings.`,
      );
    await after();
  };

  const choose = async (account: string | null) => {
    const r = await commands.chatgptChoose(account);
    if (r.status === "error") setMessage(r.error.message);
    await after();
  };

  if (!billing)
    return message ? (
      <p className="notice" role="alert" id="chatgpt-message">
        {message}
      </p>
    ) : null;
  const onPlan = billing.choice === "chatgpt";
  return (
    <div className="codex-billing" id="codex-billing">
      {billing.accounts.length === 0 &&
        (billing.key_saved ? (
          <div className="invite" id="chatgpt-invite">
            <p>
              <span className="badge">New</span> Use your ChatGPT plan in this app.
            </p>
            <ContinueButton id="chatgpt-continue" onClick={() => void signIn(null)} disabled={waiting} />
          </div>
        ) : (
          <div className="card use-plan">
            <p>
              <strong>Use your ChatGPT plan</strong>
            </p>
            <p>Complete eligible AI requests in this app with usage included in your ChatGPT plan or credits balance.</p>
            <ContinueButton id="chatgpt-continue" onClick={() => void signIn(null)} disabled={waiting} />
          </div>
        ))}
      {billing.accounts.length > 0 && (
        <fieldset id="codex-pays">
          <legend>Codex runs on</legend>
          {billing.accounts.map((a) => (
            <div key={a.id} className="account" data-account={a.id}>
              <label>
                <input
                  type="radio"
                  name="codex-pays"
                  checked={onPlan && a.active}
                  onChange={() => void choose(a.id)}
                />{" "}
                ChatGPT · {a.label}
              </label>{" "}
              {onPlan && a.active && <span className="badge">Active</span>}{" "}
              <span className="muted">{status(a)}</span>{" "}
              {a.signed_in && !a.plan && (
                <button type="button" onClick={() => void signIn(a.id, true)} disabled={waiting}>
                  Enable ChatGPT plan usage
                </button>
              )}{" "}
              {a.signed_in ? (
                <button type="button" className="sign-out" onClick={() => void signOut(a)}>
                  Sign out
                </button>
              ) : (
                <button type="button" className="sign-in" onClick={() => void signIn(a.id)} disabled={waiting}>
                  Sign in
                </button>
              )}
            </div>
          ))}
          <div className="account">
            <label>
              <input type="radio" name="codex-pays" checked={!onPlan} onChange={() => void choose(null)} /> OpenAI API key
              {!billing.key_saved && <span className="muted"> (add one below)</span>}
            </label>
          </div>
          <div className="add-account">
            <span className="muted">Add another ChatGPT account or workspace:</span>
            <ContinueButton id="chatgpt-add" onClick={() => void signIn(null)} disabled={waiting} />
          </div>
        </fieldset>
      )}
      {waiting && (
        <p className="muted" id="chatgpt-waiting" role="status">
          Finish signing in in your browser…{" "}
          <button type="button" onClick={() => void commands.chatgptCancel()}>
            Cancel
          </button>
        </p>
      )}
      {message && (
        <p className="notice" role="alert" id="chatgpt-message">
          {message}
        </p>
      )}
      {notice && <PlanNotice account={notice} onDone={() => setNotice(null)} />}
    </div>
  );
}
