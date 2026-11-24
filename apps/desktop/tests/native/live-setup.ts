import { openTool } from "./interaction";
import { browser, $, $$, expect } from "@wdio/globals";
import { writeFileSync } from "node:fs";
import { baseline, corrected, verifyStatementRule, snapshot, readRule, findClaimArticle } from "../../scripts/live-setup-lineage.mjs";
import { waitForTerminal } from "../../scripts/live-setup-observation.mjs";

describe(process.env.TOPICS_TEST_REHEARSAL === "1" ? "Scripted synthetic setup rehearsal through native services" : "Genuine Codex synthetic setup through native services", () => {
  it(
    "authorizes synthetic notes, saves once and corrects a sourced claim",
    async () => {
      const path = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
      const artifact = process.env.TOPICS_TEST_ARTIFACT_PREFIX || "evidence/native-live-setup";
      await $("#root-path").waitForDisplayed();
      await $("#root-path").setValue(path);
      await $("button=Use notes folder").click();
      await $("button=Analyse sources").click();
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
      await $('button=Knowledge graph').click();
      await openTool("Set up folder");
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
      const getState = async () => browser.execute(async () => {
        const core = (window as any).__TAURI__.core;
        const root = (await core.invoke("workspace_root_get", { request: {} })).root;
        const jobs = await core.invoke("setup_jobs_list", { request: { root_id: root.root_id } });
        return { root, job: jobs[0] };
      });
      let state: any;
      await waitForTerminal(async () => { state = await getState(); return state.job; }, (observation: unknown) => {
        writeFileSync(process.env.TOPICS_TEST_OBSERVATION!, JSON.stringify(observation, null, 2));
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

      const claims = state.job.graph.items.filter((item: any) => item.candidate.kind === "claim" && item.candidate.claim.sources.some((source: any) => source.kind === "passage"));
      expect(claims.length).toBeGreaterThan(0);
      const claim = claims[0].candidate.claim;
      const topic = state.job.graph.items.find((item: any) => item.candidate.kind === "topic" && item.candidate.topic.id === claim.subject_id);
      expect(topic).toBeTruthy();
      const topicName = topic.candidate.topic.name;
      const selected = { topicId: claim.subject_id, logicalId: claim.id, revisionId: claim.revision_id, predicate: claim.predicate, value: claim.value };
      const replacement = "updated after review";
      const statementText = "Synthetic correction: the value is updated after review.";
      const readProjection = async () => browser.execute(async (rootId, topicId) => {
        const core = (window as any).__TAURI__.core;
        return core.invoke("setup_topic_review", { request: { root_id: rootId, topic_id: topicId } });
      }, state.root.root_id, selected.topicId);
      const readStatement = async (statementId: string) => browser.execute(async (rootId, statementId) => {
        return (window as any).__TAURI__.core.invoke("workspace_statement_get", { request: { root_id: rootId, statement_id: statementId } });
      }, state.root.root_id, statementId);
      await browser.saveScreenshot(`${artifact}-draft.png`);
      await $("button=Knowledge graph").click();
      await $(`button[aria-label="${topicName}"]`).waitForDisplayed();
      await expect($(`button[aria-label="${topicName}"]`)).toHaveText(expect.stringContaining("Pending"));
      await $("button=Open discovery").click();
      await $("button=Review starting map").click();
      await $("h2=Save starting map").waitForDisplayed();
      await browser.saveScreenshot(`${artifact}-receipt.png`);
      await $("button=Save starting map").click();
      await browser.waitUntil(async () => !(await $("h2=Save starting map").isExisting()), { timeout: 30000 });
      writeFileSync(process.env.TOPICS_TEST_EXPECTED!, JSON.stringify({topicName}));
      await $('button=Knowledge graph').click();
      await $(`button[aria-label="${topicName}"]`).waitForDisplayed();
      await $("#topic-search").setValue(topicName);
      const hit = await $(".graph-hits button");
      await hit.waitForDisplayed();
      await hit.click();
      await expect($(`button[aria-label="${topicName}"]`)).toHaveAttribute("aria-pressed","true");
      if(await $(".review-tools summary").isExisting()) await $(".review-tools summary").click();
      await $(".context-review").waitForDisplayed();
      await browser.waitUntil(
        async () =>
          (await $(".context-review").getText()).includes("From your notes"),
        { timeout: 10000 },
      );
      const original = baseline(await readProjection(), selected);
      const claimArticle = await findClaimArticle($$(".context-review article"), claim.predicate, claim.value);
      expect(claimArticle).toBeTruthy();
      await claimArticle!.$("button=That’s wrong").click();
      await $(".context-review textarea").setValue(
        statementText,
      );
      await $(".context-review input").setValue(replacement);
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
      const beforeContext = await readProjection();
      const lineageBefore = corrected(beforeContext, selected, replacement, statementText);
      expect(lineageBefore.old).toEqual(original);
      const statementBefore = await readStatement(lineageBefore.statementId);
      const ruleBefore = readRule(path, lineageBefore.head.revision_id);
      verifyStatementRule(lineageBefore, statementBefore, ruleBefore, selected, replacement);
      const confirmedBefore = beforeContext.reviews.filter((entry: any) => entry.state === "confirmed_by_you").map((entry: any) => entry.record_id).sort();
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
      const projection = await readProjection();
      expect(projection.context.map((entry: any) => entry.text)).toContain(
        "Synthetic context: role changed after review.",
      );
      expect(projection.reviews.filter((entry: any) => entry.state === "confirmed_by_you").map((entry: any) => entry.record_id).sort()).toEqual(confirmedBefore);
      const lineage = corrected(projection, selected, replacement, statementText);
      const statement = await readStatement(lineage.statementId);
      const rule = readRule(path, lineage.head.revision_id);
      verifyStatementRule(lineage, statement, rule, selected, replacement);
      const expected = snapshot(projection, lineage, statement, rule);
      writeFileSync(process.env.TOPICS_TEST_EXPECTED!, JSON.stringify({ topicName, selected, replacement, statementText, expected }));
      await browser.saveScreenshot(`${artifact}-reviewed.png`);
    },
  );
});
