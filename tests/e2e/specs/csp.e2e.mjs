// SPDX-License-Identifier: AGPL-3.0-or-later
// A script that arrives as data never runs (plan section 15, session 5B). A shared project is someone else's files:
// its name, and the build/ folder it carries (part ids, step titles, findings), are shown before the user
// trusts anything. Here every one of those strings is markup that would run a script if the window put it in the page
// as HTML; the window must show it as the text it is. Then the content security policy itself, the last line if a
// string ever does get through: no inline script, no eval, no request to anywhere but the app. The chat's markdown
// has the same test in chat.e2e.mjs.
import { $, expect } from "@wdio/globals";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { invoke } from "../lib/app.mjs";
import { emmet, projectDir } from "../lib/projects.mjs";

const dir = projectDir("csp");
/** Markup that sets window.__pwned if it's ever parsed as HTML: an image whose load fails, and a script. */
const IMG = '<img src=x onerror="window.__pwned=1">';
const SCRIPT = "<script>window.__pwned=1</script>";
const pwned = () => browser.execute(() => window.__pwned ?? null);

/** Rewrites a build file's JSON. */
function edit(name, change) {
  const path = join(dir, "build", name);
  const v = JSON.parse(readFileSync(path, "utf8"));
  change(v);
  writeFileSync(path, JSON.stringify(v, null, 2));
}

describe("a script in a project's data doesn't run", () => {
  before(() => {
    emmet(["run", "--project", dir]);
    emmet(["check", "--project", dir]);
    edit("steps.json", (steps) => {
      for (const s of steps.steps) {
        s.title = `${IMG}${SCRIPT}`;
        s.group = `${SCRIPT}${IMG}`;
      }
    });
    // The window makes its parts list from the placements and its own parts index, not from a build/bom.json a
    // project carries; a part id is the string a project can put there.
    edit("placements.json", (p) => {
      p.model = `${IMG}${SCRIPT}`;
      p.parts[0].part = IMG;
      p.parts[1].id = SCRIPT;
    });
    edit("validation.json", (v) => {
      v.clean = false;
      v.counts = { floating: 1 };
      v.findings = [{ id: "F-0001", rule: "floating", severity: "warning", parts: ["base"], source: [], at: [0, 0, 0], message: `${IMG}${SCRIPT}`, hint: `${SCRIPT}${IMG}` }];
    });
    const toml = join(dir, "emmet.toml");
    writeFileSync(toml, readFileSync(toml, "utf8").replace(/^name = .*$/m, `name = '${IMG}${SCRIPT}'`));
    // As it would arrive from someone else: not trusted, so the app shows the build it carries and runs nothing.
    rmSync(join(dir, ".emmet", "trust"), { force: true });
  });

  it("the project's name, part ids, step titles, and findings show as text", async () => {
    const { root, name } = await invoke("open_recent", { path: dir });
    expect(name).toBe(`${IMG}${SCRIPT}`);
    await browser.waitUntil(() => browser.execute((r) => window.__emmet.project() === r, root));
    await expect($("#banner-untrusted")).toBeExisting();
    await expect($("#project-chip")).toHaveText(expect.stringContaining(IMG));
    // Each tab shows the markup as words, so the strings did reach the page.
    const words = {};
    for (const tab of ["parts", "steps", "checks", "options"]) {
      await (await $(`#tab-${tab}`)).click();
      await browser.pause(300);
      words[tab] = await browser.execute(() => document.body.innerText);
      expect(await pwned()).toBe(null);
    }
    expect(words.parts).toContain(IMG);
    expect(words.steps).toContain(SCRIPT);
    expect(words.checks).toContain(IMG);
    // Nothing in the page was made from the markup.
    const made = await browser.execute(() => ({
      images: document.querySelectorAll("img[src='x'], img[onerror]").length,
      scripts: [...document.scripts].filter((s) => !s.src).length,
    }));
    expect(made).toEqual({ images: 0, scripts: 0 });
    expect(await pwned()).toBe(null);
  });

  it("the content security policy refuses inline script, eval, and requests to anywhere else", async () => {
    const result = await browser.execute(async () => {
      const blocked = [];
      const note = (e) => blocked.push(e.effectiveDirective);
      document.addEventListener("securitypolicyviolation", note);
      // Markup parsed into the page, as a string that got past React would be: each way in sets its own flag.
      const box = document.createElement("div");
      box.innerHTML = '<img src="data:image/png;base64,AAAA" onerror="window.__ran_handler=1"><a href="javascript:window.__ran_link=1">x</a>';
      document.body.append(box);
      box.querySelector("a").click();
      const script = document.createElement("script");
      script.textContent = "window.__ran_script=1";
      document.body.append(script);
      // From a timer, as the page's own code would call it: WebView2 lets the script a driver injects use eval
      // whatever the policy says (as DevTools' console may), so a call made here directly proves nothing.
      const evaled = await new Promise((resolve) =>
        setTimeout(() => {
          try {
            (0, eval)("window.__ran_eval=1");
            resolve("ran");
          } catch (e) {
            resolve(e.name);
          }
        }, 0),
      );
      const fetched = await fetch("https://example.com/").then(
        () => "sent",
        () => "refused",
      );
      await new Promise((r) => setTimeout(r, 500));
      document.removeEventListener("securitypolicyviolation", note);
      box.remove();
      script.remove();
      const ran = ["handler", "link", "script", "eval"].filter((k) => window[`__ran_${k}`]);
      return { ran, evaled, fetched, blocked: [...new Set(blocked)].sort() };
    });
    console.log(`csp: ${JSON.stringify(result)}`);
    expect(result.ran).toEqual([]);
    expect(result.evaled).toBe("EvalError");
    expect(result.fetched).toBe("refused");
    expect(result.blocked).toEqual(expect.arrayContaining(["connect-src", "script-src-attr", "script-src-elem"]));
  });
});
