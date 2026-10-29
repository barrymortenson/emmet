// SPDX-License-Identifier: AGPL-3.0-or-later
// The chat pane v0 (plan section 10; Phase 2 designs it): the agent picker with who pays, a streaming message list,
// tool-call, file-change, approval, and question cards, and an open composer: a message sent while the agent works is
// a pending redirect (Edit, Cancel, Stop and send now) until it's delivered. Stop (or Esc) ends the turn at once. After
// a crash, Resume continues the same session in one click. Codex on the ChatGPT plan shows "Using ChatGPT plan" by the
// composer and OpenAI's limit card for a usage-limit error (1E).
import { useCallback, useEffect, useLayoutEffect, useReducer, useRef, useState } from "react";
import {
  type AgentEvent,
  type AgentKind,
  type AgentView,
  type ApprovalKind,
  type ChatRecord,
  type ChatState,
  commands,
  type Decision,
  events,
  type ProjectView,
  type Question,
  type SteerPath,
  type UsageSnapshot,
} from "./bindings";
import { AgentSetup } from "./Agents";
import { LimitCard, PlanLabel } from "./ChatGpt";

/** A redirect that nothing has delivered in this long offers "Stop and send now" (plan section 10). */
const STUCK_MS = 20_000;

type Item =
  | { kind: "user"; key: string; text: string; note?: string; delivered?: SteerPath | null; at: number }
  | { kind: "text" | "reasoning"; key: string; message: string; text: string }
  | { kind: "tool"; key: string; id: string; name: string; input: unknown; result: { ok: boolean; summary: string } | null }
  | { kind: "file"; key: string; path: string; change: string }
  | { kind: "approval"; key: string; id: string; approval: ApprovalKind; tool: string; detail: string; decided: string | null }
  | { kind: "question"; key: string; id: string; question: Question; closed: string | null }
  | { kind: "error"; key: string; message: string; limit: boolean }
  | { kind: "status"; key: string; text: string; resume?: boolean };

type Chat = {
  items: Item[];
  running: boolean;
  /** The open question's id. */
  question: string | null;
  badge: string | null;
  /** On the user's ChatGPT plan (Codex's only plan). */
  plan: boolean;
  model: string | null;
  exited: boolean;
  /** Deliveries that came before the redirect's own bubble (the event can beat chat_steer's reply). */
  early: Record<string, SteerPath>;
  n: number;
};

type Action =
  | { type: "reset"; records: ChatRecord[]; live: boolean }
  | { type: "user"; text: string; note?: string; at: number }
  | { type: "event"; event: AgentEvent }
  | { type: "decided"; id: string; how: string }
  | { type: "note"; id: string; text: string | null };

const empty: Chat = { items: [], running: false, question: null, badge: null, plan: false, model: null, exited: false, early: {}, n: 0 };

const DECIDED: Record<Decision, string> = {
  allow_once: "Allowed once",
  allow_session: "Allowed for this session",
  deny: "Denied",
};

