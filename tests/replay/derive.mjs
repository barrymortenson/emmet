#!/usr/bin/env node
// SPDX-License-Identifier: AGPL-3.0-or-later
// Builds the derived replay fixtures (tests/replay/{claude,codex}/derived-*.jsonl) from 0B's recordings: the flows
// that use Emmet's real tools (edit, run_model, ask_user) and the failure cases 1C's replay tests need (a crash
// mid-stream, a malformed line, an unknown event, a question canceled by Stop, a redirect sent as the next turn).
// Every line is one of 0B's, cloned and changed only where this script says; each file's meta line names its source.
// The fakes call the real MCP servers, so a tool result here is a placeholder the fake replaces.
//
// Usage: node tests/replay/derive.mjs   (rewrites the derived files; the replay tests read them)
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
// What the rung 1 runs write: the cottage template, the hand-written rung 1 reference (1D's emmet eval dry run).
const cottage = readFileSync(join(here, "../../crates/emmet-core/templates/cottage/model.py"), "utf8");
const rung1 = readFileSync(join(here, "../../docs/spec/ladder.md"), "utf8")
  .split("## Rung 1")[1]
  .split("\n")
  .filter((l) => l.startsWith(">"))
  .map((l) => l.slice(1).trim())
  .join(" ");
const style = {
  question: "Which style should the cottage have?",
  header: "Style",
  options: [
    { label: "Classic: tan walls and a dark red roof", description: "Recommended" },
    { label: "Modern: white walls and a black roof" },
  ],
};
const finished = "The cottage is up: 8 x 6 studs, 6 bricks tall, 93 parts, 0 errors. Sliders: width, depth, wall rows, wall color, roof color, chimney.";
const load = (cli, name) =>
  readFileSync(join(here, cli, `${name}.jsonl`), "utf8")
    .split("\n")
    .filter(Boolean)
    .map((l) => JSON.parse(l));
const clone = (v) => structuredClone(v);
const save = (cli, name, from, what, lines) => {
  const meta = clone(lines[0]);
  meta.msg.meta = { ...meta.msg.meta, scenario: name, derived_from: from, what };
  const out = [meta, ...lines.slice(1)];
  writeFileSync(join(here, cli, `derived-${name}.jsonl`), out.map((l) => JSON.stringify(l)).join("\n") + "\n");
};
const pick = (lines, pred, nth = 0) => {
  const found = lines.filter(pred);
  if (!found[nth]) throw new Error(`no line #${nth} matches ${pred}`);
  return clone(found[nth]);
};
const exit = (code, crash = false) => ({ t: 0, dir: "note", msg: { exit: { code, signal: null, ...(crash && { crash }) } } });

