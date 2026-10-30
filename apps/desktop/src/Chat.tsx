// SPDX-License-Identifier: AGPL-3.0-or-later
// The chat, docked along the bottom of the viewer (plan section 14, docs/design/frozen, session 4B). Its top: the agent
// picker (switching starts a fresh conversation and keeps this one), the plan badge, the agent's status with Stop and
// the time, Undo turn after a turn that changed the model, History, and Start fresh. Then the card that needs you (a
// question, an approval, a pending redirect, a crash, a sign-in, a plan limit, a failed build) or the latest line; then
// the composer, which stays open while the agent works, with the model and effort beside Send. History opens upward.
//
// What the agent did is said in plain words (summaries.ts): tool cards and file cards, never a tool's name, a path, or
// code on their closed line (law 2). A message the app wrote for the user ("Ask agent to fix") shows its plain words;
// the details go to the agent as the turn's context, with what the user has selected and the view.
import { DropdownMenu, Popover } from "radix-ui";
import { useCallback, useEffect, useLayoutEffect, useMemo, useReducer, useRef, useState } from "react";
import {
  type AgentEvent,
  type AgentKind,
  type AgentView,
  type ApprovalKind,
  type ChatState,
  commands,
  type Decision,
  events,
  type ModelChoice,
  type Origin,
  type Problem,
  type ProjectView,
  type Question,
  type SteerPath,
  type TurnContext,
  type UsageSnapshot,
  type VersionView,
} from "./bindings";
import { AgentSetup } from "./Agents";
import { LimitCard } from "./ChatGpt";
import { Markdown } from "./markdown";
import { editedFiles, effortName, fileCard, isEdit, isHidden, modelName, toolCard, doing, toolName } from "./summaries";
import { AlertCard, Badge, Button, Choose, Icon, type IconName, Kbd, MenuItems, Resizable, Spinner, TextField } from "./ui";
import { limitWarning, PlanBadge, until } from "./Usage";
import { type Row, VirtualList } from "./virtual";

/** A redirect that nothing has delivered in this long offers "Stop and send now" (plan section 10). */
const STUCK_MS = 20_000;
const NAME: Record<AgentKind, string> = { claude: "Claude", codex: "Codex" };
const CLI: Record<AgentKind, string> = { claude: "Claude Code", codex: "Codex" };
const FROM: Record<Origin, [IconName, string]> = {
  checks: ["triangle-alert", "Sent from Checks"],
  settings: ["sliders-horizontal", "Sent from Build settings"],
  build: ["octagon-alert", "Sent from the failed build"],
  import: ["file-up", "Sent from the imported file"],
};

type Item =
  | { k: "divider"; key: string; text: string }
  | { k: "user"; key: string; text: string; from: Origin | null; note?: string; delivered?: SteerPath | null; at: number }
  | { k: "agent" | "thinking"; key: string; message: string; text: string }
  | { k: "tool"; key: string; id: string; name: string; input: unknown; result: { ok: boolean; summary: string } | null }
  | { k: "file"; key: string; path: string; change: "added" | "modified" | "removed"; tool?: string }
  | { k: "approval"; key: string; id: string; kind: ApprovalKind; tool: string; detail: string; decided: Decision | "closed" | null }
  | { k: "question"; key: string; id: string; question: Question; answer: string | null; closed: boolean }
  | { k: "note"; key: string; text: string }
  | { k: "crash"; key: string }
  | { k: "error"; key: string; message: string; limit: boolean };

type Chat = {
  items: Item[];
  running: boolean;
  /** When the turn began (Date.now()), for the status's time; and how long the last one took. */
  began: number | null;
  took: number | null;
  question: string | null;
  stopped: boolean;
  crashed: boolean;
  /** From the session's start: the model, who pays, and whether it's Codex on the ChatGPT plan. */
  model: string | null;
  plan: boolean;
  models: ModelChoice[];
  /** Live plan usage (Claude's rate_limit_event). */
  limits: UsageSnapshot | null;
  /** Tokens this conversation used, where the CLI reports them. */
  tokens: { input: number; output: number };
  early: Record<string, SteerPath>;
  n: number;
};

type Action =
  | { type: "load"; state: ChatState; live: boolean }
  | { type: "user"; text: string; from: Origin | null; note?: string; at: number }
  | { type: "event"; event: AgentEvent; at: number }
  | { type: "divider"; text: string }
  | { type: "decided"; id: string; how: Decision }
  | { type: "answered"; id: string; answer: string }
  | { type: "note"; id: string; text: string | null }
  | { type: "restarted" };

const empty: Chat = {
  items: [],
  running: false,
  began: null,
  took: null,
  question: null,
  stopped: false,
  crashed: false,
  model: null,
  plan: false,
  models: [],
  limits: null,
  tokens: { input: 0, output: 0 },
  early: {},
  n: 0,
};

const CHANGE: Record<string, "added" | "modified" | "removed"> = { added: "added", modified: "modified", removed: "removed" };

