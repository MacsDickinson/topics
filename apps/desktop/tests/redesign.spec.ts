import { test, expect } from "@playwright/test";
import { workspaceFixture } from "./fixtures/workspace";

test("first run has one focused task and shortcuts cannot dismiss folder connection", async ({
  page,
}) => {
  await workspaceFixture(page, { empty: true });
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(
    "Start with your notes",
  );
  await expect(page.getByRole("searchbox")).toHaveCount(0);
  await page.keyboard.press("Meta+k");
  await expect(page.getByLabel("Notes folder path")).toBeVisible();
  await expect(page.getByRole("searchbox")).toHaveCount(0);
  await page.keyboard.press("Tab");
  await expect(
    page.getByRole("link", { name: "Skip to content" }),
  ).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("main")).toBeFocused();
});

test("both search shortcuts work across screens and select the existing query", async ({
  page,
}) => {
  await workspaceFixture(page, { populated: true });
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  await page.getByRole("button", { name: "Ledger", exact: true }).click();
  await page.getByLabel("Message about this topic").fill("Keep this draft");
  await page.keyboard.press("Meta+k");
  const search = page.getByRole("searchbox", { name: "Search notes" });
  await expect(search).toBeFocused();
  await search.fill("Ledger");
  // Hold the shortcut frame so the test observes both sides of the race.
  await page.evaluate(() => {
    const original = window.requestAnimationFrame;
    const pending: FrameRequestCallback[] = [];
    (window as any).releaseShortcutFrame = () => {
      window.requestAnimationFrame = original;
      for (const callback of pending) original(callback);
      return pending.length;
    };
    window.requestAnimationFrame = (callback) => {
      pending.push(callback);
      return pending.length;
    };
  });
  await page.keyboard.press("Control+k");
  expect(
    await search.evaluate((el: HTMLInputElement) => [
      el.selectionStart,
      el.selectionEnd,
    ]),
  ).toEqual([6, 6]);
  expect(
    await page.evaluate(() => (window as any).releaseShortcutFrame()),
  ).toBeGreaterThan(0);
  await expect
    .poll(() =>
      search.evaluate((el: HTMLInputElement) => [
        el.selectionStart,
        el.selectionEnd,
      ]),
    )
    .toEqual([0, 6]);
  await expect(search).toBeFocused();
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  await expect(page.getByLabel("Message about this topic")).toHaveValue(
    "Keep this draft",
  );
  await page.getByRole("button", { name: "Search", exact: true }).click();
  await expect(
    page.getByRole("searchbox", { name: "Search notes" }),
  ).toBeFocused();
});

test("manual capture and folder settings restore focus to their actual triggers on cancel", async ({
  page,
}) => {
  await workspaceFixture(page, { populated: true });
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  const contextual = page
    .getByRole("main")
    .getByRole("button", { name: "New topic", exact: true });
  await contextual.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByLabel("Topic name", { exact: true })).toBeFocused();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(contextual).toBeFocused();
  const folder = page.getByRole("button", { name: "Change notes folder" });
  await folder.click();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(folder).toBeFocused();
});

test("creating and approving a manual proposal keeps focus on the resulting Inbox", async ({
  page,
}) => {
  await workspaceFixture(page);
  await page.getByRole("button", { name: "New topic", exact: true }).click();
  await page
    .getByLabel("Topic name", { exact: true })
    .fill("Synthetic focus project");
  await page.getByRole("button", { name: "Create proposal" }).click();
  await expect(
    page.getByRole("heading", { name: "Inbox", exact: true }),
  ).toBeFocused();
  await page.getByRole("button", { name: "Approve selected changes" }).click();
  await expect(
    page.getByRole("heading", { name: "Inbox", exact: true }),
  ).toBeFocused();
  await expect(page.getByText("Accepted", { exact: true })).toBeVisible();
});

test("topic selection stays visible during a delayed read and disclosures use native keyboard semantics", async ({
  page,
}) => {
  await workspaceFixture(page, { populated: true });
  await page.evaluate(() => {
    (window as any).defer.workspace_topic_get = true;
  });
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  const topic = page.getByRole("button", { name: "Ledger", exact: true });
  await topic.focus();
  await page.keyboard.press("Enter");
  await expect(topic).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByText("Opening topic…", { exact: true })).toBeVisible();
  await page.evaluate(() => {
    const w = window as any;
    w.defer.workspace_topic_get.resolve({
      topic: w.topics[0],
      current_claims: w.claims,
      history: [],
      connections: [],
      unresolved: [],
      warnings: [],
    });
  });
  await expect(
    page.getByRole("heading", { name: "Current understanding" }),
  ).toBeVisible();
  await expect(topic).toBeFocused();
  const disclosure = page.locator(".claim-provenance summary").first();
  await disclosure.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(".claim-provenance").first()).toHaveAttribute(
    "open",
    "",
  );
  await expect(page.locator(".claim-provenance blockquote")).toHaveText(
    "Ledger launch is Friday",
  );
  await page.keyboard.press("Space");
  await expect(page.locator(".claim-provenance").first()).not.toHaveAttribute(
    "open",
    "",
  );
});

for (const scheme of ["light", "dark"] as const) {
  test(`${scheme} long paths, selected notes and text selection work at 320px`, async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme: scheme, reducedMotion: "reduce" });
    await page.setViewportSize({ width: 320, height: 800 });
    await workspaceFixture(page);
    const longPath = `Projects/${"synthetic-long-path-".repeat(10)}note.md`;
    const content = `Synthetic note\n\n${"unbroken-".repeat(100)}\n\nEnd of note.`;
    await page.evaluate(
      ({ longPath, content }) => {
        const w = window as any,
          invoke = w.__TAURI_INTERNALS__.invoke;
        w.__TAURI_INTERNALS__.invoke = async (
          command: string,
          args: unknown,
        ) => {
          const result = await invoke(command, args);
          if (command === "workspace_sources_list")
            result.sources[0].path = longPath;
          if (command === "workspace_source_get") {
            result.summary.path = longPath;
            result.content = content;
          }
          return result;
        };
      },
      { longPath, content },
    );
    await page.getByRole("button", { name: "Sources", exact: true }).click();
    const source = page.getByRole("button", { name: longPath, exact: true });
    await source.focus();
    await page.keyboard.press("Enter");
    await expect(source).toHaveAttribute("aria-pressed", "true");
    await expect(source).toBeFocused();
    await expect(page.getByLabel("Source sensitivity")).toBeEnabled();
    expect(
      await page.locator(".note-content").evaluate((el) => {
        const range = document.createRange();
        range.selectNodeContents(el);
        const selection = window.getSelection();
        selection?.removeAllRanges();
        selection?.addRange(range);
        return selection?.toString();
      }),
    ).toBe(content);
    expect(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    ).toBe(true);
    expect(
      await page
        .locator(".note-content")
        .evaluate((el) => getComputedStyle(el).userSelect),
    ).not.toBe("none");
    const controls = page.locator(".sidebar button");
    for (const control of await controls.all()) {
      expect(
        await control.evaluate(
          (el) => el.getBoundingClientRect().right <= innerWidth,
        ),
      ).toBe(true);
    }
  });
}
