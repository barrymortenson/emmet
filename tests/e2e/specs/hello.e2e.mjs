// SPDX-License-Identifier: AGPL-3.0-or-later
// The E2E hello-world (ADR 0002): the real app starts, shows its version, and responds to a click.
import { $, expect } from "@wdio/globals";

describe("the app window", () => {
  it("shows Emmet and greets after a click", async () => {
    await expect($("h1")).toHaveText("Emmet");
    await expect($(".version")).toHaveText(expect.stringContaining("Version "));
    await expect($("#greeting")).not.toBeExisting();
    await $("#hello").click();
    await expect($("#greeting")).toHaveText("Hello from Emmet");
  });
});