function apply(c: Chat, e: AgentEvent, at: number): Chat {
  const key = `k${c.n}`;
  const next = { ...c, n: c.n + 1 };
  const add = (item: Item, extra: Partial<Chat> = {}) => ({ ...next, ...extra, items: [...c.items, item] });
  const update = (pred: (i: Item) => boolean, f: (i: Item) => Item, extra: Partial<Chat> = {}) => ({ ...next, ...extra, items: c.items.map((i) => (pred(i) ? f(i) : i)) });
  switch (e.type) {
    case "started":
      return { ...next, model: e.model, plan: e.billing?.kind === "plan" && e.billing.name === "ChatGPT", crashed: false };
    case "models":
      return { ...next, models: e.models };
    case "text":
    case "reasoning": {
      const k = e.type === "text" ? "agent" : "thinking";
      const last = c.items[c.items.length - 1];
      if (last && last.k === k && last.message === e.message) return { ...next, items: [...c.items.slice(0, -1), { ...last, text: last.text + e.delta }] };
      return add({ k, key, message: e.message, text: e.delta });
    }
    case "tool_call":
      if (isHidden(e.name)) return next;
      // An edit is a file card for each file it changes, in words; the raw call is on the card, opened.
      if (isEdit(e.name)) {
        const files = editedFiles(e.name, e.input);
        if (files.length) return { ...next, items: [...c.items, ...files.map((path, i) => ({ k: "file" as const, key: `${key}-${i}`, path, change: "modified" as const, tool: e.id }))] };
      }
      return add({ k: "tool", key, id: e.id, name: e.name, input: e.input, result: null });
    case "tool_result":
      return update((i) => i.k === "tool" && i.id === e.id, (i) => ({ ...i, result: { ok: e.ok, summary: e.summary } }) as Item);
    case "file_change": {
      // A file an edit card already names, or the same file again, is one card.
      const said = c.items.slice(-6).some((i) => i.k === "file" && i.path.replaceAll("\\", "/").endsWith(e.path));
      if (said) return next;
      return add({ k: "file", key, path: e.path, change: CHANGE[e.kind] ?? "modified" });
    }
    case "approval":
      return add({ k: "approval", key, id: e.id, kind: e.kind, tool: e.tool, detail: e.detail, decided: null });
    case "approval_closed":
      return update((i) => i.k === "approval" && i.id === e.id && !i.decided, (i) => ({ ...i, decided: "closed" }) as Item);
    case "question":
      return add({ k: "question", key, id: e.id, question: e.question, answer: null, closed: false }, { question: e.id });
    case "question_closed":
      return update((i) => i.k === "question" && i.id === e.id, (i) => ({ ...i, closed: true, answer: (i as { answer: string | null }).answer ?? (e.reason === "stopped" ? "Stopped before an answer" : null) }) as Item, {
        question: c.question === e.id ? null : c.question,
      });
    case "note_delivered": {
      const found = c.items.some((i) => i.k === "user" && i.note === e.id);
      // A redirect sent as the next turn starts that turn.
      const turn = e.path === "next_turn" ? { running: true, began: at, stopped: false } : {};
      if (!found) return { ...next, ...turn, early: { ...c.early, [e.id]: e.path } };
      return update((i) => i.k === "user" && i.note === e.id, (i) => ({ ...i, delivered: e.path }) as Item, turn);
    }
    case "usage":
      return { ...next, tokens: { input: c.tokens.input + e.input, output: c.tokens.output + e.output } };
    case "limits":
      return { ...next, limits: e.snapshot };
    case "turn_done": {
      const took = c.began != null ? at - c.began : null;
      const s = { ...next, running: false, question: null, took };
      if (e.reason === "crashed") return c.crashed ? s : { ...s, crashed: true, items: [...c.items, { k: "crash", key }] };
      if (e.reason === "interrupted") return { ...s, stopped: true, items: [...c.items, { k: "note", key, text: `You stopped the turn at ${clock(took ?? 0)}.` }] };
      return s;
    }
    case "error":
      // A crash is its own card (the turn's end says it); its details go to Copy the details, not the screen.
      if (e.code === "crashed") return next;
      return add({ k: "error", key, message: e.message, limit: e.code === "quota_exceeded" });
    case "exited": {
      const mid = c.running && !c.crashed;
      return { ...(mid ? add({ k: "crash", key }) : next), running: false, question: null, crashed: c.crashed || mid };
    }
    default:
      return next;
  }
}

function reduce(c: Chat, a: Action): Chat {
  switch (a.type) {
    case "load": {
      let s: Chat = { ...empty, models: a.state.models };
      a.state.sessions.forEach((v, i) => {
        if (i > 0) {
          const prev = a.state.sessions[i - 1]!.info.kind;
          s = reduce(s, { type: "divider", text: dividerText(v.info.kind, prev) });
        }
        // Each conversation starts with no turn running: an earlier one's last turn is over.
        s = { ...s, running: false, question: null, crashed: false };
        for (const r of v.records) {
          if (r.user != null) s = reduce(s, { type: "user", text: r.user, from: r.from ?? null, at: r.at ?? 0 });
          else if (r.event) s = apply(s, r.event, r.at ?? 0);
        }
      });
      // A chat read back that isn't running: its last turn is over (a crash shows only while it's live).
      return a.live ? s : { ...s, running: false, question: null, crashed: false, took: null };
    }
    case "user": {
      const delivered = a.note ? (c.early[a.note] ?? null) : undefined;
      const turn = a.note ? {} : { running: true, began: a.at, stopped: false, crashed: false };
      return { ...c, ...turn, n: c.n + 1, items: [...c.items, { k: "user", key: `k${c.n}`, text: a.text, from: a.from, note: a.note, delivered, at: a.at }] };
    }
    case "event":
      return apply(c, a.event, a.at);
    case "restarted":
      return { ...c, crashed: false, took: null };
    case "divider":
      return { ...c, n: c.n + 1, stopped: false, took: null, tokens: { input: 0, output: 0 }, items: [...c.items, { k: "divider", key: `k${c.n}`, text: a.text }] };
    case "decided":
      return { ...c, items: c.items.map((i) => (i.k === "approval" && i.id === a.id ? { ...i, decided: a.how } : i)) };
    case "answered":
      return { ...c, items: c.items.map((i) => (i.k === "question" && i.id === a.id ? { ...i, answer: a.answer } : i)) };
    case "note":
      return {
        ...c,
        items: c.items.flatMap((i) => {
          if (i.k !== "user" || i.note !== a.id) return [i];
          return a.text == null ? [] : [{ ...i, text: a.text }];
        }),
      };
  }
}

const dividerText = (kind: AgentKind, prev: AgentKind) =>
  kind === prev ? "A fresh conversation. The last one is kept." : `${NAME[kind]} · a fresh conversation. The ${NAME[prev]} one is kept.`;

