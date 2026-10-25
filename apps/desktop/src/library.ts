// SPDX-License-Identifier: AGPL-3.0-or-later
// The viewer's part source: the LDraw library through the host's emmet-lib:// protocol (plan section 13).
import type { PartSource, Placement } from "@emmet/viewer";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { Scene, Value } from "./bindings";

interface Pack {
  parts: Record<string, string[]>;
  files: Record<string, string>;
  missing: Record<string, string[]>;
}

/** `emmet-lib://localhost/` on macOS, `http://emmet-lib.localhost/` on Windows. */
const base = () => convertFileSrc("", "emmet-lib");

export const library: PartSource = {
  // One request for every new part of a model; each part's files become one packed .mpd.
  async parts(ids) {
    const r = await fetch(`${base()}pack?parts=${encodeURIComponent(ids.join(","))}`);
    if (!r.ok) throw new Error(`the library answered ${r.status}`);
    const pack = (await r.json()) as Pack;
    const parts: Record<string, string> = {};
    for (const [id, names] of Object.entries(pack.parts)) {
      if (pack.missing[id]?.length) continue;
      parts[id] = names.map((n) => `0 FILE ${n}\n${pack.files[n] ?? ""}`).join("\n");
    }
    return { parts, missing: pack.missing };
  },
  async colors() {
    const r = await fetch(`${base()}LDConfig.ldr`);
    if (!r.ok) throw new Error("the library has no LDConfig.ldr; run emmet library install");
    return r.text();
  },
};

/** Base64 for a PNG on its way to the host. */
export function base64(bytes: Uint8Array): string {
  let s = "";
  for (let i = 0; i < bytes.length; i += 0x8000) s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(s);
}

/** A scene's placements for the viewer. The bindings type each matrix entry `number | null`, since serde writes a
 * non-finite f64 as null; emmet never writes one. */
export function placements(scene: Scene): Placement[] {
  return scene.placements as Placement[];
}

/** A value for a command that takes JSON. */
export function json(v: object): Value {
  return v as unknown as Value;
}
