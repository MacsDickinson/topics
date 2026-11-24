import { openTool } from "./interaction";
import { browser, $, expect } from "@wdio/globals";

describe("Synthetic setup restart", () => {
  it(
    process.env.TOPICS_TEST_GRAPH_ONLY === "1"
      ? "recovers the saved map from app-owned data"
      : "recovers the saved map and correction from app-owned data",
    async () => {
      await $("button=Knowledge graph").waitForDisplayed();
      await $("button=Knowledge graph").click();
      await $('button[aria-label="Synthetic Alex A42"]').waitForDisplayed();
      await $('button[aria-label="Synthetic Alex A42"]').click();
      if (process.env.TOPICS_TEST_GRAPH_ONLY === "1") {
        await expect($(".topic-detail h2")).toHaveText("Synthetic Alex A42");
        return;
      }
      await browser.waitUntil(
        async () =>
          (await $(".context-review").getText()).includes("Confirmed by you"),
        { timeout: 30000 },
      );
      const text = await $(".context-review").getText();
      expect(text).toContain("supports Ledger");
      expect(text).toContain("Synthetic context: role changed after review.");
    },
  );
  it("reopens the approved period judgment and restores the disputed passages through History", async function () {
    if (process.env.TOPICS_TEST_GRAPH_ONLY === "1") this.skip();
    await $("button=Knowledge graph").click();
    await $('button[aria-label="Orchard"]').click();
    await expect($(".topic-detail h2")).toHaveText("Orchard");
    await expect($(".context-review")).toHaveText(
      expect.stringContaining("Confirmed by you"),
    );
    await expect($(".validation-item")).not.toExist();
    await openTool("History");
    const judgment = await $(
      '(//article[contains(@class,"operation")][.//h2[text()="Apply exact setup review receipt"]])[last()]',
    );
    await expect(judgment).toHaveText(
      expect.stringContaining("Apply exact setup review receipt"),
    );
    await judgment.$("button=Undo").click();
    await $("button=Knowledge graph").click();
    await $('button[aria-label="Orchard"]').click();
    await $(".validation-item").waitForDisplayed();
    await expect($(".validation-item")).toHaveText(
      expect.stringContaining(
        "Sam owns the Orchard project from January through June.",
      ),
    );
    await expect($(".validation-item")).toHaveText(
      expect.stringContaining("Priya owns the Orchard project in June."),
    );
  });
});
