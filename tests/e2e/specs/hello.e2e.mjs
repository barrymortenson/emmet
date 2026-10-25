// SPDX-License-Identifier: AGPL-3.0-or-later
// The E2E hello-world (ADR 0002), on 1B's window: the real app starts with no project, says so, and shows its version.
import { $, expect } from "@wdio/globals";

describe("the app window", () => {
  it("starts with no project and shows its version", async () => {
    await expect($("h1")).toHaveText("Emmet");
    await expect($("#status")).toHaveText("No project open");
    await expect($(".versions")).toHaveText(expect.stringContaining("Emmet 0.1.0"));
    await expect($(".empty")).toBeDisplayed();
  });
});
