import { openTool } from "./fixtures/workspace";
import { test, expect, type Page } from "@playwright/test";
import { workspaceFixture, ids, source } from "./fixtures/workspace";

async function startMap(page: Page) {
  await openTool(page, "Set up folder");
  await page
    .getByRole("button", { name: "Review manifest", exact: true })
    .click();
  await page.getByLabel("Authorize this exact manifest for this job").check();
  await page
    .getByRole("button", { name: "Build starting map", exact: true })
    .click();
  await expect
    .poll(() =>
      page.evaluate(() =>
        (window as any).calls.some((c: any) => c.command === "setup_job_watch"),
      ),
    )
    .toBe(true);
}
async function capture(page: Page, appearance: string, name: string) {
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  ).toBe(true);
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.screenshot({
    path: `evidence/redesign/canonical/${appearance}-${name}.png`,
    fullPage: true,
  });
}

for (const appearance of ["light", "dark", "narrow"] as const) {
  test.describe(appearance, () => {
    test.beforeEach(async ({ page }) => {
      await page.setViewportSize({
        width: appearance === "narrow" ? 600 : 1440,
        height: 960,
      });
      await page.emulateMedia({
        colorScheme: appearance === "dark" ? "dark" : "light",
        reducedMotion: "reduce",
      });
    });
    test(`${appearance} exact updated scope requires fresh keyboard consent`, async ({
      page,
    }) => {
      await workspaceFixture(page);
      await startMap(page);
      await page.evaluate(() => {
        const w = window as any,
          next = structuredClone(w.setupJob);
        next.status = "needs_consent";
        next.revision = 2;
        next.coverage = {
          ...next.coverage,
          selected: 2,
          processed: ["Meetings/completed.md"],
          outstanding: ["Meetings/one.md"],
          complete: false,
        };
        w.nextManifest = {
          ...structuredClone(w.setupManifest),
          batches: [],
          excluded: [{ path: "Meetings/one.md", reason: "No longer eligible" }],
        };
        for (const fn of Object.values(w.listeners))
          (fn as Function)({ event: "setup_job_event", id: 2, payload: next });
      });
      const preview = page.getByRole("button", {
        name: "Preview updated manifest",
        exact: true,
      });
      await preview.focus();
      await page.keyboard.press("Enter");
      await expect(
        page.getByRole("heading", {
          name: "Removed from the updated selection",
        }),
      ).toBeVisible();
      const allow = page.getByLabel(
        "Authorize this updated manifest for the remaining job",
      );
      const resume = page.getByRole("button", {
        name: "Authorize and resume",
        exact: true,
      });
      await expect(allow).not.toBeChecked();
      await expect(resume).toBeDisabled();
      await capture(page, appearance, "updated-scope");
      await allow.focus();
      await page.keyboard.press("Space");
      await expect(resume).toBeEnabled();
      await preview.click();
      await expect(allow).not.toBeChecked();
      await expect(resume).toBeDisabled();
      await allow.check();
      await resume.click();
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
      expect(
        await page.evaluate(() =>
          (window as any).calls.filter(
            (c: any) => c.command === "setup_job_create",
          ),
        ),
      ).toHaveLength(1);
    });
    test(`${appearance} supplemental context and connection approval remain separate`, async ({
      page,
    }) => {
      await workspaceFixture(page, { populated: true });
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
      await page.getByRole("button", { name: "Knowledge graph", exact: true }).click();
      await page.getByRole("button", { name: "Ledger", exact: true }).click();
      await page
        .getByRole("button", { name: "Add context", exact: true })
        .click();
      await page
        .getByLabel("Your context")
        .fill("The meeting was provisional; retain the original statement.");
      await page
        .getByRole("button", { name: "Save context", exact: true })
        .click();
      await expect(
        page
          .getByRole("blockquote")
          .filter({ hasText: "The meeting was provisional" }),
      ).toBeVisible();
      await capture(page, appearance, "supplemental-context");
      const confirm = page.getByRole("button", {
        name: "Confirm this connection",
        exact: true,
      });
      await confirm.focus();
      await page.keyboard.press("Enter");
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
      expect(
        await page.evaluate(() =>
          (window as any).calls.filter((c: any) =>
            [
              "setup_decide",
              "setup_confirm_preview",
              "setup_correction_preview",
            ].includes(c.command),
          ),
        ),
      ).toHaveLength(0);
      await capture(page, appearance, "connection-receipt");
    });
    test(`${appearance} named diff and topic-group receipt preserve the reading context`, async ({
      page,
    }) => {
      await workspaceFixture(page);
      await startMap(page);
      await expect(page.getByRole("status")).toContainText(
        "Selected notes analysed. Ready to save.",
      );
      await expect(
        page.getByText(
          "1 excluded source; exclusions are not part of the selected coverage.",
        ),
      ).toBeVisible();
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
            (fn as Function)({
              event: "setup_job_event",
              id: 2,
              payload: next,
            });
        },
        { source },
      );
      await page.getByRole("button", { name: "Ledger", exact: true }).click();
      await expect(
        page.getByRole("heading", { name: "Proposed change" }),
      ).toBeVisible();
      await expect(
        page.getByText("launch: Monday", { exact: true }),
      ).toBeVisible();
      await capture(page, appearance, "named-diff");
      await page
        .getByRole("button", { name: "Review topic group", exact: true })
        .click();
      await expect(
        page.getByRole("heading", { name: "Save topic group", exact: true }),
      ).toBeVisible();
      expect(
        await page.evaluate(() =>
          (window as any).calls.filter(
            (c: any) => c.command === "setup_decide",
          ),
        ),
      ).toHaveLength(0);
      await capture(page, appearance, "topic-group-receipt");
    });
  });
}
