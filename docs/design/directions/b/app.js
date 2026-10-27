// SPDX-License-Identifier: AGPL-3.0-or-later
// Direction B, Canvas (Barry's revision, 2026-10-27): the viewer takes the left of the window, the options panel the
// right, and the chat docks along the bottom of the viewer. What the agent is doing is a capsule at the top center of
// the viewer. The chat is light by default (the latest line, or the card that needs you, then the composer); its
// history opens upward and pushes the viewer up, never over the model. Options leads with the build settings (style,
// scale, detail, size), which steer the agent, then the model's own tweaks. Warm, roomy, friendly.
(() => {
  const D = window.EMMET_DATA;
  const U = window.EMMET_UI;
  const S = window.EMMET_STATES;
  const esc = U.esc;
  // The brand's ant (brand/mark-ant.svg): in the project chip, which opens About.
  const ANT = `<img class="b-ant" src="../../brand/mark-ant.svg" alt="">`;

  // History opens where the conversation itself is the point: picking an agent, and right after a switch. Elsewhere
  // "History" opens it (a local toggle).
  const HISTORY = new Set(["switch-agent", "switched"]);
  const historyOpen = (s) => (s.history === "open" ? true : s.history === "closed" ? false : HISTORY.has(s.id));
  const tabOf = (s) => s.drawer || S.byId[s.id].drawer || "params";

  // The viewer: the render, the project and the agent's capsule on top, views and the scrubber at the right.
  function viewer(s, ctx) {
    const b = s.build;
    const failed = b.state === "failed" ? `<div class="b-vnote" data-tone="bad">${U.icon("warn")}Build failed · the last good model is still shown</div>` : "";
    const ro = b.state === "readonly" ? `<div class="b-vnote" data-tone="warn">${U.icon("lock")}Read-only · model.py hasn't run</div>` : "";
    const pin = b.state === "findings" ? `<button type="button" class="b-pin" data-tab="checks"><span class="b-pin-dot">${U.icon("warn")}</span><span class="b-pin-text"><b>9 parts float</b><small>F-0001 · the chimney starts 9 plates up</small></span></button>` : "";
    const step = s.step || (b.parts ? D.stepOf(b.parts) : undefined);
    const scrub = b.state === "empty" ? "" : U.scrubber({ ...s, step });
    return `<header class="b-title"><div class="b-top-left">${U.lights()}<button type="button" class="b-chip b-project" data-act="about" title="Emmet Builder: About, Settings">${ANT}${esc(s.imported ? D.imported.name : D.project.name)} ${U.icon("chev")}</button></div>
      <span class="e-grow"></span><div class="b-tools-row">${scrub}${U.views()}</div></header>${s.banner ? `<div class="b-banner">${U.banner(s.banner)}</div>` : ""}
      <section class="b-viewer" aria-label="3D viewer">${U.shot(s.render, ctx.theme)}${pin}<div class="b-vnotes">${failed}${ro}</div>
      <div class="b-tools">${U.animating(s)}</div></section>`;
  }

  // The options panel: the build in words and the project's tools, then Options, Checks, Parts, Steps.
  function panel(s) {
    const b = s.build;
    const tone = U.buildTone(b);
    const ic = b.state === "building" ? U.spinner() : U.icon(tone === "bad" || tone === "warn" ? (b.state === "readonly" ? "lock" : "warn") : tone === "muted" ? "info" : "check");
    const t = tabOf(s);
    const tabs = U.tabs(t, s).replace(">Parameters<", ">Options<");
    let body;
    if (t === "params") {
      // The model's knobs need a model, and a trusted one; until then the section is left out.
      const tweaks = b.state === "empty" ? "" : b.state === "readonly"
        ? `<div class="b-tweaks"><p class="b-sec">Tweaks</p><p class="b-locked">${U.icon("lock")} Trust the project to use these</p></div>`
        : `<div class="b-tweaks"><p class="b-sec">Tweaks</p>${U.params(s)}</div>`;
      // Build settings can be switched off: the prompt alone then decides. On, before the first build or with a change
      // waiting they're open; after it, one summary line.
      const on = s.briefOn !== "off";
      // Before the first build the settings are open; after it they're one summary line, Change opens them, and Done
      // folds them back (pending changes stay, with Undo and Rebuild on the summary).
      const full = b.state === "empty" || (s.briefDirty && s.brief !== "closed") || s.brief === "open";
      const toggle = `<label class="b-switch" title="${on ? "Turn build settings off: the prompt alone decides" : "Turn build settings on"}"><input type="checkbox" role="switch" data-local="briefOn=${on ? "off" : "on"}"${on ? " checked" : ""} aria-label="Build settings"><span></span></label>`;
      const done = on && full && b.state !== "empty" ? `<button type="button" class="e-btn e-quiet e-sm" data-local="brief=closed">Done</button>` : "";
      body = `<div class="b-sec b-sec-row"><span>Build settings</span><span class="b-sec-tools">${done}${toggle}</span></div>${on ? U.buildSettings(s, { compact: !full, open: "brief=open" }) : `<p class="b-off">Off: the prompt decides</p>`}${tweaks}`;
    } else body = U.panel(t, s);
    return `<aside class="b-panel" aria-label="Options"><div class="b-panel-head" data-tone="${tone}">${ic}<span class="b-build">${esc(U.buildText(b))}</span>
        <span class="b-tools-top"><button type="button" class="e-icon-btn" title="New (⌘N)">${U.icon("plus")}</button><button type="button" class="e-icon-btn" title="Open (⌘O)">${U.icon("folder")}</button><button type="button" class="e-icon-btn" data-act="export" title="Export">${U.icon("download")}</button></span></div>
      <div class="b-usage-row">${U.usageMini(s)}</div>${tabs}<div class="b-panel-body" data-tab="${t}">${body}</div>
      ${s.overlay === "usage" || s.overlay === "export" ? `<div class="b-pop b-pop-${s.overlay}">${U.overlay(s)}</div>` : ""}</aside>`;
  }

  // Above the composer: the card that needs you (a question, an approval, a redirect waiting, an error, a crash, a
  // sign-in), or else one short line, the latest word from the agent or what it's running.
  function active(s) {
    if (s.agent.state === "signedout") return U.signedOut();
    if (s.error) return U.chat({ ...s, chat: [] });
    if (s.pending) return U.pending(s.pending, U.agentOf(s).name);
    const last = s.chat[s.chat.length - 1];
    if (last && ["question", "approval", "crash"].includes(last.t) && !last.answered) return U.message(last, s);
    return "";
  }
  function latest(s) {
    if (s.dirty || s.briefDirty) return "";
    const last = s.chat[s.chat.length - 1];
    if (s.build.state === "findings" && s.agent.state === "idle") return `<div class="b-latest b-latest-warn">${U.icon("warn")}<span>The last build has 1 problem: 9 parts float.</span><span class="e-grow"></span>${U.btn("Ask agent to fix", "fix", "e-primary e-sm")}</div>`;
    if (!last) return "";
    if (last.t === "tool" || last.t === "file" || last.t === "note") return U.message(last, s);
    if (last.t === "thinking") return U.message(last, s).replace('<details class="e-think">', '<details class="e-think" open>');
    if (last.t === "agent") return `<div class="b-latest"><span class="b-latest-who">${esc(U.agentOf(s).name)}</span>${U.message(last, s)}</div>`;
    return "";
  }

  function chat(s) {
    const a = U.agentOf(s);
    const open = historyOpen(s);
    const n = s.chat.length;
    // What the agent is doing (note 1) leads the chat, beside the agent it's about. An untrusted project is a warning
    // for the agent too: it can't run the model until the user trusts it.
    const now = U.agentState(s.banner === "untrusted" ? { ...s, agent: { ...s.agent, state: "offline" } } : s);
    const top = `<div class="b-chat-top"><button type="button" class="b-agent" data-act="agents" aria-haspopup="menu">${esc(a.name)} ${U.icon("chev")}</button><span class="e-badge">${esc(a.plan)}</span><div class="b-now">${now}</div>
      ${n ? `<button type="button" class="e-btn e-quiet e-sm" data-local="history=${open ? "closed" : "open"}" aria-expanded="${open}">${U.icon("chat")}<span>${open ? "Hide history" : `History · ${n}`}</span></button>` : ""}
      <button type="button" class="e-btn e-quiet e-sm" title="Start fresh: clear the context, keep the agent">${U.icon("fresh")}<span>Start fresh</span></button></div>`;
    const card = active(s);
    const mid = open
      ? `<div class="b-msgs" data-scroll="end">${n ? U.chat(s) : ""}${card}</div>`
      : card ? `<div class="b-active">${card}</div>` : (() => { const l = latest(s); return l ? `<div class="b-line">${l}</div>` : ""; })();
    const pops = s.overlay === "agents" || s.overlay === "model" ? `<div class="b-pop b-pop-${s.overlay}">${U.overlay(s)}</div>` : "";
    return `<section class="b-chat" data-open="${open}" aria-label="Chat">${top}${mid}${U.composer(s, { rows: s.composer ? 3 : 1 })}${pops}</section>`;
  }

  function workspace(s, ctx) {
    return `<div class="b-app"><div class="b-left">${viewer(s, ctx)}${chat(s)}</div>${panel(s)}</div>`;
  }

  // First run: one big friendly card on the canvas, with the model as the backdrop.
  const BACKDROP = { welcome: "cottage", library: "stage1", agents: "stage2", start: "empty" };
  function firstRun(s, ctx) {
    const { rail, body } = U.firstRun(s.step);
    return `<div class="b-fr"><div class="b-frame-fr">${U.shot(BACKDROP[s.step], ctx.theme)}</div><div class="b-top-left">${U.lights()}</div>
      <section class="b-fr-card"><img class="b-wordmark" src="../../brand/wordmark-emmet-builder-ant-${ctx.theme}.svg" alt="Emmet Builder">${rail}<div class="b-fr-body">${body}</div></section></div>`;
  }

  window.EMMET_PROTO.direction({
    id: "b", name: "B · Canvas", tagline: "The viewer on the left, options on the right, the chat along the bottom",
    render: (s, ctx) => (s.kind === "first-run" ? firstRun(s, ctx) : workspace(s, ctx)),
  });
})();