function apply(c: Chat, e: AgentEvent): Chat {
  const key = `k${c.n}`;
  const next = { ...c, n: c.n + 1 };
  const add = (item: Item) => ({ ...next, items: [...c.items, item] });
  const update = (pred: (i: Item) => boolean, f: (i: Item) => Item) => ({
    ...next,
    items: c.items.map((i) => (pred(i) ? f(i) : i)),
  });
  switch (e.type) {
    case "started": {
      const who = e.billing?.kind === "api_key" ? "API key" : e.billing?.kind === "plan" ? `${e.billing.name ? cap(e.billing.name) + " " : ""}plan` : null;
      const plan = e.billing?.kind === "plan" && e.billing.name === "ChatGPT";
      return { ...next, model: e.model, badge: who, plan, exited: false };
    }
    case "text":
    case "reasoning": {
      const last = c.items[c.items.length - 1];
      if (last && last.kind === e.type && last.message === e.message) {
        return { ...next, items: [...c.items.slice(0, -1), { ...last, text: last.text + e.delta }] };
      }
      return add({ kind: e.type, key, message: e.message, text: e.delta });
    }
    case "tool_call":
      // ToolSearch only loads Emmet's deferred tools (ADR 0001): it isn't the agent doing anything.
      if (e.name === "ToolSearch") return next;
      return add({ kind: "tool", key, id: e.id, name: e.name, input: e.input, result: null });
    case "tool_result":
      return update(
        (i) => i.kind === "tool" && i.id === e.id,
        (i) => ({ ...i, result: { ok: e.ok, summary: e.summary } }) as Item,
      );
    case "file_change": {
      const change = e.kind === "added" ? "Added" : e.kind === "removed" ? "Removed" : "Changed";
      const last = c.items[c.items.length - 1];
      if (last?.kind === "file" && last.path === e.path) return next;
      return add({ kind: "file", key, path: e.path, change });
    }
    case "approval":
      return add({ kind: "approval", key, id: e.id, approval: e.kind, tool: e.tool, detail: e.detail, decided: null });
    case "approval_closed":
      return update(
        (i) => i.kind === "approval" && i.id === e.id && !i.decided,
        (i) => ({ ...i, decided: "Closed" }) as Item,
      );
    case "question":
      return { ...add({ kind: "question", key, id: e.id, question: e.question, closed: null }), question: e.id };
    case "question_closed": {
      const done = update(
        (i) => i.kind === "question" && i.id === e.id,
        (i) => ({ ...i, closed: e.reason === "stopped" ? "Stopped before an answer" : (i as { closed: string | null }).closed ?? "Answered" }) as Item,
      );
      return { ...done, question: c.question === e.id ? null : c.question };
    }
    case "note_delivered": {
      const found = c.items.some((i) => i.kind === "user" && i.note === e.id);
      // A redirect sent as the next turn starts that turn.
      const running = e.path === "next_turn" ? true : c.running;
      if (!found) return { ...next, running, early: { ...c.early, [e.id]: e.path } };
      return { ...update((i) => i.kind === "user" && i.note === e.id, (i) => ({ ...i, delivered: e.path }) as Item), running };
    }
    case "turn_done": {
      const s = { ...next, running: false, question: null };
      if (e.reason === "interrupted") return { ...s, items: [...c.items, { kind: "status", key, text: "You stopped the turn." }] };
      return s;
    }
    case "error":
      return add({ kind: "error", key, message: e.message, limit: c.plan && e.code === "quota_exceeded" });
    case "exited":
      return {
        ...add({ kind: "status", key, text: "The agent stopped. The project is as it left it.", resume: true }),
        running: false,
        question: null,
        exited: true,
      };
    default:
      return next;
  }
}

function reduce(c: Chat, a: Action): Chat {
  switch (a.type) {
    case "reset": {
      let s: Chat = { ...empty };
      for (const r of a.records) {
        if (r.user != null) s = reduce(s, { type: "user", text: r.user, at: r.at ?? 0 });
        else if (r.event) s = apply(s, r.event);
      }
      // A chat read back from history that isn't running: its last turn is over.
      return a.live ? s : { ...s, running: false, question: null };
    }
    case "user": {
      const delivered = a.note ? (c.early[a.note] ?? null) : undefined;
      return {
        ...c,
        n: c.n + 1,
        running: true,
        items: [...c.items, { kind: "user", key: `k${c.n}`, text: a.text, note: a.note, delivered, at: a.at }],
      };
    }
    case "event":
      return apply(c, a.event);
    case "decided":
      return { ...c, items: c.items.map((i) => (i.kind === "approval" && i.id === a.id ? { ...i, decided: a.how } : i)) };
    case "note":
      return {
        ...c,
        items: c.items.flatMap((i) => {
          if (i.kind !== "user" || i.note !== a.id) return [i];
          return a.text == null ? [] : [{ ...i, text: a.text }];
        }),
      };
  }
}

function cap(s: string): string {
  return s ? s[0]!.toUpperCase() + s.slice(1) : s;
}

