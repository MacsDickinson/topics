import { readFileSync } from "node:fs";
import { afterRestart, readRule } from "../../scripts/live-setup-lineage.mjs";
import { browser, $, expect } from "@wdio/globals";

describe("Synthetic setup restart", () => {
  it(
    "recovers the saved map and correction from app-owned data",
    async () => {
      const { topicName, selected, replacement, statementText, expected } = JSON.parse(readFileSync(process.env.TOPICS_TEST_EXPECTED!, "utf8"));
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
      const { projection, statement } = await browser.execute(async (topicId, statementId) => {
        const core = (window as any).__TAURI__.core;
        const root = (await core.invoke("workspace_root_get", { request: {} })).root;
        const projection = await core.invoke("setup_topic_review", { request: { root_id: root.root_id, topic_id: topicId } });
        const statement = await core.invoke("workspace_statement_get", { request: { root_id: root.root_id, statement_id: statementId } });
        return { projection, statement };
      }, selected.topicId, expected.statement.id);
      const rule = readRule(process.env.TOPICS_TEST_WORKSPACE_ROOT!, expected.head.revision_id);
      afterRestart(projection, selected, replacement, statementText, statement, rule, expected);
    },
  );
});
