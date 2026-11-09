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
    await browser.execute(async () => {
      const trigger = document.querySelector('button[aria-label="From date: Select date"]') as HTMLElement;
      trigger.scrollIntoView({block: "center", behavior: "instant"});
      let previous = "", stable = 0;
      await new Promise<void>((resolve, reject) => {
        let frames = 0;
        const check = () => {
          const r = trigger.getBoundingClientRect();
          const current = [r.x, r.y, r.width, r.height, window.scrollX, window.scrollY].map(x => Math.round(x * 10)).join(",");
          stable = current === previous ? stable + 1 : 0;
          previous = current;
          if (stable >= 8) return resolve();
          if (++frames > 180) return reject(new Error("Date trigger did not settle"));
          requestAnimationFrame(check);
        };
        requestAnimationFrame(check);
      });
    });
    await $('button[aria-label="From date: Select date"]').click();
    await $(".board-date-chip").waitForDisplayed();
    await browser.keys(["Escape"]);
    await browser.waitUntil(async () => !(await $(".board-date-chip").isExisting()));
    expect(await browser.execute(async () => document.activeElement?.getAttribute("aria-label"))).toEqual("From date: Select date");
    await $('button[aria-label="From date: Select date"]').click();
    await $(".board-date-chip").waitForDisplayed();
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