/** One line for a tool card: what the agent is doing, in plain words. */
function describe(name: string, input: unknown, root: string): string {
  const i = (input ?? {}) as Record<string, unknown>;
  const s = (k: string) => (typeof i[k] === "string" ? (i[k] as string) : "");
  // A path inside the project, relative to it; anything else in full.
  const file = (p: string) => {
    const norm = (x: string) => x.replaceAll("\\", "/");
    const base = norm(root).replace(/\/$/, "") + "/";
    return norm(p).startsWith(base) ? norm(p).slice(base.length) : p;
  };
  // A command names files by their full path; inside the project, the relative one says the same, shorter.
  const cmd = (c: string) => {
    const base = root.replaceAll("\\", "/").replace(/\/$/, "") + "/";
    return c.replaceAll("\\", "/").split(base).join("").split(base.slice(0, -1)).join(".");
  };
  const emmet = name.match(/^mcp__emmet(?:_host)?__(.+)$/);
  if (emmet) {
    const tool = emmet[1]!;
    const words: Record<string, string> = {
      run_model: "Run the model",
      render: "Look at the model",
      search_parts: `Search parts: ${s("query")}`,
      part_info: `Check part ${s("id")}`,
      rules: "Read Emmet's rules",
      api: "Read the DSL reference",
      set_params: "Set parameters",
      get_params: "Read the parameters",
      list_colors: "List colors",
      highlight: "Highlight parts",
      ask_user: "Ask you a question",
    };
    return words[tool] ?? tool.replace(/_/g, " ");
  }
  switch (name) {
    case "Edit":
    case "MultiEdit":
      return `Edit ${file(s("file_path"))}`;
    case "Write":
      return `Write ${file(s("file_path"))}`;
    case "Read":
      return `Read ${file(s("file_path"))}`;
    case "Bash":
    case "PowerShell":
    case "shell":
      return `Run: ${cmd(s("command") || JSON.stringify(i["command"] ?? ""))}`;
    case "apply_patch":
      return `Edit ${Array.isArray(i["files"]) ? (i["files"] as string[]).map(file).join(", ") : "files"}`;
    default:
      return name;
  }
}

const KIND_WORDS: Record<ApprovalKind, string> = {
  command: "Run a command",
  edit: "Change a file",
  permission: "Use more access",
  tool: "Use a tool",
};

type Props = {
  project: ProjectView | null;
  /** A message from elsewhere in the app for the agent ("Ask agent to fix"); each new `n` sends it once. */
  ask: { text: string; n: number } | null;
  /** Bumped when Settings changes an agent's sign-in: detect again. */
  agentsChanged: number;
  /** A quiet line before the next turn: a plan limit at 90 % or more. */
  warning: string | null;
  /** The agent's turn ended (plan usage is fetched then, D-36). */
  onTurnEnd: (kind: AgentKind) => void;
  onLimits: (snapshot: UsageSnapshot) => void;
  /** A line for the status bar, or null. */
  onStatus: (text: string | null) => void;
  /** The agent the chat is with. */
  onAgent: (kind: AgentKind | null) => void;
  /** Opens Settings → Agents, where Codex's sign-in lives (1E). */
  onSettings: () => void;
};

