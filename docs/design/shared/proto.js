// SPDX-License-Identifier: AGPL-3.0-or-later
// The prototype harness each direction runs in: routing (#s=<screen>&t=light|dark, &bare for review.html's frames), the
// review bar, the clicks that move through a flow, and #audit, which renders every screen in both themes and checks
// the contrast of every visible text element against what's really behind it. Works from file:// with no server.
window.EMMET_PROTO = (() => {
  const { screens, byId, FLOWS } = window.EMMET_STATES;
  const UI = window.EMMET_UI;
  const W = 1280, H = 800;
  let dir = null;
  let local = {};

  const hash = () => Object.fromEntries(location.hash.slice(1).split("&").filter(Boolean).map((kv) => kv.split("=").map(decodeURIComponent)));
  const state = () => {
    const h = hash();
    return { s: byId[h.s] ? h.s : "built", t: h.t === "dark" ? "dark" : "light", bare: "bare" in h, audit: "audit" in h };
  };
  function go(s, t) {
    const cur = state();
    local = {};
    const parts = [`s=${s ?? cur.s}`, `t=${t ?? cur.t}`].concat(cur.bare ? ["bare"] : []);
    location.hash = parts.join("&");
  }

  function chrome(cur, screen) {
    const opts = screens.map((x, i) => `<option value="${x.id}"${x.id === cur.s ? " selected" : ""}>${i + 1}. ${UI.esc(x.title)}</option>`).join("");
    const flows = FLOWS.filter((f) => f.path.includes(cur.s)).map((f) => f.name);
    const covers = screen.covers.concat(flows.map((f) => "Flow: " + f));
    return `<div class="p-bar"><a class="p-home" href="${window.EMMET_ROOT}review.html">All directions</a><b>${UI.esc(dir.name)}</b><span class="p-tag">${UI.esc(dir.tagline)}</span>
      <span class="p-grow"></span><button type="button" class="p-btn" data-p="prev" title="Previous screen (PageUp)">◀</button>
      <select class="p-select" aria-label="Screen">${opts}</select><button type="button" class="p-btn" data-p="next" title="Next screen (PageDown)">▶</button>
      <span class="p-seg" role="group" aria-label="Theme"><button type="button" class="p-btn" data-p="light" aria-pressed="${cur.t === "light"}">Light</button><button type="button" class="p-btn" data-p="dark" aria-pressed="${cur.t === "dark"}">Dark</button></span></div>
      <div class="p-note">${covers.map((c) => `<span class="p-chip">${UI.esc(c)}</span>`).join("")}<span>${UI.esc(screen.note || "Click the buttons in the mockup to move through the flow.")}</span></div>`;
  }

  function fit() {
    const stage = document.querySelector(".p-stage");
    const win = document.querySelector(".e-window");
    if (!stage || !win) return;
    const cs = getComputedStyle(stage);
    const w = stage.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight);
    const h = stage.clientHeight - parseFloat(cs.paddingTop) - parseFloat(cs.paddingBottom);
    const k = Math.min(1, w / W, h / H);
    win.style.transform = `scale(${k})`;
    win.style.marginLeft = `${Math.max(0, (w - W * k) / 2)}px`;
  }

  function render(cur = state()) {
    const screen = byId[cur.s];
    document.documentElement.dataset.theme = cur.t;
    document.documentElement.classList.toggle("p-bare", cur.bare);
    const view = { ...screen, ...local };
    // About is a dialog over any direction's window; every other overlay is the direction's to place.
    const modal = view.overlay === "about" ? `<div class="e-modal">${UI.overlay(view)}</div>` : "";
    const app = `<div class="e-window" data-kind="${screen.kind}" style="width:${W}px;height:${H}px">${dir.render(view, { theme: cur.t, local })}${modal}</div>`;
    document.body.innerHTML = (cur.bare ? "" : chrome(cur, screen)) + `<div class="p-stage">${app}</div>`;
    document.title = `${dir.name}: ${screen.title}`;
    // A chat opens at its newest message, as the app's does.
    for (const el of document.querySelectorAll("[data-scroll=end]")) el.scrollTop = el.scrollHeight;
    fit();
  }

  // Clicks: a screen's `on` map first, then what the chrome around every screen does.
  const GLOBAL = { agents: "switch-agent", model: "model-picker", usage: "usage", export: "export", pick: "switched", about: "about", close: "built" };
  function act(name, el) {
    const s = byId[state().s];
    // A trigger clicked again while its overlay is open closes it.
    const next = s.on?.[name] || GLOBAL[name];
    if (next) return go(next === s.id ? "built" : next);
    el.classList.add("p-flash");
    setTimeout(() => el.classList.remove("p-flash"), 400);
  }
  function onClick(e) {
    const p = e.target.closest("[data-p]");
    if (p) {
      const i = screens.findIndex((x) => x.id === state().s);
      const k = p.dataset.p;
      if (k === "prev" || k === "next") go(screens[(i + (k === "next" ? 1 : screens.length - 1)) % screens.length].id);
      else go(undefined, k);
      return;
    }
    // A direction's own view toggles (data-local="key=value"), kept until the screen changes.
    const loc = e.target.closest("[data-local]");
    if (loc) { const [k, v] = loc.dataset.local.split("="); local[k] = v; render(); return; }
    const tab = e.target.closest("[data-tab]");
    if (tab) { local.drawer = tab.dataset.tab; local.overlay = null; render(); return; }
    const a = e.target.closest("[data-act]");
    if (a) { e.preventDefault(); act(a.dataset.act, a); return; }
    const s = byId[state().s];
    if (s.overlay && !e.target.closest(".e-menu") && e.target.closest(".e-window")) go("built");
  }
  function onInput(e) {
    const r = e.target.closest("input[type=range][data-param]");
    if (!r) return;
    const out = r.parentElement.querySelector("output");
    const p = window.EMMET_DATA.params.find((x) => x.name === r.dataset.param);
    if (out && p) out.textContent = `${r.value} ${p.unit}`;
  }
  function onChange(e) {
    if (e.target.closest("input[type=range][data-param]")) { const s = byId[state().s]; if (s.on?.slider) go(s.on.slider); }
    const sel = e.target.closest(".p-select");
    if (sel) go(sel.value);
  }
  function onKey(e) {
    if (e.key === "Escape") { const s = byId[state().s]; if (s.on?.stop) go(s.on.stop); else if (s.overlay) go("built"); }
    if (e.key === "PageDown" || e.key === "PageUp") document.querySelector(`[data-p=${e.key === "PageDown" ? "next" : "prev"}]`)?.click();
  }

  // #audit: WCAG 2.2 AA contrast of every visible text element, against the first opaque background behind it.
  // Text whose background search reaches the render first is reported: it has no surface of its own.
  const parseRgb = (s) => { const m = s.match(/rgba?\(([^)]+)\)/); if (!m) return null; const [r, g, b, a = 1] = m[1].split(/[ ,/]+/).filter(Boolean).map(Number); return [r, g, b, a]; };
  const lum = ([r, g, b]) => [r, g, b].map((v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; }).reduce((s, v, i) => s + v * [0.2126, 0.7152, 0.0722][i], 0);
  const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };
  const over = (top, under) => top.slice(0, 3).map((c, i) => c * top[3] + under[i] * (1 - top[3]));
  function audit() {
    const cur = state();
    const results = [];
    let checked = 0;
    for (const t of ["light", "dark"]) for (const s of screens) {
      local = {};
      render({ s: s.id, t, bare: true });
      for (const el of document.querySelectorAll(".e-window *")) {
        const own = [...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim());
        if (!own || !el.getClientRects().length) continue;
        const cs = getComputedStyle(el);
        if (cs.visibility === "hidden" || el.closest("[aria-hidden=true]")) continue;
        // Background layers from the element outward, until an opaque one; then composite them back up.
        const layers = [];
        let hit = null;
        for (let n = el; n && n !== document.body; n = n.parentElement) {
          if (n !== el && n.querySelector(":scope > img.e-shot")) { hit = "over the render"; break; }
          const c = parseRgb(getComputedStyle(n).backgroundColor);
          if (c && c[3] > 0) { layers.push(c); if (c[3] >= 0.999) break; }
        }
        let bg = null;
        if (!hit && layers.length && layers[layers.length - 1][3] >= 0.999) {
          bg = layers.pop().slice(0, 3);
          while (layers.length) bg = over(layers.pop(), bg);
        } else hit = hit || "no opaque background";
        checked++;
        const size = parseFloat(cs.fontSize), bold = Number(cs.fontWeight) >= 700;
        const need = size >= 24 || (bold && size >= 18.66) ? 3 : 4.5;
        let fg = parseRgb(cs.color);
        fg = bg ? over(fg, bg) : fg;
        const r = hit ? 0 : ratio(fg, bg);
        if (hit || r < need) results.push({ theme: t, screen: s.id, text: el.textContent.trim().slice(0, 40), ratio: Math.round(r * 100) / 100, need, why: hit || "low contrast" });
      }
    }
    window.EMMET_AUDIT = { direction: dir.id, screens: screens.length, themes: 2, checked, failures: results };
    console.log(`[audit] ${dir.id}: ${checked} text elements, ${results.length} failures across ${screens.length} screens × 2 themes`, results);
    // Opened with #audit (for example by headless Chrome with --dump-dom), the page ends on the result.
    if (cur.audit) document.body.innerHTML = `<pre id="p-audit">${UI.esc(JSON.stringify(window.EMMET_AUDIT, null, 1))}</pre>`;
    else render(cur);
    return window.EMMET_AUDIT;
  }

  function direction(d) {
    dir = d;
    document.addEventListener("click", onClick);
    document.addEventListener("input", onInput);
    document.addEventListener("change", onChange);
    document.addEventListener("keydown", onKey);
    window.addEventListener("hashchange", () => render());
    window.addEventListener("resize", fit);
    if (state().audit) audit();
    else render();
  }
  return { direction, go, audit, screens };
})();
