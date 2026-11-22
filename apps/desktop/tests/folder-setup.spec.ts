import { test, expect } from "@playwright/test";
import { workspaceFixture, ids, source } from "./fixtures/workspace";

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
  await page.locator(".consent").getByText("Local-only (1)").click();
  await expect(page.locator(".consent").getByText("Meetings/private.md: Local-only")).toBeVisible();
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
  await page.getByRole("button", { name: "Through date: Select date" }).click();
  await page.getByLabel("Through date date (YYYY-MM-DD)").fill("2026-11-04");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(
    page.getByRole("heading", { name: "Review the exact manifest" }),
  ).toHaveCount(0);
});

test("selected completion stays distinct from excluded sources", async ({
  page,
}) => {
  await workspaceFixture(page);
  await setupJob(page);
  await expect(page.getByRole("status")).toContainText(
    "Selected notes analysed",
  );
  await expect(page.getByRole("status")).toContainText("1 of 1 selected notes");
  await expect(
    page.getByText("1 excluded source", { exact: false }).first(),
  ).toBeVisible();
  await expect(page.getByText(/entire folder fully analysed/i)).toHaveCount(0);
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

test("job watch ignores foreign and older snapshots and displays a newer matching update", async ({
  page,
}) => {
  await workspaceFixture(page);
  await setupJob(page);
  await expect
    .poll(() =>
      page.evaluate(() =>
        (window as any).calls.some((c: any) => c.command === "setup_job_watch"),
      ),
    )
    .toBe(true);
  await page.evaluate(() => {
    const w = window as any;
    for (const payload of [
      {
        ...structuredClone(w.setupJob),
        root_id: "other-root",
        revision: 50,
        status: "paused",
      },
      { ...structuredClone(w.setupJob), revision: 0, status: "paused" },
    ])
      for (const fn of Object.values(w.listeners))
        (fn as Function)({ event: "setup_job_event", id: 2, payload });
  });
  await expect(page.getByRole("status")).toContainText(
    "Selected notes analysed",
  );
  await page.evaluate(() => {
    const w = window as any;
    const payload = {
      ...structuredClone(w.setupJob),
      revision: 2,
      status: "finished_with_gaps",
      coverage: {
        ...w.setupJob.coverage,
        complete: false,
        failed: ["Meetings/one.md"],
      },
    };
    for (const fn of Object.values(w.listeners))
      (fn as Function)({ event: "setup_job_event", id: 2, payload });
  });
  await expect(page.getByRole("status")).toContainText(
    "finished with gaps: selected coverage is incomplete",
  );
  await expect(
    page.getByRole("button", { name: "Retry failed" }),
  ).toBeVisible();
});

test("tree builds missing intermediate folders from inventory paths and selects descendants", async ({
  page,
}) => {
  await workspaceFixture(page, { nested: true });
  await page.getByRole("button", { name: "Set up folder" }).click();
  await expect(page.getByLabel("Projects", { exact: true })).toBeChecked();
  await expect(page.getByLabel("Archive", { exact: true })).toBeChecked();
  await page.getByLabel("Projects", { exact: true }).uncheck();
  await expect(page.getByLabel("Archive", { exact: true })).not.toBeChecked();
  await expect(
    page.getByRole("button", { name: "Review manifest" }),
  ).toBeDisabled();
});

test("changed scope requires a new manifest and explicit consent before resuming", async ({
  page,
}) => {
  await workspaceFixture(page);
  await setupJob(page);
  await expect
    .poll(() =>
      page.evaluate(() =>
        (window as any).calls.some((c: any) => c.command === "setup_job_watch"),
      ),
    )
    .toBe(true);
  await page.evaluate(() => {
    const w = window as any;
    const payload = {
      ...structuredClone(w.setupJob),
      status: "needs_consent",
      revision: 2,
    };
    for (const fn of Object.values(w.listeners))
      (fn as Function)({ event: "setup_job_event", id: 2, payload });
  });
  await page.evaluate(() => {
    const w = window as any;
    w.nextManifest = {
      ...structuredClone(w.setupManifest),
      batches: [],
      excluded: [{ path: "Meetings/one.md", reason: "No longer eligible" }],
    };
  });
  await page.getByRole("button", { name: "Preview updated manifest" }).click();
  await expect(
    page.getByRole("heading", { name: "Removed from the updated selection" }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Authorize and resume" }),
  ).toBeDisabled();
  await page
    .getByLabel("Authorize this updated manifest for the remaining job")
    .check();
  await page.getByRole("button", { name: "Authorize and resume" }).click();
  const request = await page.evaluate(
    () =>
      (window as any).calls.find(
        (c: any) => c.command === "setup_job_reconsent",
      ).args.request,
  );
  expect(request).toMatchObject({
    root_id: ids.root,
    job_id: "job-one",
    expected_revision: 2,
    manifest_id: "manifest-one",
  });
});

test("connection confirmation previews only the selected connection", async ({
  page,
}) => {
  await workspaceFixture(page);
  await showDraft(page);
  await page.getByRole("button", { name: "Approve selected changes" }).click();
  await page.evaluate(
    ({ ids, source }) => {
      (window as any).reviewConnection = {
        id: "connection-one",
        from: { kind: "topic", id: ids.topic },
        relation: "relates to",
        to: { kind: "topic", id: ids.topic },
        uncertainty: null,
        sources: [source],
        valid_from: null,
        valid_to: null,
      };
    },
    { ids, source },
  );
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  await page.getByRole("button", { name: "Ledger", exact: true }).click();
  await page.getByRole("button", { name: "Confirm this connection" }).click();
  await expect(
    page.getByRole("heading", { name: "Confirm displayed connections" }),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        (window as any).calls.find(
          (c: any) => c.command === "setup_connection_confirm_preview",
        ).args.request.connection_ids,
    ),
  ).toEqual(["connection-one"]);
});

test("a returning root opens Today and still offers setup", async ({
  page,
}) => {
  await workspaceFixture(page, { returning: true });
  await expect(
    page.getByRole("heading", { name: "Today", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Set up folder" }).click();
  await expect(
    page.getByRole("heading", { name: "Understand this folder" }),
  ).toBeVisible();
});

test("new evidence shows the prior understanding without accepting the proposed replacement", async ({
  page,
}) => {
  await workspaceFixture(page);
  await setupJob(page);
  await expect
    .poll(() =>
      page.evaluate(() =>
        (window as any).calls.some((c: any) => c.command === "setup_job_watch"),
      ),
    )
    .toBe(true);
  await page.evaluate(
    ({ source }) => {
      const w = window as any;
      w.claims = [
        {
          ...w.setupJob.graph.items.find(
            (item: any) => item.item_id === "item-claim",
          ).candidate.claim,
          value: "Monday",
        },
      ];
      const next = structuredClone(w.setupJob);
      next.revision = 2;
      next.graph.issues.push({
        id: "replacement",
        issue_kind: "replacement",
        key: "Launch differs",
        item_ids: ["item-claim"],
        sources: [source],
        deferred: false,
      });
      for (const fn of Object.values(w.listeners))
        (fn as Function)({ event: "setup_job_event", id: 2, payload: next });
    },
    { source },
  );
  await page.getByRole("button", { name: "Ledger", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Proposed change" }),
  ).toBeVisible();
  await expect(page.getByText("launch: Monday", { exact: true })).toBeVisible();
  await expect(
    page.getByText("launch: Friday", { exact: true }).first(),
  ).toBeVisible();
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter((c: any) => c.command === "setup_decide"),
    ),
  ).toHaveLength(0);
});

test("an oversized selected source is named and must be excluded before preview", async ({
  page,
}) => {
  await workspaceFixture(page, { oversized: true });
  await page.getByRole("button", { name: "Set up folder" }).click();
  await expect(
    page.getByText(/exceed the 64 KiB per-batch limit/),
  ).toBeVisible();
  await expect(page.getByText(/Meetings\/one.md \(293/)).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Review manifest" }),
  ).toBeDisabled();
});

test("manifest request and reviewed budget reserve room for evidence and context", async ({
  page,
}) => {
  await workspaceFixture(page);
  await page.getByRole("button", { name: "Set up folder" }).click();
  await page.getByRole("button", { name: "Review manifest" }).click();
  const request = await page.evaluate(
    () =>
      (window as any).calls.find(
        (call: any) => call.command === "setup_manifest_preview",
      ).args.request,
  );
  expect(request.limits.max_bytes_per_batch).toBe(65536);
  await expect(page.getByText(/64 KiB of notes per batch/)).toBeVisible();
  await expect(
    page.getByText(/space reserved for citations and context/i),
  ).toBeVisible();
});

test("batch failure explanations allow only exact frozen reason pairs", async ({
  page,
}) => {
  await workspaceFixture(page);
  await setupJob(page);
  await expect
    .poll(() =>
      page.evaluate(() =>
        (window as any).calls.some((c: any) => c.command === "setup_job_watch"),
      ),
    )
    .toBe(true);
  await page.evaluate(() => {
    const w = window as any;
    const details = [
      { phase: "turn", reason: "model_terminal_failed" },
      { phase: "publication", reason: "scope_not_read" },
      {
        phase: "turn",
        reason: "model_terminal_failed",
        extra: "SENTINEL_PRIVATE",
      },
      { phase: "turn", reason: "scope_not_read" },
      { phase: "other", reason: "SENTINEL_PRIVATE" },
      ["SENTINEL_PRIVATE"],
      null,
    ];
    const payload = {
      ...structuredClone(w.setupJob),
      revision: 2,
      status: "finished_with_gaps",
      coverage: {
        ...w.setupJob.coverage,
        complete: false,
        failed: ["Meetings/one.md"],
      },
      batches: details.map((detail, i) => ({
        batch_id: `batch-${i}`,
        status: "failed",
        scope_id: null,
        run_id: null,
        attempt: 1,
        error: {
          code: "SENTINEL_PRIVATE",
          message: "SENTINEL_PRIVATE",
          details: detail,
        },
        proposal_ids: [],
        completed_source_ids: [],
      })),
    };
    for (const fn of Object.values(w.listeners))
      (fn as Function)({ event: "setup_job_event", id: 2, payload });
  });
  await expect(page.getByText("The model reported a failed turn.")).toHaveCount(
    1,
  );
  await expect(
    page.getByText(
      "The selected notes were not read, so this batch was not saved.",
    ),
  ).toHaveCount(1);
  await expect(
    page.getByText("This batch could not be completed."),
  ).toHaveCount(5);
  await expect(
    page.getByText("SENTINEL_PRIVATE", { exact: false }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Retry failed" }),
  ).toBeVisible();
});


test("date-ineligible oversized notes do not block eligible selected notes", async ({ page }) => {
  await workspaceFixture(page, { dateExcludedLarge: true });
  await page.getByRole("button", { name: "Set up folder" }).click();
  await page.getByRole("button", { name: "From date: Select date" }).click();
  await page.getByLabel("From date date (YYYY-MM-DD)").fill("2026-01-01");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page.getByRole("button", { name: "Review manifest" })).toBeEnabled();
});


test("direct exclusion preserves the remaining selection and invalidates consent", async ({ page }) => {
  await workspaceFixture(page, { dateExcludedLarge: true });
  await page.getByRole("button", { name: "Set up folder" }).click();
  await expect(page.getByRole("heading", { name: "Needs attention" })).toBeVisible();
  await page.getByRole("button", { name: "Exclude Meetings/old.md from this run" }).click();
  await page.getByRole("button", { name: "Exclude Meetings/undated.md from this run" }).click();
  await expect(page.getByRole("button", { name: "Review manifest" })).toBeEnabled();
  await page.getByRole("button", { name: "Review manifest" }).click();
  const request = await page.evaluate(() => (window as any).calls.find((c: any) => c.command === "setup_manifest_preview").args.request);
  expect(request.spec.files).toEqual(["Meetings/one.md"]);
  await page.getByLabel("Authorize this exact manifest for this job").check();
  await page.getByRole("button", { name: "From date: Select date" }).click();
  await page.getByLabel("From date date (YYYY-MM-DD)").fill("2026-01-01");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page.getByRole("heading", { name: "Review the exact manifest" })).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).calls.filter((c: any) => c.command === "setup_job_create").length)).toBe(0);
});

