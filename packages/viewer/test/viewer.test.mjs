// SPDX-License-Identifier: AGPL-3.0-or-later
// Runs against the built package (dist/), the way the app will import it.
import assert from "node:assert/strict";
import { test } from "node:test";
import { createViewer } from "@emmet/viewer";

test("createViewer returns a viewer that can be disposed", () => {
  const viewer = createViewer({});
  assert.equal(typeof viewer.dispose, "function");
  viewer.dispose();
});
