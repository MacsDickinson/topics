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
  await page
    .getByRole("button", { name: "Week: Select date" })
    .scrollIntoViewIfNeeded();
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
  await page
    .getByRole("button", { name: "Week: Select date" })
    .scrollIntoViewIfNeeded();
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
  expect(calls.indexOf("pilot_feedback_snapshot_create")).toBeLessThan(
    calls.indexOf("pilot_feedback_save"),
  );
});

test("unsaved briefing topic explains how to continue and preserves feedback", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true });
  await page.getByRole("button", { name: "Correct Ledger" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("The owner needs correction.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await page
    .getByRole("button", { name: "Review scoped correction for Ledger" })
    .click();
  await expect(page.getByText(/not a saved topic yet/)).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Open folder setup" }),
  ).toBeVisible();
  expect(await page.evaluate(() => (window as any).pilotFeedback)).toHaveLength(
    1,
  );
});

test("weekly phase uses the accessible Board UI selector inside Field", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true });
  await page.getByText("Weekly review", { exact: true }).first().click();
  await page.getByRole("button", { name: "Week: Select date" }).click();
  await page.getByLabel("Week date (YYYY-MM-DD)").fill("2026-11-06");
  await page.getByRole("button", { name: "Apply" }).click();
  await page.getByLabel("Project", { exact: true }).fill("Ledger");
  await page.getByRole("button", { name: "Open weekly review" }).click();
  const control = page.getByRole("button", { name: "Phase" });
  await expect(control).toBeVisible();
  await expect(
    page.locator(".board-select").filter({ has: control }),
  ).toHaveCount(1);
  await control.scrollIntoViewIfNeeded();
  await control.click();
  await page.getByRole("option", { name: "Baseline" }).click();
  await expect(control).toContainText("Baseline");
});

test("superseded briefing item offers history without losing feedback", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true, populated: true });
  await page.getByRole("button", { name: "Correct Ledger" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("The previous value was wrong.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await page.evaluate(() => {
    (window as any).claims[0].revision_id = "new-revision";
  });
  await page
    .getByRole("button", { name: "Review scoped correction for Ledger" })
    .click();
  await expect(
    page.getByText(/original statement is no longer current/),
  ).toBeVisible();
  await page.getByRole("button", { name: "Open history" }).click();
  await expect(
    page.getByRole("heading", { name: "History", exact: true }),
  ).toBeVisible();
  expect(await page.evaluate(() => (window as any).pilotFeedback)).toHaveLength(
    1,
  );
});

test("reviews two material surfaced changes and a reported omission without retyping", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true, pilotTwoItems: true });
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("Approval was required.");
  await page.getByLabel("Evidence at briefing time").selectOption("analysed");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await page.getByRole("button", { name: "Back to Today" }).click();
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("The decision was new.");
  await page
    .getByLabel("Evidence at briefing time")
    .selectOption("first_learned");
  await page.getByRole("button", { name: "Save feedback" }).click();
  expect(
    await page.evaluate(
      () =>
        new Set((window as any).pilotFeedback.map((f: any) => f.snapshot_id))
          .size,
    ),
  ).toBe(1);
  await page.getByRole("button", { name: "Keep briefing for meeting" }).click();
  await page.getByText("Weekly review", { exact: true }).first().click();
  await page
    .getByRole("button", { name: "Week: Select date" })
    .scrollIntoViewIfNeeded();
  await page.getByRole("button", { name: "Week: Select date" }).click();
  await page.getByLabel("Week date (YYYY-MM-DD)").fill("2026-11-06");
  await page.getByRole("button", { name: "Apply" }).click();
  await page.getByLabel("Project", { exact: true }).fill("Ledger");
  await page.getByRole("button", { name: "Open weekly review" }).click();
  await page
    .getByLabel("Meeting briefing", { exact: true })
    .selectOption("snapshot-one");
  await page
    .getByRole("button", { name: "Count Ledger launch as material" })
    .click();
  await page
    .getByRole("button", { name: "Count Ledger owner as material" })
    .click();
  await expect(
    page.getByRole("button", { name: "Count Ledger launch as material" }),
  ).toBeDisabled();
  await expect(page.getByText("Approval was required.").last()).toBeVisible();
  await expect(page.getByLabel("Total preparation minutes")).toHaveValue("");
  await page.getByRole("button", { name: "Save weekly review" }).click();
  await expect(page.getByText("Known in-scope omissions: 1 / 3")).toBeVisible();
  expect(
    await page.evaluate(() => (window as any).pilotWeek.total_minutes),
  ).toBeNull();
});

test("keeps a sentence across a stale briefing and requires a fresh review", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true });
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("A material decision was missed.");
  await page.evaluate(() => {
    (window as any).pilotBriefing.briefing_token = "changed-token";
  });
  await page.getByRole("button", { name: "Save feedback" }).click();
  await expect(page.getByRole("alert")).toContainText("refresh and review");
  await expect(
    page.getByRole("button", { name: "I reviewed the refreshed briefing" }),
  ).toHaveCount(0);
  await expect(page.getByLabel("What was wrong or missed?")).toHaveValue(
    "A material decision was missed.",
  );
  await page.getByRole("button", { name: "Refresh briefing" }).click();
  await page
    .getByRole("button", { name: "I reviewed the refreshed briefing" })
    .click();
  await page.getByRole("button", { name: "Save feedback" }).click();
  await expect(page.getByText("A material decision was missed.")).toBeVisible();
});

