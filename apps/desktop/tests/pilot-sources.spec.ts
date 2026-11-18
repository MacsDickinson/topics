import { test, expect } from "@playwright/test";
import { workspaceFixture } from "./fixtures/workspace";

test("Friday feedback stays reachable when the briefing covers many notes", async ({ page }) => {
  await workspaceFixture(page, { pilot: true });
  await page.getByRole("button", { name: "Topics missed something" }).waitFor();
  await page.evaluate(() => {
    const briefing = (window as any).pilotBriefing;
    const source = briefing.sources[0];
    briefing.sources = Array.from({ length: 40 }, (_, index) => ({
      ...source, source_id: String(index), path: `Minutes/meeting-${index}.md`,
      status: index === 0 ? "pending" : "checked",
    }));
  });
  await page.getByRole("button", { name: "Refresh briefing" }).click();
  await expect(page.getByText("Awaiting batch authorization (1)", { exact: true })).toBeVisible();
  await expect(page.getByText("Minutes/meeting-39.md", { exact: true })).toBeHidden();
  await page.getByText("Analysed (39)", { exact: true }).click();
  await expect(page.getByText("Minutes/meeting-39.md", { exact: true })).toBeVisible();
});
