import { readFileSync } from "node:fs";
import { browser, $, expect } from "@wdio/globals";

describe("Synthetic setup restart", () => {
  it(
    "recovers the saved map and correction from app-owned data",
    async () => {
      const { topicName, confirmedBefore } = JSON.parse(readFileSync(process.env.TOPICS_TEST_EXPECTED!, "utf8"));
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
      const projection = await browser.execute(async (topicName) => {
        const core = (window as any).__TAURI__.core;
        const root = (await core.invoke("workspace_root_get", { request: {} })).root;
        const topics = (await core.invoke("workspace_topics_list", { request: { root_id: root.root_id } })).topics;
        const topic = topics.find((entry: any) => entry.name === topicName);
        return core.invoke("setup_topic_review", { request: { root_id: root.root_id, topic_id: topic.id } });
      }, topicName);
      expect(projection.context.map((entry: any) => entry.text)).toContain("Synthetic context: role changed after review.");
      expect(projection.reviews.filter((entry: any) => entry.state === "confirmed_by_you").map((entry: any) => entry.record_id).sort()).toEqual(confirmedBefore);
    },
  );
});