// ---------------------------------------------------------------------------------------------------------- Claude
{
  const call = load("claude", "tool-call");
  const plain = load("claude", "plain-turn");
  const interrupt = load("claude", "interrupt");
  const meta = pick(call, (l) => l.msg.meta);
  const init = pick(call, (l) => l.msg.type === "system" && l.msg.subtype === "init");
  const user = (text) => ({ t: 0, dir: "in", msg: { type: "user", message: { role: "user", content: text } } });
  const use = (msg, id, name, input) => {
    const l = pick(call, (l) => l.msg.type === "assistant" && l.msg.message.content[0].type === "tool_use");
    l.msg.message.id = msg;
    l.msg.message.content = [{ type: "tool_use", id, name, input }];
    return l;
  };
  const result = (id, content) => {
    const l = pick(call, (l) => l.dir === "out" && l.msg.type === "user");
    l.msg.message.content = [{ type: "tool_result", tool_use_id: id, content }];
    delete l.msg.tool_use_result;
    delete l.msg.tool_result_meta;
    return l;
  };
  const text = (msg, t) => {
    const l = pick(call, (l) => l.msg.type === "assistant" && l.msg.message.content[0].type === "text");
    l.msg.message.id = msg;
    l.msg.message.content = [{ type: "text", text: t }];
    return l;
  };
  const done = (t) => {
    const l = pick(call, (l) => l.msg.type === "result");
    l.msg.result = t;
    return l;
  };
  const search = (msg, id, tool) => [
    use(msg, id, "ToolSearch", { query: `select:${tool}`, max_results: 5 }),
    result(id, [{ type: "tool_reference", tool_name: tool }]),
  ];
  const placeholder = [{ type: "text", text: "(the fake replaces this with the real tool result)" }];

  save("claude", "build", "tool-call.jsonl, permissions.jsonl", "an Edit to model.py, then the real run_model", [
    meta,
    user("Make the row 10 bricks wide, then build it."),
    init,
    use("msg_0001", "toolu_edit", "Edit", {
      file_path: "<WS>/project/model.py",
      old_string: "WIDTH = 8",
      new_string: "WIDTH = 10",
      replace_all: false,
    }),
    result("toolu_edit", "The file <WS>/project/model.py has been updated successfully."),
    ...search("msg_0002", "toolu_search", "mcp__emmet__run_model"),
    use("msg_0003", "toolu_run", "mcp__emmet__run_model", {}),
    result("toolu_run", placeholder),
    text("msg_0004", "Built it: the row is 10 bricks wide now."),
    done("Built it: the row is 10 bricks wide now."),
    exit(0),
  ]);

  // Rung 1 through emmet eval (1D): a style question, the whole model written at once, run, validate.
  save("claude", "rung1", "tool-call.jsonl", "rung 1's prompt: a question, model.py written, run_model, validate", [
    meta,
    user(rung1),
    init,
    ...search("msg_0001", "toolu_search1", "mcp__emmet_host__ask_user"),
    use("msg_0002", "toolu_ask", "mcp__emmet_host__ask_user", style),
    result("toolu_ask", placeholder),
    use("msg_0003", "toolu_write", "Write", { file_path: "<WS>/project/model.py", content: cottage }),
    result("toolu_write", "File created successfully at: <WS>/project/model.py"),
    ...search("msg_0004", "toolu_search2", "mcp__emmet__run_model"),
    use("msg_0005", "toolu_run", "mcp__emmet__run_model", {}),
    result("toolu_run", placeholder),
    use("msg_0006", "toolu_validate", "mcp__emmet__validate", {}),
    result("toolu_validate", placeholder),
    text("msg_0007", finished),
    done(finished),
    exit(0),
  ]);

  const ask = {
    question: "How tall should the wall be?",
    header: "Height",
    options: [
      { label: "6 bricks", description: "Recommended: the cottage's height" },
      { label: "3 bricks", description: "A garden wall" },
    ],
  };
  const asking = [
    meta,
    user("Build a wall."),
    init,
    ...search("msg_0001", "toolu_search", "mcp__emmet_host__ask_user"),
    use("msg_0002", "toolu_ask", "mcp__emmet_host__ask_user", ask),
    result("toolu_ask", placeholder),
  ];
  save("claude", "ask-user", "tool-call.jsonl", "a real ask_user question, answered in the same turn", [
    ...asking,
    text("msg_0003", "Six bricks tall, then."),
    done("Six bricks tall, then."),
    exit(0),
  ]);
  // Stop while the question is open: the question is answered "stopped", then the interrupt lands (interrupt.jsonl).
  const tail = interrupt.slice(interrupt.findIndex((l) => l.dir === "in" && l.msg.type === "control_request"));
  // The tail's own rejected tool result (its block call) has no place here: the interrupted call was ask_user.
  save("claude", "ask-user-stopped", "tool-call.jsonl, interrupt.jsonl", "Stop while a question card is open", [
    ...asking,
    ...tail.filter((l) => l.dir !== "stub" && !(l.msg.type === "user" && Array.isArray(l.msg.message?.content))),
  ]);

  // A crash mid-turn (the CLI dies during a tool call), then a second process resumes the session.
  const upTo = call.findIndex((l) => l.msg.type === "assistant" && l.msg.message.content[0].name === "mcp__emmet__ping");
  const second = clone(plain);
  save("claude", "crash-mid-stream", "tool-call.jsonl, plain-turn.jsonl", "the CLI exits mid-turn; resume works", [
    ...call.slice(0, upTo + 1).filter((l) => l.dir !== "stub"),
    { t: 0, dir: "err", msg: "Error: the fake agent crashed on purpose (derived-crash-mid-stream)" },
    exit(1, true),
    ...second,
  ]);

  const firstOut = plain.findIndex((l) => l.dir === "out");
  save("claude", "malformed-line", "plain-turn.jsonl", "a line that isn't JSON mid-stream", [
    ...plain.slice(0, firstOut + 1),
    { t: 0, dir: "raw", msg: '{"type":"assistant","message":{' },
    ...plain.slice(firstOut + 1),
  ]);
  save("claude", "unknown-event", "plain-turn.jsonl", "event types Emmet doesn't know", [
    ...plain.slice(0, firstOut + 1),
    { t: 0, dir: "out", msg: { type: "brand_new_event", detail: "from a future Claude Code" } },
    { t: 0, dir: "out", msg: { type: "system", subtype: "brand_new_subtype" } },
    ...plain.slice(firstOut + 1),
  ]);

  // The same turn twice: a redirect sent while the first turn streams goes out as the second.
  const end = plain.findIndex((l) => l.dir === "note" && l.msg.exit);
  const turn = plain.slice(plain.findIndex((l) => l.dir === "in"), end);
  save("claude", "next-turn", "plain-turn.jsonl", "a redirect delivered as the next turn", [
    ...plain.slice(0, end),
    ...clone(turn),
    plain[end],
  ]);
}

