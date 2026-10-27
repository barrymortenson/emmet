// SPDX-License-Identifier: AGPL-3.0-or-later
// The pieces of Emmet's UI every direction shows: the same content and controls, as HTML strings. A direction decides
// where each piece goes and how it looks (its own style.css); it may also draw its own versions. No colors here: a
// part color is the only inline color, and it comes from data.js.
window.EMMET_UI = (() => {
  const D = window.EMMET_DATA;
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const md = (s) => esc(s).replace(/`([^`]+)`/g, "<code>$1</code>").replace(/\*\*([^*]+)\*\*/g, "<b>$1</b>");

  const PATHS = {
    send: "M4 12l16-8-6 16-3-7-7-1z", stop: "M7 7h10v10H7z", chev: "M6 9l6 6 6-6", check: "M5 12.5l4.5 4.5L19 7",
    x: "M6 6l12 12M18 6L6 18", warn: "M12 4l9 16H3zM12 10v4M12 17v.5", info: "M12 8v.5M12 11v6M12 3a9 9 0 100 18 9 9 0 000-18z",
    cube: "M12 3l8 4.5v9L12 21l-8-4.5v-9zM12 12l8-4.5M12 12v9M12 12L4 7.5", search: "M11 4a7 7 0 100 14 7 7 0 000-14zM16 16l4 4",
    book: "M5 4h10a3 3 0 013 3v13H8a3 3 0 01-3-3zM5 17a3 3 0 013-3h10", eye: "M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12zM12 9a3 3 0 100 6 3 3 0 000-6z",
    file: "M6 3h8l4 4v14H6zM14 3v4h4", sliders: "M4 7h10M18 7h2M4 17h4M12 17h8M14 5v4M8 15v4", list: "M8 6h12M8 12h12M8 18h12M4 6h.5M4 12h.5M4 18h.5",
    layers: "M12 3l9 5-9 5-9-5zM3 13l9 5 9-5", steps: "M4 18h5v-4h5v-4h5V6", download: "M12 4v11M7 10l5 5 5-5M5 20h14",
    upload: "M12 20V9M7 14l5-5 5 5M5 4h14", folder: "M3 6h6l2 2h10v11H3z", refresh: "M20 11a8 8 0 10-2.3 5.7M20 5v6h-6",
    clock: "M12 3a9 9 0 100 18 9 9 0 000-18zM12 7v5l3 2", spark: "M12 3v4M12 17v4M3 12h4M17 12h4M6 6l2.5 2.5M15.5 15.5L18 18M6 18l2.5-2.5M15.5 8.5L18 6",
    frame: "M4 9V4h5M15 4h5v5M20 15v5h-5M9 20H4v-5", camera: "M4 8h4l2-3h4l2 3h4v11H4zM12 10a3.5 3.5 0 100 7 3.5 3.5 0 000-7z",
    lock: "M6 11h12v9H6zM9 11V8a3 3 0 016 0v3", wifi: "M3 9a13 13 0 0118 0M6.5 12.5a8 8 0 0111 0M10 16a3 3 0 014 0M3 3l18 18",
    plus: "M12 5v14M5 12h14", play: "M8 5l11 7-11 7z", pin: "M12 21s-6-6.5-6-11a6 6 0 0112 0c0 4.5-6 11-6 11zM12 8a2 2 0 100 4 2 2 0 000-4z",
    cmd: "M9 6a3 3 0 10-3 3h12a3 3 0 10-3-3v12a3 3 0 103-3H6a3 3 0 103 3z", chat: "M4 5h16v11H9l-5 4z", fresh: "M4 5h16v11H9l-5 4zM12 8v5M9.5 10.5h5",
    paperclip: "M16 7l-7.5 7.5a2 2 0 003 3L19 10a4 4 0 00-6-6l-7.5 7.5a6 6 0 008.5 8.5L20 14",
  };
  const icon = (name, cls = "") => `<svg class="e-ic ${cls}" viewBox="0 0 24 24" aria-hidden="true"><path d="${PATHS[name] || PATHS.info}"/></svg>`;
  const spinner = () => `<span class="e-spin" role="status" aria-label="Working"></span>`;
  const lights = () => `<span class="e-lights" aria-hidden="true"><i></i><i></i><i></i></span>`;
  const btn = (label, act, cls = "", ic = "") => `<button type="button" class="e-btn ${cls}" ${act ? `data-act="${act}"` : ""}>${ic ? icon(ic) : ""}<span>${esc(label)}</span></button>`;
  const kbd = (k) => `<kbd class="e-kbd">${esc(k)}</kbd>`;
  const swatch = (color) => `<i class="e-swatch" style="background:${D.colors[color] || "transparent"}" aria-hidden="true"></i>`;
  const agentOf = (screen) => D.agents[screen.who || "claude"];

  // The render, as a square image sized to the viewer's width, lined up so build stages share one camera.
  function shot(name, theme, cls = "") {
    if (!name) return "";
    const a = D.renders[name] || { k: 1, ty: 0 };
    const src = `${window.EMMET_ROOT}shared/renders/${theme}/${name}.webp`;
    return `<img class="e-shot ${cls}" src="${src}" alt="" style="transform: translate(0, calc(-50% + ${a.ty}%)) scale(${a.k})">`;
  }

  // The build state, in words (section 14: "Built in 0.3 s · 118 parts · ✓ clean").
  function buildText(b) {
    if (!b) return "";
    if (b.state === "empty") return "Empty baseplate · waiting for the first build";
    if (b.state === "building") return `Building · ${b.parts} parts so far`;
    if (b.state === "failed") return `Build failed: model.py line ${D.buildError.line}; showing the last good model`;
    if (b.state === "readonly") return `Read-only · ${b.parts} parts · model.py won't run until you trust it`;
    if (b.state === "findings") return `Built ${b.parts} parts · ${b.findings} problem`;
    const ms = b.ms ?? D.run.ms;
    return `Built ${b.parts} parts in ${(ms / 1000).toFixed(1)} s${b.clean ? " · no problems" : ""}`;
  }
  const buildTone = (b) => ({ failed: "bad", findings: "warn", building: "busy", readonly: "warn", empty: "muted" })[b?.state] || "ok";

  // What the agent is doing right now (note 1): one line, the time, and Stop while it works.
  function agentState(screen, opts = {}) {
    const a = screen.agent || { state: "idle" };
    const who = agentOf(screen).name;
    const working = a.state === "working";
    const tone = { working: "busy", question: "attn", approval: "attn", stopped: "muted", crashed: "bad", signedout: "warn", offline: "warn" }[a.state] || "idle";
    const lead = working ? spinner() : a.state === "question" || a.state === "approval" ? icon("info") : a.state === "crashed" ? icon("warn") : icon(a.state === "idle" ? "check" : "info");
    return `<div class="e-now" data-tone="${tone}" role="status" aria-live="polite">
      ${lead}<span class="e-now-who">${esc(who)}</span><span class="e-now-line">${esc(a.line || "Idle")}</span>
      ${a.elapsed ? `<span class="e-now-time">${esc(a.elapsed)}</span>` : ""}
      ${working && opts.stop !== false ? `<button type="button" class="e-btn e-stop" data-act="stop" title="Stop the turn (Esc)">${icon("stop")}<span>Stop</span>${opts.kbd === false ? "" : kbd("Esc")}</button>` : ""}
    </div>`;
  }

  // Chat messages.
  function tool(m) {
    const st = m.status === "running" ? spinner() : m.status === "error" ? icon("x", "e-bad") : icon("check", "e-ok");
    return `<details class="e-tool" data-status="${m.status}"><summary>${icon(m.icon)}<span class="e-tool-title">${esc(m.title)}</span><span class="e-tool-detail">${esc(m.detail)}</span>${st}</summary><pre class="e-raw">${esc(m.raw)}</pre></details>`;
  }
  function file(m) {
    const lines = m.diff.map((l) => `<span class="e-diff" data-op="${l[0] === "+" ? "add" : l[0] === "-" ? "del" : ""}">${esc(l)}</span>`).join("\n");
    return `<details class="e-tool e-file"><summary>${icon("file")}<span class="e-tool-title">${esc(m.summary)}</span><span class="e-tool-detail">${esc(m.path)}</span>${icon("check", "e-ok")}</summary><pre class="e-raw">${lines}</pre></details>`;
  }
  function question(m) {
    if (m.answered) return `<div class="e-q e-q-done">${icon("check", "e-ok")}<span class="e-q-head">${esc(m.question)}</span><span class="e-q-ans">${esc(m.answered)}</span></div>`;
    const opts = m.options.map((o, i) => `<button type="button" class="e-q-opt" data-act="answer">${kbd(i + 1)}<span><b>${esc(o.label)}</b>${o.description ? `<small>${esc(o.description)}</small>` : ""}</span></button>`).join("");
    return `<div class="e-q" role="group" aria-label="${esc(m.question)}"><div class="e-q-top"><span class="e-chip">${esc(m.header)}</span><span class="e-q-wait">Waiting for your answer</span></div>
      <p class="e-q-head">${esc(m.question)}</p>${opts}
      <label class="e-q-other">${kbd(m.options.length + 1)}<input type="text" placeholder="Other: your own answer" aria-label="Other: your own answer"></label></div>`;
  }
  function approval(m) {
    return `<div class="e-appr" role="group" aria-label="${esc(m.title)}"><div class="e-q-top"><span class="e-chip">${esc(m.tool)}</span><span class="e-q-wait">Waiting for you</span></div>
      <p class="e-q-head">${esc(m.title)}</p><code class="e-cmd">${esc(m.command)}</code><p class="e-why">${esc(m.why)}</p>
      <div class="e-row">${btn("Allow once", "allow", "e-primary")}${btn("Allow for this session", "allow")}${btn("Don't allow", "deny", "e-quiet")}</div></div>`;
  }
  function message(m, screen) {
    switch (m.t) {
      case "user": return `<div class="e-msg e-user">${m.auto ? `<span class="e-meta">${icon(m.auto === "settings" ? "sliders" : "warn")} Sent from ${m.auto === "settings" ? "Build settings" : "Checks"}</span>` : ""}<p>${md(m.text)}</p>${m.delivered ? `<span class="e-meta">${icon("check")} Delivered at the agent's next step</span>` : ""}</div>`;
      case "agent": return `<div class="e-msg e-agent">${m.text.split("\n\n").map((p) => `<p>${md(p)}</p>`).join("")}</div>`;
      case "thinking": return `<details class="e-think"><summary><span>Thinking</span>${icon("chev")}</summary><p>${esc(m.text)}</p></details>`;
      case "tool": return tool(m);
      case "file": return file(m);
      case "question": return question(m);
      case "approval": return approval(m);
      case "note": return `<p class="e-note">${esc(m.text)}</p>`;
      case "divider": return `<div class="e-divider" role="separator"><span>${esc(m.text)}</span></div>`;
      case "crash": return `<div class="e-crash" role="alert">${icon("warn", "e-bad")}<div><p class="e-q-head">${esc(m.title)}</p><p>${esc(m.text)}</p><div class="e-row">${btn(`Restart ${agentOf(screen).cli}`, "restart", "e-primary", "refresh")}${btn("Copy the details", "", "e-quiet")}</div></div></div>`;
      default: return "";
    }
  }
  function errorCard() {
    const e = D.buildError;
    return `<div class="e-err" role="alert"><div class="e-err-top">${icon("warn", "e-bad")}<b>The model didn't build</b><span class="e-chip">model.py:${e.line}</span></div>
      <p>${esc(e.message)}</p><pre class="e-src">${[].concat(e.excerpt).map((l, i) => `<span class="e-ln">${e.line + i}</span>${esc(l)}`).join("\n")}</pre>
      <div class="e-row">${btn("Ask agent to fix", "fix", "e-primary")}${btn("Open model.py", "", "e-quiet", "file")}</div><p class="e-small">The last good model is still shown.</p></div>`;
  }
  // A finished run of three or more tool cards folds into one line (note 2: less in the chat); a run still going, or
  // one the turn hasn't moved past, stays open.
  function chat(screen, opts = {}) {
    const out = [];
    const list = screen.chat;
    for (let i = 0; i < list.length; ) {
      let j = i;
      while (j < list.length && (list[j].t === "tool" || list[j].t === "file") && list[j].status !== "running") j++;
      const run = list.slice(i, j);
      const moved = j < list.length && list[j].t !== "tool" && list[j].t !== "file";
      if (opts.fold !== false && run.length >= 3 && moved) {
        const built = run.filter((m) => /^Built \d+/.test(m.title)).length;
        out.push(`<details class="e-fold"><summary>${icon("check", "e-ok")}<span>${run.length} steps${built ? ` · built in ${built} stage${built > 1 ? "s" : ""}` : ""}</span>${icon("chev")}</summary>${run.map((m) => message(m, screen)).join("")}</details>`);
        i = j;
      } else if (j > i) {
        out.push(...run.map((m) => message(m, screen)));
        i = j;
      } else {
        out.push(message(list[i], screen));
        i++;
      }
    }
    return out.join("") + (screen.error ? errorCard() : "");
  }

  function pending(text, who = "Claude") {
    return `<div class="e-pending"><p>${esc(text)}</p><span class="e-meta">${icon("clock")} Goes to ${esc(who)} at its next step</span>
      <div class="e-row">${btn("Edit", "", "e-quiet e-sm")}${btn("Cancel", "cancel", "e-quiet e-sm")}${btn("Stop and send now", "deliver", "e-sm")}</div></div>`;
  }

  function composer(screen, opts = {}) {
    const a = agentOf(screen);
    const working = screen.agent?.state === "working";
    const ph = working ? "Redirect the agent…" : screen.agent?.state === "offline" ? "The agent needs a connection" : opts.placeholder || `Ask ${a.name} to build or change something`;
    return `<div class="e-composer" data-working="${working}">
      <textarea rows="${opts.rows || 2}" placeholder="${esc(ph)}" aria-label="Message">${esc(screen.composer || "")}</textarea>
      <div class="e-composer-bar">
        <button type="button" class="e-icon-btn" title="Attach a photo">${icon("paperclip")}</button>
        <span class="e-model-anchor"><button type="button" class="e-model-chip" data-act="model" title="Model and effort" aria-haspopup="dialog" aria-expanded="${screen.overlay === "model"}">${esc(a.model)} · Medium ${icon("chev")}</button>${opts.modelMenu && screen.overlay === "model" ? modelMenu(screen) : ""}</span>
        <span class="e-grow"></span>
        ${working && opts.stop !== false ? `<button type="button" class="e-btn e-stop" data-act="stop">${icon("stop")}<span>Stop</span></button>` : ""}
        <button type="button" class="e-btn e-primary e-send" data-act="send"${screen.agent?.state === "offline" ? " disabled" : ""}>${icon("send")}<span>Send</span></button>
      </div></div>`;
  }

  // The chat header: the agent picker and Start fresh (note 3), and which account pays (section 10).
  function chatHeader(screen) {
    const a = agentOf(screen);
    return `<div class="e-chat-head"><button type="button" class="e-agent-btn" data-act="agents" aria-haspopup="menu">${esc(a.name)} ${icon("chev")}</button>
      <span class="e-badge">${esc(a.name)} · ${esc(a.plan)}</span><span class="e-grow"></span>
      <button type="button" class="e-btn e-quiet e-sm" title="Start fresh: clear the context, keep the agent">${icon("fresh")}<span>Start fresh</span></button></div>`;
  }

  // Parameters (note 5): grouped, labeled in words, help from the docstring, colors limited to what the parts come in.
  function params(screen) {
    const dirty = screen.dirty || {};
    // Names that explain themselves, and the docstring's help on hover only (Barry 2026-10-27: no subtext).
    const row = (p) => {
      const v = p.name in dirty ? dirty[p.name] : p.value;
      const moved = p.name in dirty;
      let ctl;
      if (p.type === "int") {
        ctl = `<input type="range" min="${p.min}" max="${p.max}" step="${p.step || 1}" value="${v}" data-param="${p.name}" aria-label="${esc(p.label)}"><output>${v} ${esc(p.unit)}</output>`;
      } else if (p.type === "seed") {
        ctl = `<span class="e-shuffle">${btn("Shuffle", "shuffle", "e-sm", "refresh")}</span>`;
      } else if (p.type === "color") {
        const opts = D.colorChoices[p.name].map((c) => `<option${c === v ? " selected" : ""}>${esc(c)}</option>`).join("");
        ctl = `<span class="e-color">${swatch(v)}<select aria-label="${esc(p.label)}">${opts}</select></span>`;
      } else {
        ctl = `<label class="e-toggle"><input type="checkbox"${v ? " checked" : ""} aria-label="${esc(p.label)}"><span></span></label>`;
      }
      const was = p.type === "seed" ? "" : `was ${p.value}${p.unit === "%" ? " %" : ""}`;
      const tip = `${p.help[0].toUpperCase()}${p.help.slice(1)} (model.py line ${p.line})`;
      return `<div class="e-param" data-type="${p.type}" data-moved="${moved}" title="${esc(tip)}"><div class="e-param-top"><span class="e-param-name">${esc(p.label)}</span>${moved && was ? `<span class="e-chip e-chip-accent">${esc(was)}</span>` : ""}</div>${ctl}</div>`;
    };
    const n = Object.keys(dirty).length;
    const keep = n ? `<div class="e-keep"><span>${n} change${n > 1 ? "s" : ""}</span>${btn("Reset", "reset", "e-quiet e-sm")}${btn("Keep", "keep", "e-primary e-sm")}</div>` : "";
    return `<div class="e-params">${keep}${D.params.map(row).join("")}</div>`;
  }

  // Build settings: style, scale, detail, size, base. They steer the agent, so a change waits for Rebuild instead of
  // rerunning model.py. A new build starts with each one on Any, so they don't box in the prompt; style takes the
  // user's own words as well as the suggestions (Barry 2026-10-27).
  function buildSettings(screen, opts = {}) {
    const B = D.brief;
    const dirty = screen.briefDirty || {};
    const fresh = screen.build?.state === "empty";
    const val = (k) => (k in dirty ? dirty[k] : fresh ? null : B[k].value);
    const size = () => ("size" in dirty ? dirty.size : fresh ? null : B.size.value);
    if (opts.compact) {
      const sz = B.size.presets.find((p) => p.id === size());
      const parts = [val("style"), val("scale"), val("detail") && `${val("detail")} detail`, sz && `${sz.min}–${sz.max} pieces`].filter(Boolean);
      const n = Object.keys(dirty).length;
      const pending = n ? `<div class="e-keep e-rebuild"><span>${n} changed</span>${btn("Undo", "undo", "e-quiet e-sm")}${btn("Rebuild", "rebuild", "e-primary e-sm", "refresh")}</div>` : "";
      return `${pending}<div class="e-brief-sum"><span>${esc(parts.join(" · ") || "Any")}</span>${btn("Change", "", "e-sm").replace("<button", `<button data-local="${opts.open}"`)}</div>`;
    }
    const was = (k, v) => (k in dirty ? `<span class="e-chip e-chip-accent">was ${esc(v ?? "Any")}</span>` : "");
    const head = (k, v) => `<div class="e-set-head"><span class="e-set-name">${esc(B[k].label)}</span>${was(k, v)}</div>`;
    const seg = (label, items, cur) => `<div class="e-seg" role="radiogroup" aria-label="${esc(label)}">${items.map((o) => `<button type="button" role="radio" aria-checked="${o === cur}">${esc(o)}</button>`).join("")}</div>`;
    // Style: the suggestions, the user's own words as chips of their own, and a field to add more.
    const cur = val("style");
    const own = cur && !B.style.options.some((o) => o.id === cur) ? [cur] : [];
    const chips = B.style.options.map((o) => `<button type="button" role="checkbox" class="e-style" aria-checked="${o.id === cur}" title="${esc(o.sub)}">${esc(o.id)}</button>`)
      .concat(own.map((w) => `<button type="button" role="checkbox" class="e-style e-style-own" aria-checked="true" title="Your own style">${esc(w)}<span aria-label="Remove">×</span></button>`)).join("");
    const style = `<div class="e-set" data-moved="${"style" in dirty}">${head("style", B.style.value)}<div class="e-styles" role="group" aria-label="Style">${chips}</div>
      <input class="e-style-input" type="text" placeholder="Your own, like steampunk or zen garden" aria-label="Your own style"></div>`;
    const scale = `<div class="e-set" data-moved="${"scale" in dirty}">${head("scale", B.scale.value)}<label class="e-select"><select aria-label="Scale"><option${val("scale") ? "" : " selected"}>Any</option>${B.scale.options.map((o) => `<option${o.id === val("scale") ? " selected" : ""} title="${esc(o.sub)}">${esc(o.id)}</option>`).join("")}</select></label></div>`;
    const detail = `<div class="e-set" data-moved="${"detail" in dirty}">${head("detail", B.detail.value)}${seg("Detail", ["Any", ...B.detail.options.map((o) => o.id)], val("detail") ?? "Any")}</div>`;
    const pieces = `<div class="e-set">${head("size")}${seg("Size", ["Any", ...B.size.presets.map((p) => p.id)], size() ?? "Any")}</div>`;
    const base = `<div class="e-set">${head("base")}${seg("Base", B.base.options, val("base") ?? B.base.value)}</div>`;
    const n = Object.keys(dirty).length;
    const bar = n ? `<div class="e-keep e-rebuild"><span>${n} changed</span>${btn("Undo", "undo", "e-quiet e-sm")}${btn("Rebuild", "rebuild", "e-primary e-sm", "refresh")}</div>` : "";
    return `<div class="e-brief">${bar}${style}${scale}${detail}${pieces}${base}</div>`;
  }

  // Checks (law 2: every finding one click from its part and its line).
  function checks(screen) {
    const b = screen.build || {};
    if (b.state !== "findings") return `<div class="e-clean">${icon("check", "e-ok")}<div><b>No problems</b><p>Nothing collides, nothing floats, and every part is current.</p></div></div>`;
    const f = D.finding;
    const src = f.excerpt.map((l, i) => `<span class="e-ln">${f.line + i}</span>${esc(l.trim())}`).join("\n");
    return `<div class="e-finding" data-selected="${!!screen.selected}"><div class="e-err-top">${icon("warn", "e-bad")}<b>${f.parts} parts aren't held by anything</b><span class="e-chip">${esc(f.id)} · floating</span></div>
      <p>${esc(f.message)}</p><p class="e-small">${esc(f.hint)}</p>
      <pre class="e-src" title="model.py line ${f.line}">${src}</pre>
      <div class="e-row">${screen.agent?.state === "working" ? `<span class="e-fixing">${spinner()}${esc(agentOf(screen).name)} is fixing this</span>` : btn("Ask agent to fix", "fix", "e-primary")}${btn("Show in model.py", "", "e-quiet", "file")}${btn("It's fine", "", "e-quiet")}</div></div>`;
  }

  // Each line's part in its color, rendered by `emmet render` on a one-part model with the background keyed out.
  const partPic = (l) => `<img class="e-bom-pic" src="${window.EMMET_ROOT}shared/parts/${l.part}-${l.color.toLowerCase().replace(/[^a-z0-9]+/g, "-")}.webp" alt="" width="40" height="40">`;
  function bom(screen) {
    const rows = D.bom.map((l) => `<tr><td>${partPic(l)}</td><td class="e-mono">${esc(l.part)}</td><td>${esc(l.name)}</td><td>${swatch(l.color)}${esc(l.color)}</td><td class="e-num">${l.qty}</td></tr>`).join("");
    return `<div class="e-bom"><div class="e-bom-top"><b>${(screen.build?.parts) || D.run.parts} parts · ${D.bom.length} kinds</b><span class="e-small">Availability arrives with the parts data pack</span><span class="e-grow"></span>${btn("Export", "export", "e-sm", "download")}</div>
      <table><thead><tr><th aria-label="Picture"></th><th>Part</th><th>Name</th><th>Color</th><th class="e-num">Qty</th></tr></thead><tbody>${rows}</tbody></table></div>`;
  }

  function stepsList(screen) {
    const cur = screen.step || D.steps.length;
    const rows = D.steps.map((s) => `<li data-state="${s.n < cur ? "past" : s.n === cur ? "current" : "later"}"><span class="e-step-n">${s.n}</span><span>${esc(s.name)}</span><span class="e-small">${s.parts} part${s.parts > 1 ? "s" : ""}</span></li>`).join("");
    return `<div class="e-steps"><div class="e-steps-top"><b>Step ${cur} of ${D.steps.length}</b>${cur < D.steps.length ? `<span class="e-small">Later steps are hidden</span>` : ""}</div><ol>${rows}</ol></div>`;
  }

  const TABS = [["params", "Parameters", "sliders"], ["checks", "Checks", "check"], ["parts", "Parts", "list"], ["steps", "Steps", "steps"]];
  function tabs(active, screen, opts = {}) {
    const built = !!screen.build?.parts;
    const count = { checks: screen.build?.findings || "", parts: screen.build?.parts || "", steps: built ? D.steps.length : "" };
    return `<div class="e-tabs" role="tablist">${TABS.map(([id, label, ic]) => `<button type="button" role="tab" class="e-tab" data-tab="${id}" aria-selected="${id === active}">${opts.icons ? icon(ic) : ""}<span>${label}</span>${count[id] ? `<span class="e-count" data-tone="${id === "checks" ? "warn" : ""}">${count[id]}</span>` : ""}</button>`).join("")}</div>`;
  }
  const panel = (tab, screen) => ({ params, checks, parts: bom, steps: stepsList })[tab]?.(screen) || "";

  // The step scrubber (section 14's viewer toolbar; note 6).
  function scrubber(screen) {
    const b = screen.build || {};
    // A partial model (building, or stopped part way) shows the step it reached, not the last one.
    const partial = (b.state === "building" || b.state === "built") && b.parts && b.parts < D.run.parts;
    const cur = screen.step || (partial ? D.stepOf(b.parts) : D.steps.length);
    return `<div class="e-scrub"><button type="button" class="e-icon-btn" title="Previous step ([)">◀</button><span>Step ${cur}/${D.steps.length}</span><button type="button" class="e-icon-btn" title="Next step (])">▶</button></div>`;
  }
  function views() {
    return `<div class="e-views" role="group" aria-label="Views">${["Iso", "Front", "Top"].map((v, i) => `<button type="button" class="e-view" aria-pressed="${i === 0}">${v}</button>`).join("")}<button type="button" class="e-icon-btn" title="Frame (F)">${icon("frame")}</button><button type="button" class="e-icon-btn" title="Screenshot">${icon("camera")}</button></div>`;
  }
  function animating(screen, opts = {}) {
    if (!screen.animating) return "";
    if (opts.bare) return btn("Skip animation", "skip", "e-sm e-anim-skip");
    return `<div class="e-anim">${spinner()}<span>Placing parts in build order · ${screen.build.parts} so far</span>${btn("Skip", "skip", "e-sm")}</div>`;
  }

  function banner(kind) {
    const B = {
      untrusted: [icon("lock"), "Read-only: " + D.untrusted.message, D.untrusted.hint, btn("Read model.py", "", "e-quiet e-sm") + btn("Trust this project", "trust", "e-primary e-sm")],
      offline: [icon("wifi"), "You're offline.", "Building, checks, sliders, and exports still work. The agent and downloads need a connection.", btn("Try again", "", "e-quiet e-sm")],
      "old-library": [icon("download"), `Your parts library is ${D.versions.oldLibrary}. This Emmet expects ${D.versions.library}.`, "Some parts may be missing or renamed until you update (about 150 MB, a minute or two).", btn("Later", "", "e-quiet e-sm") + btn("Update the library", "update", "e-primary e-sm")],
      imported: [icon("upload"), `Opened ${D.imported.file}: ${D.imported.parts} parts in ${D.imported.steps} steps.`, "It has no model.py yet, so it has no sliders. The agent can rewrite it as a parametric model.", btn("Make it parametric", "parametric", "e-primary e-sm")],
    }[kind];
    return B ? `<div class="e-banner" data-kind="${kind}" role="status">${B[0]}<div><b>${esc(B[1])}</b><p>${esc(B[2])}</p></div><span class="e-grow"></span><div class="e-row">${B[3]}</div></div>` : "";
  }

  // Not signed in, inside the workspace (section 14 state; D-41's buttons).
  function signedOut() {
    return `<div class="e-setup-card">${icon("info")}<div><b>Claude isn't signed in</b><p>Emmet builds with your own Claude Code. Sign in opens Claude's page in your browser; Emmet never sees your password.</p>
      <div class="e-row">${btn("Sign in to Claude", "signin", "e-primary")}${btn("Use Codex instead", "other", "e-quiet")}</div><p class="e-small">${esc(D.plans.claude)}</p></div></div>`;
  }

  // Overlays.
  function agentMenu(screen) {
    const cur = screen.who || "claude";
    const item = (id, extra) => `<button type="button" role="menuitemradio" class="e-menu-item" aria-checked="${id === cur}" data-act="pick"><span>${esc(D.agents[id].name)}</span><span class="e-small">${esc(extra)}</span></button>`;
    return `<div class="e-menu" role="menu" data-for="agents">${item("claude", "Max plan")}${item("codex", "ChatGPT plan")}<p class="e-small e-menu-note">Switching starts a fresh conversation. This one is kept.</p></div>`;
  }
  function modelMenu(screen) {
    const a = agentOf(screen);
    const sel = (label, list, v) => `<label class="e-field"><span>${label}</span><select>${list.map((x) => `<option${x === v ? " selected" : ""}>${esc(x)}</option>`).join("")}</select></label>`;
    return `<div class="e-menu e-menu-wide" role="dialog" aria-label="Model and effort" data-for="model">${sel("Model", a.models, a.model)}${sel("Effort", D.efforts, "Medium")}<p class="e-small">Bigger models and higher effort use up your plan faster.</p></div>`;
  }
  function usageMini(screen = {}) {
    if (screen.who === "codex") return `<button type="button" class="e-usage" data-act="usage" title="Plan usage">Codex · Using ChatGPT plan</button>`;
    return `<button type="button" class="e-usage" data-act="usage" title="Plan usage">Claude ${D.usage.map((u) => `<span class="e-meter" title="${esc(u.label)}"><i style="width:${Math.max(u.pct, 2)}%"></i></span>`).join("")}<span>${D.usage[0].pct}% · ${D.usage[0].reset}</span></button>`;
  }
  function usagePanel() {
    const rows = D.usage.map((u) => `<div class="e-usage-row"><span>${esc(u.label)}</span><span class="e-meter e-meter-lg"><i style="width:${Math.max(u.pct, 1)}%"></i></span><span class="e-num">${u.pct}%</span><span class="e-small">resets in ${esc(u.reset)}</span></div>`).join("");
    return `<div class="e-menu e-menu-wide" role="dialog" aria-label="Plan usage" data-for="usage"><b>Claude · Max plan</b>${rows}<p class="e-small">Codex · Using ChatGPT plan · no usage report yet. Updated 8m ago.</p></div>`;
  }
  function exportMenu() {
    const item = (ic, label, sub) => `<button type="button" class="e-menu-item" role="menuitem">${icon(ic)}<span>${esc(label)}</span><span class="e-small">${esc(sub)}</span></button>`;
    return `<div class="e-menu e-menu-wide" role="menu" data-for="export"><p class="e-menu-group">Parts list</p>${item("list", "CSV", "Every part and color, with counts")}${item("list", "Rebrickable", "A parts list to import")}${item("list", "BrickLink XML", "A wanted list")}
      <p class="e-menu-group">Model</p>${item("cube", "LDraw .mpd", "Opens in Studio, LDCad, and LeoCAD")}<p class="e-menu-group">Instructions</p>${item("file", "PDF", "Step pages, A4 or Letter")}${item("file", "LPub3D-ready file", "For your own layout")}</div>`;
  }
  // About: the app icon, the version and license, what Emmet connects to (section 15), and the trademark line
  // (TRADEMARKS.md, D-30). A dialog the harness centers over the window.
  function about() {
    const theme = document.documentElement.dataset.theme;
    return `<div class="e-menu e-about" role="dialog" aria-label="About Emmet Builder"><img class="e-about-icon" src="${window.EMMET_ROOT}brand/icon-macos-ant.svg" alt="">
      <img class="e-about-word" src="${window.EMMET_ROOT}brand/wordmark-emmet-builder-ant-${theme}.svg" alt="Emmet Builder">
      <p>Version ${D.versions.emmet} · LDraw library ${D.versions.library} · Parts data ${D.versions.partsData}</p>
      <p>Free and open source under the AGPL-3.0. Your models and chats stay on this computer; Emmet has no accounts and no tracking.</p>
      <p class="e-small">Emmet connects only to: the LDraw library and parts data downloads, its own update check, and the agent you pick, which talks to Anthropic or OpenAI on your plan.</p>
      <p class="e-small e-tm">${trademark()} LDraw is a trademark of the Estate of James Jessiman. Claude is a trademark of Anthropic, and Codex and ChatGPT of OpenAI; Emmet isn't affiliated with or endorsed by them.</p>
      <div class="e-row">${btn("Licenses", "", "e-quiet e-sm")}${btn("Close", "close", "e-sm")}</div></div>`;
  }
  const overlay = (screen) => ({ agents: agentMenu, model: modelMenu, usage: usagePanel, export: exportMenu, about })[screen.overlay]?.(screen) || "";

  // First run (D-41: no terminal; which plans work; the provider notice once).
  function firstRun(step) {
    const steps = [["welcome", "Welcome"], ["library", "Parts and Python"], ["agents", "Your agent"], ["start", "Start"]];
    const at = steps.findIndex(([id]) => id === step);
    const rail = `<ol class="e-fr-steps">${steps.map(([id, label], i) => `<li data-state="${i < at ? "done" : i === at ? "current" : "later"}">${i < at ? icon("check") : `<span>${i + 1}</span>`}${label}</li>`).join("")}</ol>`;
    let body;
    if (step === "welcome") {
      body = `<h1>Build brick models by describing them</h1><p class="e-lede">Emmet Builder works with your own Claude Code or Codex. You say what to build; your agent writes a small Python model from real LDraw parts, and Emmet shows it, checks it, and gives you a parts list and building steps.</p>
        <ul class="e-fr-list"><li>${icon("cube")}Real parts, in colors they're actually made in</li><li>${icon("check")}Checks for parts that collide or float</li><li>${icon("sliders")}Pick a style, scale, and size instead of placing 400 bricks</li></ul>
        <div class="e-row">${btn("Get started", "primary", "e-primary e-lg")}</div><p class="e-small">Your models and chats stay on this computer. Emmet has no accounts and no tracking.</p>`;
    } else if (step === "library") {
      body = `<h1>Getting the parts ready</h1><p class="e-lede">Downloading the LDraw parts library (about 150 MB) and setting up Python. This takes a minute or two, and only happens once.</p>
        <div class="e-progress-row"><span>LDraw parts library ${D.versions.library}</span><span class="e-bar"><i style="width:64%"></i></span><span class="e-num">96 of 150 MB</span></div>
        <div class="e-progress-row"><span>Python for models</span><span class="e-bar"><i style="width:0%"></i></span><span class="e-small">Next</span></div>
        <p class="e-small">While you wait: set up your agent on the next screen. Everything keeps downloading.</p><div class="e-row">${btn("Continue", "primary", "e-primary e-lg")}</div>`;
    } else if (step === "agents") {
      body = `<h1>Set up your agent</h1><p class="e-lede">Emmet builds with your own Claude Code or Codex. Set up either one here, no terminal needed.</p>
        <div class="e-agent-card" data-ready="true">${icon("check", "e-ok")}<div><b>Claude Code ${D.agents.claude.version}</b><p>Signed in · Max plan</p></div><span class="e-badge">Ready</span></div>
        <div class="e-agent-card">${icon("download")}<div><b>Codex</b><p>${esc(D.plans.codex)}</p></div>${btn("Install Codex", "", "e-primary")}</div>
        <p class="e-small">${esc(D.plans.claude)}</p>
        <div class="e-notice">${icon("info")}<p>Claude Code and Codex are made by Anthropic and OpenAI, not by Emmet. Your plan and its limits are between you and them, and you use them with Emmet at your own risk.</p></div>
        <div class="e-row">${btn("Check agents again", "", "e-quiet", "refresh")}${btn("Continue", "primary", "e-primary e-lg")}</div>`;
    } else {
      body = `<h1>What do you want to build?</h1><p class="e-lede">New projects go in Documents/Emmet. You can change that in Settings.</p>
        <div class="e-start"><button type="button" class="e-start-card" data-act="example"><span class="e-thumbbox">${shot("cottage", document.documentElement.dataset.theme)}</span><b>Start with the Cottage example</b><span class="e-small">163 parts: a real door, glass windows, and a room inside</span></button>
        <button type="button" class="e-start-card" data-act="prompt"><span class="e-thumbbox">${shot("empty", document.documentElement.dataset.theme)}</span><b>New model from a prompt</b><span class="e-small">An empty baseplate and your agent</span></button></div>
        <p class="e-small">Or open a folder, or an LDraw file from Studio, LDCad, or LeoCAD.</p>`;
    }
    return { rail, body };
  }

  // The trademark line (TRADEMARKS.md; D-30), for About and the site footer.
  const trademark = () => `LEGO® is a trademark of the LEGO Group, which does not sponsor, authorize, or endorse this software.`;

  return {
    esc, md, icon, spinner, lights, btn, kbd, swatch, shot, buildText, buildTone, agentState, chat, message, pending,
    composer, chatHeader, params, buildSettings, checks, bom, stepsList, tabs, panel, scrubber, views, animating, banner, signedOut,
    overlay, usageMini, firstRun, trademark, agentOf,
  };
})();