export function ChatPane({ project, ask, agentsChanged, warning, onTurnEnd, onLimits, onStatus, onAgent, onSettings }: Props) {
  const [chat, dispatch] = useReducer(reduce, empty);
  const [agents, setAgents] = useState<AgentView[] | null>(null);
  const [state, setState] = useState<ChatState | null>(null);
  const [live, setLive] = useState<{ session: string; kind: AgentKind } | null>(null);
  /** The agent of the chat on screen, live or not, for Resume. */
  const [shown, setShown] = useState<AgentKind | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [now, setNow] = useState(() => Date.now());
  const list = useRef<HTMLDivElement | null>(null);
  /** When the agent last said anything (Date.now()), for the "still working" line. */
  const [heard, setHeard] = useState(() => Date.now());
  const session = useRef<string | null>(null);
  /** Event `at` times waiting for their commit, and the measured event → DOM times (ms): the pixels follow at the next
   * frame. */
  const painting = useRef<number[]>([]);
  const latency = useRef<number[]>([]);

  const load = useCallback(async () => {
    const s = await commands.chatState();
    setState(s);
    setLive(s.live);
    setShown(s.live?.kind ?? s.last?.kind ?? null);
    session.current = s.live?.session ?? null;
    dispatch({ type: "reset", records: s.history, live: s.live != null });
    onAgent(s.live?.kind ?? s.last?.kind ?? null);
  }, [onAgent]);

  useEffect(() => {
    void commands.agents(agentsChanged > 0).then(setAgents);
  }, [agentsChanged]);

  useEffect(() => {
    // A message about the last project's chat doesn't carry over to this one.
    setError(null);
    if (project) void load();
    else dispatch({ type: "reset", records: [], live: false });
  }, [project?.root, load]);

  useEffect(() => {
    const off = events.chatEvent.listen(({ payload }) => {
      if (payload.session !== session.current) return;
      painting.current.push(payload.at ?? Date.now());
      setHeard(Date.now());
      dispatch({ type: "event", event: payload.event });
      const e = payload.event;
      if (e.type === "limits") onLimits(e.snapshot);
      if (e.type === "turn_done" && live) onTurnEnd(live.kind);
      if (e.type === "exited") setLive(null);
    });
    return () => void off.then((f) => f());
  }, [live, onLimits, onTurnEnd]);

  // Event → pixels (plan section 10's 50 ms): measured when React has put the event's change in the DOM, which the
  // next frame paints. (A requestAnimationFrame timestamp would measure the window's own throttling when it's behind
  // another window, not Emmet.)
  useLayoutEffect(() => {
    if (!painting.current.length) return;
    const t = Date.now();
    for (const at of painting.current) latency.current.push(t - at);
    painting.current = [];
    list.current?.scrollTo({ top: list.current.scrollHeight });
  }, [chat]);

  useEffect(() => {
    if (__EMMET_TEST_HOOKS__) {
      const w = window as unknown as { __emmet?: Record<string, unknown> };
      w.__emmet = { ...(w.__emmet ?? {}), chatLatency: () => latency.current.slice() };
    }
  }, []);

  useEffect(() => {
    onStatus(chat.question ? "Waiting for your answer" : chat.running ? "The agent is working" : null);
  }, [chat.question, chat.running, onStatus]);

  // Pending redirects older than STUCK_MS offer "Stop and send now", and the "still working" line counts seconds.
  const pending = chat.items.some((i) => i.kind === "user" && i.note && i.delivered == null);
  useEffect(() => {
    if (!pending && !chat.running) return;
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, [pending, chat.running]);

  const run = async (f: () => Promise<{ status: "ok" } | { status: "error"; error: { message: string; hint: string } }>) => {
    const r = await f();
    setError(r.status === "error" ? `${r.error.message} ${r.error.hint}`.trim() : null);
    return r.status === "ok";
  };

  const start = async (kind: AgentKind, resume: boolean) => {
    const r = await commands.chatStart(kind, resume);
    if (r.status === "error") {
      setError(`${r.error.message} ${r.error.hint}`.trim());
      return;
    }
    setError(null);
    session.current = r.data;
    setLive({ session: r.data, kind });
    setShown(kind);
    onAgent(kind);
    if (!resume) dispatch({ type: "reset", records: [], live: true });
  };

  /** Sends what's in the composer, or `given` (a message from elsewhere in the app): a turn, a redirect while the agent
   * works, or the open question's Other answer. */
  const send = async (given?: string) => {
    const t = (given ?? text).trim();
    if (!t || !live) return;
    if (given == null) setText("");
    // While a question is open, what the user types is its Other answer (plan section 10).
    if (chat.question) {
      await run(() => commands.chatAnswer(chat.question!, { choices: [], other: t }));
      return;
    }
    if (chat.running) {
      const r = await commands.chatSteer(t);
      if (r.status === "ok") dispatch({ type: "user", text: t, note: r.data.id, at: Date.now() });
      else setError(r.error.message);
      return;
    }
    dispatch({ type: "user", text: t, at: Date.now() });
    setHeard(Date.now());
    await run(() => commands.chatSend(t));
  };

  const stop = () => void run(() => commands.chatStop());

  // "Ask agent to fix": the finding goes to the live session like a message the user typed.
  const asked = useRef(0);
  useEffect(() => {
    if (!ask || ask.n === asked.current) return;
    asked.current = ask.n;
    if (!live) {
      setError("Start a chat with an agent first, then ask again.");
      return;
    }
    void send(ask.text);
    // Only a new ask sends; send reads the session as it is then.
  }, [ask]);

  const onKey = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void send();
    } else if (e.key === "Escape" && chat.running) {
      e.preventDefault();
      stop();
    }
  };

  if (!project) return null;
  const agent = live ? agents?.find((a) => a.kind === live.kind) : null;
  // Who pays (D-35): what the session's start said, else what detection found.
  const pays = chat.badge ?? agent?.badge.split(" · ")[1] ?? null;

  return (
    <section className="chat" aria-labelledby="chat-title">
      <div className="chat-head">
        <h2 id="chat-title">Chat</h2>
        {live && (
          <span className="badge" id="chat-badge">
            {live.kind === "claude" ? "Claude" : "Codex"}
            {pays && ` · ${pays}`}
          </span>
        )}
        {live && chat.running && (
          <button type="button" id="chat-stop" onClick={stop} title="Stop the turn (Esc)">
            Stop
          </button>
        )}
        {live && !chat.running && (
          <button
            type="button"
            id="chat-new"
            onClick={async () => {
              await commands.chatEnd();
              session.current = null;
              setLive(null);
              dispatch({ type: "reset", records: [], live: false });
              void commands.chatState().then(setState);
            }}
          >
            New chat
          </button>
        )}
      </div>
      {!live && (
        <Picker
          agents={agents}
          last={state?.last ?? null}
          onStart={start}
          onRefresh={() => void commands.agents(true).then(setAgents)}
          onSettings={onSettings}
        />
      )}
      {agent?.warning && <p className="muted warning">{agent.warning}</p>}
      {error && (
        <p className="notice" role="alert" id="chat-error">
          {error}
        </p>
      )}
      <div className="messages" role="log" aria-live="polite" ref={list} id="chat-log">
        {chat.items.map((i) => (
          <Entry
            key={i.key}
            item={i}
            root={project.root}
            now={now}
            open={chat.question}
            onDecide={(id, d) => {
              dispatch({ type: "decided", id, how: DECIDED[d] });
              void run(() => commands.chatRespond(id, d));
            }}
            onAnswer={(id, choices, other) => void run(() => commands.chatAnswer(id, { choices, other }))}
            onNote={async (id, t) => {
              const r = await commands.chatNote(id, t);
              if (r.status === "ok" && r.data) dispatch({ type: "note", id, text: t });
            }}
            onStopAndSend={stop}
            onResume={() => live == null && shown && void start(shown, true)}
          />
        ))}
      </div>
      {/* Pinned above the composer, so it shows however the log is scrolled: an agent can think, or run its own hooks,
          for a minute without a word (1C's live check). */}
      {live && chat.running && (
        <p className="working" id="chat-working" role="status">
          {chat.question ? (
            "Waiting for your answer"
          ) : (
            <>
              <span className="spinner" aria-hidden="true" />
              {live.kind === "claude" ? "Claude Code" : "Codex"} is working
              {now - heard >= 3000 ? ` · ${Math.round((now - heard) / 1000)} s since its last update` : "…"}
            </>
          )}
        </p>
      )}
      {live && warning && !chat.running && <p className="muted warning">{warning}</p>}
      {live?.kind === "codex" && chat.plan && <PlanLabel />}
      {live && (
        <div className="composer">
          <textarea
            id="chat-input"
            aria-label={chat.question ? "Your answer, in your own words" : chat.running ? "A message for the agent while it works" : "A message for the agent"}
            placeholder={chat.question ? "Answer in your own words, or pick an option" : chat.running ? "Redirect the agent…" : "Ask the agent to build something"}
            value={text}
            rows={2}
            onChange={(e) => setText(e.target.value)}
            onKeyDown={onKey}
          />
          <button type="button" id="chat-send" onClick={() => void send()} disabled={!text.trim()}>
            Send
          </button>
        </div>
      )}
    </section>
  );
}

