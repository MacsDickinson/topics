import { test, expect } from "@playwright/test";
import { workspaceFixture, ids } from "./fixtures/workspace";

test("connecting a new folder opens local inventory before analysis", async ({
  page,
}) => {
  await workspaceFixture(page, { empty: true });
  await page.getByLabel("Notes folder path").fill("/synthetic/new-notes");
  await page.getByRole("button", { name: "Use notes folder" }).click();
  await expect(
    page.getByRole("heading", { name: "Understand this folder" }),
  ).toBeVisible();
  await expect(
    page.getByText("Suggested role", { exact: false }).first(),
  ).toBeVisible();
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter(
        (c: any) => c.command === "workspace_analysis_start",
      ),
    ),
  ).toHaveLength(0);
});

async function setupJob(page: import("@playwright/test").Page) {
  await page.getByRole("button", { name: "Set up folder" }).click();
  await expect(
    page.getByRole("heading", { name: "Choose notes to analyse" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Review manifest" }).click();
  await expect(
    page.getByText("Meetings/private.md: Local-only").first(),
  ).toBeVisible();
  await page.getByLabel("Authorize this exact manifest for this job").check();
  await page.getByRole("button", { name: "Build starting map" }).click();
  await expect(
    page.getByRole("heading", { name: "Starting map" }),
  ).toBeVisible();
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter(
        (c: any) => c.command === "setup_job_create",
      ),
    ),
  ).toHaveLength(1);
}

test("setup offers tree selection and an exact manifest before building", async ({
  page,
}) => {
  await workspaceFixture(page);
  await page.getByRole("button", { name: "Set up folder" }).click();
  await expect(
    page.getByRole("heading", { name: "Choose notes to analyse" }),
  ).toBeVisible();
  await expect(page.getByRole("list", { name: "Notes folders" })).toBeVisible();
  await page.screenshot({
    path: "evidence/folder-setup-light.png",
    fullPage: true,
  });
  await page.emulateMedia({ colorScheme: "dark" });
  await page.screenshot({
    path: "evidence/folder-setup-dark.png",
    fullPage: true,
  });
  await page.setViewportSize({ width: 480, height: 800 });
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: "evidence/folder-setup-narrow.png",
    fullPage: true,
  });
  await page.getByRole("button", { name: "Review manifest" }).click();
  await expect(
    page.getByRole("button", { name: "Build starting map" }),
  ).toBeDisabled();
  await page.getByLabel("Authorize this exact manifest for this job").check();
  await page.getByLabel("Through date").fill("2026-11-04");
  await expect(
    page.getByRole("heading", { name: "Review the exact manifest" }),
  ).toHaveCount(0);
});

test("topic map offers graph and list navigation", async ({ page }) => {
  await workspaceFixture(page);
  await setupJob(page);
  await page.getByRole("button", { name: "Graph", exact: true }).click();
  await expect(
    page.getByRole("group", { name: /Draft topic map/ }),
  ).toBeVisible();
  await expect(
    page.getByText("Inferred association · Needs your input"),
  ).toBeVisible();
  await page.screenshot({
    path: "evidence/folder-setup-graph.png",
    fullPage: true,
  });
  await page.getByRole("button", { name: "Open Ledger", exact: true }).focus();
  await page.keyboard.press("Enter");
  await expect(page.getByText("launch: Friday")).toBeVisible();
  await expect(
    page.getByText("Ledger launch is Friday", { exact: true }).first(),
  ).toBeHidden();
  await page.getByText("Sources (1)", { exact: true }).first().click();
  await expect(
    page.getByText("Ledger launch is Friday", { exact: true }).first(),
  ).toBeVisible();
});

