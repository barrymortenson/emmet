// SPDX-License-Identifier: AGPL-3.0-or-later
// Every frozen screen and state (plan session 2B), composed only from components.js. The list starts from 2A's
// (docs/design/shared/states.js: the same ids, chat, and flows, which direction B was reviewed on) and adds what the
// design system needed to settle: the Steps tab's tree, a failed download in first run, the project menu, the chat
// history, a tooltip, a toast, the plan's limit, and Settings. `covers` names the section 14 states a screen shows,
// `group` places it in the catalog, and `on` maps a click to the next screen.
window.EMMET_FROZEN_DATA = {
  // Sub-assemblies as 3B's submodels will make them (composed for these screens: today's model.py places the door
  // and windows in the walls). The door is the step view's sub-assembly in 2A.
  submodels: [
    { id: "door", name: "Door", uses: 1, parts: 2, steps: ["Frame", "Door in its frame"] },
    { id: "window", name: "Window", uses: 5, parts: 2, steps: ["Frame and glass"] },
  ],
};

window.EMMET_FROZEN = (() => {
  const D = window.EMMET_DATA;
  const C = window.EMMET_C;
  const S = window.EMMET_STATES;
  const seg = D.seg;
  const story = (...p) => p.flat();
  const esc = C.esc;
  const { icon, button, badge } = C;

  // The section 14 states, in the plan's words (tests/scripts/design.test.mjs reads them from plan.html and checks
  // each has a screen), and the key flows.
  const FLOWS = S.FLOWS.map((f) => (f.id === "sliders" ? { ...f, name: "Tweak with knobs" } : f))
    .concat([{ id: "structure", name: "Find a sub-assembly", path: ["built", "structure"] }]);

  const finished = story(seg.prompt, seg.answered, seg.start, seg.stage1, seg.stage2, seg.stage3, seg.finish);
  const built = { kind: "workspace", render: "cottage", build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Done in 1:12" }, chat: finished };
  // 2A's screens, as B drew them, with the groups and the few fixes the frozen design makes.
  const from2A = (id, extra = {}) => ({ ...S.byId[id], ...extra, id: extra.id || id });
  const screens = [
    // First run (section 14 key flow 1; D-41: no terminal).
    from2A("fr-welcome", { group: "First run" }),
    from2A("fr-library", { group: "First run" }),
    { id: "fr-library-failed", title: "First run: the download failed", kind: "first-run", step: "library", failed: true, group: "First run", covers: ["First run", "Loading library"], on: { retry: "fr-library" }, note: "Every failure in first run has one clear retry (4A). The Python setup waits; nothing is lost." },
    from2A("fr-agents", { group: "First run" }),
    from2A("fr-start", { group: "First run" }),

    // New model from a prompt: think, ask, build, redirect, approve, done.
    from2A("new-empty", { group: "Building", drawer: "options" }),
    from2A("thinking", { group: "Building" }),
    from2A("question", { group: "Building" }),
    from2A("building", { group: "Building" }),
    from2A("redirect", { group: "Building" }),
    from2A("approval", { group: "Building" }),
    from2A("built", { group: "Building", drawer: "options" }),
    { ...built, id: "history", title: "The chat's history, opened", group: "Building", history: "open", covers: [], note: "History opens upward and pushes the viewer up; the model is never covered (law 1). Drag its top edge to resize it." },

    // Knobs and build settings.
    from2A("sliders", { group: "Knobs and settings", title: "Tweak with a knob: weathering", drawer: "options" }),
    from2A("shuffled", { group: "Knobs and settings", drawer: "options" }),
    from2A("settings", { group: "Knobs and settings", drawer: "options" }),
    { ...built, id: "tooltip", title: "Build settings, with a setting's tooltip", group: "Knobs and settings", agent: { state: "idle", line: "Idle" }, drawer: "options", brief: "open", tip: "scale", covers: [], note: "Names, not subtext (Barry 2026-10-27): what a setting decides is its tooltip, shown after a pause on hover or focus." },
    from2A("restyle", { group: "Knobs and settings", drawer: "options" }),

    // Steps and the model's structure.
    from2A("step-door", { group: "Steps" }),
    from2A("steps", { group: "Steps" }),
    { ...built, id: "structure", title: "Steps: a sub-assembly, every copy highlighted", group: "Steps", agent: { state: "idle", line: "Idle" }, drawer: "steps", selected: "Window", expanded: "Window", covers: [], note: "Barry 2026-10-27 (2B): the Steps tab is the model's tree. Clicking a sub-assembly highlights every copy (the viewer dims the rest, as the finding's render shows); clicking a step shows the build up to it." },

    // Checks and failures.
    from2A("findings", { group: "Checks and failures" }),
    from2A("fixing", { group: "Checks and failures" }),
    from2A("failed", { group: "Checks and failures" }),

    // The agent: stop, crash, sign-in, switch, model, usage.
    from2A("stopped", { group: "The agent" }),
    from2A("crashed", { group: "The agent" }),
    from2A("signed-out", { group: "The agent" }),
    from2A("switch-agent", { group: "The agent" }),
    from2A("switched", { group: "The agent" }),
    from2A("model-picker", { group: "The agent" }),
    from2A("usage", { group: "The agent" }),
    { ...built, id: "limit", title: "The plan's limit reached", group: "The agent", agent: { state: "limit", line: "Claude's plan limit · resets in 2h 14m" }, usageAt: 100, limit: true, covers: [], on: { other: "switched" }, note: "4B: the limit-reached card offers the other agent. The plan badge turns warn from 90 %." },

    // The project: its menu, versions, export, import, About, Settings.
    { ...built, id: "project-menu", title: "The project menu", group: "The project", menu: "open", covers: [], note: "The project chip's menu: New, Open, versions, Export, and About. The native menus (macOS) and the title-bar menu (Windows) hold the same and more (docs/design/keyboard.md)." },
    from2A("version-save", { group: "The project" }),
    from2A("versions", { group: "The project" }),
    from2A("version-view", { group: "The project" }),
    from2A("export", { group: "The project" }),
    { ...built, id: "export-done", title: "Exported, with a toast", group: "The project", drawer: "parts", toast: true, covers: [], note: "A toast confirms what was written and where, for five seconds, with the one thing you'd do next." },
    from2A("import", { group: "The project" }),
    from2A("about", { group: "The project" }),
    { ...built, id: "settings-dialog", title: "Settings", group: "The project", overlay: "settings", covers: [], on: { close: "built" }, note: "4A's Settings, in one dialog: agent, projects folder, appearance, parts, viewer, updates, the command line tool, and diagnostics. About stays its own dialog." },

    // Trouble that isn't the agent's.
    from2A("untrusted", { group: "Banners" }),
    from2A("offline", { group: "Banners" }),
    from2A("old-library", { group: "Banners" }),
  ];
  // 2A's tab id for the options tab was "params".
  for (const s of screens) if (s.drawer === "params") s.drawer = "options";

  const byId = Object.fromEntries(screens.map((s) => [s.id, s]));
  const missing = S.STATES.filter((st) => !screens.some((x) => x.covers.includes(st)));
  if (missing.length) throw new Error("screens.js doesn't cover: " + missing.join(", "));

  // ---- Composition ------------------------------------------------------------------------------------------------

  // A key in the platform's own words: ⌘ on macOS, Ctrl on Windows.
  const key = (k, ctx) => (ctx.platform === "windows" ? k.replace("⌘", "Ctrl+").replace("⌥", "Alt+").replace("⇧", "Shift+").replace("↩", "Enter") : k);
  const historyOpen = (s) => (s.history === "open" ? true : s.history === "closed" ? false : ["switch-agent", "switched"].includes(s.id));
  const tabOf = (s) => s.drawer || "options";
  const ANT = () => `<img src="${window.EMMET_ROOT}brand/mark-ant.svg" alt="">`;

  function projectMenu(s, ctx) {
    if (s.menu === "open") return C.menu([
      { icon: "plus", text: "New", key: key("⌘N", ctx) }, { icon: "folder-open", text: "Open…", key: key("⌘O", ctx) }, "-",
      { icon: "save", text: "Save version", key: key("⌘S", ctx), local: "menu=save" }, { icon: "rotate-ccw-clock", text: "Versions…", local: "menu=versions" }, "-",
      { icon: "download", text: "Export…", key: key("⌘E", ctx), act: "export" }, "-", { icon: "info", text: "About Emmet Builder", act: "about" },
    ], { label: "Project" });
    if (s.menu === "save") return C.popover("Save version", C.textField({ value: "Steeper roof", label: "Version name" }) + `<div class="popover-actions">${button("Cancel", { variant: "quiet", size: "sm", local: "menu=closed" })}${button("Save", { variant: "primary", size: "sm", icon: "save", local: "menu=versions" })}</div>`);
    if (s.menu === "versions" && !s.view) return C.popover("Versions", `<div class="popover-head">${badge(`${D.history.length} versions`)}${button("Save version", { size: "sm", icon: "save", local: "menu=save" })}</div>${D.history.map((v) => C.versionRow(v)).join("")}`);
    return "";
  }

  function titlebar(s, ctx) {
    const pop = projectMenu(s, ctx);
    const win = ctx.platform === "windows";
    return `<header class="titlebar" data-c="titlebar" data-tauri-drag-region><div class="titlebar-start"><button type="button" class="project-chip" aria-haspopup="menu" aria-expanded="${s.menu === "open"}" data-local="menu=${s.menu === "open" ? "closed" : "open"}">${ANT()}${esc(s.imported ? D.imported.name : D.project.name)}${icon("chevron-down", "sm")}</button>
      ${win ? C.menubar() : ""}${pop ? `<div class="anchored">${pop}</div>` : ""}</div>${C.viewSwitch()}<span></span>${win ? C.captionButtons() : ""}</header>`;
  }

  function viewer(s) {
    const seen = s.view && D.history.find((v) => String(v.n) === s.view);
    const pin = s.build.state === "findings" && !seen ? C.pin(`${D.finding.parts} parts float`, { u: 0.541, v: 0.344 }, D.renders.finding) : "";
    const bar = seen ? `<div class="viewer-bar" role="status">${icon("rotate-ccw-clock")}<span><b>Version ${seen.n}: ${esc(seen.name)}</b> · ${esc(seen.when)}</span>${button("Restore", { variant: "primary", size: "sm", icon: "undo-2", act: "restore", title: "Saves the current model as a version first" })}${button("Open as new project", { size: "sm", act: "copy-out" })}${button("Back", { variant: "quiet", size: "sm", local: "view=" })}</div>` : "";
    const skip = s.animating ? `<div class="viewer-tools">${button("Skip animation", { size: "sm", act: "skip", cls: "viewer-skip" })}</div>` : "";
    const steps = !seen && s.step ? C.stepView(s.step) : "";
    const leader = s.step && D.stepView[s.step]?.sub ? `<span class="viewer-anchor" data-leader style="--u:0.408;--v:0.525;--k:${D.renders["step-4"].k};--ty:${D.renders["step-4"].ty}"></span>` : "";
    return `<section class="viewer" data-c="viewer" aria-label="3D view of ${esc(D.project.name)}">${C.render(seen ? seen.render : s.render, { alt: "" })}${pin}${steps}${leader}${bar}${skip}${s.toast ? C.toast(`Saved “Cottage parts.csv” to Downloads`, "Show in folder") : ""}</section>`;
  }

  // The options panel's tabs.
  function modelState(b) {
    return { empty: "Empty", building: "Building", failed: "Build failed", readonly: "Read-only", findings: `${b.findings} problem${b.findings > 1 ? "s" : ""}` }[b.state] || "No problems";
  }
  function setting(name, help, control, o = {}) {
    const was = o.was !== undefined ? badge(`was ${o.was}`, { tone: "accent" }) : "";
    return `<div class="setting" data-c="setting"${C.attrs({ "data-moved": o.moved ? "true" : undefined, "data-inline": o.inline })}><div class="setting-head"><span class="setting-name" title="${esc(help)}" tabindex="0">${esc(name)}</span>${was}</div>${control}${o.tip ? C.tooltip(help, "left:var(--space-sm);top:calc(var(--control-sm) + var(--space-3xs))") : ""}</div>`;
  }
  function buildSettings(s) {
    const B = D.brief;
    const dirty = s.briefDirty || {};
    const fresh = s.build.state === "empty";
    const val = (k) => (k in dirty ? dirty[k] : fresh ? null : B[k].value);
    const range = (p) => (p.max ? `${p.min.toLocaleString("en-US")}–${p.max.toLocaleString("en-US")}` : `${p.min.toLocaleString("en-US")}+`);
    const n = Object.keys(dirty).length;
    const bar = n ? `<div class="pending-bar"><span>${n} changed</span>${button("Undo", { variant: "quiet", size: "sm", act: "undo" })}${button("Rebuild", { variant: "primary", size: "sm", icon: "refresh-cw", act: "rebuild" })}</div>` : "";
    const full = fresh || s.brief === "open" || (n && s.brief !== "closed");
    if (!full) {
      const size = B.size.presets.find((p) => p.id === val("size"));
      const words = [val("scale"), val("age") && `Ages ${val("age")}`, val("detail") && `${val("detail")} detail`, size && `${range(size)} pieces`, val("style")].filter(Boolean).join(" · ");
      return `${bar}<div class="summary-row"><span>${esc(words || "Any")}</span>${button("Change", { size: "sm", local: "brief=open" })}</div>`;
    }
    const ageNow = s.pickAge ?? val("age") ?? "Any";
    const preset = s.pickAge && B.age.options.find((o) => o.id === ageNow);
    const mine = (k) => { const [v, a] = (s[k] || "").split("@"); return v && a === ageNow ? v : null; };
    const pick = (k, pv, stored) => mine(k) ?? (preset ? pv : s.pickAge === "Any" ? "Any" : stored) ?? "Any";
    const detailNow = pick("pickDetail", preset?.detail, val("detail"));
    const piecesNow = pick("pickPieces", preset?.pieces, val("size"));
    const styleNow = val("style");
    const own = styleNow && !B.style.options.some((o) => o.id === styleNow) ? [styleNow] : [];
    const moved = (k) => ({ moved: k in dirty, was: k in dirty ? B[k].value ?? "Any" : undefined });
    return `${bar}<div class="settings-list">
      ${setting("Base", B.base.help, C.segmented("Base", B.base.options, val("base") ?? B.base.value))}
      ${setting("Scale", B.scale.help, C.chips("Scale", [{ id: "Any", sub: "The prompt decides" }, ...B.scale.options], val("scale") ?? "Any"), { ...moved("scale"), tip: s.tip === "scale" })}
      ${setting("Age", B.age.help, C.segmented("Age", ["Any", ...B.age.options.map((o) => ({ id: o.id, tip: o.sub }))], ageNow, { local: (o) => `pickAge=${o}` }), moved("age"))}
      ${setting("Detail", B.detail.help, C.segmented("Detail", ["Any", ...B.detail.options.map((o) => ({ id: o.id, tip: o.sub }))], detailNow, { local: (o) => `pickDetail=${o}@${ageNow}` }), moved("detail"))}
      ${setting("Pieces", B.size.help, C.chips("Pieces", ["Any", ...B.size.presets.map((p) => ({ id: p.id, name: range(p) }))], piecesNow, { local: (o) => `pickPieces=${o}@${ageNow}` }))}
      ${setting("Style", B.style.help, C.chips("Style", B.style.options, own.length ? null : styleNow, { own, role: "radio" }) + C.textField({ placeholder: "Your own, like steampunk or zen garden", label: "Your own style" }), moved("style"))}</div>`;
  }
  function knobs(s) {
    const dirty = s.dirty || {};
    const n = Object.keys(dirty).length;
    const keep = n ? `<div class="pending-bar"><span>${n} change${n > 1 ? "s" : ""}</span>${button("Reset", { variant: "quiet", size: "sm", act: "reset" })}${button("Keep", { variant: "primary", size: "sm", act: "keep" })}</div>` : "";
    return keep + D.params.map((p) => {
      const v = p.name in dirty ? dirty[p.name] : p.value;
      const help = p.help[0].toUpperCase() + p.help.slice(1);
      const was = p.name in dirty && p.type !== "seed" ? `${p.value} ${p.unit || ""}`.trim() : undefined;
      const moved = { moved: p.name in dirty, was };
      if (p.type === "int") return setting(p.label, help, C.slider(p, v, { act: "slider" }), moved);
      if (p.type === "seed") return setting(p.label, help, button("Shuffle", { size: "sm", icon: "shuffle", act: "shuffle" }), { ...moved, inline: true });
      return setting(p.label, help, C.switchC(p.label, v), { inline: true });
    }).join("");
  }
  function panelBody(s, tab) {
    const b = s.build;
    if (tab === "options") {
      const knobPart = b.state === "empty" || s.imported ? "" : b.state === "readonly"
        ? `<section class="panel-section"><div class="panel-section-head">Tweaks</div><p class="muted row">${icon("lock", "sm")}Trust the project to use these</p></section>`
        : `<section class="panel-section"><div class="panel-section-head">Tweaks</div>${knobs(s)}</section>`;
      const done = s.brief === "open" && b.state !== "empty" ? button("Done", { variant: "quiet", size: "sm", local: "brief=closed" }) : "";
      return `<section class="panel-section"><div class="panel-section-head"><span title="Together they set how the agent designs the model">Build settings</span>${done}</div>${buildSettings(s)}</section>${knobPart}`;
    }
    if (tab === "checks") return C.finding(s);
    if (tab === "parts") return `<div class="row"><b>${b.parts || D.run.parts} parts · ${D.bom.length} kinds</b></div>${C.dataTable()}`;
    const cur = s.step || D.steps.length;
    return `<div class="row"><b>Step ${cur} of ${D.steps.length}</b>${cur < D.steps.length ? `<span class="small">Later steps are hidden</span>` : ""}</div>${C.tree({ step: s.step, selected: s.selected && typeof s.selected === "string" ? s.selected : undefined, expanded: s.expanded })}`;
  }
  function panel(s, ctx) {
    const b = s.build;
    const tone = { failed: "bad", findings: "warn", building: "busy", readonly: "warn", empty: "muted" }[b.state] || "ok";
    const lead = tone === "busy" ? C.spinner("Building") : icon({ bad: "octagon-alert", warn: b.state === "readonly" ? "lock" : "triangle-alert", muted: "info", ok: "circle-check" }[tone]);
    const tab = tabOf(s);
    const counts = { checks: b.findings, parts: b.parts, steps: b.parts ? D.steps.length : 0 };
    const exp = s.overlay === "export" ? `<div class="anchored">${C.menu([{ label: "Parts list" }, { icon: "sheet", text: "CSV", sub: "Every part and color, with counts" }, { icon: "list", text: "Rebrickable", sub: "A parts list to import" }, { icon: "list", text: "BrickLink XML", sub: "A wanted list" }, { label: "Model" }, { icon: "box", text: "LDraw .mpd", sub: "Opens in Studio, LDCad, and LeoCAD" }, { label: "Instructions" }, { icon: "file-text", text: "PDF", sub: "Step pages, A4 or Letter" }, { icon: "file-text", text: "LPub3D-ready file", sub: "For your own layout" }], { label: "Export" })}</div>` : "";
    return `<aside class="panel" data-c="panel" aria-label="Options"><div class="panel-head" data-tone="${tone}">${lead}<span class="panel-head-text">${esc(modelState(b))}</span>${b.parts ? button("Export", { size: "sm", icon: "download", act: "export", kbd: undefined, title: `Export (${key("⌘E", ctx)})` }) : ""}</div>
      ${C.tabs(tab, counts)}<div class="panel-body" role="tabpanel">${panelBody(s, tab)}</div>${exp}</aside>`;
  }

  // The chat dock.
  function activeCard(s) {
    const a = C.agentOf(s);
    if (s.agent.state === "signedout") return C.alertCard({ icon: "log-in", title: `${a.name} isn't signed in`, text: `Emmet builds with your own ${a.cli}. Sign in opens ${a.name}'s page in your browser; Emmet never sees your password.`, actions: button(`Sign in to ${a.name}`, { variant: "primary", act: "signin" }) + button("Use Codex instead", { variant: "quiet", act: "other" }), foot: D.plans.claude });
    if (s.limit) return C.alertCard({ icon: "gauge", title: `You've used ${a.name}'s plan limit for now`, text: "It resets in 2h 14m. Your model and this conversation are kept.", actions: button("Continue with Codex", { variant: "primary", act: "other" }) + button("Wait for the reset", { variant: "quiet" }) });
    if (s.error) return C.alertCard({ tone: "bad", icon: "octagon-alert", title: "The model didn't build", text: D.buildError.plain, actions: button("Ask agent to fix", { variant: "primary", act: "fix" }), foot: "The last good model is still shown." });
    if (s.pending) return C.pending(s.pending, a.name);
    const last = s.chat[s.chat.length - 1];
    if (last && ["question", "approval", "crash"].includes(last.t) && !last.answered) return C.message(last, s);
    return "";
  }
  function latestLine(s) {
    if (s.dirty || s.briefDirty) return "";
    const last = s.chat[s.chat.length - 1];
    if (!last || (last.t === "tool" && last.status === "running")) return "";
    if (last.t === "thinking") return `<p class="thought">${esc(last.text)}</p>`;
    return ["tool", "file", "note", "agent"].includes(last.t) ? C.message(last, s) : "";
  }
  function dock(s, ctx) {
    const a = C.agentOf(s);
    const open = historyOpen(s);
    const card = activeCard(s);
    const says = card && !s.pending;
    const quiet = s.agent.line === "Idle" || says || s.banner;
    const undo = !quiet && s.agent.state === "idle" && /^Done/.test(s.agent.line || "") ? button("Undo turn", { variant: "quiet", size: "sm", icon: "undo-2", act: "undo-turn", title: "Restore the model from before this turn (version 6)" }) : "";
    const pct = s.usageAt ?? D.usage[0].pct;
    const plan = badge(`${a.plan}${a === D.agents.claude ? ` · ${pct}%` : ""}`, { act: "usage", tone: pct >= 90 && a === D.agents.claude ? "warn" : undefined, title: "Plan usage" });
    const n = s.chat.length;
    const top = `<div class="dock-top"><button type="button" class="agent-picker" data-act="agents" aria-haspopup="menu" aria-expanded="${s.overlay === "agents"}">${esc(a.name)}${icon("chevron-down", "sm")}</button>${plan}${quiet ? `<span class="grow"></span>` : C.status(s)}${undo}
      ${n ? button(open ? "Hide history" : `History · ${n}`, { variant: "quiet", size: "sm", icon: "messages-square", local: `history=${open ? "closed" : "open"}`, expanded: open }) : ""}${button("Start fresh", { variant: "quiet", size: "sm", icon: "message-square-plus", title: "Clear the context and keep the agent" })}</div>`;
    const mid = open ? `<div class="dock-history" data-scroll="end">${n ? C.chat(s) : ""}${card}</div>` : card ? `<div class="dock-active">${card}</div>` : (() => { const l = latestLine(s); return l ? `<div class="dock-line">${l}</div>` : ""; })();
    let pop = "";
    if (s.overlay === "agents") pop = C.menu([{ text: "Claude", sub: "Max plan", checked: (s.who || "claude") === "claude", act: "pick" }, { text: "Codex", sub: "ChatGPT plan", checked: s.who === "codex", act: "pick" }, { note: "Switching starts a fresh conversation. This one is kept." }], { role: "menu", label: "Agent" });
    if (s.overlay === "usage") pop = C.popover(`${a.name} · ${a.plan}`, D.usage.map((u) => `<div class="meter-row"><span>${esc(u.label)}</span>${C.meter(u.label, u.pct)}<span class="num">${u.pct}%</span><span class="small">resets in ${esc(u.reset)}</span></div>`).join("") + `<p class="small">Codex · Using ChatGPT plan · no usage report yet. Updated 8m ago.</p>`);
    return `<section class="dock" data-c="dock" data-state="${open ? "open" : "closed"}" aria-label="Chat">${open ? C.resizable("horizontal", "Resize the chat history", { value: 58, min: 20, max: 80 }) : ""}${top}${mid}${C.composer(s)}${pop ? `<div class="anchored">${pop}</div>` : ""}</section>`;
  }

  function about() {
    return C.dialog("About Emmet Builder", `<img class="about-icon" src="${window.EMMET_ROOT}icon/app-icon-macos.svg" alt=""><img class="about-wordmark" src="${window.EMMET_ROOT}brand/wordmark-emmet-builder-ant-${window.EMMET_THEME}.svg" alt="Emmet Builder">
      <p>Version ${D.versions.emmet} · LDraw library ${D.versions.library} · Parts data ${D.versions.partsData}</p>
      <p>Free and open source under the AGPL-3.0. Your models and chats stay on this computer; Emmet has no accounts and no tracking.</p>
      <p class="small">Emmet connects only to: the LDraw library and parts data downloads, its own update check, and the agent you pick, which talks to Anthropic or OpenAI on your plan.</p>
      <p class="small">LEGO® is a trademark of the LEGO Group, which does not sponsor, authorize, or endorse this software. LDraw is a trademark of the Estate of James Jessiman. Claude is a trademark of Anthropic, and Codex and ChatGPT of OpenAI; Emmet isn't affiliated with or endorsed by them.</p>
      <div class="dialog-actions">${button("Licenses", { variant: "quiet", size: "sm" })}${button("Close", { size: "sm", act: "close" })}</div>`, { cls: "about" });
  }
  function settingsDialog(ctx) {
    const row = (label, control) => `<div class="setting" data-c="setting" data-inline><span class="setting-name">${esc(label)}</span>${control}</div>`;
    const folder = ctx.platform === "windows" ? "Documents\\Emmet" : "Documents/Emmet";
    return C.dialog("Settings", `<p class="dialog-title">Settings</p>
      <section class="panel-section"><div class="panel-section-head">Agent</div>${row("Start with", C.select("Start with", "Claude", { hideLabel: true }))}${row("Model and effort", C.select("Model and effort", "Opus 5.5 · Medium", { hideLabel: true }))}</section>
      <section class="panel-section"><div class="panel-section-head">Projects</div>${row(`New projects go in ${folder}`, button("Change…", { size: "sm" }))}</section>
      <section class="panel-section"><div class="panel-section-head">Look</div>${row("Appearance", C.segmented("Appearance", ["System", "Light", "Dark"], "System"))}${row("Color names", C.segmented("Color names", ["LDraw", "BrickLink"], "LDraw"))}${row("Viewer quality", C.segmented("Viewer quality", ["Fast", "Balanced", "Best"], "Balanced"))}</section>
      <section class="panel-section"><div class="panel-section-head">Emmet</div>${row("Updates", `<span class="small">Checked today · 0.1.0 is the latest</span>`)}${row("Command line tool", button("Install", { size: "sm", icon: "terminal" }))}${row("Diagnostics", button("Copy diagnostics", { size: "sm", icon: "copy" }))}</section>
      <div class="dialog-actions">${button("Done", { variant: "primary", size: "sm", act: "close" })}</div>`);
  }

  function workspace(s, ctx) {
    const win = ctx.platform === "windows";
    const main = `<div class="app-main">${win ? "" : titlebar(s, ctx)}${s.banner ? C.banner(s.banner) : ""}${viewer(s)}${dock(s, ctx)}</div>`;
    const modal = s.overlay === "about" ? about() : s.overlay === "settings" ? settingsDialog(ctx) : "";
    return `<div class="app" data-c="app">${win ? titlebar(s, ctx) : ""}${main}${C.resizable("vertical", "Resize the options panel", { value: 384, min: 320, max: 520 })}${panel(s, ctx)}${modal}</div>`;
  }

  // First run: one big card over the model.
  const BACKDROP = { welcome: "cottage", library: "stage1", agents: "stage2", start: "empty" };
  function firstRun(s, ctx) {
    const steps = [["welcome", "Welcome"], ["library", "Parts and Python"], ["agents", "Your agent"], ["start", "Start"]];
    const at = steps.findIndex(([id]) => id === s.step);
    const rail = `<ol class="rail" aria-label="Setup">${steps.map(([id, label], i) => `<li${C.attrs({ "aria-current": i === at ? "step" : undefined, "data-state": i < at ? "done" : i === at ? "current" : "later" })}>${i < at ? icon("check", "sm") : `<b>${i + 1}</b>`}${label}</li>`).join("")}</ol>`;
    let body;
    if (s.step === "welcome") {
      body = `<h1>Build brick models by describing them</h1><p class="lede">Emmet Builder works with your own Claude Code or Codex. You say what to build; your agent writes a small Python model from real LDraw parts, and Emmet shows it, checks it, and gives you a parts list and building steps.</p>
        <ul class="feature-list"><li>${icon("brick", "lg")}Real parts, in colors they're actually made in</li><li>${icon("circle-check", "lg")}Checks for parts that collide or float</li><li>${icon("sliders-horizontal", "lg")}Pick a style, scale, and size instead of placing 400 bricks</li></ul>
        <div class="row">${button("Get started", { variant: "primary", size: "lg", act: "primary" })}</div><p class="small">Your models and chats stay on this computer. Emmet has no accounts and no tracking.</p>`;
    } else if (s.step === "library") {
      const lib = s.failed ? C.progress(`LDraw parts library ${D.versions.library}`, 64, "Stopped at 96 of 150 MB", { error: true }) : C.progress(`LDraw parts library ${D.versions.library}`, 64, "96 of 150 MB");
      const err = s.failed ? C.alertCard({ tone: "bad", icon: "wifi-off", title: "The download stopped", text: "The connection dropped at 96 of 150 MB. Emmet picks up where it stopped.", actions: button("Try again", { variant: "primary", icon: "refresh-cw", act: "retry" }) }) : "";
      body = `<h1>Getting the parts ready</h1><p class="lede">Downloading the LDraw parts library (about 150 MB) and setting up Python. This takes a minute or two, and only happens once.</p>
        ${lib}${C.progress("Python for models", 0, "Next")}${err}
        ${s.failed ? "" : `<p class="small">While you wait: set up your agent on the next screen. Everything keeps downloading.</p><div class="row">${button("Continue", { variant: "primary", size: "lg", act: "primary" })}</div>`}`;
    } else if (s.step === "agents") {
      body = `<h1>Set up your agent</h1><p class="lede">Emmet builds with your own Claude Code or Codex. Set up either one here, no terminal needed.</p>
        ${C.setupCard({ ready: true, icon: "circle-check", title: `Claude Code ${D.agents.claude.version}`, text: "Signed in · Max plan", action: badge("Ready") })}
        ${C.setupCard({ icon: "download", title: "Codex", text: D.plans.codex, action: button("Install Codex", { variant: "primary" }) })}
        <p class="small">${esc(D.plans.claude)}</p>
        <div class="notice">${icon("info")}<p>Claude Code and Codex are made by Anthropic and OpenAI, not by Emmet. Your plan and its limits are between you and them, and you use them with Emmet at your own risk.</p></div>
        <div class="row">${button("Check agents again", { variant: "quiet", icon: "refresh-cw" })}<span class="grow"></span>${button("Continue", { variant: "primary", size: "lg", act: "primary" })}</div>`;
    } else {
      const folder = ctx.platform === "windows" ? "Documents\\Emmet" : "Documents/Emmet";
      body = `<h1>What do you want to build?</h1><p class="lede">New projects go in ${folder}. You can change that in Settings.</p>
        <div class="start-cards"><button type="button" class="start-card" data-act="example"><span class="viewer" data-c="viewer">${C.render("cottage")}</span><b>Start with the Cottage example</b><span class="small">163 parts: a real door, glass windows, and a room inside</span></button>
        <button type="button" class="start-card" data-act="prompt"><span class="viewer" data-c="viewer">${C.render("empty")}</span><b>New model from a prompt</b><span class="small">An empty baseplate and your agent</span></button></div>
        <p class="small">Or open a folder, or an LDraw file from Studio, LDCad, or LeoCAD.</p>`;
    }
    const caption = ctx.platform === "windows" ? C.captionButtons() : "";
    return `<div class="first-run" data-c="first-run"><section class="viewer" data-c="viewer" aria-hidden="true">${C.render(BACKDROP[s.step])}</section>${caption}
      <section class="first-run-card" aria-label="Set up Emmet"><img class="wordmark" src="${window.EMMET_ROOT}brand/wordmark-emmet-builder-ant-${window.EMMET_THEME}.svg" alt="Emmet Builder">${rail}${body}</section></div>`;
  }

  const compose = (s, ctx) => (s.kind === "first-run" ? firstRun(s, ctx) : workspace(s, ctx));
  return { screens, byId, FLOWS, STATES: S.STATES, compose };
})();
