// SPDX-License-Identifier: AGPL-3.0-or-later
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export const repo = join(fileURLToPath(new URL(".", import.meta.url)), "..", "..", "..");
export const out = join(repo, "tests", "e2e", "out");
export const os = process.platform === "win32" ? "windows" : process.platform === "darwin" ? "macos" : process.platform;
export const bin = (name) => join(repo, "target", "debug", process.platform === "win32" ? `${name}.exe` : name);
export const appBinary = bin("emmet-desktop");
