// SPDX-License-Identifier: AGPL-3.0-or-later
// Every screen and state 2A designs, from plan section 14 ("States every screen designs for" and "Key flows") and
// Barry's notes (docs/design/notes.md). Each direction renders every entry; review.html and storyboards.md use the
// same ids. `covers` names the section 14 states an entry shows; `on` maps a click in the mockup to the next screen.
window.EMMET_STATES = (() => {
  const S = window.EMMET_DATA.seg;
  const story = (...parts) => parts.flat();

  // Section 14's state list, word for word, and its nine key flows.
  const STATES = [
    "Empty", "First run", "Loading library", "Building", "Build failed", "Checks with findings", "Agent thinking",
    "Waiting for approval", "Question waiting for an answer", "Redirect pending", "Stopped by the user",
    "Build animating (with Skip)", "Agent not installed or not signed in", "Agent crashed", "Offline",
    "Untrusted project", "An old library version",
  ];
  const FLOWS = [
    { id: "first-run", name: "First run", path: ["fr-welcome", "fr-library", "fr-agents", "fr-start"] },
    { id: "new", name: "New model from a prompt", path: ["fr-start", "new-empty", "thinking", "question", "building", "redirect", "approval", "built"] },
    { id: "sliders", name: "Tweak with sliders", path: ["built", "sliders", "shuffled"] },
    { id: "settings", name: "Change the build settings", path: ["built", "settings", "restyle"] },
    { id: "fix", name: "Fix a finding", path: ["findings", "fixing", "built"] },
    { id: "export", name: "Export", path: ["built", "export"] },
    { id: "import", name: "Open an existing LDraw file", path: ["import"] },
    { id: "switch", name: "Switch agent mid-project", path: ["switch-agent", "switched"] },
    { id: "answer", name: "Answer a question", path: ["question", "building"] },
    { id: "watch", name: "Watch, redirect, stop", path: ["building", "redirect", "approval", "built", "stopped"] },
  ];

  const busy = (line, elapsed, extra = {}) => ({ state: "working", line, elapsed, ...extra });
  const screens = [
    // First run (section 14 key flow 1; D-41 setup with no terminal).
    { id: "fr-welcome", title: "First run: welcome", kind: "first-run", step: "welcome", covers: ["First run"], on: { primary: "fr-library" } },
    { id: "fr-library", title: "First run: parts library and Python", kind: "first-run", step: "library", covers: ["First run", "Loading library"], on: { primary: "fr-agents" } },
    { id: "fr-agents", title: "First run: set up an agent", kind: "first-run", step: "agents", covers: ["First run", "Agent not installed or not signed in"], on: { primary: "fr-start" }, note: "Note 10: \"Look again\" becomes \"Check agents again\"." },
    { id: "fr-start", title: "First run: start", kind: "first-run", step: "start", covers: ["First run"], on: { example: "built", prompt: "new-empty" }, note: "Note 9: New offers the Cottage example or a model from a prompt. Note 7: projects go to Documents/Emmet." },

    // New model from a prompt, watched, redirected, approved.
    { id: "new-empty", title: "New model from a prompt", kind: "workspace", render: "empty", covers: ["Empty"], build: { state: "empty" }, agent: { state: "idle" }, chat: [], composer: S.prompt[0].text, drawer: null, on: { send: "thinking" }, note: "Note 8: the baseplate is a parameter; an empty model starts on it." },
    { id: "thinking", title: "Agent thinking", kind: "workspace", render: "empty", covers: ["Agent thinking"], build: { state: "empty" }, agent: busy("Thinking about the cottage", "0:04"), chat: story(S.prompt, S.thinking), on: { stop: "stopped" } },
    { id: "question", title: "Question waiting for an answer", kind: "workspace", render: "empty", covers: ["Question waiting for an answer"], build: { state: "empty" }, agent: { state: "question", line: "Waiting for your answer", elapsed: "0:09" }, chat: story(S.prompt, S.thinking, S.question), on: { answer: "building" } },
    { id: "building", title: "Building, brick by brick", kind: "workspace", render: "stage1", animating: true, covers: ["Building", "Build animating (with Skip)"], build: { state: "building", parts: 52 }, agent: busy("Building the walls, row 3 of 6", "0:22"), chat: story(S.prompt, S.answered, S.start, S.stage1Running), on: { stop: "stopped", skip: "redirect", send: "redirect" }, note: "Note 1: the live line sits where the user looks, with a spinner and the time." },
    { id: "redirect", title: "Redirect pending", kind: "workspace", render: "stage2", covers: ["Redirect pending", "Building"], build: { state: "building", parts: 108 }, agent: busy("Building the walls and ceiling", "0:31"), chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2Running), pending: S.pending[0].text, on: { stop: "stopped", deliver: "approval", cancel: "building" } },
    { id: "approval", title: "Waiting for approval", kind: "workspace", render: "stage2", covers: ["Waiting for approval"], build: { state: "built", parts: 108 }, agent: { state: "approval", line: "Waiting for you to allow a web request", elapsed: "0:38" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.delivered, S.ack, S.approval), on: { allow: "built", deny: "built" } },
    { id: "built", title: "Built and clean", kind: "workspace", render: "cottage", covers: [], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Done in 1:12" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.delivered, S.ack, S.stage3, S.finish), drawer: "params", on: { slider: "sliders" } },

    // Tweak, steps, fix, fail.
    { id: "sliders", title: "Tweak with a knob: weathering", kind: "workspace", render: "weathered-1", covers: [], build: { state: "built", parts: 163, clean: true, ms: 188 }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.delivered, S.ack, S.stage3, S.finish), drawer: "params", dirty: { weathering: 20 }, on: { keep: "built", reset: "built", shuffle: "shuffled" }, note: "Barry 2026-10-27: the model's knobs change its character (weathering, features, another variation), not raw width or color; a drag is a draft until Keep." },
    { id: "shuffled", title: "Shuffle: another variation", kind: "workspace", render: "weathered-2", covers: [], build: { state: "built", parts: 163, clean: true, ms: 188 }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.delivered, S.ack, S.stage3, S.finish), drawer: "params", dirty: { weathering: 20, seed: 2 }, on: { keep: "built", reset: "built", shuffle: "sliders" }, note: "Shuffle picks another random arrangement of the same design, at once and free." },
    // Build settings (a proposal for direction B): change one, then ask the agent to rework the model for it.
    { id: "settings", title: "Build settings: a change waiting to rebuild", kind: "workspace", render: "cottage", covers: [], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.delivered, S.ack, S.stage3, S.finish), drawer: "params", briefDirty: { style: "Steampunk", detail: "Medium" }, on: { rebuild: "restyle", undo: "built" }, note: "Barry 2026-10-27: build settings steer the agent; style takes your own words (here, steampunk); a change asks the agent to rework the model." },
    { id: "restyle", title: "The agent reworks the model for new settings", kind: "workspace", render: "cottage", covers: ["Agent thinking"], build: { state: "built", parts: 163, clean: true }, agent: busy("Reworking the cottage: steampunk, medium detail", "0:06"), chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.delivered, S.ack, S.stage3, S.finish, S.restyleAsk, S.restyleThinking), drawer: "params", on: { stop: "stopped" } },
    { id: "steps", title: "Steps: the build up to step 9", kind: "workspace", render: "stage2", covers: [], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.delivered, S.ack, S.stage3, S.finish), drawer: "steps", step: 9, note: "Note 6: a step shows the model built up to it; later steps are hidden." },
    { id: "findings", title: "Checks with findings", kind: "workspace", render: "finding", covers: ["Checks with findings"], build: { state: "findings", parts: 160, findings: 1 }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stage3, S.finish), drawer: "checks", selected: true, on: { fix: "fixing" } },
    { id: "fixing", title: "The agent fixes a finding", kind: "workspace", render: "finding", covers: ["Building"], build: { state: "findings", parts: 160, findings: 1 }, agent: busy("Fixing F-0001: lowering the chimney", "0:07"), chat: story(S.fixAsk, S.fixing), drawer: "checks", on: { stop: "stopped", done: "built" } },
    { id: "failed", title: "Build failed", kind: "workspace", render: "cottage", covers: ["Build failed"], build: { state: "failed", parts: 163 }, agent: { state: "idle", line: "Idle" }, chat: story(S.flipAsk, S.flipEdit), error: true, on: { fix: "fixing" }, note: "The last good model stays in the viewer (law 1)." },

    // Interruptions and trouble.
    { id: "stopped", title: "Stopped by the user", kind: "workspace", render: "stage2", covers: ["Stopped by the user"], build: { state: "built", parts: 108 }, agent: { state: "stopped", line: "Stopped by you" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stopped), on: { send: "thinking" } },
    { id: "crashed", title: "Agent crashed", kind: "workspace", render: "stage2", covers: ["Agent crashed"], build: { state: "built", parts: 108 }, agent: { state: "crashed", line: "Claude Code stopped unexpectedly" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2Running, S.crashed), on: { restart: "building" } },
    { id: "signed-out", title: "Agent not signed in", kind: "workspace", render: "cottage", covers: ["Agent not installed or not signed in"], build: { state: "built", parts: 163, clean: true }, agent: { state: "signedout", line: "Claude isn't signed in" }, chat: [], on: { signin: "built", other: "switched" } },
    { id: "switch-agent", title: "Switch agent: the picker", kind: "workspace", render: "cottage", covers: [], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stage3, S.finish), overlay: "agents", on: { pick: "switched" }, note: "Note 3: the picker and Start fresh in the chat header, like nurb. The MVP's agents are Claude Code and Codex (D-43)." },
    { id: "switched", title: "Switched to Codex mid-project", kind: "workspace", render: "cottage", covers: ["Agent thinking"], build: { state: "built", parts: 163, clean: true }, agent: busy("Reading the project", "0:05", { who: "codex" }), who: "codex", chat: story([{ t: "divider", text: "Codex · a fresh conversation. The Claude one is kept." }], S.garage, S.codexStart), on: { stop: "stopped" } },
    { id: "model-picker", title: "Model and effort", kind: "workspace", render: "cottage", covers: [], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stage3, S.finish), overlay: "model", note: "Note 4: model and effort beside Send, with \"Bigger models and higher effort use up your plan faster.\"" },
    { id: "usage", title: "Plan usage", kind: "workspace", render: "cottage", covers: [], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stage3, S.finish), overlay: "usage" },
    { id: "about", title: "About Emmet Builder", kind: "workspace", render: "cottage", covers: [], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stage3, S.finish), overlay: "about", on: { close: "built" }, note: "The brand: the ant icon and wordmark, the version, what Emmet connects to, and the trademark line (D-30)." },
    { id: "export", title: "Export", kind: "workspace", render: "cottage", covers: [], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stage3, S.finish), drawer: "parts", overlay: "export" },
    { id: "import", title: "Open an existing LDraw file", kind: "workspace", render: "cottage", covers: [], imported: true, build: { state: "built", parts: 163 }, agent: { state: "idle", line: "Idle" }, chat: [], banner: "imported", drawer: "steps", on: { parametric: "thinking" } },
    { id: "untrusted", title: "Untrusted project", kind: "workspace", render: "cottage", covers: ["Untrusted project"], build: { state: "readonly", parts: 163 }, agent: { state: "idle", line: "Read-only until you trust it" }, chat: [], banner: "untrusted", on: { trust: "built" } },
    { id: "offline", title: "Offline", kind: "workspace", render: "cottage", covers: ["Offline"], build: { state: "built", parts: 163, clean: true }, agent: { state: "offline", line: "Offline: the agent needs a connection" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stage3, S.finish), banner: "offline", drawer: "params" },
    { id: "old-library", title: "An old library version", kind: "workspace", render: "cottage", covers: ["An old library version"], build: { state: "built", parts: 163, clean: true }, agent: { state: "idle", line: "Idle" }, chat: story(S.prompt, S.answered, S.start, S.stage1, S.stage2, S.stage3, S.finish), banner: "old-library", on: { update: "built" } },
  ];

  // Every section 14 state must be covered by at least one screen.
  const missing = STATES.filter((s) => !screens.some((x) => x.covers.includes(s)));
  if (missing.length) throw new Error("states.js doesn't cover: " + missing.join(", "));
  return { STATES, FLOWS, screens, byId: Object.fromEntries(screens.map((s) => [s.id, s])) };
})();