function Picker({
  agents,
  last,
  onStart,
  onRefresh,
  onSettings,
}: {
  agents: AgentView[] | null;
  last: { kind: AgentKind } | null;
  onStart: (kind: AgentKind, resume: boolean) => void;
  onRefresh: () => void;
  onSettings: () => void;
}) {
  if (!agents) return <p className="muted">Looking for Claude Code and Codex…</p>;
  return (
    <div className="picker" role="group" aria-label="Agents">
      {agents.map((a) => {
        const ok = a.detection.path != null && a.detection.signed_in;
        return (
          <div key={a.kind} className="agent">
            <button type="button" id={`agent-${a.kind}`} disabled={!ok} onClick={() => onStart(a.kind, false)}>
              {a.name}
            </button>
            {ok && <span className="badge">{a.badge}</span>}
            {ok && last?.kind === a.kind && (
              <button type="button" id={`resume-${a.kind}`} onClick={() => onStart(a.kind, true)}>
                Resume last chat
              </button>
            )}
            {ok && a.warning && <p className="muted fix">{a.warning}</p>}
            {ok && a.detection.note && <p className="muted">{a.detection.note}</p>}
          </div>
        );
      })}
      <AgentSetup agents={agents} only="unready" onChanged={onRefresh} onSettings={onSettings} />
      <button type="button" id="agents-again" onClick={onRefresh}>
        Check agents again
      </button>
    </div>
  );
}

