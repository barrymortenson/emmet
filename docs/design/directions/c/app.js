// SPDX-License-Identifier: AGPL-3.0-or-later
// Direction C, Command: command-first and keyboard-driven. One bar across the top is the composer and the command
// palette, and its right end says what the agent is doing. A question, an approval, or a problem docks under the bar,
// never over the model. The viewer is the big center, an inspector on the right holds the parameters, checks, parts,
// and steps, and the conversation is a timeline strip along the bottom that opens into a transcript. Dense and precise.
(() => {
  const D = window.EMMET_DATA;
  const U = window.EMMET_UI;
  const esc = U.esc;
  const cut = (s, n) => (s.length > n ? s.slice(0, n - 1).trimEnd() + "…" : s);

  // The transcript drawer opens and closes in place (a DOM toggle, so the mockup keeps its screen).
  document.addEventListener("click", (e) => {
    const t = e.target.closest("[data-c=transcript]");
    if (!t) return;
    const app = t.closest(".c-app");
    const open = app.dataset.transcript !== "open";
    app.dataset.transcript = open ? "open" : "closed";
    for (const b of app.querySelectorAll("[data-c=transcript]")) b.setAttribute("aria-expanded", String(open));
    const list = app.querySelector(".c-transcript-list");
    if (list) list.scrollTop = list.scrollHeight;
  });

  // The command bar: project, the one input (ask or command), then what the agent is doing, then agent and model.
  function statusSeg(s) {
    const a = s.agent || { state: "idle" };
    const who = U.agentOf(s).name;
    if (a.state === "working") {
      return `<div class="c-live" data-tone="busy" role="status" aria-live="polite">${U.spinner()}<span class="c-live-line"><b>${esc(who)}</b> ${esc(a.line)}</span><span class="c-time">${esc(a.elapsed || "")}</span>
        <button type="button" class="c-stop" data-act="stop" title="Stop the turn">${U.icon("stop")}<span>Stop</span>${U.kbd("Esc")}</button></div>`;
    }
    const tone = { question: "attn", approval: "attn", crashed: "bad", signedout: "warn", offline: "warn", stopped: "muted" }[a.state] || "idle";
    const ic = { question: "info", approval: "info", crashed: "warn", signedout: "warn", offline: "wifi", stopped: "stop" }[a.state] || "check";
    const line = a.state === "question" || a.state === "approval" ? `${a.line} ↓` : a.line || "Idle";
    return `<div class="c-live" data-tone="${tone}" role="status">${U.icon(ic)}<span class="c-live-line"><b>${esc(who)}</b> ${esc(line)}</span>${a.elapsed ? `<span class="c-time">${esc(a.elapsed)}</span>` : ""}</div>`;
  }

  function commandBar(s) {
    const a = U.agentOf(s);
    const working = s.agent?.state === "working";
    const offline = s.agent?.state === "offline";
    const ph = working ? `Redirect ${a.name}: it reads this at its next step` : offline ? "Offline: commands work, the agent needs a connection" : `Ask ${a.name}, or type a command: /export, /step 8, width=12`;
    return `<header class="c-bar">${U.lights()}
      <button type="button" class="c-proj" title="Projects (⌘O)">${esc(s.imported ? D.imported.name : D.project.name)} ${U.icon("chev")}</button>
      <label class="c-cmd" data-working="${working}">${U.icon(working ? "chat" : "cmd")}<input type="text" value="${esc(s.composer || "")}" placeholder="${esc(ph)}" aria-label="Ask or type a command">
        <button type="button" class="c-attach" title="Attach a photo">${U.icon("paperclip")}</button>${U.kbd("⌘K")}
        <button type="button" class="c-send" data-act="send" title="Send (⏎)"${offline ? " disabled" : ""}>${U.icon("send")}<span>Send</span></button></label>
      ${statusSeg(s)}
      <button type="button" class="c-pick" data-act="agents" aria-haspopup="menu" title="Agent (switching starts a fresh conversation)">${esc(a.name)} ${U.icon("chev")}</button>
      <button type="button" class="c-pick c-pick-model" data-act="model" title="Model and effort">${esc(a.model)} · Medium ${U.icon("chev")}</button>
      <button type="button" class="c-iconbtn" title="Start fresh: clear the context, keep the agent">${U.icon("fresh")}</button>
    </header>`;
  }

  // The dock under the bar: whatever needs the user now. It pushes the viewer down; it never covers it.
  function question(m) {
    const opts = m.options.map((o, i) => `<button type="button" class="c-opt" data-act="answer">${U.kbd(i + 1)}<span><b>${esc(o.label)}</b>${o.description ? ` <small>${esc(o.description)}</small>` : ""}</span></button>`).join("");
    return `<div class="c-dock" data-tone="attn" role="group" aria-label="${esc(m.question)}"><div class="c-dock-head">${U.icon("info")}<span class="c-tag">${esc(m.header)}</span><b>Claude asks: ${esc(m.question)}</b><span class="c-grow"></span><span class="c-hint">Press 1–${m.options.length + 1}, or type your own answer</span></div>
      <div class="c-opts">${opts}<label class="c-opt c-other">${U.kbd(m.options.length + 1)}<input type="text" placeholder="Other: your own answer" aria-label="Other: your own answer"></label></div></div>`;
  }
  function approval(m) {
    return `<div class="c-dock" data-tone="attn" role="group" aria-label="${esc(m.title)}"><div class="c-dock-head">${U.icon("info")}<span class="c-tag">${esc(m.tool)}</span><b>Claude wants to open a web page</b><code class="c-code">${esc(m.command)}</code><span class="c-grow"></span><span class="c-hint">The sandbox can't contain this, so it asks</span></div>
      <p class="c-why">${esc(m.why)}</p>
      <div class="c-opts"><button type="button" class="c-opt c-opt-primary" data-act="allow">${U.kbd("1")}<span><b>Allow once</b></span></button><button type="button" class="c-opt" data-act="allow">${U.kbd("2")}<span><b>Allow for this session</b></span></button><button type="button" class="c-opt" data-act="deny">${U.kbd("3")}<span><b>Don't allow</b></span></button></div></div>`;
  }
  function pending(text, who) {
    return `<div class="c-dock" data-tone="queued"><div class="c-dock-head">${U.icon("clock")}<span class="c-tag">Queued</span><b>“${esc(text)}”</b><span class="c-hint">Goes to ${esc(who)} at its next step</span><span class="c-grow"></span>
      ${U.btn("Edit", "", "e-quiet e-sm")}${U.btn("Cancel", "cancel", "e-quiet e-sm")}${U.btn("Stop and send now", "deliver", "e-sm")}</div></div>`;
  }
  function failed() {
    const e = D.buildError;
    return `<div class="c-dock" data-tone="bad" role="alert"><div class="c-dock-head">${U.icon("warn")}<span class="c-tag">model.py:${e.line}</span><b>The model didn't build</b><span class="c-hint">The last good model is still shown</span><span class="c-grow"></span>${U.btn("Open model.py", "", "e-quiet e-sm", "file")}${U.btn("Ask agent to fix", "fix", "e-primary e-sm")}</div>
      <pre class="c-src"><span class="c-err">${esc(e.message)}</span>\n<span class="e-ln">${e.line}</span>${esc(e.excerpt.trim())}</pre></div>`;
  }
  function crashed(s, m) {
    return `<div class="c-dock" data-tone="bad" role="alert"><div class="c-dock-head">${U.icon("warn")}<b>${esc(m.title)}</b><span class="c-hint">${esc(m.text)}</span><span class="c-grow"></span>${U.btn("Copy the details", "", "e-quiet e-sm")}${U.btn(`Restart ${U.agentOf(s).cli}`, "restart", "e-primary e-sm", "refresh")}</div></div>`;
  }
  function signedOut() {
    return `<div class="c-dock" data-tone="warn"><div class="c-dock-head">${U.icon("info")}<b>Claude isn't signed in</b><span class="c-hint">Sign in opens Claude's page in your browser; Emmet never sees your password</span><span class="c-grow"></span>${U.btn("Use Codex instead", "other", "e-quiet e-sm")}${U.btn("Sign in to Claude", "signin", "e-primary e-sm")}</div>
      <p class="c-why">${esc(D.plans.claude)}</p></div>`;
  }
  function dock(s) {
    const last = s.chat[s.chat.length - 1];
    const out = [];
    if (s.banner) out.push(`<div class="c-dock c-dock-banner" data-tone="${s.banner === "imported" ? "info" : "warn"}">${U.banner(s.banner)}</div>`);
    if (s.agent?.state === "signedout") out.push(signedOut());
    if (last?.t === "question" && !last.answered) out.push(question(last));
    if (last?.t === "approval") out.push(approval(last));
    if (last?.t === "crash") out.push(crashed(s, last));
    if (s.pending) out.push(pending(s.pending, U.agentOf(s).name));
    if (s.error) out.push(failed());
    return `<div class="c-docks">${out.join("")}</div>`;
  }

  // The step scrubber, with the step's name: the model shows only what's built up to it (note 6).
  function scrubber(step, chosen) {
    const name = D.steps[step - 1]?.name || "";
    return `<div class="e-scrub" title="[ and ] move a step"><button type="button" class="e-icon-btn" title="Previous step ([)">◀</button><span>step ${step}/${D.steps.length}</span><span class="c-scrub-name">${esc(name)}${chosen ? " · later steps hidden" : ""}</span><button type="button" class="e-icon-btn" title="Next step (])">▶</button></div>`;
  }

  // The viewer: the render, a mono readout, views and the step scrubber, and the finding's pin.
  function viewer(s, ctx) {
    const b = s.build;
    const step = s.step || (b.parts && b.state !== "failed" ? D.stepOf(b.parts) : D.steps.length);
    const tone = U.buildTone(b);
    const pin = b.state === "findings" ? `<button type="button" class="c-pin" data-tab="checks" title="F-0001: 8 floating parts">${U.icon("warn")}<span>F-0001 · 8 floating</span></button>` : "";
    return `<section class="c-viewer" aria-label="3D viewer">${U.shot(s.render, ctx.theme)}
      <div class="c-hud"><span class="c-readout" data-tone="${tone}">${b.state === "building" ? U.spinner() : U.icon(tone === "bad" || tone === "warn" ? "warn" : b.state === "readonly" ? "lock" : "check")}<span>${esc(U.buildText(b))}</span></span>
        <span class="c-grow"></span>${U.views()}</div>
      ${pin}${U.animating(s)}
      <div class="c-vfoot"><span class="c-grow"></span>${b.state === "empty" ? "" : scrubber(step, s.step)}<span class="c-grow"></span></div></section>`;
  }

  // The inspector: tabs for parameters, checks, parts, and steps.
  function inspector(s) {
    const tab = s.drawer || "params";
    if (s.build.state === "empty") return `<aside class="c-insp" aria-label="Inspector">${U.tabs(null, s)}<div class="c-panel"><p class="c-empty">Parameters, checks, parts, and steps appear after the first build. Type what to build in the bar above, then press ⏎.</p></div></aside>`;
    // Read-only until trusted: the values show, the controls wait.
    const ro = s.build.state === "readonly" && tab === "params";
    let body = ro ? `<p class="c-ro-note">${U.icon("lock")}Trust the project to use its sliders.</p><fieldset class="c-ro" disabled>${U.panel(tab, s)}</fieldset>` : U.panel(tab, s);
    // An imported file has no model.py, so no parameters until the agent writes one.
    if (s.imported && tab === "params") body = `<p class="c-empty">${esc(D.imported.file)} has no parameters: it's a plain LDraw file. "Make it parametric" asks the agent to write a model.py, and its sliders appear here.</p>`;
    return `<aside class="c-insp" aria-label="Inspector">${U.tabs(tab, s)}<div class="c-panel" data-tab="${tab}">${body}</div></aside>`;
  }

  // The conversation as a timeline: one compact chip per event, the newest at the right edge.
  function chip(m) {
    switch (m.t) {
      case "user": return { cls: "you", ic: m.auto ? "warn" : "chat", text: m.auto ? "From Checks: Fix F-0001" : `You: ${cut(m.text, 46)}${m.delivered ? " · delivered" : ""}` };
      case "agent": return { cls: "say", ic: "spark", text: cut(m.text, 70) };
      case "thinking": return { cls: "think", ic: "spark", text: "Thinking" };
      case "tool": return { cls: m.status === "running" ? "run" : m.status === "error" ? "bad" : "done", ic: m.icon, text: m.status === "running" ? `${m.title}: ${cut(m.detail, 40)}` : m.title, spin: m.status === "running" };
      case "file": return { cls: "done", ic: "file", text: `${m.path}: ${m.summary.replace(/^Wrote the model · /, "wrote ")}` };
      case "question": return { cls: m.answered ? "done" : "attn", ic: "info", text: m.answered ? `${m.header}: ${cut(m.answered, 22)}` : `Asked: ${m.header}` };
      case "approval": return { cls: "attn", ic: "info", text: "Web request?" };
      case "note": return { cls: "note", ic: "stop", text: cut(m.text, 56) };
      case "crash": return { cls: "bad", ic: "warn", text: "Claude Code stopped" };
      case "divider": return { cls: "div", ic: "refresh", text: cut(m.text, 40) };
      default: return null;
    }
  }
  function timeline(s) {
    // A tool still "running" when the turn was stopped or the agent crashed shows as cut off, not as working.
    const ended = s.chat.some((m) => m.t === "crash" || m.t === "note");
    const chips = s.chat.map((m) => (ended && m.t === "tool" && m.status === "running" ? { cls: "note", ic: "stop", text: `${m.detail}: cut off` } : chip(m))).filter(Boolean);
    // Keep what fits in the strip, newest last; older chips fold into a count at the left.
    let room = 1110, keep = [];
    for (let i = chips.length - 1; i >= 0; i--) {
      const w = chips[i].text.length * 6.6 + 44;
      if (room - w < 0 && keep.length) break;
      room -= w;
      keep.unshift(chips[i]);
    }
    const earlier = chips.length - keep.length;
    const html = keep.map((c, i) => `${i ? '<i class="c-join" aria-hidden="true"></i>' : ""}<span class="c-chip" data-k="${c.cls}">${c.spin ? U.spinner() : U.icon(c.ic)}<span>${esc(c.text)}</span></span>`).join("");
    const empty = !chips.length ? `<span class="c-chip" data-k="note">${U.icon("chat")}<span>${s.agent?.state === "signedout" ? "Sign in to start a conversation" : s.imported ? "No conversation yet: ask the agent to make this model parametric" : "No conversation yet"}</span></span>` : "";
    return `<div class="c-tl"><button type="button" class="c-tl-toggle" data-c="transcript" aria-expanded="false" title="Transcript">${U.icon("list")}<span>Transcript</span>${U.kbd("⌘L")}</button>
      <div class="c-tl-track">${earlier ? `<span class="c-chip" data-k="more">+${earlier} earlier</span><i class="c-join" aria-hidden="true"></i>` : ""}${html}${empty}</div></div>`;
  }
  function transcript(s) {
    const body = s.chat.length ? U.chat(s) : `<p class="c-empty">No conversation yet.</p>`;
    return `<section class="c-transcript" aria-label="Transcript"><div class="c-transcript-head"><b>Transcript</b><span class="c-dimtext">${esc(U.agentOf(s).name)} · ${esc(U.agentOf(s).plan)} · code and tool output folded</span><span class="c-grow"></span><button type="button" class="c-iconbtn" data-c="transcript" aria-expanded="true" title="Close the transcript">${U.icon("x")}</button></div>
      <div class="c-transcript-list" data-scroll="end">${body}</div></section>`;
  }

  function statusbar(s) {
    const a = U.agentOf(s);
    return `<footer class="c-status"><span>${esc(a.cli)} ${esc(a.version)} · ${esc(a.plan)}</span><span class="c-sep">│</span><span>ldraw ${D.versions.library}</span><span class="c-sep">│</span><span>parts-data ${D.versions.partsData}</span><span class="c-sep">│</span><span>emmet ${D.versions.emmet}</span>
      <span class="c-grow"></span><span class="c-keys">${U.kbd("⌘K")} command ${U.kbd("Esc")} stop ${U.kbd("[")}${U.kbd("]")} step</span>${s.who === "codex" ? `<button type="button" class="e-usage" data-act="usage">Codex · Using ChatGPT plan</button>` : U.usageMini()}</footer>`;
  }

  function workspace(s, ctx) {
    const pop = { agents: "c-pop-agents", model: "c-pop-model", usage: "c-pop-usage", export: "c-pop-export" }[s.overlay];
    const open = s.id === "switched" ? "open" : "closed";
    return `<div class="c-app" data-transcript="${open}">${commandBar(s)}${dock(s)}
      <div class="c-body">${viewer(s, ctx)}${inspector(s)}</div>${transcript(s)}${timeline(s)}${statusbar(s)}
      ${pop ? `<div class="c-pop ${pop}">${U.overlay(s)}</div>` : ""}</div>`;
  }

  // First run: a setup checklist with live progress lines. No commands are asked of anyone.
  function firstRun(s) {
    const { body } = U.firstRun(s.step);
    const steps = [["welcome", "Welcome", "What Emmet does"], ["library", "Parts and Python", "LDraw library and a Python for models"], ["agents", "Your agent", "Claude Code or Codex, signed in"], ["start", "Start", "The example, or your own prompt"]];
    const at = steps.findIndex(([id]) => id === s.step);
    const list = steps.map(([id, label, sub], i) => `<li data-state="${i < at ? "done" : i === at ? "current" : "later"}"><span class="c-fr-mark">${i < at ? U.icon("check") : i === at ? U.icon("play") : `<span>${i + 1}</span>`}</span><span><b>${label}</b><small>${sub}</small></span></li>`).join("");
    const log = {
      welcome: ["ready   no account, no tracking", "next    LDraw parts library (about 150 MB)"],
      library: [`fetch   ldraw ${D.versions.library}   96 / 150 MB   3.1 MB/s`, "verify  sha256   waiting", "python  3.13 for models   next", "note    set up your agent meanwhile"],
      agents: [`ok      ldraw ${D.versions.library} · 37,261 files`, "ok      python 3.13 ready", `found   Claude Code ${D.agents.claude.version} · Max plan`, "missing Codex · not installed"],
      start: [`ok      ldraw ${D.versions.library} · python 3.13`, `ok      Claude Code ${D.agents.claude.version} · Max plan`, "ok      projects folder Documents/Emmet"],
    }[s.step];
    return `<div class="c-fr"><header class="c-bar c-bar-plain">${U.lights()}<b class="c-fr-name">Emmet Builder</b><span class="c-dimtext">setup · ${at + 1} of 4</span></header>
      <div class="c-fr-body"><aside class="c-fr-side"><ol class="c-fr-list">${list}</ol><div class="c-fr-log" aria-label="Progress"><p class="c-fr-log-h">Progress</p>${log.map((l) => `<code>${esc(l)}</code>`).join("")}</div></aside>
      <main class="c-fr-main">${body}</main></div></div>`;
  }

  window.EMMET_PROTO.direction({
    id: "c", name: "C · Command", tagline: "Command-first: one bar to ask or command, an inspector, the turn as a timeline",
    render: (s, ctx) => (s.kind === "first-run" ? firstRun(s) : workspace(s, ctx)),
  });
})();
