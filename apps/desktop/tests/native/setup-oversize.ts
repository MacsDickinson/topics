import { browser, $, expect } from "@wdio/globals";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const digest = (path: string) => createHash("sha256").update(readFileSync(path)).digest("hex");
describe("Synthetic native setup size and date regression", () => {
  it("filters undated and old large notes, explicitly excludes an eligible large note, and preserves privacy and bytes", async () => {
    const rootPath = process.env.TOPICS_TEST_WORKSPACE_ROOT!;
    const names = ["small.md", "old.md", "undated.md", "large.md", "private.md"];
    const before = names.map((name) => digest(join(rootPath, name)));
    await $("#root-path").waitForDisplayed();
    await $("#root-path").setValue(rootPath);
    await $("button=Use notes folder").click();
    await $("h1=Understand this folder").waitForDisplayed();
    const rootId = await browser.execute(async () => {
      const core = (window as any).__TAURI__.core;
      const root = (await core.invoke("workspace_root_get", { request: {} })).root;
      const sources = (await core.invoke("workspace_sources_list", { request: { root_id: root.root_id } })).sources;
      const source = sources.find((entry: any) => entry.path === "private.md");
      const detail = await core.invoke("workspace_source_get", { request: {root_id: root.root_id, source_id: source.source_id} });
      await core.invoke("workspace_sensitivity_set", { request: {root_id: root.root_id, target: {kind: "source", source_id: source.source_id, locator: detail.citation.locator}, sensitivity: "local_only"} });
      return root.root_id;
    });
    await $('button[aria-label="Topics"]').click();
    await $("button=Set up folder").click();
    await $('button[aria-label="From date: Select date"]').waitForExist();
    // Embedded driver click scrolls after activation, dismissing the nonmodal popover.
    // DOM activation isolates the native date/filter behavior; it is not a pointer test.
    await browser.execute(async () => (document.querySelector('button[aria-label="From date: Select date"]') as HTMLButtonElement).click());
    await browser.pause(500);
    await $(".board-date-chip").setValue("2026-01-01");
    await $("button=Apply").click();
    expect(await $(".notice").getText()).toContain("large.md");
    expect(await $(".notice").getText()).not.toContain("old.md");
    expect(await $(".notice").getText()).not.toContain("undated.md");
    expect(await $("button=Review manifest").isEnabled()).toEqual(false);
    await $("button=Exclude large.md from this run").click();
    expect(await $("button=Review manifest").isEnabled()).toEqual(true);
    await $("button=Review manifest").click();
    await $("h2=Review the exact manifest").waitForDisplayed();
    const manifestSources = await browser.execute(async () => Array.from(document.querySelectorAll(".consent > ul > li"), (entry) => entry.textContent));
    expect(manifestSources.length).toEqual(1);
    expect(manifestSources[0]).toContain("small.md");
    expect(manifestSources[0]).not.toContain("large.md");
    const state = await browser.execute(async (rootId) => {
      const core = (window as any).__TAURI__.core;
      return {inventory: await core.invoke("setup_inventory", {request: {root_id: rootId, roles: []}}), jobs: await core.invoke("setup_jobs_list", {request: {root_id: rootId}})};
    }, rootId);
    expect(state.inventory.excluded.some((entry: any) => entry.path === "private.md" && entry.reason.includes("Local-only"))).toEqual(true);
    expect(state.jobs.length).toEqual(0);
    expect(names.map((name) => digest(join(rootPath, name)))).toEqual(before);
  });
});