test("dates validate and BoardUI role selection sends canonical values", async ({ page }) => {
  await workspaceFixture(page);
  await page.getByRole("button", { name: "Set up folder" }).click();
  await page.getByRole("button", { name: "Suggested role for Meetings" }).click();
  await page.getByRole("option", { name: "project" }).click();
  await page.getByRole("button", { name: "From date: Select date" }).click();
  await page.getByLabel("From date date (YYYY-MM-DD)").fill("2026-12-01");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page.getByText(/No selected notes match these dates/)).toBeVisible();
  await page.getByRole("button", { name: "Through date: Select date" }).click();
  await page.getByLabel("Through date date (YYYY-MM-DD)").fill("2026-02-30");
  await expect(page.getByText("Enter a valid date in YYYY-MM-DD format.")).toBeVisible();
  await page.getByLabel("Through date date (YYYY-MM-DD)").fill("2026-11-01");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page.getByRole("alert")).toContainText("From date must be on or before through date");
  await expect(page.getByRole("button", { name: "Review manifest" })).toBeDisabled();
  await page.getByRole("button", { name: /From date: 2026-12-01/ }).click();
  await page.getByRole("button", { name: "Clear" }).click();
  await page.getByRole("button", { name: "Review manifest" }).click();
  const request = await page.evaluate(() => (window as any).calls.find((c: any) => c.command === "setup_manifest_preview").args.request);
  expect(request.spec.date_from).toBeNull();
  expect(request.spec.date_to).toBe("2026-11-01");
  expect(request.roles).toEqual([{path: "Meetings", role: "project"}]);
});

