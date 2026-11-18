import { test, expect } from "@playwright/test";
import { workspaceFixture } from "./fixtures/workspace";

test("opening a configured workspace starts a local briefing refresh and reports failures honestly", async ({
  page,
}) => {
  await workspaceFixture(page, { returning: true });
  await expect(
    page.getByRole("heading", { name: "Today", exact: true }),
  ).toBeVisible();
  await expect
    .poll(() =>
      page.evaluate(() =>
        (window as any).calls.some((c: any) => c.command === "pilot_briefing"),
      ),
    )
    .toBe(true);
  await expect(page.getByText(/briefing unavailable/i)).toBeVisible();
  await expect(
    page.getByText(/no new changes in analysed sources/i),
  ).toHaveCount(0);
});

test("keeps a sourced briefing, captures and reopens feedback, and records an honest weekly review", async ({
  page,
}) => {
  await workspaceFixture(page, {
    returning: true,
    pilot: true,
    populated: true,
  });
  await expect(
    page.getByText("Meetings/one.md", { exact: true }).first(),
  ).toBeVisible();
  await page.getByRole("button", { name: "Keep briefing for meeting" }).click();
  await expect(
    page.getByText(/Later source changes cannot rewrite/),
  ).toBeVisible();
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("The release needs approval.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await page.getByRole("button", { name: "Back to Today" }).click();
  await page.getByRole("button", { name: /Open briefing/ }).click();
  await expect(page.getByText("The release needs approval.")).toBeVisible();
  await page.getByRole("button", { name: "Edit feedback" }).click();
  await page.getByLabel("Evidence at briefing time").selectOption("analysed");
  await page.getByRole("button", { name: "Save feedback" }).click();
  expect(await page.evaluate(() => (window as any).pilotFeedback)).toHaveLength(
    1,
  );
  await page.getByText("Weekly review", { exact: true }).first().click();
  await page.getByRole("button", { name: "Week: Select date" }).scrollIntoViewIfNeeded();
  await page.getByRole("button", { name: "Week: Select date" }).click();
  await page.getByLabel("Week date (YYYY-MM-DD)").fill("2026-11-06");
  await page.getByRole("button", { name: "Apply" }).click();
  await page.getByLabel("Project", { exact: true }).fill("Ledger");
  await page.getByRole("button", { name: "Open weekly review" }).click();
  await expect(page.getByText("Known in-scope omissions: 1 / 3")).toBeVisible();
  await expect(page.getByText("Draft quality unavailable.")).toBeVisible();
  await page.getByLabel("Total preparation minutes").fill("25");
  await page.getByRole("button", { name: "Save weekly review" }).click();
  expect(
    await page.evaluate(() => (window as any).pilotWeek.total_minutes),
  ).toBe(25);
});

test("shows available draft measures from the synthetic fixture", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true, draftEnabled: true });
  await page.getByText("Weekly review", { exact: true }).first().click();
  await page.getByRole("button", { name: "Week: Select date" }).scrollIntoViewIfNeeded();
  await page.getByRole("button", { name: "Week: Select date" }).click();
  await page.getByLabel("Week date (YYYY-MM-DD)").fill("2026-11-06");
  await page.getByRole("button", { name: "Apply" }).click();
  await page.getByLabel("Project", { exact: true }).fill("Ledger");
  await page.getByRole("button", { name: "Open weekly review" }).click();
  await expect(
    page.getByText("Drafts: 1 unchanged, 1 edited, 1 wrong, 1 undecided."),
  ).toBeVisible();
});

test("a reported wrong item opens the existing scoped correction with its original revision", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true, populated: true });
  await page.getByRole("button", { name: "Keep briefing for meeting" }).click();
  await page.getByRole("button", { name: "Correct Ledger" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("The launch is Monday.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await page
    .getByRole("button", { name: "Review scoped correction for Ledger" })
    .click();
  await expect(page.getByLabel("Your statement")).toHaveValue(
    "The launch is Monday.",
  );
  await page.getByLabel("Replacement").fill("Monday");
  await page.getByRole("button", { name: "Preview correction" }).click();
  const request = await page.evaluate(
    () =>
      (window as any).calls.find(
        (c: any) => c.command === "setup_correction_preview",
      ).args.request,
  );
  expect(request).toMatchObject({
    topic_id: "00000000-0000-4000-8000-000000000002",
    revision_ids: ["00000000-0000-4000-8000-000000000007"],
    rule_scope: { source_paths: ["Meetings/one.md"] },
  });
});

test("names pending and unavailable sources and requires a fresh view after a stale snapshot", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true });
  await expect(
    page.getByRole("button", { name: "Keep briefing for meeting" }),
  ).toBeVisible();
  await page.evaluate(() => {
    (window as any).pilotBriefing.briefing_token = "new-token";
    (window as any).pilotBriefing.sources[0].status = "pending";
  });
  await page.getByRole("button", { name: "Keep briefing for meeting" }).click();
  await expect(page.getByRole("alert")).toContainText("refresh and review");
  await page.getByRole("button", { name: "Refresh briefing" }).click();
  await expect(page.getByText(/Awaiting batch authorization/)).toBeVisible();
  await expect(
    page.getByText(/No new changes in analysed sources/),
  ).toHaveCount(0);
  await page.evaluate(() => {
    (window as any).pilotBriefing.sources[0].status = "unavailable";
  });
  await page.getByRole("button", { name: "Refresh briefing" }).click();
  await expect(page.getByText(/Source unavailable/)).toBeVisible();
});

test("a miss can be reported directly from an empty Today briefing", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true });
  await page.evaluate(() => {
    (window as any).pilotBriefing.items = [];
  });
  await page.getByRole("button", { name: "Refresh briefing" }).click();
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("A decision was missed.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await expect(page.getByText("A decision was missed.")).toBeVisible();
  const calls = await page.evaluate(() =>
    (window as any).calls
      .filter((c: any) => c.command.startsWith("pilot_"))
      .map((c: any) => c.command),
  );
  expect(calls.indexOf("pilot_snapshot_create")).toBeLessThan(
    calls.indexOf("pilot_feedback_save"),
  );
});