/** An elapsed time as m:ss (docs/design/content.md). */
function clock(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/** What the agent is doing now, in the present (note 1): the running tool, thinking, or writing. */
function activity(items: Item[]): string {
  for (let i = items.length - 1; i >= 0; i--) {
    const it = items[i]!;
    if (it.k === "tool" && !it.result) return doing(it.name);
    if (it.k === "thinking") return "Thinking";
    if (it.k === "agent") return "Writing";
    if (it.k === "user") return "Reading your message";
    if (it.k === "tool" || it.k === "file") return "Working";
  }
  return "Working";
}

/** Each approval's question and its tag, in words. */
function approvalWords(kind: ApprovalKind, tool: string, who: string): { title: string; tag: string; why: string } {
  if (/^WebFetch$|web/i.test(tool)) return { title: "Open a web page?", tag: "Web request", why: `${who} wants to read it. It's outside what the agent can do on its own.` };
  if (/^WebSearch$/.test(tool)) return { title: "Search the web?", tag: "Web search", why: `${who} wants to look something up. Part and color questions don't need the web.` };
  switch (kind) {
    case "command":
      return { title: "Run a command?", tag: "Command", why: "It needs more than the agent's sandbox allows." };
    case "edit":
      return { title: "Change a file?", tag: "File", why: "It's outside what Emmet lets the agent change on its own." };
    case "permission":
      return { title: "Give the agent more access?", tag: "Access", why: "It asked to work outside its sandbox." };
    default:
      return { title: "Use a tool?", tag: "Tool", why: "It's a tool Emmet doesn't allow on its own." };
  }
}

/** What "Ask agent to fix" and the other app-written messages send: the plain words shown, the details for the agent. */
export type Ask = { text: string; detail: string; from: Origin; n: number };

type Props = {
  project: ProjectView;
  ask: Ask | null;
  /** The failed build, for its card (the details go to the agent with "Ask agent to fix"). */
  buildError: { problem: Problem; agent: boolean; shown: boolean } | null;
  onAskBuild: () => void;
  onRebuild: () => void;
  agentsChanged: number;
  /** What the user sees now: the selection and the view, sent with each message. */
  context: () => TurnContext;
  onSettings: () => void;
  onUndone: (saved: VersionView, who: string) => void;
  /** The agent the chat is with, for the keyboard map and Settings. */
  onAgent: (kind: AgentKind | null) => void;
};

export function Dock({ project, ask, buildError, onAskBuild, onRebuild, agentsChanged, context, onSettings, onUndone, onAgent }: Props) {
  const [chat, dispatch] = useReducer(reduce, empty);
  const [agents, setAgents] = useState<AgentView[] | null>(null);
  const [state, setState] = useState<ChatState | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [now, setNow] = useState(() => Date.now());
  const [undo, setUndo] = useState<number | null>(null);
  const [open, setOpen] = useState(false);
  const [height, setHeight] = useState(() => Number(localStorage.getItem("emmet.history")) || 58);
  const [usage, setUsage] = useState<UsageSnapshot | null>(null);
  const [turns, setTurns] = useState(0);
  const session = useRef<string | null>(null);
  /** The message on its way to the host, which a redirect waits for. */
  const sending = useRef<Promise<unknown> | null>(null);
  const input = useRef<HTMLTextAreaElement | null>(null);
  /** Event `at` times waiting for their commit, and the measured event → DOM times (ms). */
  const painting = useRef<number[]>([]);
  const latency = useRef<number[]>([]);

  const kind = state?.agent ?? null;
  const current = state?.sessions[state.sessions.length - 1]?.info ?? null;
  const who = kind ? NAME[kind] : "the agent";
  const agent = kind ? agents?.find((a) => a.kind === kind) : null;
  const ready = !!agent && agent.detection.path != null && agent.detection.signed_in;

  const load = useCallback(async () => {
    const s = await commands.chatState();
    setState(s);
    setUndo(s.undo);
    session.current = s.live?.session ?? null;
    dispatch({ type: "load", state: s, live: s.live != null });
    onAgent(s.agent);
  }, [onAgent]);

  useEffect(() => {
    void commands.agents(agentsChanged > 0).then(setAgents);
  }, [agentsChanged]);

  useEffect(() => {
    setError(null);
    void load();
  }, [project.root, load]);

  useEffect(() => {
    const offs = [
      events.chatEvent.listen(({ payload }) => {
        if (payload.session !== session.current) return;
        painting.current.push(payload.at ?? Date.now());
        dispatch({ type: "event", event: payload.event, at: payload.at ?? Date.now() });
        if (payload.event.type === "turn_done") setTurns((n) => n + 1);
        if (payload.event.type === "exited") session.current = null;
      }),
      events.chatUndo.listen(({ payload }) => setUndo(payload.undo)),
    ];
    return () => offs.forEach((off) => void off.then((f) => f()));
  }, []);

  // Event → pixels (plan section 10's 50 ms): measured when React has put the event's change in the DOM.
  useLayoutEffect(() => {
    if (!painting.current.length) return;
    const t = Date.now();
    for (const at of painting.current) latency.current.push(t - at);
    painting.current = [];
  }, [chat]);

  useEffect(() => {
    if (__EMMET_TEST_HOOKS__) {
      const w = window as unknown as { __emmet?: Record<string, unknown> };
      w.__emmet = { ...(w.__emmet ?? {}), chatLatency: () => latency.current.slice() };
    }
  }, []);

  const pending = chat.items.find((i): i is Extract<Item, { k: "user" }> => i.k === "user" && !!i.note && i.delivered == null) ?? null;
  // The status's time, and a stuck redirect's "Stop and send now".
  useEffect(() => {
    if (!pending && !chat.running) return;
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, [pending, chat.running]);

  const run = async (f: () => Promise<{ status: "ok"; data?: unknown } | { status: "error"; error: { message: string; hint: string } }>) => {
    const r = await f();
    setError(r.status === "error" ? `${r.error.message} ${r.error.hint}`.trim() : null);
    return r.status === "ok";
  };

  /** Sends `t`: a turn, a redirect while the agent works, or the open question's Other answer. */
  const send = async (t: string, from: Origin | null = null, detail: string | null = null) => {
    t = t.trim();
    if (!t) return;
    const ctx = { ...context(), detail };
    // While a question is open, what the user types is its Other answer (plan section 10).
    if (chat.question && !from) {
      dispatch({ type: "answered", id: chat.question, answer: t });
      await run(() => commands.chatAnswer(chat.question!, { choices: [], other: t }));
      return;
    }
    if (chat.running) {
      // A redirect typed while the message before it is still reaching the agent waits for it: otherwise the agent,
      // with no turn running yet, would take the redirect as a turn of its own.
      await sending.current;
      const r = await commands.chatSteer(t, ctx);
      if (r.status === "ok") dispatch({ type: "user", text: t, from, note: r.data.id, at: Date.now() });
      else setError(r.error.message);
      return;
    }
    dispatch({ type: "user", text: t, from, at: Date.now() });
    const call = commands.chatSend(t, ctx, from);
    sending.current = call;
    const r = await call;
    if (r.status === "error") {
      setError(`${r.error.message} ${r.error.hint}`.trim());
      dispatch({ type: "event", event: { type: "turn_done", reason: "failed" }, at: Date.now() });
      return;
    }
    setError(null);
    if (session.current !== r.data) {
      session.current = r.data;
      void commands.chatState().then(setState);
    }
  };

  const submit = () => {
    const t = text;
    setText("");
    void send(t);
  };
  // Stop as the first message is still on its way waits for it, so the agent it starts is the one stopped.
  const stop = () => void (async () => {
    await sending.current;
    await run(() => commands.chatStop());
  })();

  // "Ask agent to fix", Rebuild, Make it parametric: sent like a message, once each.
  const asked = useRef(0);
  useEffect(() => {
    if (!ask || ask.n === asked.current) return;
    asked.current = ask.n;
    void send(ask.text, ask.from, ask.detail);
  }, [ask]);

  const fresh = async (k: AgentKind | null) => {
    const r = await commands.chatFresh(k);
    if (r.status === "error") return setError(r.error.message);
    session.current = null;
    setUndo(null);
    await load();
  };
  const pickModel = async (model: string | null, effort: string | null) => {
    const r = await commands.chatModel(model, effort);
    if (r.status === "error") setError(r.error.message);
    else setState((s) => s && { ...s, sessions: s.sessions.map((v, i) => (i === s.sessions.length - 1 ? { ...v, info: r.data } : v)) });
  };
  const restart = async () => {
    const r = await commands.chatRestart();
    if (r.status === "error") return setError(`${r.error.message} ${r.error.hint}`.trim());
    session.current = r.data;
    dispatch({ type: "restarted" });
    setError(null);
  };
  const undoTurn = async () => {
    const r = await commands.chatUndo();
    if (r.status === "error") return setError(r.error.message);
    setUndo(null);
    onUndone(r.data, who);
  };

  // What the specs read: the turn, the open question, a crash, Undo turn's version, the live session.
  const probe = useRef({});
  const reload = useRef(load);
  reload.current = load;
  probe.current = { running: chat.running, question: chat.question, crashed: chat.crashed, stopped: chat.stopped, undo, session: session.current, items: chat.items.length, agent: state?.agent ?? null };
  useEffect(() => {
    if (!__EMMET_TEST_HOOKS__) return;
    const w = window as unknown as { __emmet?: Record<string, unknown> };
    // The dock spec's screens: a turn's events played into the chat as the host would send them, without an agent.
    const play = (evs: AgentEvent[], user?: string, o: { from?: Origin; note?: string; at?: number } = {}) => {
      if (user) dispatch({ type: "user", text: user, from: o.from ?? null, note: o.note, at: o.at ?? Date.now() });
      for (const event of evs) dispatch({ type: "event", event, at: Date.now() });
    };
    w.__emmet = { ...(w.__emmet ?? {}), chat: () => probe.current, chatPlay: play, reloadChat: () => reload.current() };
  }, []);

  const offered = chat.models.length ? chat.models : (state?.models ?? []);
  const limit = (chat.limits ?? usage)?.status === "limited" && kind === "claude";
  const lastError = [...chat.items].reverse().find((i) => i.k === "error") as Extract<Item, { k: "error" }> | undefined;
  const codexLimit = !!lastError?.limit && kind === "codex" && chat.plan && !chat.running && chat.items[chat.items.length - 1] === lastError;
  const last = chat.items[chat.items.length - 1];
  const openCard = last && ["question", "approval", "crash"].includes(last.k) && !(last.k === "question" && last.closed) && !(last.k === "approval" && last.decided) && !(last.k === "crash" && !chat.crashed);

  // The card that needs you, most urgent first.
  let card: React.ReactNode = null;
  if (agents && kind && !ready && !chat.running)
    card = (
      <div className="dock-fix" id="agent-fix">
        <AgentSetup agents={agents} only="unready" onChanged={() => void commands.agents(true).then(setAgents)} onSettings={onSettings} />
        {(() => {
          const other = agents.find((a) => a.kind !== kind && a.detection.path != null && a.detection.signed_in);
          return other ? <Button label={`Use ${NAME[other.kind]} instead`} variant="quiet" size="sm" id="use-other" onClick={() => void fresh(other.kind)} /> : null;
        })()}
      </div>
    );
  else if (codexLimit) card = <LimitCard />;
  else if (limit) {
    const w = (chat.limits ?? usage)?.windows.find((x) => (x.percent ?? 0) >= 100) ?? (chat.limits ?? usage)?.windows[0];
    const other = agents?.find((a) => a.kind === "codex" && a.detection.path != null && a.detection.signed_in);
    card = (
      <AlertCard
        icon="gauge"
        id="claude-limit"
        title="You've used Claude's plan limit for now"
        text={`${w?.resets_at ? `It resets in ${until(w.resets_at, now)}. ` : ""}Your model and this conversation are kept.`}
        actions={
          <>
            {other && <Button label="Continue with Codex" variant="primary" id="continue-other" onClick={() => void fresh("codex")} />}
            <Button label="Wait for the reset" variant={other ? "quiet" : "primary"} onClick={() => setUsage(null)} />
          </>
        }
      />
    );
  } else if (buildError?.shown)
    card = buildError.agent ? (
      <AlertCard
        tone="bad"
        icon="octagon-alert"
        id="build-failed"
        title="The model didn't build"
        text="The agent's last change has a mistake in it, so the new model couldn't be built."
        foot="The last good model is still shown."
        actions={<Button label="Ask agent to fix" variant="primary" id="ask-fix-build" onClick={onAskBuild} />}
      />
    ) : (
      <AlertCard
        tone="bad"
        icon="octagon-alert"
        id="build-failed"
        title="The build stopped"
        text={`${buildError.problem.message} ${buildError.problem.hint}`.trim()}
        foot="The last good model is still shown."
        actions={<Button label="Rebuild" variant="primary" icon="refresh-cw" id="rebuild" onClick={onRebuild} />}
      />
    );
  else if (pending)
    card = (
      <Pending
        item={pending}
        who={who}
        now={now}
        onNote={async (id, t) => {
          const r = await commands.chatNote(id, t);
          if (r.status === "ok" && r.data) dispatch({ type: "note", id, text: t });
        }}
        onStopAndSend={stop}
      />
    );
  else if (openCard) card = <Message item={last!} who={kind ?? "claude"} chat={chat} actions={actions()} />;

  function actions() {
    return {
      decide: (id: string, d: Decision) => {
        dispatch({ type: "decided", id, how: d });
        void run(() => commands.chatRespond(id, d));
      },
      answer: (id: string, choices: string[], other: string | null) => {
        dispatch({ type: "answered", id, answer: [...choices, ...(other ? [other] : [])].join(", ") });
        void run(() => commands.chatAnswer(id, { choices, other }));
      },
      restart: () => void restart(),
    };
  }

  // The status, said once (docs/design/content.md): what the agent is doing, with the time and Stop; quiet when a card
  // says it.
  const quiet = !!card && !pending;
  let status: { tone: string; line: string; time?: string; icon?: IconName } | null = null;
  if (chat.running && !chat.question && !(last?.k === "approval" && !last.decided)) status = { tone: "busy", line: activity(chat.items), time: chat.began ? clock(now - chat.began) : undefined };
  else if (chat.crashed) status = { tone: "bad", line: `${kind ? CLI[kind] : "The agent"} stopped unexpectedly`, icon: "octagon-alert" };
  else if (limit) status = { tone: "attn", line: "Claude's plan limit", icon: "gauge" };
  else if (chat.stopped) status = { tone: "muted", line: "Stopped by you", icon: "square" };
  else if (!chat.running && chat.took != null) status = { tone: "done", line: `Done in ${clock(chat.took)}`, icon: "circle-check" };
  const canUndo = !chat.running && undo != null && status?.tone === "done";

  // The card that needs you shows once: below the history, not in it too.
  const rows = useMemo(() => fold(openCard ? chat.items.slice(0, -1) : chat.items, kind ?? "claude", chat, actions), [chat, kind, openCard]);
  // The latest line when nothing needs you: the last step or the agent's last words.
  const lastLine = !card && last && ["tool", "file", "note", "agent", "thinking"].includes(last.k) && !(last.k === "tool" && !last.result) ? <Message item={last} who={kind ?? "claude"} chat={chat} actions={actions()} line /> : null;
  // Said before the next turn: a Claude window at 90% or more, or an API key billing instead of the plan (D-35).
  const warning = chat.running ? null : (agent?.warning ?? (kind === "claude" ? limitWarning(chat.limits ?? usage) : null));

  const problem = error && (
    <p className="notice" role="alert" id="chat-error">
      <Icon name="info" />
      <span>{error}</span>
    </p>
  );

  const onKey = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey && !e.nativeEvent.isComposing) {
      e.preventDefault();
      submit();
    } else if (e.key === "Escape" && chat.running) {
      e.preventDefault();
      stop();
    }
  };

  const model = current?.model ?? null;
  const effort = current?.effort ?? null;
  const chip = `${model ? (offered.find((m) => m.id === model)?.name ?? modelName(model)) : modelName(chat.model)}${effort ? ` · ${effortName(effort)}` : ""}`;

  return (
    <section className="dock" data-c="dock" data-state={open ? "open" : "closed"} aria-label="Chat" id="dock" tabIndex={-1} style={{ "--dock-history": height / 100 } as React.CSSProperties}>
      {open && (
        <Resizable
          orientation="horizontal"
          label="Resize the chat history"
          value={height}
          min={20}
          max={80}
          invert
          onChange={(v) => {
            setHeight(v);
            localStorage.setItem("emmet.history", String(v));
          }}
        />
      )}
      <div className="dock-top">
        <DropdownMenu.Root>
          <DropdownMenu.Trigger asChild>
            <button type="button" className="agent-picker" id="agent-picker" aria-label={`Agent: ${who}`}>
              {kind ? NAME[kind] : "Agent"}
              <Icon name="chevron-down" size="sm" />
            </button>
          </DropdownMenu.Trigger>
          <DropdownMenu.Portal>
            <DropdownMenu.Content className="menu" data-c="menu" aria-label="Agent" side="top" align="start" sideOffset={6} id="agent-menu">
              <MenuItems
                items={[
                  ...(["claude", "codex"] as AgentKind[]).map((k) => {
                    const a = agents?.find((x) => x.kind === k);
                    const ok = !!a && a.detection.path != null && a.detection.signed_in;
                    return {
                      id: `agent-${k}`,
                      text: NAME[k],
                      sub: ok ? (a!.detection.note ?? a!.badge.split(" · ")[1]) : a?.detection.path ? "Not signed in" : "Not set up",
                      checked: k === kind,
                      onSelect: () => void (k !== kind && fresh(k)),
                    };
                  }),
                  { note: "Switching starts a fresh conversation. This one is kept." },
                  "-",
                  { id: "agents-again", text: "Check agents again", icon: "refresh-cw", onSelect: () => void commands.agents(true).then(setAgents) },
                ]}
              />
            </DropdownMenu.Content>
          </DropdownMenu.Portal>
        </DropdownMenu.Root>
        {kind && <PlanBadge kind={kind} live={chat.limits} turns={turns} onSnapshot={setUsage} plan={chat.plan} tokens={chat.tokens} named={agent?.badge.split(" · ")[1] ?? null} />}
        {status && !quiet ? (
          <div className="status" data-c="status" data-tone={status.tone} role="status" aria-live="polite" id="chat-status">
            {status.tone === "busy" ? <Spinner label={status.line} /> : <Icon name={status.icon ?? "circle-check"} size="sm" />}
            <span className="status-line">{status.line}</span>
            {status.time && <span className="status-time">{status.time}</span>}
            {status.tone === "busy" && <Button label="Stop" size="sm" icon="square" kbd="Esc" title="Stop the turn (Esc)" id="chat-stop" onClick={stop} />}
          </div>
        ) : (
          <span className="grow" />
        )}
        {canUndo && <Button label="Undo turn" variant="quiet" size="sm" icon="undo-2" compact id="undo-turn" title={`Undo turn: restore the model from before this turn (version ${undo})`} onClick={() => void undoTurn()} />}
        {(chat.items.length > 0 || open) && (
          <Button label={open ? "Hide history" : `History · ${chat.items.filter((i) => i.k !== "divider").length}`} variant="quiet" size="sm" icon="messages-square" compact id="history-toggle" aria-expanded={open} onClick={() => setOpen(!open)} />
        )}
        <Button label="Start fresh" variant="quiet" size="sm" icon="message-square-plus" compact id="start-fresh" title="Start fresh: clear the context and keep the agent" disabled={chat.running} onClick={() => void fresh(null)} />
      </div>
      {open ? (
        <div className="dock-history-wrap">
          <VirtualList rows={rows} className="dock-history" id="chat-log" label="Chat history" />
          {card && (
            <div className="dock-active" id="dock-active">
              {card}
            </div>
          )}
          {problem}
        </div>
      ) : card ? (
        <div className="dock-active" id="dock-active">
          {card}
          {problem}
        </div>
      ) : (
        (lastLine || warning || problem) && (
          <div className="dock-line" id="dock-line">
            {warning ? (
              <p className="small" id="limit-warning">
                {warning}
              </p>
            ) : (
              lastLine
            )}
            {problem}
          </div>
        )
      )}
      <div className="composer" data-c="composer">
        <textarea
          id="chat-input"
          ref={input}
          rows={text.includes("\n") || text.length > 80 ? 3 : 1}
          placeholder={chat.running ? `Redirect ${who}…` : `Ask ${who} to build or change something`}
          aria-label={chat.question ? "Your answer, in your own words" : chat.running ? `A message for ${who} while it works` : "Message"}
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={onKey}
        />
        <div className="composer-bar">
          <span className="composer-model">
            <Popover.Root>
              <Popover.Trigger asChild>
                <button type="button" className="model-chip" id="model-chip" aria-haspopup="dialog">
                  {chip}
                  <Icon name="chevron-down" size="sm" />
                </button>
              </Popover.Trigger>
              <Popover.Portal>
                <Popover.Content className="popover" data-c="popover" aria-label="Model and effort" side="top" align="start" sideOffset={6} id="model-popover">
                  <span className="popover-title">Model and effort</span>
                  <ModelPicker kind={kind} models={offered} model={model} effort={effort} onPick={pickModel} />
                  <p className="small">Bigger models and higher effort use up your plan faster.</p>
                </Popover.Content>
              </Popover.Portal>
            </Popover.Root>
          </span>
          <span className="grow" />
          <Button label="Send" variant="primary" size="sm" icon="send" id="chat-send" disabled={!text.trim() || !kind} onClick={submit} />
        </div>
      </div>
    </section>
  );
}

