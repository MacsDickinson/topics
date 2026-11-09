import { readFileSync } from "node:fs";
import { browser, $, expect } from "@wdio/globals";

describe("Synthetic setup restart", () => {
  it(
    "recovers the saved map and correction from app-owned data",
    async () => {
      const { topicName } = JSON.parse(readFileSync(process.env.TOPICS_TEST_EXPECTED!, "utf8"));
      await $('button[aria-label="Topics"]').waitForDisplayed();
      await $('button[aria-label="Topics"]').click();
      await  $(`button[aria-label="${topicName}"]`).waitForDisplayed();
      await  $(`button[aria-label="${topicName}"]`).click();
      await browser.waitUntil(
        async () =>
          (await $(".context-review").getText()).includes("Confirmed by you"),
        { timeout: 30000 },
      );
      const text = await $(".context-review").getText();
      expect(text).toContain("updated after review");
      expect(text).toContain("Synthetic context: role changed after review.");
    },
  );
});
