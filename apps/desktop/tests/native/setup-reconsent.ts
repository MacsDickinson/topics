import { browser, $, expect } from "@wdio/globals";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

describe("Native setup reconsent", () => {
  it("retains completed batches and requires an updated exact manifest before sending changed notes", async () => {
    const notes = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
    const log = process.env.TOPICS_TEST_SETUP_LOG!;
    await $("#root-path").waitForDisplayed();
    await $("#root-path").setValue(notes);
    await $("button=Use notes folder").click();
    await $("h1=Understand this folder").waitForDisplayed();
    await $("button=Review manifest").click();
    await $("label*=Authorize this exact manifest").$("input").click();
    await $("button=Build starting map").click();
    await $("h1=Starting map").waitForDisplayed();
    await browser.waitUntil(
      () => {
        try {
          return readFileSync(log, "utf8").trim().split("\n").length >= 2;
        } catch {
          return false;
        }
      },
      { timeout: 120000, timeoutMsg: "Second scripted turn did not start" },
    );
    await $("button=Pause").waitForDisplayed();
    await $("button=Pause").click();
    await $("button=Resume").waitForDisplayed();
    const before = await browser.execute(async () => {
      const core = (window as any).__TAURI__.core;
      const root = (await core.invoke("workspace_root_get", { request: {} }))
        .root;
      return {
        root,
        job: (
          await core.invoke("setup_jobs_list", {
            request: { root_id: root.root_id },
          })
        )[0],
      };
    });
    expect(before.job.coverage.processed.length).toEqual(20);
    writeFileSync(
      join(notes, "meetings/040.md"),
      "# Synthetic Alex A42\n\nAlex A42 leads Ledger; changed locally.\n",
    );
    writeFileSync(join(notes, "meetings/new.md"), "# New synthetic note\n");
    writeFileSync(process.env.TOPICS_TEST_MODE_FILE!, "graph");
    await $("button=Resume").click();
    await $("button=Preview updated manifest").waitForDisplayed({
      timeout: 120000,
    });
    await $("button=Preview updated manifest").click();
    await browser.waitUntil(
      async () => (await $(".consent").getText()).includes("changed bytes"),
      { timeout: 15000 },
    );
    expect(await $(".consent").getText()).not.toContain(
      "meetings/new.md — new selection",
    );
    await $("label*=Authorize this updated manifest").$("input").click();
    await $("button=Authorize and resume").click();
    await browser.waitUntil(
      async () =>
        (await $(".run-panel [role=status]").getText()).startsWith(
          "Selected notes analysed",
        ),
      { timeout: 120000 },
    );
    const after = await browser.execute(
      async (rootId, jobId) =>
        (window as any).__TAURI__.core.invoke("setup_job_get", {
          request: { root_id: rootId, job_id: jobId },
        }),
      before.root.root_id,
      before.job.job_id,
    );
    expect(after.coverage.processed.length).toEqual(41);
    expect(after.coverage.transmission_authorizations).toEqual(2);
    expect(after.prior_consents.length).toEqual(1);
    const turns = readFileSync(log, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    const paths = turns.flatMap((turn) =>
      turn.context.sources.map((source: any) => source.summary.path),
    );
    for (const path of before.job.coverage.processed)
      expect(paths.filter((entry) => entry === path).length).toEqual(1);
    expect(paths).not.toContain("meetings/new.md");
  });
});