function ModelPicker({ kind, models, model, effort, onPick }: { kind: AgentKind | null; models: ModelChoice[]; model: string | null; effort: string | null; onPick: (m: string | null, e: string | null) => void }) {
  if (!kind) return null;
  const DEFAULT = "default";
  const chosen = models.find((m) => m.id === (model ?? "")) ?? models[0];
  const list = models.length ? models : [{ id: "", name: "Default", efforts: [], default_effort: null }];
  return (
    <>
      <label className="select" data-c="select">
        <span className="select-label">Model</span>
        <Choose label="Model" id="model-select" value={model ?? DEFAULT} items={list.map((m) => [m.id || DEFAULT, m.name])} onChange={(v) => onPick(v === DEFAULT ? null : v, null)} />
      </label>
      <label className="select" data-c="select">
        <span className="select-label">Effort</span>
        <Choose
          label="Effort"
          id="effort-select"
          value={effort ?? DEFAULT}
          items={[[DEFAULT, chosen?.default_effort ? `Default (${effortName(chosen.default_effort)})` : `${CLI[kind]}'s default`], ...(chosen?.efforts ?? []).map((e): [string, string] => [e, effortName(e)])]}
          onChange={(v) => onPick(model, v === DEFAULT ? null : v)}
        />
      </label>
    </>
  );
}