function Entry({
  item,
  root,
  now,
  open,
  onDecide,
  onAnswer,
  onNote,
  onStopAndSend,
  onResume,
}: {
  item: Item;
  root: string;
  now: number;
  open: string | null;
  onDecide: (id: string, d: Decision) => void;
  onAnswer: (id: string, choices: string[], other: string | null) => void;
  onNote: (id: string, text: string | null) => void;
  onStopAndSend: () => void;
  onResume: () => void;
}) {
  switch (item.kind) {
    case "user":
      return <UserBubble item={item} now={now} onNote={onNote} onStopAndSend={onStopAndSend} />;
    case "text":
      return <p className="msg agent">{item.text}</p>;
    case "reasoning":
      return (
        <details className="msg reasoning">
          <summary>Thinking</summary>
          <p>{item.text}</p>
        </details>
      );
    case "tool":
      return (
        <details className={`card tool ${item.result && !item.result.ok ? "failed" : ""}`} data-tool={item.name}>
          <summary>
            {describe(item.name, item.input, root)}
            {item.result == null ? " …" : item.result.ok ? "" : " (failed)"}
          </summary>
          <pre>{JSON.stringify(item.input, null, 2)}</pre>
          {item.result && <pre className="result">{item.result.summary}</pre>}
        </details>
      );
    case "file":
      return (
        <p className="card file" data-file={item.path}>
          {item.change} <code>{item.path}</code>
        </p>
      );
    case "approval":
      return (
        <div className="card approval" role="group" aria-label="Approval" data-approval={item.id}>
          <p>
            <strong>{KIND_WORDS[item.approval]}</strong> ({item.tool})
          </p>
          <pre>{item.detail}</pre>
          {item.decided ? (
            <p className="muted">{item.decided}</p>
          ) : (
            <div className="actions">
              <button type="button" onClick={() => onDecide(item.id, "allow_once")}>
                Allow once
              </button>
              <button type="button" onClick={() => onDecide(item.id, "allow_session")}>
                Allow for this session
              </button>
              <button type="button" onClick={() => onDecide(item.id, "deny")}>
                Deny
              </button>
            </div>
          )}
        </div>
      );
    case "question":
      return <QuestionCard item={item} open={open === item.id} onAnswer={onAnswer} />;
    case "error":
      if (item.limit)
        return (
          <div>
            <LimitCard />
            <details className="muted">
              <summary>What OpenAI said</summary>
              <pre>{item.message}</pre>
            </details>
          </div>
        );
      return (
        <p className="card error-card" role="alert">
          {item.message}
        </p>
      );
    case "status":
      return (
        <p className="status-line">
          {item.text}{" "}
          {item.resume && (
            <button type="button" id="chat-resume" onClick={onResume}>
              Resume
            </button>
          )}
        </p>
      );
  }
}

