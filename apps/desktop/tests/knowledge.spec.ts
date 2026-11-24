import { test, expect } from "@playwright/test";
import { workspaceFixture, ids, topic, claim } from "./fixtures/workspace";
import { resolve } from "node:path";

test("graph, topic lookup and a disputed launch date use the displayed judgment receipt", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1220, height: 850 });
  await workspaceFixture(page, { populated: true });
  await page.evaluate(
    ({ ids, topic, claim }) => {
      const w = window as any;
      const checks = {
        conflicting: 0,
        uncertain: 0,
        stale: 0,
        unverified: 0,
        deferred: 0,
      };
      const topics = [
        { ...topic, name: "Ledger migration" },
        { ...topic, id: "robin", name: "Robin Shah", topic_kind: "person" },
        {
          ...topic,
          id: "handover",
          name: "Operations handover",
          topic_kind: "thread",
        },
        {
          ...topic,
          id: "rehearsal",
          name: "Release rehearsal",
          topic_kind: "project",
        },
        {
          ...topic,
          id: "checklist",
          name: "Launch checklist",
          topic_kind: "thread",
        },
      ];
      w.topics = topics.slice(0, 4);
      const decision = { id: "job-one", kind: "setup_draft" };
      const nodes = topics.map((topic, i) => ({
        topic,
        status: i === 4 ? "pending" : "accepted",
        assertions: 1,
        verified: i === 1,
        checks: { ...checks, conflicting: i === 0 ? 1 : 0 },
        pending: i === 4 ? decision : null,
      }));
      const edge = (i: number, relation: string, from: string, to: string) => ({
        id: "edge-" + i,
        from,
        to,
        relation,
        status: i === 4 ? "pending" : "accepted",
        inferred: i === 4,
        confirmed: i === 1,
        pending: i === 4 ? decision : null,
        sources: 1,
        valid_from: null,
        valid_to: null,
      });
      w.graphOverride = {
        root_id: ids.root,
        nodes,
        edges: [
          edge(1, "leads", "robin", ids.topic),
          edge(2, "requires", ids.topic, "handover"),
          edge(3, "precedes", "rehearsal", ids.topic),
          edge(4, "supports", "checklist", ids.topic),
        ],
        checks: { ...checks, conflicting: 1 },
        warnings: [],
        pending: [
          {
            decision,
            state: "awaiting_decision",
            topics: 1,
            assertions: 1,
            relationships: 1,
            needs_input: 1,
            created_at: claim.recorded_at,
          },
        ],
      };
      w.claims = [
        claim,
        {
          ...claim,
          id: "monday",
          revision_id: "monday-r1",
          value: "Monday",
          sources: [
            {
              kind: "passage",
              source: {
                ...claim.sources[0].source,
                path: "Meetings/launch-review.md",
                excerpt: "The cutover moves to Monday after rehearsal.",
              },
            },
          ],
        },
      ];
      const assertions = w.claims.map((c: any) => ({
        ...c,
        claim_id: c.id,
        status: "accepted",
        review: "from_notes",
        uncertainty: null,
        evidence: c.sources.map((s: any) => ({ ...s, stale: false })),
        check: "conflicting",
        pending: null,
      }));
      w.detailOverrides = {
        [ids.topic]: {
          topic: topics[0],
          status: "accepted",
          pending: null,
          assertions,
          relationships: [],
          earlier: [],
          timelines: [],
          context: [],
          warnings: [],
          checks: nodes[0].checks,
          validation: [
            {
              id: "launch-check",
              kind: "conflicting",
              predicate: "Launch date",
              revision_ids: assertions.map((a: any) => a.revision_id),
              connection_id: null,
              deferred: false,
            },
          ],
        },
      };
      const invoke = w.__TAURI_INTERNALS__.invoke;
      w.__TAURI_INTERNALS__.invoke = async (cmd: string, args: any) => {
        const r = args?.request;
        if (cmd === "validation_defer") {
          w.detailOverrides[ids.topic].validation[0].deferred = r.deferred;
          return w.detailOverrides[ids.topic];
        }
        if (cmd === "validation_preview") {
          if (r.action.kind === "confirm_both")
            return {
              receipt: { kind: "confirm_both", connections: [] },
              confirmed: w.claims,
              after: [],
              before: [],
              statement: null,
            };
          const value = r.action.value || "Monday";
          return {
            receipt: {
              receipt_id: "judgment-receipt",
              fingerprint: "reviewed-bytes",
              connections: [],
            },
            statement: { text: "Ledger launch is " + value + "." },
            before: w.claims,
            after: w.claims.map((c: any) => ({
              ...c,
              value,
              supersedes: [c.revision_id],
              revision_id: c.revision_id + "-new",
            })),
            confirmed: [],
            rules: [],
          };
        }
        if (cmd === "setup_decide" && r.receipt_id === "judgment-receipt") {
          w.approval = r;
          w.detailOverrides[ids.topic].validation = [];
          w.graphOverride.nodes[0].checks = { ...checks };
          return { status: "completed" };
        }
        return invoke(cmd, args);
      };
    },
    { ids, topic, claim },
  );
  // Re-enter through regular navigation so the fixture is read via IPC.
  await page.getByRole("button", { name: "Sources", exact: true }).click();
  await page
    .getByRole("button", { name: "Knowledge graph", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Robin Shah", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("awaiting decision")).toBeVisible();
  await expect(page.locator(".graph-discoveries")).toContainText(
    "1 open question",
  );
  if (process.env.TOPICS_CAPTURE_GRAPH === "1")
    await page.screenshot({
      path: resolve("../../docs/site/img/workspace-graph.png"),
    });
  await page.getByRole("button", { name: "Text view", exact: true }).click();
  await expect(
    page.getByText(/Robin Shah → leads → Ledger migration/),
  ).toBeVisible();
  await page.getByRole("button", { name: "Show graph", exact: true }).click();
  await page.keyboard.press("Meta+k");
  const search = page.getByRole("searchbox", { name: "Search topics" });
  await expect(search).toBeFocused();
  await search.fill("Ledger");
  const hit = page.locator(".graph-hits button");
  await hit.focus();
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("button", { name: "Ledger migration", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  const check = page.locator(".validation-item");
  await expect(check.locator("blockquote")).toHaveCount(2);
  await expect(check).toContainText("The cutover moves to Monday");
  await check.getByRole("button", { name: "Defer", exact: true }).click();
  await check.getByRole("button", { name: "Bring back" }).click();
  if (process.env.TOPICS_CAPTURE_GRAPH) {
    await page
      .locator(".topic-identity")
      .evaluate((e) => e.scrollIntoView({ block: "start" }));
    await page.screenshot({
      path: resolve("../../docs/site/img/claim-validation.png"),
    });
  }
  await check.getByRole("button", { name: "Both values apply" }).click();
  await expect(check.getByLabel("Judgment receipt")).toContainText(
    "Keep both assertions",
  );
  await check.getByRole("button", { name: "Cancel" }).click();
  await check.getByRole("button", { name: "Confirm Monday" }).click();
  await expect(check.getByLabel("Judgment receipt")).toContainText("Monday");
  await check.getByRole("button", { name: "Cancel" }).click();
  await check
    .locator("summary")
    .filter({ hasText: "Correct the value" })
    .click();
  await check.getByLabel("Correct value").fill("Thursday");
  await check.getByRole("button", { name: "Preview correction" }).click();
  await expect(check.getByLabel("Judgment receipt")).toContainText("Thursday");
  expect(await page.evaluate(() => (window as any).approval)).toBeUndefined();
  await check.getByRole("button", { name: "Approve judgment" }).click();
  await expect(check).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).approval)).toEqual({
    root_id: ids.root,
    receipt_id: "judgment-receipt",
    expected_fingerprint: "reviewed-bytes",
  });
});