test("date picker cancels drafts, returns focus, and handles keyboard dismissal", async ({ page }) => {
  await workspaceFixture(page);
  await page.getByRole("button", { name: "Set up folder" }).click();
  const trigger = page.getByRole("button", { name: "From date: Select date" });
  await trigger.focus();
  await page.keyboard.press("Enter");
  await page.getByLabel("From date date (YYYY-MM-DD)").fill("2026-11-01");
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(trigger).toBeFocused();
  await trigger.click();
  await expect(page.getByLabel("From date date (YYYY-MM-DD)")).toHaveValue("");
  await page.keyboard.press("Escape");
  await expect(trigger).toBeFocused();
});

test("BoardUI calendar exposes its month panel and editable date chip", async ({ page }) => {
  await workspaceFixture(page);
  await page.getByRole("button", { name: "Set up folder" }).click();
  await page.getByRole("button", { name: "From date: Select date" }).click();
  await expect(page.locator(".board-month-panel")).toBeVisible();
  await expect(page.locator(".board-month-panel svg")).toHaveCount(2);
  await expect(page.locator(".board-date-chip")).toBeVisible();
});

test("empty date draft cannot apply an old value", async ({ page }) => {
  await workspaceFixture(page);
  await page.getByRole("button", { name: "Set up folder" }).click();
  await page.getByRole("button", { name: "From date: Select date" }).click();
  await page.getByLabel("From date date (YYYY-MM-DD)").fill("2026-01-01");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page.getByRole("button", { name: "From date: 2026-01-01" })).toBeFocused();
  await page.getByRole("button", { name: "From date: 2026-01-01" }).click();
  await page.getByLabel("From date date (YYYY-MM-DD)").fill("");
  await expect(page.getByRole("button", { name: "Apply" })).toBeDisabled();
  await page.getByRole("button", { name: "Clear" }).click();
  await expect(page.getByRole("button", { name: "From date: Select date" })).toBeFocused();
});