type Actions = {
  decide: (id: string, d: Decision) => void;
  answer: (id: string, choices: string[], other: string | null) => void;
  restart: () => void;
};

/** The history's rows: three or more finished steps fold into one line once the turn moves past them (note 2). */
function fold(items: Item[], who: AgentKind, chat: Chat, actions: () => Actions): Row[] {
  const out: Row[] = [];
  const step = (i: Item) => (i.k === "tool" && !!i.result) || i.k === "file";
  for (let i = 0; i < items.length; ) {
    let j = i;
    while (j < items.length && step(items[j]!)) j++;
    const run = items.slice(i, j);
    if (run.length >= 3 && j < items.length) {
      const built = run.filter((m) => m.k === "tool" && toolName(m.name) === "run_model" && m.result?.ok).length;
      out.push({
        key: `run-${run[0]!.key}`,
        node: (
          <details className="tool-run" data-c="tool-card">
            <summary>
              <Icon name="circle-check" />
              <span>
                {run.length} steps{built ? ` · built in ${built} stage${built > 1 ? "s" : ""}` : ""}
              </span>
              <Icon name="chevron-down" size="sm" />
            </summary>
            {run.map((m) => (
              <Message key={m.key} item={m} who={who} chat={chat} actions={actions()} />
            ))}
          </details>
        ),
      });
      i = j;
    } else if (j > i) {
      for (const m of run) out.push({ key: m.key, node: <Message item={m} who={who} chat={chat} actions={actions()} /> });
      i = j;
    } else {
      const m = items[i]!;
      out.push({ key: m.key, node: <Message item={m} who={who} chat={chat} actions={actions()} /> });
      i++;
    }
  }
  return out;
}

