// SPDX-License-Identifier: AGPL-3.0-or-later
// Every component in every state (components.html), light and dark side by side: the reference 4A builds each one
// to. Hover, press, and focus are drawn still with data-show; the app gets them from the pointer and the keyboard.
// The order and the Radix primitive of each follow docs/design/components.md.
(() => {
  const C = window.EMMET_C;
  const D = window.EMMET_DATA;
  const GEN = window.EMMET_GEN;
  const H = window.EMMET_H;
  const F = window.EMMET_FROZEN;
  const { button: b, iconButton, badge, icon, esc } = C;
  const weathering = D.params.find((p) => p.name === "weathering");
  const seg = D.seg;
  const screen = (id, extra = {}) => ({ ...F.byId[id], ...extra });
  // A frame for what positions itself against the window (a dialog, a toast, a pin): a small window of its own.
  const frame = (html, cls = "") => `<div class="viewer${cls}" data-c="viewer" style="height:${cls ? 220 : 260}px">${html}</div>`;
  const onRender = (html, render = "cottage") => frame(C.render(render) + html);

  const SPECS = [
    ["base", "Type, color, and focus", "none: the window's own", () => [
      ["Body", "<p>The cottage is up: 163 parts, 0 errors.</p>"], ["Muted and small", `<p class="muted">Later steps are hidden</p><p class="small">Updated 8m ago</p>`],
      ["Tones", `<p><span class="ok">No problems</span> · <span class="warn">1 problem</span> · <span class="bad">Build failed</span> · <span class="accent">Waiting for you</span></p>`],
      ["Focus ring", b("Focused", { show: "focus" })],
    ]],
    ["icon", "Icon", "lucide-react, and Emmet's glyphs", () => [
      ["Emmet glyphs", GEN.glyphs.map((g) => `<span title="${g}">${icon(g, "lg")}</span>`).join(" ")],
      ["Sizes", `${icon("brick", "sm")} ${icon("brick")} ${icon("brick", "lg")} ${icon("brick", "xl")}`],
    ]],
    ["spinner", "Spinner", "none (role status)", () => [["Working", C.spinner()]]],
    ["kbd", "Key", "none (kbd)", () => [["Keys", `${C.kbd("Esc")} ${C.kbd("⌘↩")} ${C.kbd("1")} ${C.kbd("[")} ${C.kbd("]")}`]]],
    ["button", "Button", "none (button; Slot for links)", () => ["primary", undefined, "quiet"].map((v) => [v || "secondary", ["", "hover", "press", "focus"].map((show) => b(show || "Rest", { variant: v, show: show || undefined })).join(" ") + " " + b("Disabled", { variant: v, disabled: true }) + " " + b("Busy", { variant: v, busy: true })])
      .concat([["Sizes and icons", `${b("Small", { size: "sm", icon: "save" })} ${b("Medium", { icon: "download" })} ${b("Large", { size: "lg", variant: "primary" })} ${b("Stop", { size: "sm", icon: "square", kbd: "Esc" })}`]])],
    ["icon-button", "Icon button", "none (button) + Tooltip", () => [["States", `${iconButton("paperclip", "Rest")} ${iconButton("paperclip", "Hover", { show: "hover" })} ${iconButton("scan", "On", { on: true })} ${iconButton("camera", "Focus", { show: "focus" })} ${iconButton("camera", "Disabled", { disabled: true })} ${iconButton("x", "Small", { size: "sm" })}`]]],
    ["badge", "Badge and count", "none", () => [["Kinds", `${badge("Max plan · 16%", { act: "usage" })} ${badge("Max plan · 92%", { act: "usage", tone: "warn" })} ${badge("was 0 %", { tone: "accent" })} ${badge("1", { tone: "warn" })} ${badge("163", { tone: "count" })} ${badge("Ready")}`]]],
    ["text-field", "Text field", "none (input)", () => [["Empty", C.textField({ placeholder: "Your own, like steampunk or zen garden", label: "Style" })], ["Filled", C.textField({ value: "Steeper roof", label: "Name" })], ["Focus", C.textField({ value: "Steeper roof", label: "Name", show: "focus" })], ["Invalid", C.textField({ value: "", placeholder: "A name is needed", label: "Name", invalid: true })], ["Disabled", C.textField({ value: "Read-only", label: "Name", disabled: true })]]],
    ["slider", "Slider", "Slider", () => [["Rest", C.slider(weathering, 0)], ["Moved", C.slider(weathering, 20)], ["Hover", C.slider(weathering, 20, { show: "hover" })], ["Dragging", C.slider(weathering, 20, { show: "press" })], ["Focus", C.slider(weathering, 20, { show: "focus" })], ["Disabled", C.slider(weathering, 10, { disabled: true })]]],
    ["switch", "Switch", "Switch", () => [["Off and on", `${C.switchC("Roof", false)} ${C.switchC("Roof", true)}`], ["Hover and focus", `${C.switchC("Roof", false, { show: "hover" })} ${C.switchC("Roof", true, { show: "focus" })}`], ["Disabled", `${C.switchC("Roof", false, { disabled: true })} ${C.switchC("Roof", true, { disabled: true })}`]]],
    ["segmented", "Segmented control", "ToggleGroup (single)", () => [["Rest", C.segmented("Detail", ["Any", "Low", "Medium", "High"], "Low")], ["Hover and focus", C.segmented("Base", D.brief.base.options, "32 × 32", { show: (id) => (id === "None" ? "hover" : id === "48 × 48" ? "focus" : undefined) })]]],
    ["chips", "Chips", "ToggleGroup", () => [["Nothing picked", C.chips("Scale", ["Any", ...D.brief.scale.options], null)], ["Picked", C.chips("Scale", ["Any", ...D.brief.scale.options], "Figure scale")], ["Your own words", C.chips("Style", D.brief.style.options.slice(0, 4), null, { own: ["Steampunk"] })], ["Hover and focus", C.chips("Pieces", ["Any", "50–250", "250–1,000"], "Any", { show: (id) => (id === "50–250" ? "hover" : id === "250–1,000" ? "focus" : undefined) })]]],
    ["select", "Select", "Select", () => [["Closed", C.select("Effort", "Medium")], ["Hover", C.select("Effort", "Medium", { show: "hover" })], ["Open", C.select("Effort", "Medium", { open: true }) + C.menu([{ text: "Low", checked: false }, { text: "Medium", checked: true, highlighted: true }, { text: "High", checked: false }], { role: "listbox", label: "Effort" })], ["Disabled", C.select("Model", "Opus 5.5", { disabled: true })]]],
    ["color-picker", "Color picker", "Popover + RadioGroup", () => [["Closed", C.colorPicker("Wall color", "Tan", D.colorChoices.wall_color)], ["Open", C.colorPicker("Wall color", "Tan", D.colorChoices.wall_color, { open: true })]]],
    ["tabs", "Tabs", "Tabs", () => [["With counts", C.tabs("options", { checks: 1, parts: 163, steps: 31 })], ["Another tab", C.tabs("steps", { parts: 163, steps: 31 })]]],
    ["tooltip", "Tooltip", "Tooltip", () => [["Shown", `<div style="height:44px">${C.tooltip(D.brief.scale.help, "left:0;top:0")}</div>`]]],
    ["menu", "Menu", "DropdownMenu", () => [["Actions", C.menu([{ icon: "plus", text: "New", key: "⌘N" }, { icon: "folder-open", text: "Open…", key: "⌘O", highlighted: true }, "-", { icon: "save", text: "Save version", key: "⌘S" }, { icon: "rotate-ccw-clock", text: "Versions…", disabled: true }])], ["Choices", C.menu([{ text: "Claude", sub: "Max plan", checked: true }, { text: "Codex", sub: "ChatGPT plan", checked: false }, { note: "Switching starts a fresh conversation. This one is kept." }])], ["Groups", C.menu([{ label: "Parts list" }, { icon: "sheet", text: "CSV", sub: "Every part and color, with counts" }, { label: "Model" }, { icon: "box", text: "LDraw .mpd", sub: "Opens in Studio, LDCad, and LeoCAD" }])]]],
    ["menubar", "Menu bar (Windows)", "Menubar", () => [["Closed", C.menubar()], ["File open", C.menubar("File")]]],
    ["popover", "Popover", "Popover", () => [["Usage", C.popover("Claude · Max plan", D.usage.map((u) => `<div class="meter-row"><span>${esc(u.label)}</span>${C.meter(u.label, u.pct)}<span class="num">${u.pct}%</span><span class="small">resets in ${esc(u.reset)}</span></div>`).join(""))]]],
    ["dialog", "Dialog", "Dialog", () => [["Open", frame(C.dialog("Save version", `<p class="dialog-title">Discard the change?</p><p class="muted">The weathering you set isn't kept yet.</p><div class="dialog-actions">${b("Cancel", { variant: "quiet", size: "sm" })}${b("Discard", { variant: "primary", size: "sm" })}</div>`), " h-frame")]]],
    ["toast", "Toast", "Toast", () => [["Shown", frame(C.toast("Saved “Cottage parts.csv” to Downloads", "Show in folder"), " h-frame")]]],
    ["progress", "Progress", "Progress", () => [["Loading", C.progress("LDraw parts library 2026-09", 64, "96 of 150 MB")], ["Done", C.progress("Python for models", 100, "Ready")], ["Failed", C.progress("LDraw parts library 2026-09", 64, "Stopped at 96 of 150 MB", { error: true })]]],
    ["meter", "Meter", "none (role meter)", () => [["Low", C.meter("Current session", 16)], ["From 90 %", C.meter("Current session", 92)]]],
    ["resizable", "Resizable panes", "none (role separator; Emmet's own)", () => [["Vertical: rest, hover, dragging, focus", `<div class="row" style="height:56px">${["", "hover", "dragging", "focus"].map((st) => C.resizable("vertical", "Resize", { show: st === "hover" || st === "focus" ? st : undefined, dragging: st === "dragging" })).join("")}</div>`], ["Horizontal", `<div style="width:160px">${C.resizable("horizontal", "Resize", { show: "hover" })}</div>`]]],
    ["tree", "Tree", "none (role tree; Emmet's own)", () => [["Steps with sub-assemblies", C.tree({ step: 4, expanded: "Door", hover: 5 }).replace(/(<li class="tree-group" role="presentation">Walls, row 2)[\s\S]*<\/ul>/, "</ul>")], ["A sub-assembly selected", C.tree({ selected: "Window" }).replace(/(<li class="tree-group" role="presentation">Start)[\s\S]*<\/ul>/, "</ul>")]]],
    ["data-table", "Data table", "none (table)", () => [["Parts list: a row hovered, a row selected", C.dataTable({ limit: 4, hover: 1, selected: 2 })]]],
    ["chat-message", "Chat message", "none; Collapsible for thinking", () => [["The user's", C.message(seg.prompt[0])], ["Sent from Checks", C.message(seg.fixAsk[0])], ["Delivered mid-turn", C.message(seg.delivered[0])], ["The agent's", C.message(seg.finish[3])], ["Thinking", C.message(seg.thinking[0])], ["A note", C.message(seg.stopped[0])], ["A divider", C.message({ t: "divider", text: "Codex · a fresh conversation. The Claude one is kept." })], ["Pending redirect", C.pending(seg.pending[0].text, "Claude")]]],
    ["tool-card", "Tool card", "Collapsible", () => [["Running", C.toolCard(seg.stage1Running[0])], ["Done", C.toolCard(seg.stage1[0])], ["Open", C.toolCard(seg.stage1[1], { open: true })], ["Failed", C.toolCard(seg.flipEdit[1])], ["A change, in words", C.fileCard(seg.start[2])], ["Folded run", C.chat({ chat: story(seg.start, seg.stage1, seg.ack) }).replace(/<div class="message"[\s\S]*$/, "")]]],
    ["question-card", "Question card", "none (group); keys 1–5", () => [["Waiting", C.questionCard(seg.question[0])], ["Answered", C.questionCard(seg.answered[0])]]],
    ["approval-card", "Approval card", "none (group); keys 1–3", () => [["Waiting", C.approvalCard(seg.approval[0], {})]]],
    ["alert-card", "Alert card", "none (role alert or status)", () => [["Build failed", C.alertCard({ tone: "bad", icon: "octagon-alert", title: "The model didn't build", text: D.buildError.plain, actions: b("Ask agent to fix", { variant: "primary" }), foot: "The last good model is still shown." })], ["Crashed", C.message(seg.crashed[0])], ["Plan limit", C.alertCard({ icon: "gauge", title: "You've used Claude's plan limit for now", text: "It resets in 2h 14m. Your model and this conversation are kept.", actions: b("Continue with Codex", { variant: "primary" }) + b("Wait for the reset", { variant: "quiet" }) })]]],
    ["finding-row", "Finding row", "none (option in a listbox)", () => [["A finding", C.finding(screen("findings", { selected: false }))], ["Selected", C.finding(screen("findings"))], ["Being fixed", C.finding(screen("fixing"))], ["Allowed", C.finding(screen("findings", { selected: false }), { allowed: true })], ["No problems", C.finding(screen("built"))]]],
    ["banner", "Banner", "none (role status)", () => ["untrusted", "offline", "old-library", "imported"].map((k) => [k, C.banner(k)])],
    ["status", "Agent status", "none (role status, polite)", () => [["Working", C.status(screen("building"))], ["A question", C.status(screen("question"))], ["An approval", C.status(screen("approval"))], ["Crashed", C.status(screen("crashed"))], ["Stopped", C.status(screen("stopped"))], ["Done", C.status(screen("built"))]]],
    ["composer", "Composer", "none (textarea)", () => [["Idle", C.composer(screen("built"))], ["Working: a redirect", C.composer(screen("building"))], ["Offline", C.composer(screen("offline"))], ["Written", C.composer(screen("new-empty"))]]],
    ["parts-box", "Parts box and step", "none", () => [["A step", onRender(C.stepView(9), "step-9")], ["With a sub-assembly", onRender(C.stepView(4), "step-4")]]],
    ["pin", "Pin", "none (button)", () => [["On its parts", onRender(C.pin(`${D.finding.parts} parts float`, { u: 0.541, v: 0.344 }, D.renders.finding), "finding")]]],
    ["setup-card", "Setup card", "none", () => [["Ready", C.setupCard({ ready: true, icon: "circle-check", title: "Claude Code 2.1.310", text: "Signed in · Max plan", action: badge("Ready") })], ["Needs installing", C.setupCard({ icon: "download", title: "Codex", text: D.plans.codex, action: b("Install Codex", { variant: "primary" }) })], ["Installing", C.setupCard({ icon: "download", title: "Codex", text: "Installing with OpenAI's installer…", action: b("Installing", { busy: true, disabled: true }) })]]],
    ["version-row", "Version row", "none (menuitem in a list)", () => [["Yours", C.versionRow(D.history[1])], ["Saved by Emmet", C.versionRow(D.history[0])], ["Hover", C.versionRow(D.history[3], { show: "hover" })]]],
    ["view-switch", "View switch", "ToggleGroup + icon buttons", () => [["Iso", C.viewSwitch("Iso")], ["Top", C.viewSwitch("Top")]]],
    ["caption-buttons", "Caption buttons (Windows)", "none (Tauri window controls)", () => [["Rest", `<div style="position:relative;height:48px">${C.captionButtons()}</div>`]]],
  ];
  const story = (...p) => p.flat();
  const REGIONS = ["app", "titlebar", "viewer", "dock", "panel", "setting", "first-run", "about"];

  const theme = (t, rows) => `<div class="window" data-theme="${t}" data-platform="macos">${rows.map(([label, html]) => `<div class="h-state"><span>${esc(label)}</span><div>${html}</div></div>`).join("")}</div>`;
  const tokens = (id) => (GEN.tokens[id] || []).map((t) => `<code>${t}</code>`).join(" ");
  const body = SPECS.map(([id, name, radix, states]) => {
    const html = {};
    for (const t of ["light", "dark"]) { window.EMMET_THEME = t; html[t] = theme(t, states()); }
    return `<section class="h-spec" id="${id}"><div class="h-spec-head"><h2>${esc(name)}</h2><a href="../components.md#${id}">${id}</a><span class="h-muted">Radix: ${esc(radix)}</span></div>
      <p class="h-spec-tokens">${tokens(id)}</p><div class="h-pair">${html.light}${html.dark}</div></section>`;
  }).join("");
  const regions = REGIONS.map((id) => `<tr><td><a href="../components.md#${id}">${id}</a></td><td class="h-spec-tokens">${tokens(id)}</td></tr>`).join("");
  document.body.innerHTML = `<main class="h-page"><h1>Emmet, frozen: every component in every state</h1>
    <p>Each component in <a href="../components.md">the inventory</a>, light and dark, with the tokens its section of <a href="components.css">components.css</a> uses.
    Hover, press, and focus are drawn still here; in the app they come from the pointer and the keyboard. <a href="index.html">All screens</a>.</p>
    ${body}<h2>Regions</h2><p class="h-muted">The window's regions are the screens themselves; each is described in the inventory.</p><table class="h-table"><tbody>${regions}</tbody></table></main>`;
  for (const w of document.querySelectorAll(".window")) H.settle(w);
})();
