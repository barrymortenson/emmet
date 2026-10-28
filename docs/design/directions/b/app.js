// SPDX-License-Identifier: AGPL-3.0-or-later
// Direction B, Canvas (Barry's revision, 2026-10-27): the viewer takes the left of the window, the options panel the
// right, and the chat docks along the bottom of the viewer, led by what the agent is doing. The chat is light by default (the latest line, or the card that needs you, then the composer); its
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

  // The viewer: the render, with the project at the left of its top row and the views centered. No step scrubber: the
  // Steps tab picks a step, and the step's number shows over the viewer (Barry 2026-10-27).
  function viewer(s, ctx) {
    const b = s.build;
    // Say each thing once (Barry 2026-10-27): a failed build and a read-only project are told by the chat card and the
    // banner, so the viewer only carries the finding's pin, which points at the parts.
    const pin = b.state === "findings" ? `<button type="button" class="b-pin" data-tab="checks"><span class="b-pin-dot">${U.icon("warn")}</span><b>9 parts float</b></button>` : "";
    // Versions (plan section 07): the project menu opens Save version and the list; a version opened from the list
    // takes the viewer, read-only, with a bar to restore it, copy it out, or go back.
    const seen = s.view && D.history.find((v) => String(v.n) === s.view);
    const pop = s.menu === "open" ? projectMenu() : s.menu === "save" ? saveVersion() : s.menu === "versions" && !seen ? versionsList() : "";
    const bar = seen ? `<div class="b-vbar" role="status">${U.icon("clock")}<span title="Version ${seen.n}: ${esc(seen.name)} · ${esc(seen.when)} · ${seen.parts} parts"><b>Version ${seen.n}: ${esc(seen.name)}</b> · ${esc(seen.when)}</span>
        <button type="button" class="e-btn e-primary e-sm" data-act="restore" title="Saves the current model as a version first">${U.icon("undo")}<span>Restore</span></button>
        <button type="button" class="e-btn e-sm" data-act="copy-out" title="Copies this version to a new project, to try another direction">Open as new</button>
        <button type="button" class="e-btn e-quiet e-sm" data-local="view=">Back</button></div>` : "";
    return `<header class="b-title"><div class="b-top-left">${U.lights()}<button type="button" class="b-chip b-project" data-local="menu=${s.menu === "open" ? "closed" : "open"}" aria-haspopup="menu">${ANT}${esc(s.imported ? D.imported.name : D.project.name)} ${U.icon("chev")}</button>${pop}</div>
      ${U.views()}<div class="b-title-right"></div></header>${s.banner ? `<div class="b-banner">${U.banner(s.banner)}</div>` : ""}
      <section class="b-viewer" aria-label="3D viewer" data-version="${!!seen}">${U.shot(seen ? seen.render : s.render, ctx.theme)}${seen ? "" : pin + U.stepView(s)}${bar}
      <div class="b-tools">${U.animating(s, { bare: true })}</div></section>`;
  }

  // The project menu, from the project chip: what used to be the panel's New and Open icons, and About.
  function projectMenu() {
    const item = (ic, label, act, key, local) => `<button type="button" class="e-menu-item" role="menuitem"${act ? ` data-act="${act}"` : ""}${local ? ` data-local="${local}"` : ""}>${U.icon(ic)}<span>${esc(label)}</span>${key ? `<span class="e-small">${esc(key)}</span>` : ""}</button>`;
    return `<div class="b-pop b-pop-project"><div class="e-menu" role="menu">${item("plus", "New", "", "⌘N")}${item("folder", "Open…", "", "⌘O")}<hr class="b-hr">${item("save", "Save version", "", "⌘S", "menu=save")}${item("clock", "Versions…", "", "", "menu=versions")}<hr class="b-hr">${item("download", "Export…", "export", "⌘E")}<hr class="b-hr">${item("info", "About Emmet Builder", "about")}</div></div>`;
  }

  // Save version: a name (the next number until the user types one), and Save. Nothing else to decide.
  function saveVersion() {
    return `<div class="b-pop b-pop-project"><div class="e-menu e-menu-wide b-save" role="dialog" aria-label="Save version"><b>Save version</b>
      <input class="e-style-input" type="text" value="Steeper roof" aria-label="Version name">
      <div class="b-save-row"><button type="button" class="e-btn e-quiet e-sm" data-local="menu=closed">Cancel</button><button type="button" class="e-btn e-primary e-sm" data-local="menu=versions">${U.icon("save")}<span>Save</span></button></div></div></div>`;
  }

  // The versions list, newest first: the user's named versions and Emmet's own from before each agent turn, each with
  // its render. Clicking one opens it in the viewer to compare.
  function versionsList() {
    const rows = D.history.map((v) => `<button type="button" class="b-ver" data-local="view=${v.n}" data-auto="${!!v.auto}">
        <img src="${window.EMMET_ROOT}shared/renders/${document.documentElement.dataset.theme || "light"}/${v.render}.webp" alt="">
        <span><b>${esc(v.name)}</b><span class="e-small">${esc(v.when)} · ${v.parts} parts${v.auto ? " · automatic" : ""}</span></span></button>`).join("");
    return `<div class="b-pop b-pop-project"><div class="e-menu e-menu-wide b-vers" role="dialog" aria-label="Versions"><div class="b-vers-head"><b>Versions</b>
      <button type="button" class="e-btn e-sm" data-local="menu=save">${U.icon("save")}<span>Save version</span></button></div>${rows}</div></div>`;
  }

  // The model's state in a few words; the tab counts already give the parts and steps.
  function modelState(b) {
    return { empty: "Empty", building: "Building", failed: "Build failed", readonly: "Read-only", findings: `${b.findings} problem${b.findings > 1 ? "s" : ""}` }[b.state] || "No problems";
  }

  // The options panel: the build in words and the project's tools, then Options, Checks, Parts, Steps.
  function panel(s) {
    const b = s.build;
    const tone = U.buildTone(b);
    const ic = U.icon(tone === "bad" || tone === "warn" ? (b.state === "readonly" ? "lock" : "warn") : tone === "busy" ? "cube" : tone === "muted" ? "info" : "check");
    const t = tabOf(s);
    const tabs = U.tabs(t, s).replace(">Parameters<", ">Options<");
    let body;
    if (t === "params") {
      // The model's knobs need a model, and a trusted one; until then the section is left out.
      // No model.py, no tweaks: an imported file has none until the agent makes it parametric.
      const tweaks = b.state === "empty" || s.imported ? "" : b.state === "readonly"
        ? `<div class="b-tweaks"><p class="b-sec">Tweaks</p><p class="b-locked">${U.icon("lock")} Trust the project to use these</p></div>`
        : `<div class="b-tweaks"><p class="b-sec">Tweaks</p>${U.params(s)}</div>`;
      // Before the first build the settings are open; after it they're one summary line, Change opens them, and Done
      // folds them back (pending changes stay, with Undo and Rebuild on the summary).
      const full = b.state === "empty" || (s.briefDirty && s.brief !== "closed") || s.brief === "open";
      // No on/off switch (Barry): every setting starts on Any, which already leaves the prompt to decide.
      const done = full && b.state !== "empty" ? `<button type="button" class="e-btn e-quiet e-sm" data-local="brief=closed">Done</button>` : "";
      body = `<div class="b-sec b-sec-row"><span title="Together they set how the agent designs the model">Build settings</span><span class="b-sec-tools">${done}</span></div>${U.buildSettings(s, { compact: !full, open: "brief=open" })}${tweaks}`;
    } else body = U.panel(t, s);
    return `<aside class="b-panel" aria-label="Options"><div class="b-panel-head" data-tone="${tone}">${ic}<span class="b-build">${esc(modelState(b))}</span>
        ${b.parts ? `<button type="button" class="e-btn e-sm" data-act="export">${U.icon("download")}<span>Export</span></button>` : ""}</div>${tabs}<div class="b-panel-body" data-tab="${t}">${body}</div>
      ${s.overlay === "export" ? `<div class="b-pop b-pop-export">${U.overlay(s)}</div>` : ""}</aside>`;
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
    if (!last) return "";
    // A running tool is what the status already says; a finished one or a note is worth its line.
    if (last.t === "tool" && last.status === "running") return "";
    if (last.t === "tool" || last.t === "file" || last.t === "note") return U.message(last, s);
    // What it's thinking, as plain text: the status above already says it's working (one busy indicator, Barry).
    if (last.t === "thinking") return `<p class="b-thought">${esc(last.text)}</p>`;
    // No name in front: the agent chip above already says who's talking (say each thing once, Barry).
    if (last.t === "agent") return U.message(last, s);
    return "";
  }

  function chat(s) {
    const a = U.agentOf(s);
    const open = historyOpen(s);
    const n = s.chat.length;
    // What the agent is doing (note 1) leads the chat, beside the agent it's about. An untrusted project is a warning
    // for the agent too: it can't run the model until the user trusts it.
    // The status shows only when it says something the screen doesn't: not "Idle", and not when the card under it
    // (a question, an approval, a crash, a sign-in, an error) says it bigger.
    const card = active(s);
    const says = card && !s.pending;  // a pending redirect doesn't say the agent is still working
    const quiet = s.agent.line === "Idle" || (s.agent.state === "idle" && !s.agent.line) || says || s.banner;
    // After a turn that changed the model, the status offers to undo it: Emmet saved a version just before (section 07).
    const undo = !quiet && s.agent.state === "idle" && /^Done/.test(s.agent.line || "") ? `<button type="button" class="e-btn e-quiet e-sm" data-act="undo-turn" title="Restore the model from before this turn (version 6)">${U.icon("undo")}<span>Undo turn</span></button>` : "";
    const now = quiet ? "" : U.agentState(s, { kbd: false }) + undo;
    // The plan badge carries the usage meter and opens the usage popover; the panel no longer has a usage row.
    const plan = `<button type="button" class="e-badge b-plan" data-act="usage" title="Plan usage">${esc(a.plan)}${a === D.agents.claude ? ` · ${D.usage[0].pct}%` : ""}</button>`;
    const top = `<div class="b-chat-top"><button type="button" class="b-agent" data-act="agents" aria-haspopup="menu">${esc(a.name)} ${U.icon("chev")}</button>${plan}<div class="b-now">${now}</div>
      ${n ? `<button type="button" class="e-btn e-quiet e-sm" data-local="history=${open ? "closed" : "open"}" aria-expanded="${open}">${U.icon("chat")}<span>${open ? "Hide history" : `History · ${n}`}</span></button>` : ""}
      <button type="button" class="e-btn e-quiet e-sm" title="Start fresh: clear the context, keep the agent">${U.icon("fresh")}<span>Start fresh</span></button></div>`;
    const mid = open
      ? `<div class="b-msgs" data-scroll="end">${n ? U.chat(s) : ""}${card}</div>`
      : card ? `<div class="b-active">${card}</div>` : (() => { const l = latest(s); return l ? `<div class="b-line">${l}</div>` : ""; })();
    // The model and effort popover opens from its chip, in the composer (like nurb's); the others from the chat's top row.
    const pops = ["agents", "usage"].includes(s.overlay) ? `<div class="b-pop b-pop-${s.overlay}">${U.overlay(s)}</div>` : "";
    // One Stop: the status has it (and Esc), so the composer doesn't.
    return `<section class="b-chat" data-open="${open}" aria-label="Chat">${top}${mid}${U.composer(s, { rows: s.composer ? 3 : 1, stop: false, modelMenu: true })}${pops}</section>`;
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