const TOOL_ICON_DONE = (ok: boolean | undefined) => (ok === false ? <Icon name="x" size="sm" /> : <Icon name="check" size="sm" />);

function Message({ item, who, chat, actions, line }: { item: Item; who: AgentKind; chat: Chat; actions: Actions; line?: boolean }) {
  const name = NAME[who];
  switch (item.k) {
    case "divider":
      return (
        <div className="message" data-c="chat-message" data-from="divider" role="separator">
          <span>{item.text}</span>
        </div>
      );
    case "user":
      return (
        <div className="message" data-c="chat-message" data-from="user" data-note={item.note}>
          {item.from && (
            <span className="message-meta">
              <Icon name={FROM[item.from][0]} size="sm" />
              {FROM[item.from][1]}
            </span>
          )}
          <p>{item.text}</p>
          {item.delivered && (
            <span className="message-meta">
              <Icon name="check" size="sm" />
              Delivered at {name}'s next step
            </span>
          )}
        </div>
      );
    case "agent":
      return (
        <div className="message" data-c="chat-message" data-from="agent">
          <Markdown src={item.text} />
        </div>
      );
    case "thinking":
      if (line) return <p className="thought">{item.text}</p>;
      return (
        <details className="message thinking" data-c="chat-message" data-from="thinking">
          <summary>
            Thinking <Icon name="chevron-down" size="sm" />
          </summary>
          <p>{item.text}</p>
        </details>
      );
    case "tool": {
      const c = toolCard(item.name, item.input, item.result);
      const st = !item.result ? "running" : item.result.ok ? "done" : "error";
      return (
        <details className="tool-card" data-c="tool-card" data-status={st} data-tool={toolName(item.name)}>
          <summary>
            <Icon name={c.icon} />
            <span className="tool-card-title">{c.title}</span>
            <span className="tool-card-detail">{c.detail}</span>
            <span className={st === "error" ? "bad" : st === "done" ? "ok" : "accent"}>{st === "running" ? <Spinner label={c.title} /> : TOOL_ICON_DONE(item.result?.ok)}</span>
          </summary>
          <pre className="tool-card-raw">{`${toolName(item.name)}(${JSON.stringify(item.input ?? {})})${item.result ? ` → ${item.result.summary}` : ""}`}</pre>
        </details>
      );
    }
    case "file":
      return (
        <div className="tool-card" data-c="tool-card" data-status="done" data-file={item.path}>
          <div className="tool-card-row">
            <Icon name="file-pen" />
            <span className="tool-card-title">{fileCard(item.path, item.change)}</span>
            <span className="tool-card-detail" />
            <span className="ok">
              <Icon name="check" size="sm" />
            </span>
          </div>
        </div>
      );
    case "approval": {
      const w = approvalWords(item.kind, item.tool, name);
      if (item.decided)
        return (
          <p className="message" data-c="chat-message" data-from="note" data-approval={item.id}>
            {item.decided === "deny" ? `You said no: ${w.title.replace(/\?$/, "").toLowerCase()}. ${name} carries on without it.` : item.decided === "closed" ? `${w.title} The question went away.` : `You allowed it${item.decided === "allow_session" ? " for this session" : " once"}: ${w.title.replace(/\?$/, "").toLowerCase()}.`}
          </p>
        );
      return <ApprovalCard item={item} words={w} onDecide={actions.decide} />;
    }
    case "question":
      return <QuestionCard item={item} open={chat.question === item.id} onAnswer={actions.answer} />;
    case "note":
      return (
        <p className="message" data-c="chat-message" data-from="note">
          {item.text}
        </p>
      );
    case "crash":
      if (!chat.crashed)
        return (
          <p className="message" data-c="chat-message" data-from="note">
            {CLI[who]} stopped unexpectedly. The project was untouched.
          </p>
        );
      return (
        <AlertCard
          tone="bad"
          icon="octagon-alert"
          id="crash-card"
          title={`${CLI[who]} stopped unexpectedly`}
          text="It stopped in the middle of a turn. Your project is untouched: the model and the last build are as they were."
          actions={
            <>
              <Button label={`Restart ${CLI[who]}`} variant="primary" icon="refresh-cw" id="chat-restart" onClick={actions.restart} />
              <Button label="Copy the details" variant="quiet" icon="copy" id="crash-details" onClick={() => void commands.diagnostics()} />
            </>
          }
        />
      );
    case "error":
      if (item.limit && who === "codex") return <LimitCard />;
      return (
        <p className="message" data-c="chat-message" data-from="note" role="alert">
          {item.message}
        </p>
      );
  }
}

