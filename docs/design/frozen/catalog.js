// SPDX-License-Identifier: AGPL-3.0-or-later
// The frozen design's catalog (index.html): every screen and state by group, with links to it at both sizes, in both
// themes, on both platforms; which screen shows each section 14 state; and the key flows.
(() => {
  const F = window.EMMET_FROZEN;
  const esc = window.EMMET_C.esc;
  const link = (id, t, w, p = "macos") => `screen.html#s=${id}&t=${t}&w=${w}&p=${p}`;
  const groups = [...new Set(F.screens.map((s) => s.group))];
  const n = Object.fromEntries(F.screens.map((s, i) => [s.id, i + 1]));
  const rows = (g) => F.screens.filter((s) => s.group === g).map((s) => `<tr><td>${n[s.id]}</td><td><a href="${link(s.id, "light", 1440)}"><b>${esc(s.title)}</b></a>${s.note ? `<br><span class="h-muted">${esc(s.note)}</span>` : ""}</td>
    <td>${s.covers.map((c) => `<span class="h-chip">${esc(c)}</span>`).join("")}</td>
    <td>${[1440, 1280].map((w) => ["light", "dark"].map((t) => `<a href="${link(s.id, t, w)}">${w} ${t}</a>`).join(" · ")).join("<br>")}<br><a href="${link(s.id, "light", 1440, "windows")}">Windows</a></td></tr>`).join("");
  const states = F.STATES.map((st) => `<tr><td>${esc(st)}</td><td>${F.screens.filter((s) => s.covers.includes(st)).map((s) => `<a href="${link(s.id, "light", 1440)}">${n[s.id]}. ${esc(s.title)}</a>`).join("<br>")}</td></tr>`).join("");
  const flows = F.FLOWS.map((f) => `<tr><td>${esc(f.name)}</td><td>${f.path.map((id) => `<a href="${link(id, "light", 1440)}">${n[id]}</a>`).join(" → ")}</td></tr>`).join("");
  document.body.innerHTML = `<main class="h-page">
    <h1>Emmet, frozen: every screen and state</h1>
    <p>Direction B, Canvas, as Barry picked it in 2A (H-07), turned into a design system and frozen in session 2B. Every screen is composed only from
    <a href="components.html">the components</a> (<a href="../components.md">inventory</a>), which take every color, size, and duration from
    <a href="../../../apps/desktop/src/styles/tokens.css">tokens.css</a>. Each screen opens at 1440 × 900 or 1280 × 800, light or dark, on macOS or Windows,
    and lists the components and tokens it uses. Phase 4 builds to these exactly; after sign-off (<a href="../SIGNOFF.md">SIGNOFF.md</a>), a change needs
    a short design change note and Barry's yes.</p>
    <p>Also: <a href="../keyboard.md">the keyboard map and menus</a> · <a href="../content.md">the content and tone guide</a> · <a href="../icons.md">the icon set</a> ·
    <a href="../icon/app-icon-macos.svg">the app icon</a> · <a href="screen.html#audit">the audit</a> (every screen, both sizes, themes, and platforms).</p>
    <h2>${F.screens.length} screens and states</h2>
    <table class="h-table"><thead><tr><th>#</th><th>Screen</th><th>Section 14 states</th><th>Open</th></tr></thead><tbody>
    ${groups.map((g) => `<tr><th colspan="4">${esc(g)}</th></tr>${rows(g)}`).join("")}</tbody></table>
    <h2>Section 14's states</h2><table class="h-table"><thead><tr><th>State</th><th>Shown on</th></tr></thead><tbody>${states}</tbody></table>
    <h2>Key flows</h2><table class="h-table"><thead><tr><th>Flow</th><th>Screens, in order</th></tr></thead><tbody>${flows}</tbody></table></main>`;
})();
