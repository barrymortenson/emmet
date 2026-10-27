// SPDX-License-Identifier: AGPL-3.0-or-later
// Direction A, Workbench: plan section 14's layout brief, done well. The project tree on the left, the viewer in the
// center with the drawer under it, the chat at full height on the right with what the agent is doing pinned at its top,
// and a status bar. Calm, fairly dense, native-feeling; for people who'll spend hours in it.
(() => {
  const D = window.EMMET_DATA;
  const U = window.EMMET_UI;
  const esc = U.esc;

  function titlebar(s) {
    const b = s.build;
    return `<header class="a-title">${U.lights()}
      <button type="button" class="a-project">${esc(s.imported ? D.imported.name : D.project.name)} ${U.icon("chev")}</button>
      <span class="a-build" data-tone="${U.buildTone(b)}">${b.state === "building" ? U.spinner() : U.icon(b.state === "failed" || b.state === "findings" ? "warn" : b.state === "readonly" ? "lock" : "check")}<span>${esc(U.buildText(b))}</span></span>
      <span class="e-grow"></span>
      <button type="button" class="a-tbtn" title="New (⌘N)">${U.icon("plus")}</button><button type="button" class="a-tbtn" title="Open (⌘O)">${U.icon("folder")}</button>
      <button type="button" class="a-tbtn" data-act="export" title="Export">${U.icon("download")}</button></header>`;
  }

  function tree(s) {
    // Only the steps built so far (a build in progress, or the step the scrubber shows).
    const cur = s.step || (s.build.parts ? D.stepOf(s.build.parts) : D.steps.length);
    const groups = [["Baseplate", [1]], ["Walls", [2, 3, 4, 5, 6, 7]], ["Ceiling", [8]], ["Roof", [9, 10, 11]], ["Chimney", [12]]];
    const rows = groups.map(([g, ns]) => {
      const parts = ns.filter((i) => s.step || i <= cur).reduce((n, i) => n + D.steps[i - 1].parts, 0);
      const later = ns[0] > cur;
      if (later && !s.step) return "";
      return `<li data-later="${later}"><span>${U.icon(g === "Baseplate" ? "layers" : "cube")}${esc(g)}</span><span class="a-dim">${parts}</span></li>`;
    }).join("");
    const f = s.build.state === "findings";
    return `<nav class="a-rail" aria-label="Project">
      <p class="a-h">Model</p><ul class="a-tree"><li class="a-root"><span>${U.icon("chev")}${esc(s.imported ? D.imported.file : "cottage")}</span><span class="a-dim">${s.build.parts || 0}</span></li>${s.build.state === "empty" ? "" : rows}</ul>
      <p class="a-h">Checks</p><p class="a-check" data-tone="${f ? "warn" : "ok"}">${U.icon(f ? "warn" : "check")}${f ? "1 problem: 8 floating parts" : s.build.state === "empty" ? "Waiting for the first build" : "No problems"}</p>
      <p class="a-h">Files</p><ul class="a-files">${(s.imported ? [D.imported.file, "CARD.md"] : ["model.py", "params.json", "CARD.md"]).map((f) => `<li>${U.icon("file")}${esc(f)}</li>`).join("")}</ul>
    </nav>`;
  }

  function viewer(s, ctx) {
    const pin = s.build.state === "findings" ? `<button type="button" class="a-pin" data-tab="checks" title="F-0001: 8 floating parts">${U.icon("warn")}<span>8 floating</span></button>` : "";
    const failed = s.build.state === "failed" ? `<div class="a-vnote" data-tone="bad">${U.icon("warn")}Build failed · the last good model is still shown</div>` : "";
    const ro = s.build.state === "readonly" ? `<div class="a-vnote" data-tone="warn">${U.icon("lock")}Read-only · model.py hasn't run</div>` : "";
    return `<section class="a-viewer" aria-label="3D viewer">${U.shot(s.render, ctx.theme)}
      <div class="a-vtop">${U.views()}<span class="e-grow"></span>${failed}${ro}</div>
      ${pin}${U.animating(s)}${s.build.state === "empty" ? "" : `<div class="a-vbottom">${U.scrubber(s)}</div>`}</section>`;
  }

  function drawer(s) {
    const tab = s.drawer || "params";
    if (s.build.state === "readonly" && tab === "params") return `<section class="a-drawer">${U.tabs(tab, s)}<p class="a-empty">${U.icon("lock")} Sliders run model.py, so they wait until you trust the project. Parts and Steps work now.</p></section>`;
    if (s.build.state === "empty") return `<section class="a-drawer a-drawer-empty">${U.tabs(null, s)}<p class="a-empty">Parameters, checks, parts, and steps appear after the first build.</p></section>`;
    return `<section class="a-drawer">${U.tabs(tab, s)}<div class="a-panel" data-tab="${tab}">${U.panel(tab, s)}</div></section>`;
  }

  function chat(s) {
    const a = s.agent;
    const pending = s.pending ? U.pending(s.pending, U.agentOf(s).name) : "";
    let body;
    if (a.state === "signedout") body = U.signedOut();
    else if (!s.chat.length) body = `<div class="a-chat-empty">${U.icon("chat")}<p><b>${s.imported ? "Ask the agent about this model" : "Describe what to build"}</b></p><p class="a-dim">${s.imported ? "It can make it parametric, so it gets sliders." : "Your agent writes model.py from real parts and builds it here, stage by stage."}</p></div>`;
    else body = U.chat(s);
    return `<aside class="a-chat" aria-label="Chat">${U.chatHeader(s)}
      <div class="a-nowslot">${a.state === "idle" ? "" : U.agentState(s)}</div>
      <div class="a-msgs" data-scroll="end">${body}${pending}</div>${U.composer(s)}
      ${s.overlay === "agents" ? `<div class="a-pop a-pop-agents">${U.overlay(s)}</div>` : ""}
      ${s.overlay === "model" ? `<div class="a-pop a-pop-model">${U.overlay(s)}</div>` : ""}</aside>`;
  }

  function statusbar(s) {
    const a = s.agent;
    return `<footer class="a-status"><span class="a-st" data-tone="${a.state === "working" ? "busy" : a.state === "question" || a.state === "approval" ? "attn" : a.state === "crashed" ? "bad" : "idle"}">${a.state === "working" ? U.spinner() : U.icon(a.state === "idle" ? "check" : "info")}${esc(U.agentOf(s).name)}: ${esc(a.line || "Idle")}</span>
      <span class="e-grow"></span><span>LDraw library ${D.versions.library} · Parts data ${D.versions.partsData}</span>${U.usageMini(s)}</footer>`;
  }

  function workspace(s, ctx) {
    const overlayAt = { usage: "a-pop-usage", export: "a-pop-export" }[s.overlay];
    return `<div class="a-app">${titlebar(s)}<div class="a-banner">${s.banner ? U.banner(s.banner) : ""}</div>
      <div class="a-body">${tree(s)}<main class="a-main">${viewer(s, ctx)}${drawer(s)}</main>${chat(s)}</div>${statusbar(s)}
      ${overlayAt ? `<div class="a-pop ${overlayAt}">${U.overlay(s)}</div>` : ""}</div>`;
  }

  function firstRun(s) {
    const { rail, body } = U.firstRun(s.step);
    return `<div class="a-fr"><header class="a-title a-title-plain">${U.lights()}<span class="a-fr-name">Emmet Builder</span></header>
      <div class="a-fr-body"><aside class="a-fr-rail"><img class="a-fr-mark" src="../../brand/icon-macos-ant.svg" alt="">${rail}</aside><main class="a-fr-main">${body}</main></div></div>`;
  }

  window.EMMET_PROTO.direction({
    id: "a", name: "A · Workbench", tagline: "The brief, done well: tree, viewer and drawer, chat on the right",
    render: (s, ctx) => (s.kind === "first-run" ? firstRun(s) : workspace(s, ctx)),
  });
})();