test("weekly measurements retain unknown and zero and reject invalid values", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true });
  await page.getByText("Weekly review", { exact: true }).first().click();
  await page.getByRole("button", { name: "Week: Select date" }).click();
  await page.getByLabel("Week date (YYYY-MM-DD)").fill("2026-11-06");
  await page.getByRole("button", { name: "Apply" }).click();
  await page.getByLabel("Project", { exact: true }).fill("Ledger");
  await page.getByRole("button", { name: "Open weekly review" }).click();
  await expect(page.getByLabel("Sources", { exact: true })).toHaveValue("");
  await page.getByLabel("Sources", { exact: true }).fill("0");
  await page.getByLabel("Total preparation minutes").fill("-1");
  await page.getByRole("button", { name: "Save weekly review" }).click();
  await expect(page.getByRole("alert")).toContainText(
    "nonnegative whole number",
  );
  expect(await page.evaluate(() => (window as any).pilotWeek)).toBeNull();
  await page.getByLabel("Total preparation minutes").fill("");
  await page.getByRole("button", { name: "Save weekly review" }).click();
  const week = await page.evaluate(() => (window as any).pilotWeek);
  expect(week.source_count).toBe(0);
  expect(week.total_minutes).toBeNull();
  expect(week.decision_count).toBeNull();
});

test("midweek feedback is a separate view until the meeting is kept", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true, pilotTwoItems: true });
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("Approval was missed midweek.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await expect(
    page.getByRole("heading", { name: "Saved feedback view" }),
  ).toBeVisible();
  expect(
    await page.evaluate(() =>
      (window as any).calls.some(
        (c: any) => c.command === "pilot_feedback_snapshot_create",
      ),
    ),
  ).toBe(true);
  await expect(page.getByText(/not a kept meeting/i)).toBeVisible();
  await page.getByRole("button", { name: "Keep briefing for meeting" }).click();
  await expect(
    page.getByRole("heading", { name: "Saved meeting briefing" }),
  ).toBeVisible();
  await expect(page.getByText("Approval was missed midweek.")).toBeVisible();
  await page.getByRole("button", { name: "Edit feedback" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("Approval was missed before the meeting.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  const entry = await page.evaluate(() => (window as any).pilotFeedback[0]);
  expect(entry.snapshot_id).toBe("feedback-one");
  await page.getByText("Weekly review", { exact: true }).first().click();
  await page
    .getByRole("button", { name: "Week: Select date" })
    .scrollIntoViewIfNeeded();
  await page.getByRole("button", { name: "Week: Select date" }).click();
  await page.getByLabel("Week date (YYYY-MM-DD)").fill("2026-11-06");
  await page.getByRole("button", { name: "Apply" }).click();
  await page.getByLabel("Project", { exact: true }).fill("Ledger");
  await page.getByRole("button", { name: "Open weekly review" }).click();
  await expect(
    page
      .getByLabel("Meeting briefing", { exact: true })
      .locator('option[value="feedback-one"]'),
  ).toHaveCount(0);
  await page
    .getByLabel("Meeting briefing", { exact: true })
    .selectOption("snapshot-one");
  await expect(
    page.getByText("2 briefing items still need a materiality decision."),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Count Ledger launch as material" })
    .click();
  await expect(
    page.getByText("1 briefing item still needs a materiality decision."),
  ).toBeVisible();
});

test("a meeting retains midweek correction provenance when the original item is absent", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true, populated: true });
  await page.getByRole("button", { name: "Correct Ledger" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("The launch is Monday.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await page.getByRole("button", { name: "Back to Today" }).click();
  await page.evaluate(() => {
    (window as any).pilotBriefing.briefing_token = "new-meeting-token";
    (window as any).pilotBriefing.items = [];
  });
  await page.getByRole("button", { name: "Refresh briefing" }).click();
  await page.getByRole("button", { name: "Keep briefing for meeting" }).click();
  await expect(page.getByText("The launch is Monday.")).toBeVisible();
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
  expect(request.revision_ids).toEqual([
    "00000000-0000-4000-8000-000000000007",
  ]);
  expect(request.rule_scope.source_paths).toEqual(["Meetings/one.md"]);
});

test("reused identical feedback view can be kept with the latest observed token", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true });
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("An omission was reported.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await page.getByRole("button", { name: "Back to Today" }).click();
  await page.evaluate(() => {
    (window as any).pilotBriefing.briefing_token = "new-identical-token";
  });
  await page.getByRole("button", { name: "Refresh briefing" }).click();
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("A second omission was reported.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await expect(
    page.getByRole("button", { name: "Keep briefing for meeting" }),
  ).toBeEnabled();
  await page.getByRole("button", { name: "Keep briefing for meeting" }).click();
  const request = await page.evaluate(
    () =>
      (window as any).calls
        .filter((c: any) => c.command === "pilot_snapshot_create")
        .at(-1).args.request,
  );
  expect(request.briefing_token).toBe("new-identical-token");
  await expect(
    page.getByRole("heading", { name: "Saved meeting briefing" }),
  ).toBeVisible();
});

test("an older feedback view cannot be kept over changed evidence", async ({
  page,
}) => {
  await workspaceFixture(page, { pilot: true });
  await page.getByRole("button", { name: "Topics missed something" }).click();
  await page
    .getByLabel("What was wrong or missed?")
    .fill("An earlier omission.");
  await page.getByRole("button", { name: "Save feedback" }).click();
  await page.getByRole("button", { name: "Back to Today" }).click();
  await page.evaluate(() => {
    (window as any).pilotBriefing.briefing_token = "changed-token";
    (window as any).pilotBriefing.items[0].value = "Monday";
  });
  await page.getByRole("button", { name: "Refresh briefing" }).click();
  await page
    .getByRole("button", { name: /Open briefing \(feedback view\)/ })
    .click();
  await expect(
    page.getByRole("button", { name: "Keep briefing for meeting" }),
  ).toBeDisabled();
  await expect(page.getByText(/Return to Today and refresh/)).toBeVisible();
});