function Pending({ item, who, now, onNote, onStopAndSend }: { item: Extract<Item, { k: "user" }>; who: string; now: number; onNote: (id: string, text: string | null) => void; onStopAndSend: () => void }) {
  const [editing, setEditing] = useState<string | null>(null);
  return (
    <div className="message" data-c="chat-message" data-from="pending" data-note={item.note} id="pending">
      {editing == null ? <p>{item.text}</p> : <TextField label="Edit your message" value={editing} onChange={setEditing} id="pending-edit" />}
      <span className="message-meta">
        <Icon name="clock" size="sm" />
        Goes to {who} at its next step
      </span>
      <div className="row">
        {editing == null ? (
          <Button label="Edit" size="sm" variant="quiet" id="pending-edit-button" onClick={() => setEditing(item.text)} />
        ) : (
          <Button
            label="Save"
            size="sm"
            variant="quiet"
            id="pending-save"
            onClick={() => {
              onNote(item.note!, editing);
              setEditing(null);
            }}
          />
        )}
        <Button label="Cancel" size="sm" variant="quiet" id="pending-cancel" onClick={() => onNote(item.note!, null)} />
        {now - item.at > STUCK_MS && <Button label="Stop and send now" size="sm" id="pending-deliver" onClick={onStopAndSend} />}
      </div>
    </div>
  );
}