async function showDraft(page: import("@playwright/test").Page) {
  await page
    .getByRole("button", { name: "Analyse notes", exact: true })
    .click();
  await page.getByLabel("Folders", { exact: true }).fill("Meetings");
  await page.getByRole("button", { name: "Preview selection" }).click();
  await page
    .getByLabel("Allow these unknown-sensitivity notes for this run")
    .check();
  await page
    .getByRole("button", { name: "Start analysis", exact: true })
    .click();
  await page.evaluate(() => {
    const w = window as any;
    w.showProposal();
    w.emitRun(1, { kind: "completed" });
  });
  await page.getByRole("button", { name: /Inbox/ }).click();
}

test("draft map exposes one exact starting-map receipt without record checkboxes", async ({
  page,
}) => {
  await workspaceFixture(page);
  await setupJob(page);
  await expect(
    page.getByRole("button", { name: "Review starting map" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Review starting map" }).click();
  await expect(
    page.getByRole("heading", { name: "Save starting map" }),
  ).toBeVisible();
  await expect(page.getByText(/0 remain open/)).toBeVisible();
  await expect(
    page.getByRole("checkbox", { name: /^Include (topic|claim|connection)/ }),
  ).toHaveCount(0);
});

test("a saved topic offers contextual decisions and collapsed evidence", async ({
  page,
}) => {
  await workspaceFixture(page);
  await showDraft(page);
  await page.getByRole("button", { name: "Approve selected changes" }).click();
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  await page.getByRole("button", { name: "Ledger", exact: true }).click();
  await expect(
    page.getByText("From your notes", { exact: false }).first(),
  ).toBeVisible();
  for (const name of [
    "Confirm these 1 statements",
    "That’s wrong",
    "Add context",
    "Not now",
  ]) {
    await expect(
      page.getByRole("button", { name, exact: true }).first(),
    ).toBeVisible();
  }
  await expect(
    page.getByText("Ledger launch is Friday", { exact: true }).first(),
  ).toBeHidden();
});

test("folder chooser cancellation preserves the path and selection waits for explicit use", async ({
  page,
}) => {
  await workspaceFixture(page, { empty: true });
  await page.getByLabel("Notes folder path").fill("/synthetic/manual");
  await page.getByRole("button", { name: "Choose folder…" }).click();
  await expect(page.getByLabel("Notes folder path")).toHaveValue(
    "/synthetic/manual",
  );
  await page.evaluate(() => {
    (window as any).chosenRoot = "/synthetic/chosen";
  });
  await page.getByRole("button", { name: "Choose folder…" }).click();
  await expect(page.getByLabel("Notes folder path")).toHaveValue(
    "/synthetic/chosen",
  );
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter(
        (c: any) => c.command === "workspace_root_set",
      ),
    ),
  ).toHaveLength(0);
});

test("context is saved as the exact user statement without correction or confirmation", async ({
  page,
}) => {
  await workspaceFixture(page);
  await showDraft(page);
  await page.getByRole("button", { name: "Approve selected changes" }).click();
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  await page.getByRole("button", { name: "Ledger", exact: true }).click();
  await page.getByRole("button", { name: "Add context", exact: true }).click();
  await page.getByLabel("Your context").fill("The meeting was provisional.");
  await page.getByRole("button", { name: "Save context" }).click();
  await expect
    .poll(() => page.evaluate(() => (window as any).savedContext))
    .toBe("The meeting was provisional.");
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter((c: any) =>
        ["setup_correction_preview", "setup_decide"].includes(c.command),
      ),
    ),
  ).toHaveLength(0);
});

test("confirmation previews only the enumerated visible revisions", async ({
  page,
}) => {
  await workspaceFixture(page);
  await showDraft(page);
  await page.getByRole("button", { name: "Approve selected changes" }).click();
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  await page.getByRole("button", { name: "Ledger", exact: true }).click();
  await page
    .getByRole("button", { name: "Confirm these 1 statements" })
    .click();
  await expect(
    page.getByRole("heading", { name: "Confirm displayed statements" }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        (window as any).calls.find(
          (c: any) => c.command === "setup_confirm_preview",
        ).args.request.revision_ids,
    ),
  ).toEqual([ids.revision]);
});
