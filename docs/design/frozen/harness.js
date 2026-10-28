// SPDX-License-Identifier: AGPL-3.0-or-later
// The frozen design's viewer (plan session 2B), from file:// with no server. screen.html#s=<screen>&t=light|dark&
// w=1440|1280&p=macos|windows shows one screen at its design size with the components and tokens it uses; &bare
// leaves out the frame; #audit renders every screen at both sizes, in both themes, on both platforms and checks it:
// the contrast of every visible text against what's really behind it, every element that claims a component is one
// in the inventory, no icon is missing, no inline style but placement and content, and nothing spills out of the
// window. index.html and components.html use the same helpers.
window.EMMET_H = (() => {
  const F = window.EMMET_FROZEN;
  const C = window.EMMET_C;
  const GEN = window.EMMET_GEN;
  const SIZES = { 1440: 900, 1280: 800 };
  const esc = C.esc;
  let local = {};
  let shown = null;

  const hash = () => Object.fromEntries(location.hash.slice(1).split("&").filter(Boolean).map((kv) => kv.split("=").map(decodeURIComponent)));
  function state() {
    const h = hash();
    return { s: F.byId[h.s] ? h.s : "built", t: h.t === "dark" ? "dark" : "light", w: h.w === "1280" ? 1280 : 1440, p: h.p === "windows" ? "windows" : "macos", bare: "bare" in h, audit: "audit" in h, uses: h.uses !== "0" };
  }
  function go(change) {
    const cur = { ...state(), ...change };
    location.hash = [`s=${cur.s}`, `t=${cur.t}`, `w=${cur.w}`, `p=${cur.p}`].concat(cur.bare ? ["bare"] : [], cur.uses ? [] : ["uses=0"]).join("&");
  }

  // One screen's window, at its design size. Tokens and components see data-theme and data-platform on it.
  function windowHtml(screen, cur) {
    window.EMMET_THEME = cur.t;
    const view = { ...screen, ...local };
    const lights = cur.p === "macos" ? `<span class="h-lights" aria-hidden="true"><i></i><i></i><i></i></span>` : "";
    return `<div class="window" data-theme="${cur.t}" data-platform="${cur.p}" style="width:${cur.w}px;height:${SIZES[cur.w]}px">${lights}${F.compose(view, { platform: cur.p })}</div>`;
  }

  // What a rendered screen uses: its components in the order they first appear, and their tokens by kind.
  const KINDS = [["Color", /^--(bg|fg|ui|fill|on)-|^--(line|scrim|focus-color)$/], ["Type", /^--(font|text|leading|weight)/], ["Space", /^--space/], ["Radius", /^--radius/], ["Elevation", /^--elevation/], ["Motion", /^--(duration|ease|rise)/], ["Size and layout", /./]];
  function uses(root) {
    const comps = [...new Set([...root.querySelectorAll("[data-c]")].map((e) => e.dataset.c))];
    const tokens = [...new Set(["base", ...comps].flatMap((c) => GEN.tokens[c] || []))].sort();
    const kinds = KINDS.map(([name, re]) => [name, tokens.filter((t) => re.test(t) && !KINDS.slice(0, KINDS.findIndex((k) => k[0] === name)).some(([, r]) => r.test(t)))]);
    return { comps, tokens, kinds };
  }
  function usesHtml(u) {
    return `<h2>Components (${u.comps.length})</h2><p>${u.comps.map((c) => `<a class="h-chip" href="../components.md#${c}">${esc(c)}</a>`).join("")}</p>
      <h2>Tokens (${u.tokens.length})</h2>${u.kinds.filter(([, l]) => l.length).map(([k, l]) => `<p><b>${k}</b><br>${l.map((t) => `<code>${t}</code>`).join(" ")}</p>`).join("")}
      <p>From <a href="../../../apps/desktop/src/styles/tokens.css">tokens.css</a> through <a href="components.css">components.css</a>; <a href="components.html">every component in every state</a>.</p>`;
  }

  function chrome(cur, screen) {
    const opts = F.screens.map((x, i) => `<option value="${x.id}"${x.id === cur.s ? " selected" : ""}>${i + 1}. ${esc(x.title)}</option>`).join("");
    const seg = (key, vals) => `<span class="h-seg" role="group">${vals.map(([v, label]) => `<button type="button" class="h-btn" data-h="${key}=${v}" aria-pressed="${String(cur[key]) === String(v)}">${label}</button>`).join("")}</span>`;
    const flows = F.FLOWS.filter((f) => f.path.includes(cur.s)).map((f) => "Flow: " + f.name);
    return `<div class="h-bar"><a href="index.html">All screens</a><b>Emmet, frozen</b><span class="h-grow"></span>
      <button type="button" class="h-btn" data-h="prev" title="Previous screen (PageUp)">◀</button><select class="h-select" aria-label="Screen">${opts}</select><button type="button" class="h-btn" data-h="next" title="Next screen (PageDown)">▶</button>
      ${seg("w", [[1440, "1440 × 900"], [1280, "1280 × 800"]])}${seg("t", [["light", "Light"], ["dark", "Dark"]])}${seg("p", [["macos", "macOS"], ["windows", "Windows"]])}${seg("uses", [[true, "Uses"], [false, "Hide"]])}</div>
      <div class="h-note">${screen.covers.concat(flows).map((c) => `<span class="h-chip">${esc(c)}</span>`).join("")}<span>${esc(screen.note || "Click the buttons in the window to move through the flow.")}${cur.p === "windows" ? " On a Mac, the Windows screens fall back to the system font." : ""}</span></div>`;
  }

  // After layout: a sub-assembly's leader runs down from its box, then across to where it goes (4C draws it the
  // same way, from the part's screen position).
  function leaders(root) {
    for (const v of root.querySelectorAll(".viewer")) {
      const sub = v.querySelector(".sub-assembly");
      const to = v.querySelector("[data-leader]");
      if (!sub || !to) continue;
      const k = v.getBoundingClientRect().width / v.offsetWidth;
      const vr = v.getBoundingClientRect(), sr = sub.getBoundingClientRect(), tr = to.getBoundingClientRect();
      const x0 = (sr.left - vr.left) / k + 24, y0 = (sr.bottom - vr.top) / k, x1 = (tr.left - vr.left) / k, y1 = (tr.top - vr.top) / k;
      v.insertAdjacentHTML("beforeend", `<svg class="leader" aria-hidden="true"><path d="M${x0} ${y0}V${y1}H${x1 - 4}M${x1 - 12} ${y1 - 5}L${x1 - 3} ${y1}L${x1 - 12} ${y1 + 5}"/></svg>`);
    }
  }
  function settle(root) {
    leaders(root);
    for (const el of root.querySelectorAll("[data-scroll=end]")) el.scrollTop = el.scrollHeight;
  }
  function fit() {
    const stage = document.querySelector(".h-stage");
    const win = stage?.querySelector(".window");
    if (!win) return;
    const cs = getComputedStyle(stage);
    const w = stage.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
    const h = stage.clientHeight - parseFloat(cs.paddingTop) - parseFloat(cs.paddingBottom);
    const k = Math.min(1, w / win.offsetWidth, h / win.offsetHeight);
    win.style.transform = `scale(${k})`;
    win.style.marginLeft = `${Math.max(0, (w - win.offsetWidth * k) / 2)}px`;
  }

  function render(cur = state()) {
    if (cur.s !== shown) { local = {}; shown = cur.s; }
    const screen = F.byId[cur.s];
    document.documentElement.dataset.theme = cur.t;
    document.body.className = `h h-screen${cur.bare ? " h-bare" : ""}`;
    document.body.innerHTML = (cur.bare ? "" : chrome(cur, screen)) + `<div class="h-body"><div class="h-stage">${windowHtml(screen, cur)}</div>${cur.bare || !cur.uses ? "" : `<aside class="h-uses" aria-label="What this screen uses"></aside>`}</div>`;
    document.title = `Emmet frozen: ${screen.title}`;
    const win = document.querySelector(".window");
    settle(win);
    const aside = document.querySelector(".h-uses");
    if (aside) aside.innerHTML = usesHtml(uses(win));
    fit();
  }

  // Clicks in the window move through the flows; the bar's controls change the screen, size, theme, or platform.
  const GLOBAL = { agents: "switch-agent", model: "model-picker", usage: "usage", export: "export", pick: "switched", about: "about", close: "built" };
  function onClick(e) {
    const h = e.target.closest("[data-h]");
    if (h) {
      const k = h.dataset.h;
      const i = F.screens.findIndex((x) => x.id === state().s);
      if (k === "prev" || k === "next") return go({ s: F.screens[(i + (k === "next" ? 1 : F.screens.length - 1)) % F.screens.length].id });
      const [key, v] = k.split("=");
      return go({ [key]: key === "w" ? Number(v) : key === "uses" ? v === "true" : v });
    }
    if (!e.target.closest(".h-stage .window")) return;
    const loc = e.target.closest("[data-local]");
    if (loc) { const [k, v] = loc.dataset.local.split("="); local[k] = v; return render(); }
    const tab = e.target.closest("[data-tab]");
    if (tab) { local.drawer = tab.dataset.tab; local.overlay = null; return render(); }
    const a = e.target.closest("[data-act]");
    const s = F.byId[state().s];
    if (a) {
      e.preventDefault();
      const next = s.on?.[a.dataset.act] || GLOBAL[a.dataset.act];
      if (next) return go({ s: next === s.id ? "built" : next });
      a.classList.add("h-flash");
      return setTimeout(() => a.classList.remove("h-flash"), 400);
    }
    if (s.overlay && !e.target.closest(".menu, .popover, .dialog")) go({ s: "built" });
  }
  function onKey(e) {
    if (e.key === "Escape") { const s = F.byId[state().s]; if (s.on?.stop) go({ s: s.on.stop }); else if (s.overlay) go({ s: "built" }); }
    if (e.key === "PageDown" || e.key === "PageUp") document.querySelector(`[data-h=${e.key === "PageDown" ? "next" : "prev"}]`)?.click();
  }

  // ---- The audit ----------------------------------------------------------------------------------------------------
  const parseRgb = (s) => { const m = s.match(/rgba?\(([^)]+)\)/); if (!m) return null; const [r, g, b, a = 1] = m[1].split(/[ ,/]+/).filter(Boolean).map(Number); return [r, g, b, a]; };
  const lum = ([r, g, b]) => [r, g, b].map((v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }).reduce((s, v, i) => s + v * [0.2126, 0.7152, 0.0722][i], 0);
  const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
  const over = (top, under) => top.slice(0, 3).map((c, i) => c * top[3] + under[i] * (1 - top[3]));
  // Inline styles a screen may carry: placement in the render's terms, a fill's share, a part's own color, and where
  // a floating piece sits (Radix places those at run time).
  const INLINE_OK = /^(--u|--v|--k|--ty|width|left|top|right|bottom|background)$/;

  function check(win, where, out) {
    const wr = win.getBoundingClientRect();
    for (const el of win.querySelectorAll("*")) {
      const cs = getComputedStyle(el);
      if (!el.getClientRects().length || cs.visibility === "hidden") continue;
      const fail = (why, extra = {}) => out.push({ ...where, why, el: `${el.tagName.toLowerCase()}.${[...el.classList].join(".")}`, text: (el.textContent || "").trim().slice(0, 40), ...extra });
      if (el.dataset.c && !GEN.tokens[el.dataset.c]) fail("not in the inventory: " + el.dataset.c);
      if (el.matches("svg[data-missing]")) fail("missing icon: " + el.dataset.missing);
      for (const prop of el.style) if (!INLINE_OK.test(prop) && !el.classList.contains("window")) fail("one-off inline style: " + prop);
      // Spilling out of the window (scrolled-away content and the render, which is cropped by design, aside).
      const r = el.getBoundingClientRect();
      const clipped = el.closest(".viewer-render, .dock-history, .dock-active, .panel-body, .tree, .first-run-card");
      if (!clipped && (r.right > wr.right + 1 || r.bottom > wr.bottom + 1 || r.left < wr.left - 1)) fail("outside the window");
      // Text: its own text nodes, a field's value, or an empty field's placeholder.
      const field = el.matches("input:not([type=range], [type=checkbox], [type=radio]), textarea");
      const placeholder = field && el.placeholder && !el.value;
      const own = [...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()) || (field && !!el.value);
      if (!own && !placeholder) continue;
      // Text for screen readers only (clipped to nothing, as .visually-hidden and a compact button's label are).
      if (el.closest("[aria-hidden=true], .visually-hidden") || cs.clipPath === "inset(50%)") continue;
      // An unwanted cut: text cut off without an ellipsis.
      if (cs.overflow !== "visible" && cs.textOverflow !== "ellipsis" && el.scrollWidth > el.clientWidth + 1 && !field) fail("text cut off");
      const layers = [];
      let hit = null;
      for (let n = el; n && n !== document.body; n = n.parentElement) {
        if (n !== el && n.querySelector(":scope > img.viewer-render")) { hit = "over the render"; break; }
        const c = parseRgb(getComputedStyle(n).backgroundColor);
        if (c && c[3] > 0) { layers.push(c); if (c[3] >= 0.999) break; }
      }
      let bg = null;
      if (!hit && layers.length && layers[layers.length - 1][3] >= 0.999) { bg = layers.pop().slice(0, 3); while (layers.length) bg = over(layers.pop(), bg); }
      else hit = hit || "no opaque background";
      const size = parseFloat(cs.fontSize), bold = Number(cs.fontWeight) >= 700;
      const need = size >= 24 || (bold && size >= 18.66) ? 3 : 4.5;
      let fg = parseRgb(placeholder ? getComputedStyle(el, "::placeholder").color : cs.color);
      fg = bg ? over(fg, bg) : fg;
      const r2 = hit ? 0 : ratio(fg, bg);
      out.checked++;
      if (hit || r2 < need) fail(hit || "low contrast", { ratio: Math.round(r2 * 100) / 100, need });
    }
  }
  function audit() {
    const out = [];
    out.checked = 0;
    let renders = 0;
    for (const p of ["macos", "windows"]) for (const t of ["light", "dark"]) for (const w of [1440, 1280]) for (const s of F.screens) {
      local = {};
      const cur = { s: s.id, t, w, p };
      document.documentElement.dataset.theme = t;
      document.body.className = "h h-bare";
      document.body.innerHTML = `<div class="h-stage">${windowHtml(s, cur)}</div>`;
      const win = document.querySelector(".window");
      settle(win);
      check(win, { screen: s.id, theme: t, size: w, platform: p }, out);
      renders++;
    }
    const result = { screens: F.screens.length, renders, checked: out.checked, components: Object.keys(GEN.tokens).length, failures: out.slice(0, 200), failureCount: out.length };
    window.EMMET_AUDIT = result;
    console.log(`[audit] ${renders} renders, ${out.checked} text elements, ${out.length} failures`, out);
    document.body.className = "h";
    document.body.innerHTML = `<pre id="h-audit">${esc(JSON.stringify(result, null, 1))}</pre>`;
    return result;
  }

  function start() {
    document.addEventListener("click", onClick);
    document.addEventListener("keydown", onKey);
    document.addEventListener("change", (e) => { if (e.target.matches(".h-select")) go({ s: e.target.value }); });
    window.addEventListener("hashchange", () => render());
    window.addEventListener("resize", fit);
    if (state().audit) audit();
    else render();
  }
  return { start, uses, windowHtml, settle, SIZES };
})();
