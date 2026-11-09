import { test, expect, type Page } from "@playwright/test";
import { workspaceFixture, ids, source } from "./fixtures/workspace";

type Kind = "correction" | "connection" | "statements";
const headings = {
  correction: "Review correction",
  connection: "Confirm displayed connections",
  statements: "Confirm displayed statements",
};
const actions = {
  correction: "Approve correction",
  connection: "Confirm connection",
  statements: "Confirm statements",
};
const commands = {
  correction: "setup_correction_preview",
  connection: "setup_connection_confirm_preview",
  statements: "setup_confirm_preview",
};

async function openReview(page: Page, multipleTopics = false) {
  await workspaceFixture(page, { populated: true, multipleTopics });
  await page.evaluate(
    ({ ids, source }) => {
      const w = window as any,
        invoke = w.__TAURI_INTERNALS__.invoke;
      w.reviewConnection = {
        id: "connection-one",
        from: { kind: "topic", id: ids.topic },
        relation: "relates to",
        to: { kind: "topic", id: ids.topic },
        uncertainty: null,
        sources: [source],
        valid_from: null,
        valid_to: null,
      };
      w.reviewPreviews = {};
      w.heldPreviews = [];
      w.previewSerial = 0;
      w.__TAURI_INTERNALS__.invoke = async (command: string, args: any) => {
        if (command === "setup_decide") {
          w.calls.push({ command, args });
          const receipt = Object.values(w.reviewPreviews).find(
            (r: any) => r.receipt_id === args.request.receipt_id,
          );
          // Capture the authorized request without simulating persisted acceptance.
          const result = {
            receipt,
            status: "conflict",
            operations: [],
            saved_ids: [],
            deferred_ids: [],
            error: {
              code: "conflict",
              message: "Synthetic decision captured; no storage change.",
              details: null,
            },
          };
          if (w.holdDecision)
            return new Promise((resolve) => {
              w.releaseDecision = () => resolve(result);
            });
          return result;
        }
        const result = await invoke(command, args);
        const kind =
          command === "setup_correction_preview"
            ? "correction"
            : command === "setup_connection_confirm_preview"
              ? "connection"
              : command === "setup_confirm_preview"
                ? "statements"
                : null;
        if (kind) {
          const receipt = kind === "correction" ? result.receipt : result;
          const serial = ++w.previewSerial;
          receipt.receipt_id = `${kind}-${serial}`;
          receipt.fingerprint = serial
            .toString(16)
            .padStart(
              64,
              kind === "correction" ? "c" : kind === "connection" ? "d" : "e",
            );
          w.reviewPreviews[kind] = structuredClone(receipt);
          if (w.holdPreview === command)
            return new Promise((resolve) =>
              w.heldPreviews.push(() => resolve(result)),
            );
        }
        return result;
      };
    },
    { ids, source },
  );
  await page.getByRole("button", { name: "Topics", exact: true }).click();
  await page.getByRole("button", { name: "Ledger", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Confirm this connection" }),
  ).toBeVisible();
}

async function begin(page: Page, kind: Kind, wait = true) {
  if (kind === "correction") {
    await page
      .getByRole("button", { name: "That’s wrong", exact: true })
      .click();
    await page
      .getByLabel("Your statement")
      .fill("The calendar confirms Monday.");
    await page.getByLabel("Replacement", { exact: true }).fill("Monday");
    await page
      .getByRole("button", { name: "Preview correction", exact: true })
      .click();
  } else {
    await page
      .getByRole("button", {
        name:
          kind === "connection"
            ? "Confirm this connection"
            : "Confirm these 1 statements",
        exact: true,
      })
      .click();
  }
  if (wait)
    await expect(
      page.getByRole("heading", { name: headings[kind], exact: true }),
    ).toBeVisible();
}

async function approveVisible(page: Page, kind: Kind) {
  const expected = await page.evaluate(
    (kind) => (window as any).reviewPreviews[kind],
    kind,
  );
  await expect(
    page.getByRole("heading", { name: headings[kind], exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: actions[kind], exact: true }).click();
  const sent = await page.evaluate(() =>
    (window as any).calls
      .filter((c: any) => c.command === "setup_decide")
      .map((c: any) => c.args.request),
  );
  expect(sent).toEqual([
    {
      root_id: ids.root,
      receipt_id: expected.receipt_id,
      expected_fingerprint: expected.fingerprint,
    },
  ]);
  await expect(page.getByRole("alert")).toHaveText(
    "Synthetic decision captured; no storage change.",
  );
}

for (const [from, to] of [
  ["correction", "connection"],
  ["correction", "statements"],
  ["connection", "correction"],
  ["statements", "correction"],
] as const) {
  test(`${from} to ${to} approves exactly the displayed receipt`, async ({
    page,
  }) => {
    await openReview(page);
    await begin(page, from);
    await begin(page, to);
    await expect(
      page.getByRole("heading", { name: headings[from], exact: true }),
    ).toHaveCount(0);
    await expect(
      page.getByRole("button", { name: actions[from], exact: true }),
    ).toHaveCount(0);
    if (to === "correction")
      await expect(page.locator(".context-review .consent")).toContainText(
        "The calendar confirms Monday.",
      );
    else await expect(page.getByText(/Future rule:/)).toHaveCount(0);
    await approveVisible(page, to);
  });
}

test("cancelled correction response cannot replace a newer connection preview", async ({
  page,
}) => {
  await openReview(page);
  await page.evaluate((command) => {
    (window as any).holdPreview = command;
  }, commands.correction);
  await begin(page, "correction", false);
  await expect
    .poll(() => page.evaluate(() => (window as any).heldPreviews.length))
    .toBe(1);
  await page
    .getByRole("button", { name: "Cancel correction", exact: true })
    .click();
  await begin(page, "connection");
  await page.evaluate(() => (window as any).heldPreviews.shift()());
  await expect(
    page.getByRole("button", { name: "Approve correction", exact: true }),
  ).toHaveCount(0);
  await approveVisible(page, "connection");
});

test("edited correction invalidates visible and pending approvals until a fresh preview", async ({
  page,
}) => {
  await openReview(page);
  await begin(page, "correction");
  await page.getByLabel("Replacement", { exact: true }).fill("Tuesday");
  await expect(
    page.getByRole("button", { name: "Approve correction", exact: true }),
  ).toHaveCount(0);
  await page.evaluate((command) => {
    (window as any).holdPreview = command;
  }, commands.correction);
  await page
    .getByRole("button", { name: "Preview correction", exact: true })
    .click();
  await expect
    .poll(() => page.evaluate(() => (window as any).heldPreviews.length))
    .toBe(1);
  await page
    .getByLabel("Your statement")
    .fill("The updated calendar confirms Wednesday.");
  await page.getByLabel("Replacement", { exact: true }).fill("Wednesday");
  await page.evaluate(() => {
    const w = window as any;
    w.holdPreview = null;
    w.heldPreviews.shift()();
  });
  await expect(
    page.getByRole("button", { name: "Approve correction", exact: true }),
  ).toHaveCount(0);
  await page
    .getByRole("button", { name: "Preview correction", exact: true })
    .click();
  await expect(page.locator(".context-review .consent")).toContainText(
    "The updated calendar confirms Wednesday.",
  );
  await expect(page.locator(".context-review .consent")).toContainText(
    "Wednesday",
  );
  await approveVisible(page, "correction");
});

test("cancelled connection response cannot replace a newer statement preview", async ({
  page,
}) => {
  await openReview(page);
  await page.evaluate((command) => {
    (window as any).holdPreview = command;
  }, commands.connection);
  await begin(page, "connection", false);
  await expect
    .poll(() => page.evaluate(() => (window as any).heldPreviews.length))
    .toBe(1);
  await page.getByRole("button", { name: "Not now", exact: true }).click();
  await begin(page, "statements");
  await page.evaluate(() => (window as any).heldPreviews.shift()());
  await expect(
    page.getByRole("button", { name: "Confirm connection", exact: true }),
  ).toHaveCount(0);
  await approveVisible(page, "statements");
});

test("changing notes root invalidates an in-flight correction without submitting it", async ({
  page,
}) => {
  await openReview(page);
  await page.evaluate((command) => {
    (window as any).holdPreview = command;
  }, commands.correction);
  await begin(page, "correction", false);
  await expect
    .poll(() => page.evaluate(() => (window as any).heldPreviews.length))
    .toBe(1);
  await page
    .getByRole("button", { name: "Change notes folder", exact: true })
    .click();
  await page.getByLabel("Notes folder path").fill("/synthetic/other-root");
  await page
    .getByRole("button", { name: "Use notes folder", exact: true })
    .click();
  await expect(
    page.getByRole("heading", { name: "Understand this folder" }),
  ).toBeVisible();
  await page.evaluate(() => (window as any).heldPreviews.shift()());
  await expect(
    page.getByRole("button", { name: "Approve correction", exact: true }),
  ).toHaveCount(0);
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter((c: any) => c.command === "setup_decide"),
    ),
  ).toHaveLength(0);
});