function ApprovalCard({ item, words, onDecide }: { item: Extract<Item, { k: "approval" }>; words: { title: string; tag: string; why: string }; onDecide: (id: string, d: Decision) => void }) {
  const ref = useRef<HTMLDivElement | null>(null);
  useEffect(() => ref.current?.focus(), []);
  const keys: Record<string, Decision> = { "1": "allow_once", "2": "allow_session", "3": "deny" };
  return (
    <div
      className="approval-card"
      data-c="approval-card"
      role="group"
      aria-label={words.title}
      data-approval={item.id}
      tabIndex={-1}
      ref={ref}
      onKeyDown={(e) => {
        const d = keys[e.key];
        if (d && (e.target as HTMLElement).tagName !== "INPUT") {
          e.preventDefault();
          onDecide(item.id, d);
        }
      }}
    >
      <div className="card-top">
        <Badge text={words.tag} />
      </div>
      <p className="card-title">{words.title}</p>
      <code className="approval-what">{item.detail}</code>
      <p className="muted">{words.why}</p>
      <div className="row">
        <Button label="Allow once" variant="primary" kbd="1" onClick={() => onDecide(item.id, "allow_once")} />
        <Button label="Allow for this session" kbd="2" onClick={() => onDecide(item.id, "allow_session")} />
        <Button label="Don't allow" variant="quiet" kbd="3" onClick={() => onDecide(item.id, "deny")} />
      </div>
    </div>
  );
}

function QuestionCard({ item, open, onAnswer }: { item: Extract<Item, { k: "question" }>; open: boolean; onAnswer: (id: string, choices: string[], other: string | null) => void }) {
  const q = item.question;
  const [picked, setPicked] = useState<string[]>([]);
  const [other, setOther] = useState("");
  const card = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    if (open) card.current?.focus();
  }, [open]);
  if (item.closed || item.answer != null)
    return (
      <div className="question-done" data-c="question-card" data-question={item.id}>
        <Icon name="check" size="sm" />
        <span className="muted">{q.question}</span>
        <b>{item.answer ?? "Answered"}</b>
      </div>
    );
  const choose = (label: string) => {
    if (!q.multi) onAnswer(item.id, [label], null);
    else setPicked((p) => (p.includes(label) ? p.filter((x) => x !== label) : [...p, label]));
  };
  // Number keys pick an option, and the one after the options is Other (plan section 10).
  const onKey = (e: React.KeyboardEvent) => {
    if (!open || (e.target as HTMLElement).tagName === "INPUT") return;
    const n = Number(e.key);
    if (n >= 1 && n <= q.options.length) {
      e.preventDefault();
      choose(q.options[n - 1]!.label);
    } else if (n === q.options.length + 1) {
      e.preventDefault();
      card.current?.querySelector<HTMLInputElement>(".question-other input")?.focus();
    }
  };
  return (
    <div className="question-card" data-c="question-card" role="group" aria-label={q.question} tabIndex={-1} ref={card} onKeyDown={onKey} data-question={item.id}>
      {q.header && (
        <div className="card-top">
          <Badge text={q.header} />
        </div>
      )}
      <p className="card-title">{q.question}</p>
      {q.options.map((o, i) => (
        <button type="button" key={o.label} className="question-option" aria-pressed={q.multi ? picked.includes(o.label) : undefined} onClick={() => choose(o.label)}>
          <Kbd k={String(i + 1)} />
          <span>
            <b>{o.label}</b>
            {o.description && <small>{o.description}</small>}
          </span>
        </button>
      ))}
      <form
        className="question-other"
        onSubmit={(e) => {
          e.preventDefault();
          if (other.trim() || picked.length) onAnswer(item.id, picked, other.trim() || null);
        }}
      >
        <Kbd k={String(q.options.length + 1)} />
        <TextField label="Other: your own answer" placeholder="Other: your own answer" value={other} onChange={setOther} id={`other-${item.id}`} />
        {(q.multi || other.trim()) && <Button label="Answer" size="sm" type="submit" id={`answer-${item.id}`} />}
      </form>
    </div>
  );
}
