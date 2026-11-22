import { browser, $, expect } from "@wdio/globals";
import { writeFileSync } from "node:fs";

describe("Native setup failure explanation", () => {
  it("shows the safe model failure explanation and retries the failed batch", async () => {
    await $("#root-path").waitForDisplayed();
    await $("#root-path").setValue(process.env.TOPICS_TEST_WORKSPACE_ROOT!);
    await $("button=Use notes folder").click();
    await $("h1=Understand this folder").waitForDisplayed();
    await $("button=Review manifest").click();
    await $("label*=Authorize this exact manifest").$("input").click();
    await $("button=Build starting map").click();
    await $("h1=Starting map").waitForDisplayed();
    await browser.waitUntil(
      async () =>
        (await $(".run-panel").getText()).includes(
          "The model reported a failed turn.",
        ),
      { timeout: 120000 },
    );
    const text = await $(".run-panel").getText();
    expect(text).not.toContain("model_terminal_failed");
    expect(text).not.toContain("phase");
    writeFileSync(process.env.TOPICS_TEST_MODE_FILE!, "graph");
    await $("button=Retry failed").click();
    await browser.waitUntil(
      async () =>
        (await $(".run-panel [role=status]").getText()).includes(
          "Selected notes analysed",
        ),
      { timeout: 120000 },
    );
  });
});
