// SPDX-License-Identifier: AGPL-3.0-or-later
// Emmet's components, frozen (plan session 2B), as HTML builders: one per entry in docs/design/components.md, styled
// only by components.css. Each root carries data-c="<id>" (the screen's Uses list and the audit read it), and its
// markup follows the Radix primitive 4A will use: the same roles and data-state values. Content comes from 2A's
// real data (docs/design/shared/data.js); no colors here except a part's own, which data.js holds.
window.EMMET_C = (() => {
  const D = window.EMMET_DATA;
  const GEN = window.EMMET_GEN;
  const esc = (s) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);
  const md = (s) => esc(s).replace(/\*\*([^*]+)\*\*/g, "<b>$1</b>");
  const attrs = (o) => Object.entries(o).filter(([, v]) => v !== undefined && v !== null && v !== false).map(([k, v]) => (v === true ? ` ${k}` : ` ${k}="${esc(v)}"`)).join("");
  const ROOT = () => window.EMMET_ROOT || "../";
  const theme = () => window.EMMET_THEME || "light";

  // An icon by its lucide name, or an Emmet glyph (apps/desktop/src/assets/glyphs). A name the generator didn't find
  // draws nothing and is marked, so the audit reports it.
  function icon(name, size) {
    const body = GEN.icons[name];
    return `<svg class="icon" data-c="icon"${attrs({ "data-size": size, "data-missing": body ? undefined : name })} viewBox="0 0 24 24" aria-hidden="true">${body || ""}</svg>`;
  }
  // 2A's tool cards name their icons by role; these are the lucide icons for those roles.
  const TOOL_ICON = { book: "book-open", search: "search", cube: "box", check: "circle-check", eye: "eye", file: "file-pen" };

  const spinner = (label = "Working") => `<span class="spinner" data-c="spinner" role="status" aria-label="${esc(label)}"></span>`;
  const kbd = (k) => `<kbd class="kbd" data-c="kbd">${esc(k)}</kbd>`;

  // A value with its unit, as content.md writes it: "20%", "150 MB", "6 studs".
  const withUnit = (v, unit) => (!unit ? String(v) : unit === "%" ? `${v}%` : `${v} ${unit}`);

  // A button. o: variant (primary, quiet), size (sm, lg), icon, act (the flow it moves), local (a view toggle),
  // disabled, busy, title, show (hover, press, focus: the specimens' still states).
  function button(label, o = {}) {
    const lead = o.busy ? spinner(label) : o.icon ? icon(o.icon) : "";
    return `<button type="button" class="button${o.cls ? " " + o.cls : ""}" data-c="button"${attrs({ "data-variant": o.variant, "data-size": o.size, "data-compact": o.compact, "data-act": o.act, "data-local": o.local, "data-busy": o.busy, "data-show": o.show, disabled: o.disabled, title: o.title || (o.compact ? label : undefined), "aria-haspopup": o.popup, "aria-expanded": o.expanded })}>${lead}<span>${esc(label)}</span>${o.kbd ? kbd(o.kbd) : ""}${o.chevron ? icon("chevron-down", "sm") : ""}</button>`;
  }
  const iconButton = (name, label, o = {}) => `<button type="button" class="icon-button" data-c="icon-button" aria-label="${esc(label)}" title="${esc(label)}"${attrs({ "data-size": o.size, "data-act": o.act, "data-state": o.on ? "on" : undefined, "data-show": o.show, disabled: o.disabled })}>${icon(name)}</button>`;
  const badge = (text, o = {}) => o.act || o.local
    ? `<button type="button" class="badge" data-c="badge"${attrs({ "data-tone": o.tone, "data-act": o.act, "data-local": o.local, title: o.title })}>${o.icon ? icon(o.icon, "sm") : ""}${esc(text)}</button>`
    : `<span class="badge" data-c="badge"${attrs({ "data-tone": o.tone, title: o.title })}>${o.icon ? icon(o.icon, "sm") : ""}${esc(text)}</span>`;
  const textField = (o = {}) => `<input class="text-field" data-c="text-field" type="text"${attrs({ value: o.value, placeholder: o.placeholder, "aria-label": o.label, disabled: o.disabled, "aria-invalid": o.invalid, "data-show": o.show })}>`;

  // Radix Slider: Root > Track > Range, and a Thumb with role slider. The value prints beside it.
  function slider(p, value, o = {}) {
    const pct = ((value - p.min) / (p.max - p.min)) * 100;
    return `<div class="slider" data-c="slider"${attrs({ "data-disabled": o.disabled })}><span class="slider-root" data-orientation="horizontal"><span class="slider-track"><span class="slider-range" style="width:${pct}%"></span></span>
      <span class="slider-thumb" role="slider" tabindex="0" aria-label="${esc(p.label)}" aria-valuemin="${p.min}" aria-valuemax="${p.max}" aria-valuenow="${value}" aria-valuetext="${esc(withUnit(value, p.unit))}" style="left:${pct}%"${attrs({ "data-show": o.show, "data-act": o.act })}></span></span><output>${esc(withUnit(value, p.unit))}</output></div>`;
  }
  // Radix Switch: a button with role switch and data-state checked or unchecked.
  const switchC = (label, on, o = {}) => `<button type="button" class="switch" data-c="switch" role="switch" aria-checked="${!!on}" aria-label="${esc(label)}" data-state="${on ? "checked" : "unchecked"}"${attrs({ "data-disabled": o.disabled, disabled: o.disabled, "data-show": o.show })}><span class="switch-thumb" data-state="${on ? "checked" : "unchecked"}"></span></button>`;
  // Radix ToggleGroup (type single): role radio items, data-state on or off.
  function segmented(label, items, cur, o = {}) {
    return `<div class="segmented" data-c="segmented" role="radiogroup" aria-label="${esc(label)}">${items.map((it) => {
      const id = typeof it === "string" ? it : it.id;
      return `<button type="button" class="segmented-item" role="radio" aria-checked="${id === cur}" data-state="${id === cur ? "on" : "off"}"${attrs({ title: it.tip, "data-local": o.local?.(id), "data-show": o.show?.(id) })}>${esc(it.name ?? id)}</button>`;
    }).join("")}</div>`;
  }
  // Chips: a ToggleGroup that wraps. o.own: the user's own words, each removable.
  function chips(label, items, cur, o = {}) {
    const own = (o.own || []).map((w) => `<button type="button" class="chip" role="${o.role || "radio"}" aria-checked="true" data-state="on" title="Your own style">${esc(w)}<span class="chip-remove" aria-label="Remove ${esc(w)}">${icon("x", "sm")}</span></button>`).join("");
    return `<div class="chips" data-c="chips" role="${o.role === "checkbox" ? "group" : "radiogroup"}" aria-label="${esc(label)}">${items.map((it) => {
      const id = typeof it === "string" ? it : it.id;
      return `<button type="button" class="chip" role="${o.role || "radio"}" aria-checked="${id === cur}" data-state="${id === cur ? "on" : "off"}"${attrs({ title: it.sub, "data-local": o.local?.(id), "data-show": o.show?.(id) })}>${esc(it.name ?? id)}</button>`;
    }).join("")}${own}</div>`;
  }
  // Radix Select: the trigger shows the choice; open, its content is a listbox (drawn by menu()).
  function select(label, value, o = {}) {
    return `<label class="select" data-c="select"><span class="${o.hideLabel ? "visually-hidden" : "select-label"}">${esc(label)}</span><button type="button" class="select-trigger" role="combobox" aria-expanded="${!!o.open}" data-state="${o.open ? "open" : "closed"}"${attrs({ "data-disabled": o.disabled, "data-show": o.show })}><span>${esc(value)}</span>${icon("chevron-down", "sm")}</button></label>`;
  }
  // A color knob: the swatch and the color's name; open, a radio grid of the colors this part is made in.
  function colorPicker(label, value, choices, o = {}) {
    const sw = (c) => `<i class="swatch" style="background:${D.colors[c]}" aria-hidden="true"></i>`;
    const grid = o.open ? `<div class="popover" data-c="popover" role="dialog" aria-label="${esc(label)}"><span class="popover-title">${esc(label)}</span><div class="swatch-grid" role="radiogroup" aria-label="${esc(label)}">${choices.map((c) => `<button type="button" class="swatch-item" role="radio" aria-checked="${c === value}" data-state="${c === value ? "checked" : "unchecked"}" title="${esc(c)}">${sw(c)}</button>`).join("")}</div><p class="small">Only colors this part is made in.</p></div>` : "";
    return `<span class="color-picker" data-c="color-picker"><button type="button" class="select-trigger" aria-haspopup="dialog" aria-expanded="${!!o.open}" data-state="${o.open ? "open" : "closed"}">${sw(value)}<span>${esc(value)}</span>${icon("chevron-down", "sm")}</button>${grid}</span>`;
  }
  // Radix Tabs: a tablist; each trigger data-state active or inactive, with its count.
  const TABS = [["options", "Options"], ["checks", "Checks"], ["parts", "Parts"], ["steps", "Steps"]];
  function tabs(active, counts = {}) {
    return `<div class="tabs" data-c="tabs" role="tablist" aria-label="Options panel">${TABS.map(([id, label]) => `<button type="button" class="tab" role="tab" data-tab="${id}" aria-selected="${id === active}" data-state="${id === active ? "active" : "inactive"}">${esc(label)}${counts[id] ? badge(String(counts[id]), { tone: id === "checks" ? "warn" : "count" }) : ""}</button>`).join("")}</div>`;
  }
  // Radix Tooltip: role tooltip, shown on hover or focus after a pause. `at` places it (left and top, in a style
  // the screen gives, since it sits next to what it explains).
  const tooltip = (text, style) => `<div class="tooltip" data-c="tooltip" role="tooltip" style="${style}">${esc(text)}</div>`;

  // Radix DropdownMenu content: items with an icon, a label, and a key; separators; group labels; a note.
  function menu(items, o = {}) {
    const row = (it) => {
      if (it === "-") return `<div class="menu-separator" role="separator"></div>`;
      if (it.label) return `<div class="menu-label">${esc(it.label)}</div>`;
      if (it.note) return `<p class="menu-note">${esc(it.note)}</p>`;
      const role = it.checked !== undefined ? "menuitemradio" : "menuitem";
      return `<button type="button" class="menu-item" role="${role}"${attrs({ "aria-checked": it.checked !== undefined ? String(!!it.checked) : undefined, "data-state": it.checked ? "checked" : undefined, "data-act": it.act, "data-local": it.local, "data-disabled": it.disabled, "data-highlighted": it.highlighted })}>${it.icon ? icon(it.icon) : ""}<span>${esc(it.text)}</span>${it.sub ? `<span class="small">${esc(it.sub)}</span>` : ""}${it.key ? `<span class="menu-key">${esc(it.key)}</span>` : ""}${it.checked ? icon("check", "sm") : ""}</button>`;
    };
    return `<div class="menu" data-c="menu" role="${o.role || "menu"}"${attrs({ "aria-label": o.label, style: o.style })}>${items.map(row).join("")}</div>`;
  }
  const menubar = (open) => `<div class="menubar" data-c="menubar" role="menubar" aria-label="Menu">${["File", "Edit", "View", "Agent", "Help"].map((m) => `<button type="button" class="menubar-trigger" role="menuitem" aria-haspopup="menu" data-state="${m === open ? "open" : "closed"}" aria-expanded="${m === open}">${m}</button>`).join("")}</div>`;
  const popover = (label, body, o = {}) => `<div class="popover" data-c="popover" role="dialog" aria-label="${esc(label)}"${attrs({ style: o.style })}>${o.title === false ? "" : `<span class="popover-title">${esc(label)}</span>`}${body}</div>`;
  const dialog = (label, body, o = {}) => `<div class="dialog-overlay" data-c="dialog"><div class="dialog${o.cls ? " " + o.cls : ""}" role="dialog" aria-modal="true" aria-label="${esc(label)}">${body}</div></div>`;
  const toast = (text, action) => `<ol class="toast-viewport" aria-label="Notifications"><li class="toast" data-c="toast" role="status">${icon("circle-check")}<span class="toast-text">${esc(text)}</span>${action ? button(action, { size: "sm", variant: "quiet" }) : ""}${iconButton("x", "Dismiss", { size: "sm" })}</li></ol>`;
  // Radix Progress: role progressbar with the value, the label, and how much in words.
  const progress = (label, pct, amount, o = {}) => `<div class="progress" data-c="progress"${attrs({ "data-state": o.error ? "error" : pct >= 100 ? "complete" : "loading" })}><span>${esc(label)}</span><span class="small num">${esc(amount)}</span><span class="progress-track" role="progressbar" aria-label="${esc(label)}" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${pct}"><i class="progress-fill" style="width:${pct}%"></i></span></div>`;
  const meter = (label, pct, o = {}) => `<span class="meter" data-c="meter" role="meter" aria-label="${esc(label)}" aria-valuemin="0" aria-valuemax="100" aria-valuenow="${pct}"${attrs({ "data-tone": pct >= 90 ? "warn" : undefined })}><i class="meter-fill" style="width:${Math.max(pct, 2)}%"></i></span>`;
  // A separator between panes: role separator, focusable, with its value; the arrows move it.
  const resizable = (orientation, label, o = {}) => `<div class="resizable" data-c="resizable" role="separator" tabindex="0" aria-orientation="${orientation}" aria-label="${esc(label)}" aria-valuenow="${o.value ?? 50}" aria-valuemin="${o.min ?? 0}" aria-valuemax="${o.max ?? 100}"${attrs({ "data-state": o.dragging ? "dragging" : undefined, "data-show": o.show })}></div>`;

  // Each line's part in its color, rendered by `emmet render` on a one-part model (2A).
  const partPic = (part, color) => `${ROOT()}shared/parts/${part}-${color.toLowerCase().replace(/[^a-z0-9]+/g, "-")}.webp`;
  const partName = (part) => D.bom.find((l) => l.part === part)?.name || part;

  // The parts list: the picture, the name with its color under it, and the count; sorted by count.
  function dataTable(o = {}) {
    const rows = D.bom.slice(0, o.limit || D.bom.length).map((l, i) => `<tr${attrs({ "aria-selected": o.selected === i ? "true" : undefined, "data-show": o.hover === i ? "hover" : undefined })}><td class="col-pic"><img src="${partPic(l.part, l.color)}" alt=""></td><td title="${esc(l.name)}, ${esc(l.color)}"><span class="cell-name">${esc(l.name)}</span><span class="cell-sub">${esc(l.color)} · ${esc(l.part)}</span></td><td class="col-qty num">${l.qty}</td></tr>`).join("");
    return `<table class="data-table" data-c="data-table" aria-label="Parts list"><thead><tr><th class="col-pic"><span class="visually-hidden">Picture</span></th><th>Part</th><th class="col-qty num" aria-sort="descending">Qty</th></tr></thead><tbody>${rows}</tbody></table>
      <p class="table-foot">Part, color, and set data from Rebrickable (rebrickable.com). Availability arrives with the parts data pack.</p>`;
  }

  // The Steps tab's tree (Barry 2026-10-27, session 2B): sub-assemblies first, each with how many times it's used and
  // its own sub-steps; then the steps, grouped by their titles' first words. Clicking a step shows the build up to
  // it; clicking a sub-assembly highlights every copy.
  function tree(o = {}) {
    const cur = o.step || D.steps.length;
    const subs = (window.EMMET_FROZEN_DATA?.submodels || []).map((s) => {
      const open = o.expanded === s.name;
      const sel = o.selected === s.name;
      const kids = open ? s.steps.map((t, i) => `<li class="tree-item" role="treeitem" aria-level="2" aria-selected="false"><span class="tree-n">${i + 1}</span><span class="tree-name">${esc(t)}</span></li>`).join("") : "";
      return `<li class="tree-item" role="treeitem" aria-level="1" aria-expanded="${open}" aria-selected="${sel}" data-act="sub-${s.id}"><span class="tree-twisty">${icon("chevron-down", "sm")}</span>${icon("submodel")}<span class="tree-name">${esc(s.name)}</span>${badge(`× ${s.uses}`, { tone: "count" })}<span class="small">${s.parts} parts</span></li>${kids}`;
    }).join("");
    let head = null;
    const steps = D.steps.map((s) => {
      const [group, name] = s.name.includes(": ") ? s.name.split(": ") : [s.name === "Baseplate" ? "Start" : s.group, s.name];
      const top = group !== head ? `<li class="tree-group" role="presentation">${esc(group)}</li>` : "";
      head = group;
      const state = s.n < cur ? "past" : s.n === cur ? "current" : "later";
      return `${top}<li class="tree-item" role="treeitem" aria-level="1" aria-selected="${state === "current" && !o.selected}" data-step="${state}" data-act="step-${s.n}"${attrs({ "data-show": o.hover === s.n ? "hover" : undefined })}><span class="tree-n">${s.n}</span><span class="tree-name">${esc(name)}</span><span class="small">${s.parts} part${s.parts > 1 ? "s" : ""}</span></li>`;
    }).join("");
    return `<ul class="tree" data-c="tree" role="tree" aria-label="Sub-assemblies and steps">${subs ? `<li class="tree-group" role="presentation">Built apart</li>${subs}` : ""}${steps}</ul>`;
  }

  // Chat messages.
  function message(m, screen = {}) {
    const who = agentOf(screen);
    switch (m.t) {
      case "user": {
        const from = m.auto ? `<span class="message-meta">${icon(m.auto === "settings" ? "sliders-horizontal" : "triangle-alert", "sm")}Sent from ${m.auto === "settings" ? "Build settings" : "Checks"}</span>` : "";
        return `<div class="message" data-c="chat-message" data-from="user">${from}<p>${md(m.text)}</p>${m.delivered ? `<span class="message-meta">${icon("check", "sm")}Delivered at ${esc(who.name)}'s next step</span>` : ""}</div>`;
      }
      case "agent": return `<div class="message" data-c="chat-message" data-from="agent">${m.text.split("\n\n").map((p) => `<p>${md(p)}</p>`).join("")}</div>`;
      case "thinking": return `<details class="message thinking" data-c="chat-message" data-from="thinking"><summary>Thinking ${icon("chevron-down", "sm")}</summary><p>${esc(m.text)}</p></details>`;
      case "note": return `<p class="message" data-c="chat-message" data-from="note">${esc(m.text)}</p>`;
      case "divider": return `<div class="message" data-c="chat-message" data-from="divider" role="separator"><span>${esc(m.text)}</span></div>`;
      case "tool": return toolCard(m);
      case "file": return fileCard(m);
      case "question": return questionCard(m);
      case "approval": return approvalCard(m, screen);
      case "crash": return alertCard({ tone: "bad", icon: "octagon-alert", title: m.title, text: m.text, actions: button(`Restart ${who.cli}`, { variant: "primary", icon: "refresh-cw", act: "restart" }) + button("Copy the details", { variant: "quiet", icon: "copy" }) });
      default: return "";
    }
  }
  // A pending redirect: what you sent while the agent works, until its next step picks it up.
  const pending = (text, who) => `<div class="message" data-c="chat-message" data-from="pending"><p>${esc(text)}</p><span class="message-meta">${icon("clock", "sm")}Goes to ${esc(who)} at its next step</span>
    <div class="row">${button("Edit", { size: "sm", variant: "quiet" })}${button("Cancel", { size: "sm", variant: "quiet", act: "cancel" })}${button("Stop and send now", { size: "sm", act: "deliver" })}</div></div>`;

  // Radix Collapsible: the summary in words; open, the raw call (the only place a tool's own words show).
  function toolCard(m, o = {}) {
    const end = m.status === "running" ? spinner(m.title) : m.status === "error" ? icon("x", "sm") : icon("check", "sm");
    return `<details class="tool-card" data-c="tool-card" data-status="${m.status}"${attrs({ open: o.open })}><summary>${icon(TOOL_ICON[m.icon] || m.icon)}<span class="tool-card-title">${esc(m.title)}</span><span class="tool-card-detail">${esc(m.detail)}</span><span class="${m.status === "error" ? "bad" : m.status === "done" ? "ok" : "accent"}">${end}</span></summary><pre class="tool-card-raw">${esc(m.raw)}</pre></details>`;
  }
  // A change to the model, in words: no diff, no file name, no code (law 2).
  const fileCard = (m) => `<div class="tool-card" data-c="tool-card" data-status="done"><div class="tool-card-row">${icon("file-pen")}<span class="tool-card-title">${esc(m.summary)}</span><span class="tool-card-detail">${esc(m.detail || "")}</span><span class="ok">${icon("check", "sm")}</span></div></div>`;
  // Three or more finished tool cards fold into one line once the turn moves past them (note 2).
  function chat(screen) {
    const list = screen.chat || [];
    const out = [];
    for (let i = 0; i < list.length;) {
      let j = i;
      while (j < list.length && ["tool", "file"].includes(list[j].t) && list[j].status !== "running") j++;
      const run = list.slice(i, j);
      if (run.length >= 3 && j < list.length && !["tool", "file"].includes(list[j].t)) {
        const built = run.filter((m) => /^Built \d+/.test(m.title || "")).length;
        out.push(`<details class="tool-run" data-c="tool-card"><summary>${icon("circle-check")}<span>${run.length} steps${built ? ` · built in ${built} stage${built > 1 ? "s" : ""}` : ""}</span>${icon("chevron-down", "sm")}</summary>${run.map((m) => message(m, screen)).join("")}</details>`);
        i = j;
      } else if (j > i) { out.push(...run.map((m) => message(m, screen))); i = j; }
      else { out.push(message(list[i], screen)); i++; }
    }
    return out.join("");
  }

  function questionCard(m) {
    if (m.answered) return `<div class="question-done" data-c="question-card">${icon("check", "sm")}<span class="muted">${esc(m.question)}</span><b>${esc(m.answered)}</b></div>`;
    const opts = m.options.map((o, i) => `<button type="button" class="question-option" data-act="answer">${kbd(String(i + 1))}<span><b>${esc(o.label)}</b>${o.description ? `<small>${esc(o.description)}</small>` : ""}</span></button>`).join("");
    return `<div class="question-card" data-c="question-card" role="group" aria-label="${esc(m.question)}"><div class="card-top">${badge(m.header)}</div><p class="card-title">${esc(m.question)}</p>${opts}
      <label class="question-other">${kbd(String(m.options.length + 1))}${textField({ placeholder: "Other: your own answer", label: "Other: your own answer" })}</label></div>`;
  }
  function approvalCard(m, screen) {
    return `<div class="approval-card" data-c="approval-card" role="group" aria-label="${esc(m.title)}"><div class="card-top">${badge(m.tool)}</div><p class="card-title">${esc(m.title)}</p><code class="approval-what">${esc(m.command)}</code><p class="muted">${esc(m.why)}</p>
      <div class="row">${button("Allow once", { variant: "primary", act: "allow", kbd: "1" })}${button("Allow for this session", { act: "allow", kbd: "2" })}${button("Don't allow", { variant: "quiet", act: "deny", kbd: "3" })}</div></div>`;
  }
  // A card in the chat that needs you, or says something went wrong: plain words and the one action that helps.
  const alertCard = (o) => `<div class="alert-card" data-c="alert-card" role="${o.tone === "bad" ? "alert" : "status"}"${attrs({ "data-tone": o.tone })}>${icon(o.icon || "info", "lg")}<div class="alert-card-body"><b>${esc(o.title)}</b><p>${esc(o.text)}</p>${o.actions ? `<div class="row">${o.actions}</div>` : ""}${o.foot ? `<p class="small">${esc(o.foot)}</p>` : ""}</div></div>`;

  // Checks: every finding one click from its parts (law 2); its source line and hint go to the agent, not the user.
  function finding(screen, o = {}) {
    const b = screen.build || {};
    if (b.state !== "findings") return `<div class="checks-clean" data-c="finding-row">${icon("circle-check", "lg")}<div><b>No problems</b><p class="muted">Nothing collides, nothing floats, and every part is current.</p></div></div>`;
    const f = D.finding;
    const fixing = screen.agent?.state === "working";
    const allowed = o.allowed;
    return `<div class="finding" data-c="finding-row" aria-selected="${!!screen.selected}"${attrs({ "data-state": allowed ? "allowed" : undefined, "data-act": "select" })}><div class="finding-top">${icon("floating")}<span>${f.parts} parts aren't held by anything</span>${badge(allowed ? "Allowed" : "Floating")}</div><p>${esc(f.plain)}</p>
      <div class="row">${allowed ? button("Check it again", { size: "sm", variant: "quiet" }) : fixing ? `<span class="fixing">${spinner()}${esc(agentOf(screen).name)} is fixing this</span>` : button("Ask agent to fix", { variant: "primary", act: "fix" }) + button("It's fine", { variant: "quiet", title: "Allow it: the check won't flag these parts again" })}</div></div>`;
  }

  // Banners across the viewer's top: read-only, offline, an old library, an imported file. One line and one action.
  function banner(kind) {
    const B = {
      untrusted: ["lock", "Read-only: " + D.untrusted.message, D.untrusted.hint, button("Trust this project", { variant: "primary", size: "sm", act: "trust" })],
      offline: ["wifi-off", "You're offline.", "Building, checks, knobs, and exports still work. The agent and downloads need a connection.", button("Try again", { variant: "quiet", size: "sm" })],
      "old-library": ["download", `Your parts library is ${D.versions.oldLibrary}. This Emmet expects ${D.versions.library}.`, "Some parts may be missing or renamed until you update (about 150 MB, a minute or two).", button("Later", { variant: "quiet", size: "sm" }) + button("Update the library", { variant: "primary", size: "sm", act: "update" })],
      imported: ["file-up", `Opened ${D.imported.file}: ${D.imported.parts} parts in ${D.imported.steps} steps.`, "It's a fixed model, so it has no knobs yet. The agent can rebuild it with some.", button("Make it parametric", { variant: "primary", size: "sm", act: "parametric" })],
    }[kind];
    return `<div class="banner" data-c="banner" data-kind="${kind}" role="status">${icon(B[0], "lg")}<div class="banner-text"><b>${esc(B[1])}</b>${kind === "old-library" ? `<p>${esc(B[2])}</p>` : ""}</div><div class="row">${B[3]}</div></div>`;
  }

  const agentOf = (screen) => D.agents[screen.who || "claude"];
  // OpenAI's own UI for Codex on a ChatGPT plan (1E): the approved "Continue with ChatGPT" button (approved.css, not
  // restyled), and Manage usage, which opens ChatGPT's usage settings in the browser.
  const continueChatGpt = () => `<button type="button" class="continue-chatgpt"><img src="${ROOT()}../../apps/desktop/src/assets/openai/chatgpt-logo-white.svg" alt=""><span>Continue with ChatGPT</span></button>`;
  const manageUsage = (o = {}) => button("Manage usage", { variant: o.primary ? "primary" : "quiet", size: "sm", icon: "external-link", compact: o.compact, title: "Manage usage: opens ChatGPT's usage settings in your browser" });
  // What the agent is doing (note 1), with Stop and Esc while it works.
  function status(screen) {
    const a = screen.agent || { state: "idle" };
    const working = a.state === "working";
    const tone = { working: "busy", question: "attn", approval: "attn", stopped: "muted", crashed: "bad", offline: "muted", limit: "attn" }[a.state] || (/^Done/.test(a.line || "") ? "done" : "muted");
    const lead = working ? spinner(a.line) : icon({ question: "message-circle-question-mark", approval: "shield-question-mark", crashed: "octagon-alert", stopped: "square", offline: "wifi-off", limit: "gauge" }[a.state] || "circle-check", "sm");
    const stop = working ? button("Stop", { size: "sm", icon: "square", act: "stop", kbd: "Esc", title: "Stop the turn (Esc)" }) : "";
    return `<div class="status" data-c="status" data-tone="${tone}" role="status" aria-live="polite">${lead}<span class="status-line">${esc(a.line)}</span>${a.elapsed ? `<span class="status-time">${esc(a.elapsed)}</span>` : ""}${stop}</div>`;
  }

  // The composer stays open while the agent works: what you send waits as a redirect. Model and effort sit beside Send.
  function composer(screen, o = {}) {
    const a = agentOf(screen);
    const st = screen.agent?.state;
    const ph = st === "working" ? `Redirect ${a.name}…` : st === "offline" ? "The agent needs a connection" : `Ask ${a.name} to build or change something`;
    const pop = screen.overlay === "model" ? `<div class="anchored">${popover("Model and effort", select("Model", a.model, { open: false }) + select("Effort", "Medium") + `<p class="small">Bigger models and higher effort use up your plan faster.</p>`)}</div>` : "";
    return `<div class="composer" data-c="composer"><textarea rows="${screen.composer ? 3 : 1}" placeholder="${esc(ph)}" aria-label="Message">${esc(screen.composer || "")}</textarea>
      <div class="composer-bar">${iconButton("paperclip", "Attach a photo", { size: "sm" })}<span class="composer-model"><button type="button" class="model-chip" data-act="model" aria-haspopup="dialog" aria-expanded="${screen.overlay === "model"}">${esc(a.model)} · Medium ${icon("chevron-down", "sm")}</button>${pop}</span><span class="grow"></span>
      ${button("Send", { variant: "primary", size: "sm", icon: "send", act: "send", disabled: st === "offline", kbd: o.kbd ? "⌘↩" : undefined })}</div></div>`;
  }

  // A step as an instruction book draws it (plan section 08): the parts box, the step number, and a sub-assembly in
  // its own box with numbered sub-steps; the leader to where it goes is drawn once it's laid out (frozen.js).
  function stepView(n) {
    const v = D.stepView[n];
    if (!v) return "";
    const box = (parts) => `<div class="parts-box">${parts.map(([p, c, q]) => `<span class="parts-box-item" title="${esc(partName(p))}, ${esc(c)}"><img src="${partPic(p, c)}" alt="">${q}x</span>`).join("")}</div>`;
    const sub = v.sub ? `<div class="sub-assembly" aria-label="Sub-assembly: the door">${box(v.sub.parts)}<ol class="sub-steps">${v.sub.steps.map((r, i) => `<li><span class="sub-step-n">${i + 1}</span><img src="${ROOT()}shared/renders/${theme()}/${r}.webp" alt="Sub-step ${i + 1}"></li>`).join("")}</ol></div>` : "";
    return `<div class="step-view" data-c="parts-box" aria-label="Step ${n}">${box(v.parts)}<span class="step-number">${n}</span>${sub}</div>`;
  }
  // A finding's pin, placed at its parts in the render's own terms (u, v across and down the render).
  const pin = (text, at, r) => `<button type="button" class="pin viewer-anchor" data-c="pin" data-tab="checks" style="--u:${at.u};--v:${at.v};--k:${r.k};--ty:${r.ty}"><span class="pin-dot">${icon("floating", "sm")}</span>${esc(text)}</button>`;
  const setupCard = (o) => `<div class="setup-card" data-c="setup-card"${attrs({ "data-ready": o.ready })}>${icon(o.icon, "xl")}<div class="setup-card-text"><b>${esc(o.title)}</b><p>${esc(o.text)}</p></div>${o.action || ""}</div>`;
  const versionRow = (v, o = {}) => `<button type="button" class="version-row" data-c="version-row" data-auto="${!!v.auto}" data-local="view=${v.n}"${attrs({ "data-show": o.show })}><img src="${ROOT()}shared/renders/${theme()}/${v.render}.webp" alt=""><span class="version-row-text"><b>${esc(v.name)}</b><span class="small">${esc(v.when)} · ${v.parts} parts${v.auto ? " · saved by Emmet" : ""}</span></span></button>`;
  const viewSwitch = (cur = "Iso") => `<div class="view-switch" data-c="view-switch" role="group" aria-label="View"><div class="view-switch-views" role="radiogroup" aria-label="Views">${["Iso", "Front", "Top"].map((v, i) => `<button type="button" class="view-switch-item" role="radio" aria-checked="${v === cur}" data-state="${v === cur ? "on" : "off"}" title="${v} (${i + 1})">${v}</button>`).join("")}</div>${iconButton("scan", "Frame the model (F)", { size: "sm" })}${iconButton("camera", "Screenshot", { size: "sm" })}</div>`;
  const captionButtons = () => `<div class="caption-buttons" data-c="caption-buttons">${[["minimize", "minus"], ["maximize", "square"], ["close", "x"]].map(([k, ic]) => `<button type="button" class="caption-button" data-kind="${k}" aria-label="${k[0].toUpperCase() + k.slice(1)}">${icon(ic, "sm")}</button>`).join("")}</div>`;
  const render = (name, o = {}) => {
    const r = D.renders[name] || window.EMMET_FROZEN_DATA?.renders?.[name] || { k: 1, ty: 0 };
    return name ? `<img class="viewer-render" src="${ROOT()}shared/renders/${theme()}/${name}.webp" alt="${esc(o.alt || "")}" style="--k:${r.k};--ty:${r.ty}">` : "";
  };

  return {
    esc, md, attrs, icon, spinner, kbd, button, iconButton, badge, textField, slider, switchC, segmented, chips, select, colorPicker, tabs,
    tooltip, menu, menubar, popover, dialog, toast, progress, meter, resizable, dataTable, tree, message, pending, toolCard, fileCard, chat,
    questionCard, approvalCard, alertCard, finding, banner, status, composer, stepView, pin, setupCard, versionRow, viewSwitch,
    captionButtons, render, agentOf, partPic, withUnit, continueChatGpt, manageUsage,
  };
})();
