import { openTool } from "./fixtures/workspace";
import { test, expect } from "@playwright/test";
import { workspaceFixture } from "./fixtures/workspace";

test("appearance follows the system, remembers overrides and styles BoardUI overlays", async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await workspaceFixture(page);
  const root = page.locator("html");
  const picker = page.getByRole("combobox", { name: "Appearance" });
  await expect(root).toHaveAttribute("data-theme", "dark");
  await page.emulateMedia({ colorScheme: "light" });
  await expect(root).toHaveAttribute("data-theme", "light");
  await picker.selectOption("dark");
  await page.reload();
  await expect(root).toHaveAttribute("data-theme", "dark");
  await openTool(page, "Set up folder");
  await page
    .getByRole("button", { name: "Suggested role for Meetings" })
    .click();
  await expect(page.getByRole("listbox")).toHaveCSS("color-scheme", "dark");
  await page.keyboard.press("Escape");
  await picker.selectOption("light");
  await page.emulateMedia({ colorScheme: "dark" });
  await expect(root).toHaveAttribute("data-theme", "light");
  await picker.selectOption("system");
  await expect(root).toHaveAttribute("data-theme", "dark");
});

test("each current statement has one reading and review surface", async ({
  page,
}) => {
  await workspaceFixture(page, { populated: true });
  await page.getByRole("button", { name: "Knowledge graph", exact: true }).click();
  await page.getByRole("button", { name: "Ledger", exact: true }).click();
  const claim = page.locator(".understanding .accepted-claim");
  await expect(claim.getByText("Friday", { exact: true })).toHaveCount(1);
  await expect(claim.getByText("From your notes")).toBeVisible();
  await claim.getByText("Sources (1)").click();
  await expect(claim.getByText("Ledger launch is Friday")).toBeVisible();
  await claim.getByRole("button", { name: "That’s wrong" }).click();
  await expect(
    page.getByRole("button", { name: "Preview correction" }),
  ).toBeVisible();
});

for (const theme of ["light", "dark"] as const) {
  test(`${theme} correction composer keeps enabled and disabled actions visible`, async ({ page }, testInfo) => {
    await page.emulateMedia({ colorScheme: theme });
    await page.setViewportSize({ width: 1180, height: 800 });
    await workspaceFixture(page, { populated: true });
    await page.getByRole("button", { name: "Knowledge graph", exact: true }).click();
    await page.getByRole("button", { name: "Ledger", exact: true }).click();
    await page.getByRole("button", { name: "That’s wrong" }).first().click();
    const composer = page.locator(".context-composer");
    const preview = composer.getByRole("button", { name: "Preview correction" });
    const cancel = composer.getByRole("button", { name: "Cancel correction" });
    await expect(preview).toBeDisabled();
    await expect(cancel).toBeEnabled();
    const field = composer.getByRole("textbox", { name: "Your statement" });
    const boundary = await field.evaluate(el => getComputedStyle(el).borderTopColor);
    for (const button of [preview, cancel]) {
      await expect(button).toHaveCSS("border-top-color", boundary);
      await expect(button).toHaveCSS("border-top-style", "solid");
      await expect(button).toHaveCSS("border-top-width", "1px");
    }
    await composer.evaluate(el => el.scrollIntoView({ block: "center" }));
    await page.screenshot({ path: testInfo.outputPath(`composer-${theme}.png`), scale: "css" });
  });
}

test("all registered palettes resolve semantic roles and readable status colours", async ({
  page,
}) => {
  await workspaceFixture(page, { empty: true });
  const picker = page.getByLabel("Appearance");
  const ids = await picker
    .locator("option")
    .evaluateAll((options) =>
      options.map((o) => (o as HTMLOptionElement).value),
    );
  for (const id of ids.filter((id) => id !== "system")) {
    await picker.selectOption(id);
    await expect(page.locator("html")).toHaveAttribute("data-theme", id);
    const contrasts = await page.evaluate(() => {
      const style = getComputedStyle(document.documentElement);
      const probe = document.createElement("span");
      document.body.append(probe);
      function luminance(role: string) {
        const token = "--color-" + role;
        if (!style.getPropertyValue(token).trim())
          throw new Error("Missing " + token);
        probe.style.color = "var(" + token + ")";
        const channels = getComputedStyle(probe)
          .color.match(/[\d.]+/g)!
          .slice(0, 3)
          .map(Number)
          .map((channel) => {
            const srgb = channel / 255;
            return srgb <= 0.04045
              ? srgb / 12.92
              : ((srgb + 0.055) / 1.055) ** 2.4;
          });
        return (
          channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722
        );
      }
      const pairs = [
        ["text-primary", "background-panel", 4.5],
        ["text-muted", "background-sidebar", 4.5],
        ["text-accent", "background-selected", 4.5],
        ["text-on-action", "background-action", 4.5],
        ["text-warning", "background-warning", 4.5],
        ["text-danger", "background-danger", 4.5],
        ["text-success", "background-success", 4.5],
        ["text-info", "background-info", 4.5],
        ["border-focus", "background-panel", 3],
        ["border-strong", "background-panel", 3],
        ["border-strong", "background-subtle", 3],
      ] as const;
      const result = pairs.map(([ink, surface, minimum]) => {
        const a = luminance(ink),
          b = luminance(surface);
        return {
          ink,
          surface,
          minimum,
          ratio: (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05),
        };
      });
      probe.remove();
      return result;
    });
    for (const { ink, surface, minimum, ratio } of contrasts)
      expect(ratio, `${id} ${ink} on ${surface}`).toBeGreaterThanOrEqual(
        minimum,
      );
  }
});
