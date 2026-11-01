// SPDX-License-Identifier: AGPL-3.0-or-later
// The preset "auto" quality picks from the GPU's name (session 4C): every user's default, so real renderer strings.
import assert from "node:assert/strict";
import { test } from "node:test";
import { presetFor } from "@emmet/viewer";

test("auto picks Fast on a software renderer, Balanced on integrated graphics, Best otherwise", () => {
  assert.equal(presetFor("Apple M2"), "best");
  assert.equal(presetFor("ANGLE (Intel, Intel(R) UHD Graphics 620 Direct3D11 vs_5_0 ps_5_0, D3D11)"), "balanced");
  assert.equal(presetFor("ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)"), "best");
  assert.equal(presetFor("ANGLE (AMD, AMD Radeon RX 6600 Direct3D11 vs_5_0 ps_5_0, D3D11)"), "best");
  assert.equal(presetFor("ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)"), "fast");
  assert.equal(presetFor("ANGLE (Microsoft, Microsoft Basic Render Driver Direct3D11 vs_5_0 ps_5_0, D3D11)"), "fast");
  assert.equal(presetFor("llvmpipe (LLVM 15.0.7, 256 bits)"), "fast");
});
