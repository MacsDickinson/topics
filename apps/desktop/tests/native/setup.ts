import { openTool } from "./interaction";
import { browser, $, $$, expect } from "@wdio/globals";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

describe("Synthetic setup job through native services", () => {
  it(
    process.env.TOPICS_TEST_GRAPH_ONLY === "1"
      ? "authorizes 100 notes and saves the graph once"
      : "authorizes 100 notes, saves once and corrects a sourced claim",
    async () => {
      const path = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
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
      await $("button=Knowledge graph").click();
      await openTool("Set up folder");
      await $("h2=Choose notes to analyse").waitForDisplayed();
      await $("button=Review manifest").click();
      await $("h2=Review the exact manifest").waitForDisplayed();
      await $(".consent").$("summary*=Local-only").click();
      expect(await $(".consent").getText()).toContain("private/private.md");
      const authorization = await $(
        'input[type=checkbox][aria-label="Authorize this exact manifest for this job"]',
      );
      // Native label associates the checkbox even without a redundant aria-label.
      if (await authorization.isExisting()) await authorization.click();
      else await $("label*=Authorize this exact manifest").$("input").click();
      await $("button=Build starting map").click();
      await $("h1=Starting map").waitForDisplayed();
      await browser.waitUntil(
        async () =>
          (await $(".run-panel [role=status]").getText()).startsWith(
            "Selected notes analysed",
          ),
        {
          timeout: 180000,
          timeoutMsg: "Synthetic job did not complete with full coverage",
        },
      );
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
      expect(state.job.coverage.processed.length).toEqual(100);
      expect(state.job.coverage.model_calls).toEqual(5);
      expect(state.job.coverage.transmission_authorizations).toEqual(1);
      expect(
        readFileSync(process.env.TOPICS_TEST_SETUP_LOG!, "utf8")
          .trim()
          .split("\n").length,
      ).toEqual(5);
      const transmitted = readFileSync(
        process.env.TOPICS_TEST_SETUP_LOG!,
        "utf8",
      );
      expect(transmitted).not.toContain("PRIVATE_SYNTHETIC_ONLY");
      expect(transmitted).not.toContain("private/private.md");
      await $("button=Knowledge graph").click();
      await $('button[aria-label="Synthetic Alex A42"]').click();
      await $("button=Review discovery").click();
      await $("button=Synthetic Alex A42").click();
      await $("button=Review topic group").click();
      await $("h2=Save topic group").waitForDisplayed();
      await $("button=Save topic group").click();
      await browser.waitUntil(
        async () => !(await $("h2=Save topic group").isExisting()),
        { timeout: 30000 },
      );
      const saved = await browser.execute(
        async (rootId, jobId) => {
          const core = (window as any).__TAURI__.core;
          return core.invoke("setup_job_get", {
            request: { root_id: rootId, job_id: jobId },
          });
        },
        state.root.root_id,
        state.job.job_id,
      );
      expect(
        saved.graph.items.every((item: any) => item.disposition === "saved"),
      ).toEqual(true);
      if (process.env.TOPICS_TEST_GRAPH_ONLY === "1") return;
      await $("button=Knowledge graph").click();
      await expect(
        $('button[aria-label="Synthetic Alex A42"]'),
      ).toHaveAttribute("aria-pressed", "true");
      await expect($(".topic-detail .topic-identity h2")).toHaveText(
        "Synthetic Alex A42",
      );
      await expect($("button=Review discovery")).not.toExist();
      await $(".context-review").waitForDisplayed();
      await browser.waitUntil(
        async () =>
          (await $(".context-review").getText()).includes("From your notes"),
        { timeout: 10000 },
      );
      await $(".context-review").$("button=That’s wrong").click();
      await $(".context-review textarea").setValue(
        "Synthetic Alex supports Ledger; Alex does not lead Ledger.",
      );
      await $(".context-review input").setValue("supports Ledger");
      await $("button=Preview correction").click();
      await $("h3=Review correction").waitForDisplayed();
      const previewText = await $(".context-review .consent").getText();
      expect(previewText).toContain("leads Ledger");
      expect(previewText).toContain("supports Ledger");
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
      const projection = await browser.execute(async (rootId) => {
        const core = (window as any).__TAURI__.core;
        const topics = (
          await core.invoke("workspace_topics_list", {
            request: { root_id: rootId },
          })
        ).topics;
        const topic = topics.find(
          (entry: any) => entry.name === "Synthetic Alex A42",
        );
        return core.invoke("setup_topic_review", {
          request: { root_id: rootId, topic_id: topic.id },
        });
      }, state.root.root_id);
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
  it("compares two cited ownership periods, defers and approves the reviewed scope", async function () {
    if (process.env.TOPICS_TEST_GRAPH_ONLY === "1") this.skip();
    const root = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
    writeFileSync(
      join(root, "ownership-plan.md"),
      "Sam owns the Orchard project from January through June.",
    );
    writeFileSync(
      join(root, "ownership-handoff.md"),
      "Priya owns the Orchard project in June.",
    );
    // Seed two accepted passages through the real source and proposal services.
    await browser.execute(async () => {
      const core = (window as any).__TAURI__.core;
      const { root } = await core.invoke("workspace_root_get", { request: {} });
      const invoke = (cmd: string, fields: Record<string, unknown> = {}) =>
        core.invoke(cmd, { request: { root_id: root.root_id, ...fields } });
      const topic = {
        id: crypto.randomUUID(),
        name: "Orchard",
        topic_kind: "project",
        aliases: [],
        sensitivity: "unknown",
      };
      const candidates: any[] = [{ kind: "topic", topic }];
      const { sources } = await invoke("workspace_sources_list");
      for (const [path, value, start] of [
        ["ownership-plan.md", "Sam", "2026-01-01"],
        ["ownership-handoff.md", "Priya", "2026-06-01"],
      ]) {
        const { citation } = await invoke("workspace_source_get", {
          source_id: sources.find((s: any) => s.path === path).source_id,
        });
        candidates.push({
          kind: "claim",
          claim: {
            id: crypto.randomUUID(),
            revision_id: crypto.randomUUID(),
            subject_id: topic.id,
            predicate: "owner",
            value,
            valid_from: start,
            valid_to: "2026-06-30",
            recorded_at: "2026-06-01T00:00:00Z",
            sources: [{ kind: "passage", source: citation }],
            supersedes: [],
            uncertainty: null,
          },
        });
      }
      const { proposal } = await invoke("workspace_manual_propose", {
        candidates,
      });
      await invoke("workspace_proposal_decide", {
        proposal_id: proposal.id,
        expected_fingerprint: proposal.fingerprint,
        decision: { kind: "approve", candidates: proposal.candidates },
      });
    });
    await $("button=Knowledge graph").click();
    await $('button[aria-label="Orchard"]').click();
    const check = await $(".validation-item");
    await expect(check).toHaveText(
      expect.stringContaining(
        "Sam owns the Orchard project from January through June.",
      ),
    );
    await expect(check).toHaveText(
      expect.stringContaining("Priya owns the Orchard project in June."),
    );
    await $("button=Defer").click();
    await $("button=Bring back").click();
    await $("button=Defer").waitForDisplayed();
    await expect($("button=Confirm Priya")).toBeDisabled();
    await $(".validation-item input[type=checkbox]").click();
    await $("button=Confirm Priya").waitForEnabled();
    await $("button=Confirm Priya").click();
    const receipt = await $('[aria-label="Judgment receipt"]');
    await receipt.waitForDisplayed();
    await expect(receipt).toHaveText(
      expect.stringContaining("2026-01-01 through 2026-06-30"),
    );
    await expect(receipt).toHaveText(
      expect.stringContaining("2026-06-01 through 2026-06-30"),
    );
    await $("button=Approve judgment").click();
    await expect($(".validation-item")).not.toExist();
    await expect($(".context-review")).toHaveText(
      expect.stringContaining("Confirmed by you"),
    );
  });
});