test("BoardUI nonmodal role select navigates with keyboard and dismisses outside", async ({ page }) => {
  await workspaceFixture(page);
  await page.getByRole("button", { name: "Set up folder" }).click();
  const trigger = page.getByRole("button", { name: "Suggested role for Meetings" });
  await trigger.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("listbox")).toBeVisible();
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Escape");
  await expect(trigger).toBeFocused();
  await trigger.click();
  await page.getByRole("heading", { name: "Choose notes to analyse" }).click();
  await expect(page.getByRole("listbox")).toHaveCount(0);
});


test("date picker dismisses on outside press and page scroll", async ({ page }) => {
  await page.setViewportSize({ width: 480, height: 800 });
  await workspaceFixture(page);
  await page.getByRole("button", { name: "Set up folder" }).click();
  const trigger = page.getByRole("button", { name: "From date: Select date" });
  await trigger.scrollIntoViewIfNeeded();
  await trigger.click();
  await expect(page.getByRole("dialog", { name: "From date" })).toBeVisible();
  await page.getByRole("heading", { name: "Choose notes to analyse" }).click();
  await expect(page.getByRole("dialog", { name: "From date" })).toHaveCount(0);
  await trigger.click();
  await expect(page.getByRole("dialog", { name: "From date" })).toBeVisible();
  await page.evaluate(() => window.scrollBy(0, 150));
  await expect(page.getByRole("dialog", { name: "From date" })).toHaveCount(0);
});
