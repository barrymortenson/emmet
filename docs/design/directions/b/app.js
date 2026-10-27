// SPDX-License-Identifier: AGPL-3.0-or-later
// Direction B, Canvas: the model fills the window, and everything else floats on it at the edges. What the agent is
// doing is a capsule at the top center; the chat is light by default (a composer at the bottom with only the latest
// thing above it), and the full conversation opens as a sheet on the right that moves the model aside, never over it.
// Parameters, checks, parts, and steps share one card on the left. Warm, roomy, friendly.
(() => {
  const D = window.EMMET_DATA;
  const U = window.EMMET_UI;
  const S = window.EMMET_STATES;
  const esc = U.esc;

  // The sheet opens where the conversation itself is the point: picking an agent, and right after a switch, where
  // the divider says what happened to the old conversation. Elsewhere "History" opens it (a local toggle).
  const SHEET = new Set(["switch-agent", "switched"]);
  const sheetOpen = (s) => (s.drawer === "history" ? true : s.drawer === "sheet-close" ? false : SHEET.has(s.id));
  const tabOf = (s) => (s.drawer && s.drawer !== "history" && s.drawer !== "sheet-close" ? s.drawer : S.byId[s.id].drawer || "params");

  // An untrusted project is a warning for the agent too: it can't run the model until the user trusts it.
  function top(s) {
    return `<div class="b-top-left">${U.lights()}<button type="button" class="b-chip b-project">${esc(s.imported ? D.imported.name : D.project.name)} ${U.icon("chev")}</button></div>
      <div class="b-now-wrap">${U.agentState(s.banner === "untrusted" ? { ...s, agent: { ...s.agent, state: "offline" } } : s)}</div>
      <div class="b-top-right"><button type="button" class="e-icon-btn" title="New (⌘N)">${U.icon("plus")}</button><button type="button" class="e-icon-btn" title="Open (⌘O)">${U.icon("folder")}</button>
        <button type="button" class="e-icon-btn" data-act="export" title="Export">${U.icon("download")}</button><span class="b-sep"></span>${U.usageMini(s)}</div>`;
  }

  // The left card: the build in words, then Parameters, Checks, Parts, Steps as one segmented control.
  function card(s) {
    const b = s.build;
    const tone = U.buildTone(b);
    const ic = b.state === "building" ? U.spinner() : U.icon(tone === "bad" || tone === "warn" ? (b.state === "readonly" ? "lock" : "warn") : tone === "muted" ? "info" : "check");
    const head = `<div class="b-card-head" data-tone="${tone}">${ic}<span>${esc(U.buildText(b))}</span></div>`;
    if (b.state === "empty") return `<section class="b-card b-card-empty" aria-label="Model">${head}<p class="b-hint">Sliders, checks, the parts list, and steps appear here after the first build.</p></section>`;
    const t = tabOf(s);
    return `<section class="b-card" aria-label="Model">${head}${U.tabs(t, s)}<div class="b-card-body" data-tab="${t}">${U.panel(t, s)}</div></section>`;
  }

  // The finding on the model: a pin beside the chimney with its one-line reason (law 2).
  function pin(s) {
    if (s.build.state !== "findings") return "";
    return `<button type="button" class="b-pin" data-tab="checks"><span class="b-pin-dot">${U.icon("warn")}</span><span class="b-pin-text"><b>8 parts float</b><small>F-0001 · the chimney starts 9 plates up</small></span></button>`;
  }

  function canvas(s, ctx) {
    const failed = s.build.state === "failed" ? `<div class="b-vnote" data-tone="bad">${U.icon("warn")}Build failed · the last good model is still shown</div>` : "";
    return `<div class="b-frame" data-sheet="${sheetOpen(s)}">${U.shot(s.render, ctx.theme)}${pin(s)}</div>${failed}`;
  }

  function tools(s) {
    // The scrubber shows the last step the model has, also for a turn that stopped partway.
    const step = s.step || (s.build.parts ? D.stepOf(s.build.parts) : undefined);
    const scrub = s.build.state === "empty" ? "" : U.scrubber({ ...s, step });
    return `<div class="b-tools" data-sheet="${sheetOpen(s)}">${U.animating(s)}${scrub}${U.views()}</div>`;
  }

  // Above the composer: one short line, the latest word from the agent or what it's running. A card that needs the
  // user (a question, an approval, a redirect waiting, an error, a banner) docks at the bottom right instead, beside
  // the composer, where it can be as tall as it needs without reaching the model.
  function latest(s) {
    if (s.overlay === "model" || s.dirty || s.banner || s.error || s.agent.state === "signedout") return "";
    const list = s.chat;
    const last = list[list.length - 1];
    if (s.build.state === "findings" && s.agent.state === "idle") return `<div class="b-latest b-latest-warn">${U.icon("warn")}<span>The last build has 1 problem: 8 parts float.</span><span class="e-grow"></span>${U.btn("Ask agent to fix", "fix", "e-primary e-sm")}</div>`;
    if (!last) return s.imported ? "" : s.build.state === "empty" ? `<p class="b-tip">${U.icon("spark")}Describe it in your own words. ${esc(U.agentOf(s).name)} builds it from real parts, stage by stage, and you watch it grow.</p>` : "";
    if (last.t === "tool" || last.t === "file" || last.t === "note") return U.message(last, s);
    if (s.pending) return "";
    if (last.t === "thinking") return U.message(last, s).replace('<details class="e-think">', '<details class="e-think" open>');
    if (last.t === "agent") return `<div class="b-latest"><span class="b-latest-who">${esc(U.agentOf(s).name)}</span>${U.message(last, s)}</div>`;
    return "";
  }
  function side(s) {
    if (s.banner) return U.banner(s.banner);
    if (s.agent.state === "signedout") return U.signedOut();
    if (s.error) return U.chat({ ...s, chat: [] });
    if (s.pending) return U.pending(s.pending, U.agentOf(s).name);
    const last = s.chat[s.chat.length - 1];
    if (last && ["question", "approval", "crash"].includes(last.t) && !last.answered) return U.message(last, s);
    return "";
  }

  function compose(s) {
    const a = U.agentOf(s);
    const n = s.chat.length;
    const rows = s.composer ? 3 : 1;
    return `<div class="b-compose"><div class="b-compose-top"><button type="button" class="b-agent" data-act="agents" aria-haspopup="menu">${esc(a.name)} ${U.icon("chev")}</button><span class="e-badge">${esc(a.plan)}</span>
      <span class="e-grow"></span>${n ? `<button type="button" class="e-btn e-quiet e-sm" data-tab="history">${U.icon("chat")}<span>History · ${n}</span></button>` : ""}
      <button type="button" class="e-btn e-quiet e-sm" title="Start fresh: clear the context, keep the agent">${U.icon("fresh")}<span>Start fresh</span></button></div>${U.composer(s, { rows })}</div>`;
  }

  function stack(s) {
    const d = latest(s);
    const c = side(s);
    return `<div class="b-stack">${d ? `<div class="b-dock">${d}</div>` : ""}${compose(s)}</div>${c ? `<div class="b-side" role="region" aria-label="Needs you">${c}</div>` : ""}`;
  }

  function sheet(s) {
    const body = s.chat.length ? U.chat(s) : `<p class="b-hint">No messages yet.</p>`;
    const pending = s.pending ? U.pending(s.pending, U.agentOf(s).name) : "";
    const agents = s.overlay === "agents" ? `<div class="b-pop b-pop-agents">${U.overlay(s)}</div>` : "";
    return `<aside class="b-sheet" aria-label="Conversation">${U.chatHeader(s)}<div class="b-sheet-bar"><span class="e-small">Conversation · ${s.chat.length} messages</span><span class="e-grow"></span><button type="button" class="e-btn e-quiet e-sm" data-tab="sheet-close">${U.icon("x")}<span>Hide</span></button></div>
      <div class="b-msgs" data-scroll="end">${body}${pending}</div>${U.composer(s)}${agents}</aside>`;
  }

  function workspace(s, ctx) {
    const open = sheetOpen(s);
    // Popovers open at the edges: usage and export under the top-right tools, model and effort where B's cards dock.
    const pop = { usage: "b-pop-usage", export: "b-pop-export", model: "b-pop-model" }[s.overlay];
    return `<div class="b-app" data-sheet="${open}">${canvas(s, ctx)}${top(s)}${card(s)}${tools(s)}${open ? sheet(s) : stack(s)}
      ${pop ? `<div class="b-pop ${pop}">${U.overlay(s)}</div>` : ""}</div>`;
  }

  // First run: one big friendly card on the canvas, with the model as the backdrop.
  const BACKDROP = { welcome: "cottage", library: "stage1", agents: "stage2", start: "empty" };
  function firstRun(s, ctx) {
    const { rail, body } = U.firstRun(s.step);
    return `<div class="b-fr"><div class="b-frame b-frame-fr">${U.shot(BACKDROP[s.step], ctx.theme)}</div><div class="b-top-left">${U.lights()}</div>
      <section class="b-fr-card"><div class="b-brand"><span class="b-mark" aria-hidden="true"></span><b>Emmet Builder</b></div>${rail}<div class="b-fr-body">${body}</div></section></div>`;
  }

  window.EMMET_PROTO.direction({
    id: "b", name: "B · Canvas", tagline: "The model fills the window; status on top, chat light at the bottom, panels at the edges",
    render: (s, ctx) => (s.kind === "first-run" ? firstRun(s, ctx) : workspace(s, ctx)),
  });
})();