test("changing topic prevents a pending correction from restoring the old approval", async ({
  page,
}) => {
  await openReview(page, true);
  await page.evaluate((command) => {
    (window as any).holdPreview = command;
  }, commands.correction);
  await begin(page, "correction", false);
  await expect
    .poll(() => page.evaluate(() => (window as any).heldPreviews.length))
    .toBe(1);
  await page.getByRole("button", { name: "Harbor", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Harbor", exact: true }),
  ).toBeVisible();
  await page.evaluate(() => (window as any).heldPreviews.shift()());
  await expect(
    page.getByRole("button", { name: "Approve correction", exact: true }),
  ).toHaveCount(0);
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter((c: any) => c.command === "setup_decide"),
    ),
  ).toHaveLength(0);
});

test("a submitted decision locks the displayed intent and correction fields until it settles", async ({
  page,
}) => {
  await openReview(page);
  await begin(page, "correction");
  const expected = await page.evaluate(() => {
    const w = window as any;
    w.holdDecision = true;
    return w.reviewPreviews.correction;
  });
  await page
    .getByRole("button", { name: "Approve correction", exact: true })
    .click();
  await expect
    .poll(() => page.evaluate(() => typeof (window as any).releaseDecision))
    .toBe("function");
  await expect(
    page.getByRole("button", { name: "Confirm this connection", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("button", {
      name: "Confirm these 1 statements",
      exact: true,
    }),
  ).toBeDisabled();
  await expect(page.getByLabel("Your statement")).toBeDisabled();
  await expect(page.getByLabel("Replacement", { exact: true })).toBeDisabled();
  await expect(
    page
      .locator(".context-review .consent")
      .getByRole("button", { name: "Cancel", exact: true }),
  ).toBeDisabled();
  await expect(
    page.getByRole("heading", { name: "Review correction", exact: true }),
  ).toBeVisible();
  expect(
    await page.evaluate(() =>
      (window as any).calls
        .filter((c: any) => c.command === "setup_decide")
        .map((c: any) => c.args.request),
    ),
  ).toEqual([
    {
      root_id: ids.root,
      receipt_id: expected.receipt_id,
      expected_fingerprint: expected.fingerprint,
    },
  ]);
  await page.evaluate(() => (window as any).releaseDecision());
  await expect(page.getByRole("alert")).toHaveText(
    "Synthetic decision captured; no storage change.",
  );
  await expect(
    page.getByRole("button", { name: "Approve correction", exact: true }),
  ).toHaveCount(0);
});

for (const appearance of ["light", "dark", "narrow"] as const) {
  test(`${appearance} switched review previews show only their matching action`, async ({
    page,
  }) => {
    await page.setViewportSize({
      width: appearance === "narrow" ? 600 : 1440,
      height: 960,
    });
    await page.emulateMedia({
      colorScheme: appearance === "dark" ? "dark" : "light",
      reducedMotion: "reduce",
    });
    await openReview(page);
    await begin(page, "correction");
    for (const kind of ["connection", "correction", "statements"] as const) {
      await begin(page, kind);
      await expect(
        page
          .locator(".context-review .consent")
          .getByRole("button", { name: actions[kind], exact: true }),
      ).toBeEnabled();
      for (const other of (
        ["connection", "correction", "statements"] as const
      ).filter((other) => other !== kind)) {
        await expect(
          page.getByRole("heading", { name: headings[other], exact: true }),
        ).toHaveCount(0);
        await expect(
          page.getByRole("button", { name: actions[other], exact: true }),
        ).toHaveCount(0);
      }
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
      ).toBe(true);
      await page.evaluate(() => window.scrollTo(0, 0));
      await page.screenshot({
        path: `evidence/redesign/approval/${appearance}-${kind}.png`,
        fullPage: true,
      });
    }
    await approveVisible(page, "statements");
  });
}
