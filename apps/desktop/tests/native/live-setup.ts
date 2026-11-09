import { browser, $, $$, expect } from "@wdio/globals";
import { writeFileSync } from "node:fs";

describe("Genuine Codex synthetic setup through native services", () => {
  it(
    "authorizes synthetic notes, saves once and corrects a sourced claim",
    async () => {
      const path = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
      await $("#root-path").waitForDisplayed();
      await $("#root-path").setValue(path);
      await $("button=Use notes folder").click();
      await $("h1=Understand this folder").waitForDisplayed();
      await browser.execute(async () => {
        const core = (window as any).__TAURI__.core;
        const root = (await core.invoke("workspace_root_get", { request: {} }))
          .root;
        const sources = (
          await core.invoke("workspace_sources_list", {
            request: { root_id: root.root_id },
          })
        ).sources;
        const privateSource = sources.find(
          (source: any) => source.path === "private/private.md",
        );
        const detail = await core.invoke("workspace_source_get", {
          request: {
            root_id: root.root_id,
            source_id: privateSource.source_id,
          },
        });
        await core.invoke("workspace_sensitivity_set", {
          request: {
            root_id: root.root_id,
            target: {
              kind: "source",
              source_id: privateSource.source_id,
              locator: detail.citation.locator,
            },
            sensitivity: "local_only",
          },
        });
      });
      await $('button[aria-label="Topics"]').click();
      await $("button=Set up folder").click();
      await $("h2=Choose notes to analyse").waitForDisplayed();
      await $("button=Review manifest").click();
      await $("h2=Review the exact manifest").waitForDisplayed();
      expect(await $(".consent").getText()).toContain("private/private.md");
      const authorization = await $(
        'input[type=checkbox][aria-label="Authorize this exact manifest for this job"]',
      );
      // Native label associates the checkbox even without a redundant aria-label.
      if (await authorization.isExisting()) await authorization.click();
      else await $("label*=Authorize this exact manifest").$("input").click();
      await $("button=Build starting map").click();
      await $("h1=Starting map").waitForDisplayed();
      try { await browser.waitUntil(
        async () =>
          (await $(".run-panel [role=status]").getText()).startsWith(
            "Selected notes analysed",
          ),
        {
          timeout: 180000,
          timeoutMsg: "Synthetic job did not complete with full coverage",
        },
      );
      } catch (error) {
        const failed = await browser.execute(async () => { const core = (window as any).__TAURI__.core; const root = (await core.invoke("workspace_root_get", { request: {} })).root; return core.invoke("setup_jobs_list", { request: { root_id: root.root_id } }); });
        console.log("LIVE_SETUP_FAILURE", JSON.stringify(failed));
        throw error;
      }
      const state = await browser.execute(async () => {
        const core = (window as any).__TAURI__.core;
        const root = (await core.invoke("workspace_root_get", { request: {} }))
          .root;
        const jobs = await core.invoke("setup_jobs_list", {
          request: { root_id: root.root_id },
        });
        return { root, job: jobs[0] };
      });
      expect(state.job.coverage.complete).toEqual(true);
      expect(state.job.status).toEqual("finished");
      expect(
        state.job.coverage.excluded.some(
          (entry: any) => entry.path === "topics",
        ),
      ).toEqual(true);
      expect(
        state.job.coverage.excluded.some(
          (entry: any) => entry.path === "private/private.md",
        ),
      ).toEqual(true);
      expect(state.job.manifest.batches.flatMap((batch: any) => batch.sources).map((source: any) => source.source.path)).toEqual(["profiles/alex.md"]);
      expect(state.job.coverage.processed.length).toEqual(1);
      expect(state.job.coverage.transmission_authorizations).toEqual(1);
      console.log("LIVE_SETUP_JOB", JSON.stringify(state.job));
      const claims = state.job.graph.items.filter((item: any) => item.candidate.kind === "claim" && item.candidate.claim.sources.some((source: any) => source.kind === "passage"));
      expect(claims.length).toBeGreaterThan(0);
      const claim = claims[0].candidate.claim;
      const topic = state.job.graph.items.find((item: any) => item.candidate.kind === "topic" && item.candidate.topic.id === claim.subject_id);
      expect(topic).toBeTruthy();
      const topicName = topic.candidate.topic.name;
      await browser.saveScreenshot("evidence/native-live-setup-draft.png");
      await $("button=Review starting map").click();
      await $("h2=Save starting map").waitForDisplayed();
      await browser.saveScreenshot("evidence/native-live-setup-receipt.png");
      await $("button=Save starting map").click();
      await browser.waitUntil(async () => !(await $("h2=Save starting map").isExisting()), { timeout: 30000 });
      writeFileSync(process.env.TOPICS_TEST_EXPECTED!, JSON.stringify({topicName}));
      await $('button[aria-label="Topics"]').click();
      await $(`button[aria-label="${topicName}"]`).waitForDisplayed();
      await $(`button[aria-label="${topicName}"]`).click();
      await $(".context-review").waitForDisplayed();
      await browser.waitUntil(
        async () =>
          (await $(".context-review").getText()).includes("From your notes"),
        { timeout: 10000 },
      );
      await $(".context-review").$("button=That’s wrong").click();
      await $(".context-review textarea").setValue(
        "Synthetic correction: the value is updated after review.",
      );
      await $(".context-review input").setValue("updated after review");
      await $("button=Preview correction").click();
      await $("h3=Review correction").waitForDisplayed();
      const previewText = await $(".context-review .consent").getText();
      expect(previewText).toContain(claim.value);
      expect(previewText).toContain("updated after review");
      expect(previewText).toContain("Future rule");
      await $("button=Approve correction").click();
      await browser.waitUntil(
        async () =>
          (await $(".context-review").getText()).includes("Confirmed by you"),
        { timeout: 30000 },
      );
      expect((await $$(".context-review article")).length).toBeGreaterThan(0);
      await $(".context-review").$("button=Add context").click();
      await $(".context-review textarea").setValue(
        "Synthetic context: role changed after review.",
      );
      await $("button=Save context").click();
      await browser.waitUntil(
        async () =>
          (await $(".context-review").getText()).includes(
            "Synthetic context: role changed after review.",
          ),
        { timeout: 30000 },
      );
      const projection = await browser.execute(async (rootId, topicName) => {
        const core = (window as any).__TAURI__.core;
        const topics = (
          await core.invoke("workspace_topics_list", {
            request: { root_id: rootId },
          })
        ).topics;
        const topic = topics.find((entry: any) => entry.name === topicName);
        return core.invoke("setup_topic_review", {
          request: { root_id: rootId, topic_id: topic.id },
        });
      }, state.root.root_id, topicName);
      expect(projection.context.map((entry: any) => entry.text)).toContain(
        "Synthetic context: role changed after review.",
      );
      expect(
        projection.reviews.filter(
          (entry: any) => entry.state === "confirmed_by_you",
        ).length,
      ).toEqual(1);
    },
  );
});
