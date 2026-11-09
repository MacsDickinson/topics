import { browser, $, $$, expect } from "@wdio/globals";
import { readFileSync } from "node:fs";

describe("Synthetic setup job through native services", () => {
  it("authorizes one manifest, builds 100 notes in bounded turns, saves once and corrects a sourced claim", async () => {
    const path = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
    await $("#root-path").waitForDisplayed();
    await $("#root-path").setValue(path);
    await $("button=Use notes folder").click();
    await $("h1=Understand this folder").waitForDisplayed();
    await $("button=Review manifest").click();
    await $("h2=Review the exact manifest").waitForDisplayed();
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
        (await $(".run-panel [role=status]").getText()).startsWith("finished:"),
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
    expect(state.job.coverage.processed.length).toEqual(100);
    expect(state.job.coverage.model_calls).toEqual(5);
    expect(state.job.coverage.transmission_authorizations).toEqual(1);
    expect(
      readFileSync(process.env.TOPICS_TEST_SETUP_LOG!, "utf8")
        .trim()
        .split("\n").length,
    ).toEqual(5);
    await $("button=Review starting map").click();
    await $("h2=Save starting map").waitForDisplayed();
    await $("button=Save starting map").click();
    await browser.waitUntil(
      async () => !(await $("h2=Save starting map").isExisting()),
      { timeout: 30000 },
    );
    await $('button[aria-label="Topics"]').click();
    await $('button[aria-label="Synthetic Alex A42"]').waitForDisplayed();
    await $('button[aria-label="Synthetic Alex A42"]').click();
    await $(".context-review").waitForDisplayed();
    await browser.waitUntil(
      async () =>
        (await $(".context-review").getText()).includes("From your notes"),
      { timeout: 10000 },
    );
    await $(".context-review button=That’s wrong").click();
    await $(".context-review textarea").setValue(
      "Synthetic Alex uses they/them.",
    );
    await $(".context-review input").setValue("they/them");
    await $("button=Preview correction").click();
    await $("h3=Review correction").waitForDisplayed();
    await $("button=Approve correction").click();
    await browser.waitUntil(
      async () =>
        (await $(".context-review").getText()).includes("Confirmed by you"),
      { timeout: 30000 },
    );
    expect((await $$(".context-review article")).length).toBeGreaterThan(0);
  });
});
