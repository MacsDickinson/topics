import { browser, $, expect } from "@wdio/globals";

describe("Synthetic setup restart", () => {
  it(
    process.env.TOPICS_TEST_GRAPH_ONLY === "1"
      ? "recovers the saved map from app-owned data"
      : "recovers the saved map and correction from app-owned data",
    async () => {
      await $('button[aria-label="Topics"]').waitForDisplayed();
      await $('button[aria-label="Topics"]').click();
      await $('button[aria-label="Synthetic Alex A42"]').waitForDisplayed();
      await $('button[aria-label="Synthetic Alex A42"]').click();
      if (process.env.TOPICS_TEST_GRAPH_ONLY === "1") {
        await expect($(".topic-detail h1")).toHaveText("Synthetic Alex A42");
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
});
