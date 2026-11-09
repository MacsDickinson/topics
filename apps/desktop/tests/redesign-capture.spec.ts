import { test, expect } from "@playwright/test";
import { workspaceFixture, topic, claim, proposal } from "./fixtures/workspace";

// Transport-only synthetic evidence. No filesystem notes or genuine model calls.
const phase = process.env.TOPICS_CAPTURE_PHASE || "after";
for (const appearance of ["light", "dark", "narrow"] as const) {
  for (const screen of [
    "topics",
    "sources",
    "search",
    "inbox",
    "history",
    "setup",
    "analysis",
  ] as const) {
    test(`capture ${appearance} ${screen}`, async ({ page }) => {
      await page.setViewportSize({
        width: appearance === "narrow" ? 600 : 1440,
        height: 960,
      });
      await page.emulateMedia({
        colorScheme: appearance === "dark" ? "dark" : "light",
        reducedMotion: "reduce",
      });
      await workspaceFixture(page, { empty: screen === "setup" });
      await page.addInitScript(
        ({ topic, claim, proposal }) => {
          const w = window as any;
          w.topics = [
            topic,
            {
              ...topic,
              id: "person-two",
              name: "Morgan Ellis",
              topic_kind: "person",
              sensitivity: "local_only",
            },
            {
              ...topic,
              id: "thread-three",
              name: "Research questions",
              topic_kind: "thread",
              sensitivity: "unknown",
            },
          ];
          w.claims = [
            claim,
            {
              ...claim,
              id: "claim-two",
              revision_id: "revision-two",
              predicate: "next step",
              value:
                "Review the migration checklist with Morgan before the launch.",
              uncertainty: null,
            },
          ];
          w.claimHistory = [claim];
          w.proposals = [proposal];
          w.operations = [
            {
              id: "operation-one",
              description: "Accepted Ledger launch",
              status: "committed",
              created_at: "2026-11-03T10:15:00Z",
            },
            {
              id: "operation-two",
              description: "Changed source sensitivity",
              status: "undone",
              created_at: "2026-11-02T09:30:00Z",
            },
          ];
          const invoke = w.__TAURI_INTERNALS__.invoke;
          w.__TAURI_INTERNALS__.invoke = async (command: string, args: any) => {
            if (command === "search_notes")
              return [
                {
                  title: "Ledger launch planning",
                  path: "Meetings/one.md",
                  snippet:
                    "The <mark>Ledger</mark> migration is ready for review. Morgan will check the launch date.",
                },
                {
                  title: "Migration checklist",
                  path: "Projects/Ledger/migration-checklist.md",
                  snippet:
                    "Confirm ownership and the <mark>Ledger</mark> cutover date before Friday.",
                },
              ];
            const result = await invoke(command, args);
            if (command === "workspace_source_get")
              result.content =
                "# Ledger launch planning\n\nThe migration is ready for review.\n\nMorgan will check the launch date against the release calendar before we confirm it. The current target is Friday.\n\n## Next steps\n\n- Review the migration checklist\n- Confirm the final launch date\n- Keep the original meeting notes for context";
            return result;
          };
        },
        { topic, claim, proposal },
      );
      await page.reload();
      await expect(
        page.getByRole("button", { name: "Topics", exact: true }),
      ).toBeVisible();
      if (screen === "topics") {
        await page.getByRole("button", { name: "Topics", exact: true }).click();
        await page.getByRole("button", { name: "Ledger", exact: true }).click();
        await expect(
          page.getByRole("heading", { name: "Current understanding" }),
        ).toBeVisible();
      } else if (screen === "sources") {
        await page
          .getByRole("button", { name: "Sources", exact: true })
          .click();
        await page
          .getByRole("button", { name: "Meetings/one.md", exact: true })
          .click();
        await expect(page.locator(".note-content")).toContainText(
          "Ledger launch planning",
        );
      } else if (screen === "search") {
        await page
          .getByRole("button", { name: /Search/ })
          .first()
          .click();
        await page
          .getByRole("searchbox", { name: "Search notes" })
          .fill("Ledger");
        await expect(
          page.getByRole("heading", {
            name: "Ledger launch planning",
            exact: true,
          }),
        ).toBeVisible();
      } else if (screen === "inbox") {
        await page.getByRole("button", { name: /Inbox/ }).click();
        await expect(
          page.getByRole("heading", { name: "Proposed understanding" }),
        ).toBeVisible();
      } else if (screen === "history") {
        await page
          .getByRole("button", { name: "History", exact: true })
          .click();
        await expect(
          page.getByRole("heading", { name: "Accepted Ledger launch" }),
        ).toBeVisible();
      } else if (screen === "analysis") {
        await page
          .getByRole("button", { name: "Analyse notes", exact: true })
          .click();
        await page.getByLabel("Folders", { exact: true }).fill("Meetings");
        await page.getByRole("button", { name: "Preview selection" }).click();
        await expect(
          page.getByRole("heading", { name: "Review selected notes" }),
        ).toBeVisible();
      } else await expect(page.getByLabel("Notes folder path")).toBeVisible();
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
      ).toBe(true);
      await page.screenshot({
        path: `evidence/redesign/${phase}/${appearance}-${screen}.png`,
        fullPage: true,
      });
    });
  }
}