function UserBubble({
  item,
  now,
  onNote,
  onStopAndSend,
}: {
  item: Extract<Item, { kind: "user" }>;
  now: number;
  onNote: (id: string, text: string | null) => void;
  onStopAndSend: () => void;
}) {
  const [editing, setEditing] = useState<string | null>(null);
  const pending = item.note != null && item.delivered == null;
  if (!pending) {
    return (
      <p className="msg user" data-note={item.note}>
        {item.text}
        {item.note && <span className="delivered"> · Delivered</span>}
      </p>
    );
  }
  return (
    <div className="msg user pending" data-note={item.note}>
      {editing == null ? (
        <p>{item.text}</p>
      ) : (
        <input aria-label="Edit your message" value={editing} onChange={(e) => setEditing(e.target.value)} />
      )}
      <p className="muted">Delivered at the agent's next step</p>
      <div className="actions">
        {editing == null ? (
          <button type="button" onClick={() => setEditing(item.text)}>
            Edit
          </button>
        ) : (
          <button
            type="button"
            onClick={() => {
              onNote(item.note!, editing);
              setEditing(null);
            }}
          >
            Save
          </button>
        )}
        <button type="button" onClick={() => onNote(item.note!, null)}>
          Cancel
        </button>
        {now - item.at > STUCK_MS && (
          <button type="button" onClick={onStopAndSend}>
            Stop and send now
          </button>
        )}
      </div>
    </div>
  );
}

function QuestionCard({
  item,
  open,
  onAnswer,
}: {
  item: Extract<Item, { kind: "question" }>;
  open: boolean;
  onAnswer: (id: string, choices: string[], other: string | null) => void;
}) {
  const q = item.question;
  const [picked, setPicked] = useState<string[]>([]);
  const [other, setOther] = useState("");
  const card = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    if (open) card.current?.focus();
  }, [open]);
  const choose = (label: string) => {
    if (!q.multi) onAnswer(item.id, [label], null);
    else setPicked((p) => (p.includes(label) ? p.filter((x) => x !== label) : [...p, label]));
  };
  // Number keys pick an option (plan section 10).
  const onKey = (e: React.KeyboardEvent) => {
    if (!open || (e.target as HTMLElement).tagName === "INPUT") return;
    const n = Number(e.key);
    if (n >= 1 && n <= q.options.length) {
      e.preventDefault();
      choose(q.options[n - 1]!.label);
    }
  };
  return (
    <div className="card question" role="group" aria-label="Question" tabIndex={-1} ref={card} onKeyDown={onKey} data-question={item.id}>
      {q.header && <span className="badge">{q.header}</span>}
      <p>
        <strong>{q.question}</strong>
      </p>
      {item.closed ? (
        <p className="muted">{item.closed}</p>
      ) : (
        <>
          <ol className="options">
            {q.options.map((o, i) => (
              <li key={o.label}>
                <button type="button" aria-pressed={picked.includes(o.label)} onClick={() => choose(o.label)}>
                  <kbd>{i + 1}</kbd> {o.label}
                </button>
                {o.description && <span className="muted"> {o.description}</span>}
              </li>
            ))}
          </ol>
          <div className="other">
            <input aria-label="Other: your own answer" placeholder="Other…" value={other} onChange={(e) => setOther(e.target.value)} />
            <button
              type="button"
              disabled={!other.trim() && !picked.length}
              onClick={() => onAnswer(item.id, picked, other.trim() || null)}
            >
              Answer
            </button>
          </div>
        </>
      )}
    </div>
  );
}
