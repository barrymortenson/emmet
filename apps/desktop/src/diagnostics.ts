// SPDX-License-Identifier: AGPL-3.0-or-later
// Copy diagnostics (Help's and Settings'): the host's facts (versions, the library, the agents), then what the window
// alone knows: what draws it and how fast it has been. A bug report from a slow machine says so in numbers, and
// section 13's budgets can be read off any machine, an installed build included (5B). Nothing here is measured for
// the copy: these are timings the window already had.
import type { Viewer } from "@emmet/viewer";
import { commands } from "./bindings";

/** What the window has seen, kept up by App.tsx. */
export const seen: {
  viewer: Viewer | null;
  /** Milliseconds from launch to the first frame after setup. */
  startupMs: number | null;
  /** The last build put on screen: its parts, the host's run and check, and the viewer's load or diff to the frame. */
  shown: { parts: number; runMs: number | null; drawMs: number; first: boolean } | null;
} = { viewer: null, startupMs: null, shown: null };

export async function diagnosticsText(): Promise<string> {
  const base = await commands.diagnostics();
  const s = seen.viewer?.stats();
  if (!s) return base;
  const lines = [
    `Graphics: ${s.gpu} (${s.quality} quality, ${s.pixelRatio} pixels a point)`,
    seen.startupMs != null && `Start: ready ${Math.round(seen.startupMs)} ms after launch`,
    seen.shown &&
      `Last build shown: ${seen.shown.parts} parts${seen.shown.first ? ", the project's first load" : ""}, run and check ${seen.shown.runMs ?? "?"} ms, drawn in ${Math.round(seen.shown.drawMs)} ms`,
    s.movingFrameMs > 0 && `Last motion: ${s.movingFrameMs.toFixed(1)} ms a frame, ${s.drawCalls} draw calls`,
  ];
  return `${base}${lines.filter(Boolean).join("\n")}\n`;
}
