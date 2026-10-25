// SPDX-License-Identifier: AGPL-3.0-or-later
// Render mode (ADR 0005): a hidden window renders the project's last build through viewer.render, hands the PNG to
// the host, and the host writes it and exits.
import { createViewer } from "@emmet/viewer";
import { commands } from "./bindings";
import { base64, json, library, placements } from "./library";

export async function renderMode(): Promise<void> {
  try {
    const job = await commands.renderJob();
    if (!job) throw new Error("this window has no render job");
    const canvas = document.createElement("canvas");
    canvas.width = canvas.height = 64;
    document.body.append(canvas);
    const viewer = createViewer(canvas, { partSource: library, theme: "light" });
    await viewer.load(placements(job.scene));
    const { png, camera } = await viewer.render({
      view: job.view as Parameters<typeof viewer.setView>[0],
      size: job.size,
      highlight: job.highlight.length ? job.highlight : undefined,
    });
    await commands.renderDone(base64(png), json(camera));
  } catch (e) {
    await commands.renderFailed(e instanceof Error ? e.message : String(e));
  }
}