// ----------------------------------------------------------------------------------------------------------- Codex
{
  const call = load("codex", "tool-call");
  const plain = load("codex", "plain-turn");
  const untrusted = load("codex", "permissions-untrusted");
  const interrupt = load("codex", "interrupt");
  const resume = load("codex", "resume");
  const is = (l, method, type) => l.msg.method === method && (!type || l.msg.params?.item?.type === type);
  // tool-call.jsonl up to (not including) its MCP tool call, and from its final agent message on.
  const start = call.findIndex((l) => is(l, "item/started", "mcpToolCall"));
  const finalMsg = call.findIndex(
    (l) => is(l, "item/started", "agentMessage") && l.msg.params.item.phase === "final_answer",
  );
  const head = call.slice(0, start);
  const finish = (text) =>
    call
      .slice(finalMsg)
      .filter((l) => l.dir !== "stub" && l.msg.method !== "item/agentMessage/delta")
      .map((l) => {
        const c = clone(l);
        if (is(c, "item/completed", "agentMessage")) c.msg.params.item.text = text;
        if (c.msg.method === "turn/completed") c.msg.params.turn.items = [];
        return c;
      });
  // One MCP tool call as tool-call.jsonl made it: started, the tool-call elicitation and its answer, resolved,
  // completed (a placeholder the fake replaces), with the server, tool, and arguments changed.
  const tool = (server, name, args, n) => {
    const lines = call.slice(start, finalMsg).filter((l) => l.dir !== "stub" && !is(l, "item/started", "reasoning"));
    return lines
      .filter((l) => !is(l, "item/completed", "reasoning"))
      .map((l) => {
        const c = clone(l);
        const item = c.msg.params?.item;
        if (item?.type === "mcpToolCall") {
          Object.assign(item, { id: `exec-tool-${n}`, server, tool: name, arguments: args });
          if (c.msg.method === "item/completed") item.result = { content: [{ type: "text", text: "(placeholder)" }] };
        }
        if (c.msg.method === "mcpServer/elicitation/request") {
          c.msg.id = 100 + n;
          c.msg.params.serverName = server;
        }
        if (c.dir === "in" && !c.msg.method) c.msg.id = 100 + n;
        if (c.msg.method === "serverRequest/resolved") c.msg.params.requestId = 100 + n;
        return c;
      });
  };
  const turnText = (lines, text) =>
    lines.map((l) => {
      const c = clone(l);
      if (c.dir === "in" && c.msg.method === "turn/start") c.msg.params.input = [{ type: "text", text }];
      return c;
    });
  const change = untrusted
    .filter((l) => (is(l, "item/started", "fileChange") || is(l, "item/completed", "fileChange")))
    .slice(0, 2);

  save("codex", "build", "tool-call.jsonl, permissions-untrusted.jsonl", "a fileChange to model.py, then the real run_model", [
    ...turnText(head, "Make the row 10 bricks wide, then build it."),
    ...change,
    ...tool("emmet", "run_model", {}, 1),
    ...finish("Built it: the row is 10 bricks wide now."),
  ]);
  const ask = {
    question: "How tall should the wall be?",
    header: "Height",
    options: [
      { label: "6 bricks", description: "Recommended: the cottage's height" },
      { label: "3 bricks", description: "A garden wall" },
    ],
  };
  // Rung 1 through emmet eval (1D): a style question, model.py written whole (an add), run, validate.
  const write = change.map((l) => {
    const c = clone(l);
    c.msg.params.item.changes = [{ path: "<WS>/project/model.py", kind: { type: "add" }, diff: cottage }];
    return c;
  });
  save("codex", "rung1", "tool-call.jsonl, permissions-untrusted.jsonl", "rung 1's prompt: a question, model.py written, run_model, validate", [
    ...turnText(head, rung1),
    ...tool("emmet_host", "ask_user", style, 1),
    ...write,
    ...tool("emmet", "run_model", {}, 2),
    ...tool("emmet", "validate", {}, 3),
    ...finish(finished),
  ]);
  save("codex", "ask-user", "tool-call.jsonl", "a real ask_user question, answered in the same turn", [
    ...turnText(head, "Build a wall."),
    ...tool("emmet_host", "ask_user", ask, 1),
    ...finish("Six bricks tall, then."),
  ]);
  // Stop with the question open: ask_user returns "stopped" (its item completes), then turn/interrupt lands
  // (interrupt.jsonl's interrupt and the interrupted turn's end).
  const asked = tool("emmet_host", "ask_user", ask, 1);
  const from = interrupt.findIndex((l) => l.dir === "in" && l.msg.method === "turn/interrupt");
  // interrupt.jsonl's thread and turn ids, changed to tool-call.jsonl's.
  const ids = (lines) => [
    lines.find((l) => l.msg.method === "turn/started").msg.params.threadId,
    lines.find((l) => l.msg.method === "turn/started").msg.params.turn.id,
  ];
  const [fromThread, fromTurn] = ids(interrupt);
  const [toThread, toTurn] = ids(call);
  const stopTail = interrupt
    .slice(from)
    .filter((l) => l.dir !== "stub" && !(l.msg.method === "item/completed" && l.msg.params.item.type === "mcpToolCall"))
    .map((l) => JSON.parse(JSON.stringify(l).replaceAll(fromThread, toThread).replaceAll(fromTurn, toTurn)));
  save("codex", "ask-user-stopped", "tool-call.jsonl, interrupt.jsonl", "Stop while a question card is open", [
    ...turnText(head, "Build a wall."),
    ...asked,
    ...stopTail,
  ]);

  // A crash during the tool call, then resume.jsonl's second process (thread/resume).
  const resumed = resume.slice(resume.findIndex((l, i) => i > 0 && l.msg.meta));
  save("codex", "crash-mid-stream", "tool-call.jsonl, resume.jsonl", "the app-server exits mid-turn; resume works", [
    ...call.slice(0, start + 1).filter((l) => l.dir !== "stub"),
    { t: 0, dir: "err", msg: "Error: the fake agent crashed on purpose (derived-crash-mid-stream)" },
    exit(1, true),
    ...resumed,
  ]);

  const turnStarted = plain.findIndex((l) => l.msg.method === "turn/started");
  save("codex", "malformed-line", "plain-turn.jsonl", "a line that isn't JSON mid-stream", [
    ...plain.slice(0, turnStarted + 1),
    { t: 0, dir: "raw", msg: '{"method":"item/started","params":{' },
    ...plain.slice(turnStarted + 1),
  ]);
  save("codex", "unknown-event", "plain-turn.jsonl", "a notification and an item type Emmet doesn't know", [
    ...plain.slice(0, turnStarted + 1),
    { t: 0, dir: "out", msg: { method: "brand/newNotification", params: { detail: "from a future Codex" } } },
    { t: 0, dir: "out", msg: { method: "item/started", params: { item: { type: "brandNewItem", id: "x1" } } } },
    ...plain.slice(turnStarted + 1),
  ]);

  // The same turn twice: the second turn/start carries the redirect.
  const end = plain.findIndex((l) => l.dir === "note" && l.msg.exit);
  const turnAt = plain.findIndex((l) => l.dir === "in" && l.msg.method === "turn/start");
  const again = plain.slice(turnAt, end).map((l) => {
    const c = clone(l);
    if (c.dir === "in" && c.msg.id != null) c.msg.id += 10;
    if (c.dir === "out" && c.msg.id != null && !c.msg.method) c.msg.id += 10;
    return c;
  });
  save("codex", "next-turn", "plain-turn.jsonl", "a redirect delivered as the next turn", [
    ...plain.slice(0, end),
    ...again,
    plain[end],
  ]);
}
console.log("derive: ok");
