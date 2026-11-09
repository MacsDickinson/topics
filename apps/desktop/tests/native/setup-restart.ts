import { browser, $, expect } from "@wdio/globals";

describe("Synthetic setup restart", () => {
  it("recovers the saved map and correction from app-owned data", async () => {
    await $('button[aria-label="Topics"]').waitForDisplayed();
    await $('button[aria-label="Topics"]').click();
    await $('button[aria-label="Synthetic Alex A42"]').waitForDisplayed();
    await $('button[aria-label="Synthetic Alex A42"]').click();
    await browser.waitUntil(
      async () =>
        (await $(".context-review").getText()).includes("Confirmed by you"),
      { timeout: 30000 },
    );
    const text = await $(".context-review").getText();
    expect(text).toContain("they/them");
  });
});
