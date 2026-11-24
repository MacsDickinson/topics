import { openMoreTools, openTool } from "./fixtures/workspace";
import { test, expect, type Page } from '@playwright/test';
import { workspaceFixture, ids, topic, claim } from './fixtures/workspace';

const pageErrors = new WeakMap<Page, string[]>();
test.beforeEach(async ({ page }) => {
  const errors: string[] = [];
  pageErrors.set(page, errors);
  page.on('pageerror', (e) => errors.push(e.message));
});
test.afterEach(async ({ page }) => {
  expect(pageErrors.get(page)).toEqual([]);
});

async function preview(page: Page) {
  await openTool(page, 'Analyse notes');
  await page.getByLabel('Folders', { exact: true }).fill('Meetings');
  await page.getByLabel('From date').fill('2026-11-01');
  await page.getByLabel('Through date').fill('2026-11-02');
  await page.getByRole('button', { name: 'Preview selection' }).click();
  await expect(
    page.getByRole('heading', { name: 'Review selected notes' }),
  ).toBeVisible();
}
async function start(page: Page) {
  await preview(page);
  await page
    .getByLabel('Allow these unknown-sensitivity notes for this run')
    .check();
  await page
    .getByRole('button', { name: 'Start analysis', exact: true })
    .click();
  await expect(page.getByRole('button', { name: 'Stop run' })).toBeVisible();
}
async function finish(page: Page) {
  await page.evaluate(() => {
    (window as any).showProposal();
    (window as any).emitRun(1, {
      kind: 'proposal',
      proposal_id: '00000000-0000-4000-8000-000000000005',
    });
    (window as any).emitRun(2, { kind: 'completed' });
  });
  await openTool(page, /Inbox/);
  await expect(page.getByLabel('Claim value')).toHaveValue('Friday');
}

test('create a fresh root and explain an honest empty workspace', async ({
  page,
}) => {
  await workspaceFixture(page, { empty: true, inventoryEmpty: true });
  await page.getByLabel('Notes folder path').fill('/synthetic/new-notes');
  await page.getByLabel('Create a new notes folder').check();
  await page.getByRole('button', { name: 'Use notes folder' }).click();
  await expect(
    page.getByText(
      'Build your first graph',
    ),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () =>
        (window as any).calls.find(
          (c: any) => c.command === 'workspace_root_set',
        ).args,
    ),
  ).toEqual({ request: { path: '/synthetic/new-notes', create: true } });
});

test('bounded date preview names exclusions, requires consent and invalidates on selection edit', async ({
  page,
}) => {
  await workspaceFixture(page);
  await preview(page);
  await expect(
    page.getByText('Meetings/one.md', { exact: true }),
  ).toBeVisible();
  await expect(page.getByText('Unknown date', { exact: true })).toBeVisible();
  await expect(page.getByText('Local-only', { exact: true })).toBeVisible();
  await expect(
    page.getByRole('button', { name: 'Start analysis', exact: true }),
  ).toBeDisabled();
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter(
        (c: any) => c.command === 'workspace_analysis_start',
      ),
    ),
  ).toHaveLength(0);
  await page
    .getByLabel('Allow these unknown-sensitivity notes for this run')
    .check();
  await expect(
    page.getByRole('button', { name: 'Start analysis', exact: true }),
  ).toBeEnabled();
  await page.getByLabel('Through date').fill('2026-11-03');
  await expect(
    page.getByRole('heading', { name: 'Review selected notes' }),
  ).toHaveCount(0);
});

test('discovery, cited edit and human approval, accepted understanding, chat diff and history conflict', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await page.evaluate(() =>
    (window as any).emitRun(0, {
      kind: 'text_delta',
      text: 'Reading selected notes…',
    }),
  );
  await expect(
    page.getByText('Reading selected notes…', { exact: true }),
  ).toBeVisible();
  await finish(page);
  await expect(
    page.getByText('Ledger launch is Friday', { exact: true }),
  ).toBeVisible();
  await expect(page.getByText('Bytes 0–23', { exact: false })).toBeVisible();
  await page.getByLabel('Claim value').fill('Monday');
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await page.getByRole('button', { name: 'Ledger', exact: true }).click();
  await expect(page.getByText('Monday', { exact: true })).toBeVisible();
  await expect(
    page.getByText('Transcript; confirm the date.', { exact: true }),
  ).toBeVisible();
  await page
    .getByLabel('Message about this topic')
    .fill('The launch is Tuesday; Monday was a mistake.');
  await expect(
    page.getByRole('button', { name: 'Send message' }),
  ).toBeDisabled();
  await page.getByRole('button', { name: 'Preview chat sources' }).click();
  await page
    .getByLabel('Allow these unknown-sensitivity notes for this run')
    .check();
  await page.getByRole('button', { name: 'Send message' }).click();
  expect(
    await page.evaluate(
      () =>
        (window as any).calls.find(
          (c: any) => c.command === 'workspace_chat_start',
        ).args.request,
    ),
  ).toMatchObject({
    root_id: ids.root,
    topic_id: ids.topic,
    scope_id: ids.scope,
    confirm_unknown: true,
    message: 'The launch is Tuesday; Monday was a mistake.',
  });
  await page.evaluate(() => {
    const w = window as any;
    w.showCorrection();
    w.emitRun(1, { kind: 'proposal', proposal_id: 'correction-one' });
    w.emitRun(2, { kind: 'completed' });
  });
  await openTool(page, /Inbox/);
  await expect(page.getByText('Monday', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Claim value')).toHaveValue('Tuesday');
  await expect(page.getByLabel('Reason and context')).toHaveValue(
    'Monday was a mistake.',
  );
  await page.screenshot({
    path: 'evidence/workspace-correction.png',
    fullPage: true,
  });
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await expect(page.getByText('Tuesday', { exact: true })).toBeVisible();
  await page.locator('.topic-detail').getByText('Sources (1)', { exact: true }).first().click();
  await expect(
    page.getByText('Your correction', { exact: false }).first(),
  ).toBeVisible();
  await expect(
    page.getByText('The launch is Tuesday; Monday was a mistake.', {
      exact: true,
    }).first(),
  ).toBeVisible();
  await page.getByText('Revision history (1)', { exact: true }).click();
  await expect(
    page.getByText('launch: Monday', { exact: false }),
  ).toBeVisible();
  await page.screenshot({
    path: 'evidence/workspace-topic.png',
    fullPage: true,
  });
  await openTool(page, 'History');
  await page.getByRole('button', { name: 'Undo', exact: true }).first().click();
  await expect(page.getByRole('alert')).toContainText(
    'Undo the dependent claim first.',
  );
  await page.screenshot({ path: 'evidence/workspace-history.png' });
});

test('approval errors preserve edited candidates and rejected proposals never become topics', async ({
  page,
}, testInfo) => {
  await workspaceFixture(page);
  await start(page);
  await finish(page);
  await page.getByLabel('Claim value').fill('Monday');
  await page.evaluate(() => {
    (window as any).failures.workspace_proposal_decide = {
      code: 'stale_evidence',
      message: 'This passage changed. Preview and analyse the note again.',
      details: null,
    };
  });
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await expect(page.getByRole('alert')).toContainText('This passage changed.');
  const bar = page.getByRole('banner', { name: 'Workspace status' });
  const details = bar.getByRole('button', { name: 'Workspace status details' });
  await details.click();
  await expect(bar.getByRole('region', { name: 'Status details' })).toContainText('Your edits are kept here.');
  await page.screenshot({ path: testInfo.outputPath('workspace-alert.png'), scale: 'css' });
  await bar.getByRole('button', { name: 'Dismiss error' }).click();
  await expect(details).toBeFocused();
  await expect(page.getByRole('alert')).toHaveCount(0);
  await expect(page.getByLabel('Claim value')).toHaveValue('Monday');
  await page.evaluate(() => {
    delete (window as any).failures.workspace_proposal_decide;
  });
  await page.getByRole('button', { name: 'Reject proposal' }).click();
  await expect(page.getByText('Rejected', { exact: true })).toBeVisible();
  expect(await page.evaluate(() => (window as any).topics)).toEqual([]);
});

test('duplicate sequences, wrong roots and cancelled late streams cannot update workspace', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await page.evaluate(() => {
    const w = window as any;
    w.emitRun(1, { kind: 'text_delta', text: 'Once' });
    w.emitRun(1, { kind: 'text_delta', text: ' duplicate' });
    w.emitRun(2, { kind: 'text_delta', text: ' wrong' }, 'other-root');
  });
  await expect(page.getByText('Once', { exact: true })).toBeVisible();
  await expect(page.getByText(/duplicate|wrong/, { exact: false })).toHaveCount(
    0,
  );
  await page.getByRole('button', { name: 'Stop run' }).click();
  await page.evaluate(() => {
    const w = window as any;
    w.showProposal();
    w.emitRun(3, { kind: 'text_delta', text: 'LATE SECRET' });
    w.emitRun(4, { kind: 'proposal', proposal_id: 'late' });
    w.emitRun(5, { kind: 'completed' });
  });
  await expect(page.getByText('LATE SECRET', { exact: true })).toHaveCount(0);
  await openTool(page, /Inbox/);
  await expect(page.getByLabel('Claim value')).toHaveCount(0);
});

test('changing root cancels run and clears old topic, proposal and delayed reads', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await page.getByRole('button', { name: 'Change notes folder' }).click();
  await page.getByLabel('Notes folder path').fill('/synthetic/other-notes');
  await page.getByRole('button', { name: 'Use notes folder' }).click();
  await page.evaluate(() => {
    const w = window as any;
    w.emitRun(9, { kind: 'text_delta', text: 'OLD ROOT' });
    w.emitRun(10, { kind: 'completed' });
  });
  await expect(page.getByText('OLD ROOT', { exact: true })).toHaveCount(0);
  await expect(
    page.getByRole('button', { name: 'Ledger', exact: true }),
  ).toHaveCount(0);
  expect(
    await page.evaluate(() =>
      (window as any).calls
        .filter((c: any) => c.command === 'workspace_run_cancel')
        .map((c: any) => c.args.request),
    ),
  ).toContainEqual({ root_id: ids.root, run_id: ids.run });
});

test('missing runtime leaves topic browsing and manual proposal capture available', async ({
  page,
}) => {
  await workspaceFixture(page, { unavailable: true });
  await expect(
    page.getByText(
      'Codex is missing. Install Codex 0.166.0 and restart Topics.',
    ),
  ).toBeVisible();
  await openMoreTools(page);
  await expect(
    page.getByRole('complementary', { name: 'Topics workspace' }).getByRole('button', { name: 'New topic', exact: true }),
  ).toBeEnabled();
  await openTool(page, 'New topic');
  await expect(page.getByLabel('Topic name', { exact: true })).toBeVisible();
});

test('human source inventory includes local-only notes, reads inert text and persists sensitivity', async ({
  page,
}) => {
  await workspaceFixture(page);
  await page.getByRole('button', { name: 'Sources', exact: true }).click();
  await page
    .getByRole('button', { name: 'Meetings/private.md', exact: true })
    .click();
  await expect(page.locator('.source-inspector .badge')).toHaveText(
    'Local-only',
  );
  await expect(
    page.getByText('<img src=x onerror=alert(1)> Ledger launch is Friday', {
      exact: true,
    }),
  ).toBeVisible();
  await expect(page.locator('img')).toHaveCount(0);
  await page
    .getByRole('button', { name: 'Meetings/one.md', exact: true })
    .click();
  const sourceSensitivity = page.locator('select#source-sensitivity');
  await expect(sourceSensitivity).toHaveAccessibleName('Source sensitivity');
  await expect(sourceSensitivity).toHaveCount(1);
  await sourceSensitivity.selectOption('local_only');
  await expect
    .poll(() => page.evaluate(() => (window as any).sourceSensitivity))
    .toBe('local_only');
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter((c: any) =>
        ['workspace_analysis_start', 'workspace_chat_start'].includes(
          c.command,
        ),
      ),
    ),
  ).toHaveLength(0);
});

test('manual fallback stages a proposal and still needs explicit human approval', async ({
  page,
}) => {
  await workspaceFixture(page, { unavailable: true });
  await openTool(page, 'New topic');
  await page.getByLabel('Topic name', { exact: true }).fill('Quiet project');
  await page.getByRole('button', { name: 'Create proposal' }).click();
  await expect(page.getByLabel('Topic name', { exact: true })).toHaveValue(
    'Quiet project',
  );
  expect(await page.evaluate(() => (window as any).topics)).toEqual([]);
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  expect(
    await page.evaluate(
      () =>
        (window as any).calls.find(
          (c: any) => c.command === 'workspace_proposal_decide',
        ).args.request.decision.candidates[0].topic.name,
    ),
  ).toBe('Quiet project');
});

test('omitted dependencies fail visibly and the exact edited subset stays reviewable', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await finish(page);
  await page
    .getByRole('checkbox', { name: /^Include topic: Ledger/ })
    .uncheck();
  await page.evaluate(() => {
    (window as any).failures.workspace_proposal_decide = {
      code: 'invalid_input',
      message:
        'The claim needs its proposed Ledger topic. Select both changes.',
      details: null,
    };
  });
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await expect(page.getByRole('alert')).toContainText('Select both changes.');
  expect(
    await page.evaluate(() =>
      (window as any).calls
        .find((c: any) => c.command === 'workspace_proposal_decide')
        .args.request.decision.candidates.map((c: any) => c.kind),
    ),
  ).toEqual(['claim']);
  await expect(
    page.getByRole('checkbox', { name: /^Include topic: Ledger/ }),
  ).not.toBeChecked();
  expect(await page.evaluate(() => (window as any).topics)).toEqual([]);
});

test('late topic detail read after a root switch cannot reveal old-root evidence', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await finish(page);
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await page.evaluate(() => {
    (window as any).defer.workspace_topic_get = true;
  });
  await page.getByRole('button', { name: 'Ledger', exact: true }).click();
  await expect
    .poll(() =>
      page.evaluate(
        () => typeof (window as any).defer.workspace_topic_get.resolve,
      ),
    )
    .toBe('function');
  await page.getByRole('button', { name: 'Change notes folder' }).click();
  await page.getByLabel('Notes folder path').fill('/synthetic/other');
  await page.getByRole('button', { name: 'Use notes folder' }).click();
  await page.evaluate(
    ({ topic, claim }) =>
      (window as any).defer.workspace_topic_get.resolve({
        topic,
        current_claims: [{ ...claim, value: 'OLD ROOT SECRET' }],
        connections: [],
        unresolved: [],
        history: [],
        warnings: [],
      }),
    { topic, claim },
  );
  await expect(page.getByText('OLD ROOT SECRET', { exact: true })).toHaveCount(
    0,
  );
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await expect(
    page.getByRole('heading', { name: 'Build your first graph' }),
  ).toBeVisible();
});

test('stale scope, runtime errors and malformed events stay actionable without accepting data', async ({
  page,
}) => {
  await workspaceFixture(page);
  await preview(page);
  await page
    .getByLabel('Allow these unknown-sensitivity notes for this run')
    .check();
  await page.evaluate(() => {
    (window as any).failures.workspace_analysis_start = {
      code: 'stale_scope',
      message: 'Selected note bytes changed.',
      details: null,
    };
  });
  await page
    .getByRole('button', { name: 'Start analysis', exact: true })
    .click();
  await expect(page.getByRole('alert')).toContainText(
    'Preview the selection again',
  );
  await expect(
    page.getByRole('heading', { name: 'Review selected notes' }),
  ).toHaveCount(0);
  await page.evaluate(() => {
    delete (window as any).failures.workspace_analysis_start;
  });
  await start(page);
  await page.evaluate(() => {
    const w = window as any;
    w.emitRun(1, { kind: 'text_delta', text: { invalid: true } });
    w.emitRun(2, { kind: 'nonsense' });
    w.emitRun(3, {
      kind: 'error',
      error: {
        code: 'runtime_unavailable',
        message:
          'Runtime isolation check failed. Restart the verified runtime.',
        details: null,
      },
    });
    w.emitRun(4, { kind: 'text_delta', text: 'LATE AFTER ERROR' });
  });
  await expect(page.getByRole('alert')).toContainText(
    'Runtime isolation check failed.',
  );
  await expect(page.getByRole('button', { name: 'Stop run' })).toHaveCount(0);
  await expect(page.getByText('LATE AFTER ERROR', { exact: true })).toHaveCount(
    0,
  );
  expect(await page.evaluate(() => (window as any).topics)).toEqual([]);
});

test('an older list refresh cannot overwrite newer workspace data', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await page.evaluate(() => {
    const w = window as any;
    w.defer.knowledge_graph = true;
    w.showProposal();
    w.emitRun(1, { kind: 'completed' });
  });
  await expect
    .poll(() =>
      page.evaluate(
        () => typeof (window as any).defer.knowledge_graph.resolve,
      ),
    )
    .toBe('function');
  await page.evaluate(
    ({ topic }) => {
      const w = window as any;
      w.oldTopicsResolve = w.defer.knowledge_graph.resolve;
      delete w.defer.knowledge_graph;
      w.topics = [topic];
    },
    { topic },
  );
  await openTool(page, 'New topic');
  await page
    .getByLabel('Topic name', { exact: true })
    .fill('Fresh pending topic');
  await page.getByRole('button', { name: 'Create proposal' }).click();
  await expect(page.getByLabel('Topic name', { exact: true })).toHaveValue(
    'Fresh pending topic',
  );
  await page.evaluate(() => (window as any).oldTopicsResolve({ nodes: [], edges:[],pending:[],warnings:[],checks:{} }));
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await expect(
    page.getByRole('button', { name: 'Ledger', exact: true }),
  ).toBeVisible();
});

test('keyboard search works from topics and narrow screens keep controls reachable', async ({
  page,
}) => {
  await workspaceFixture(page);
  await page.setViewportSize({ width: 600, height: 700 });
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await page.keyboard.press('Meta+k');
  await expect(
    page.getByRole('searchbox', { name: 'Search topics' }),
  ).toBeFocused();
  await preview(page);
  await expect(
    page.getByLabel('Allow these unknown-sensitivity notes for this run'),
  ).toBeVisible();
  expect(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= window.innerWidth,
    ),
  ).toBe(true);
  await page.screenshot({
    path: 'evidence/workspace-narrow.png',
    fullPage: true,
  });
});

test('failed cancellation remains retryable and ignores late output until stop is confirmed', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await page.evaluate(() => {
    (window as any).failures.workspace_run_cancel = {
      code: 'internal',
      message: 'Run cancellation could not be confirmed.',
      details: null,
    };
  });
  await page.getByRole('button', { name: 'Stop run' }).click();
  await expect(page.getByRole('alert')).toContainText('could not be confirmed');
  await expect(page.getByRole('button', { name: 'Stop run' })).toBeEnabled();
  await page.evaluate(() => {
    const w = window as any;
    w.emitRun(1, { kind: 'text_delta', text: 'LATE AFTER STOP REQUEST' });
    delete w.failures.workspace_run_cancel;
  });
  await expect(
    page.getByText('LATE AFTER STOP REQUEST', { exact: true }),
  ).toHaveCount(0);
  await page.getByRole('button', { name: 'Stop run' }).click();
  await expect(page.getByRole('button', { name: 'Stop run' })).toHaveCount(0);
  await expect(
    page.getByText('Run stopped. No model changes were accepted.', {
      exact: true,
    }),
  ).toBeVisible();
});

test('accepted empty analysis outcome says no new changes without an approval card', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await page.evaluate(() => {
    const w = window as any;
    w.showProposal();
    w.proposals[0].status = 'accepted';
    w.proposals[0].candidates = [];
    w.emitRun(1, { kind: 'completed' });
  });
  await expect(
    page.getByText(
      'No new changes. Your accepted understanding is already up to date for this selection.',
      { exact: true },
    ),
  ).toBeVisible();
  const statusBar = page.getByRole('banner', { name: 'Workspace status' });
  const statusDetails = statusBar.getByRole('button', { name: 'Workspace status details' });
  await expect(statusDetails).toHaveAccessibleDescription(/No new changes/);
  await statusDetails.click();
  await statusBar.getByRole('button', { name: 'Dismiss notification' }).click();
  await expect(statusDetails).toBeFocused();
  await expect(statusDetails).toHaveAccessibleDescription(/Codex .* installed/);
  await openTool(page, /Inbox/);
  await expect(
    page.getByRole('heading', { name: 'No new changes' }),
  ).toBeVisible();
  await expect(
    page.getByRole('button', { name: 'Approve selected changes' }),
  ).toHaveCount(0);
  expect(
    await page.evaluate(() =>
      (window as any).calls.filter(
        (c: any) => c.command === 'workspace_proposal_decide',
      ),
    ),
  ).toEqual([]);
});

test('review F1 keyboard paths retain spaces and newlines until explicit preview', async ({
  page,
}) => {
  await workspaceFixture(page);
  await openTool(page, 'Analyse notes');
  const files = page.getByLabel('Markdown files', { exact: true });
  await files.pressSequentially('Team Notes/one.md');
  await files.press('Enter');
  await files.pressSequentially('Team Notes/two.md');
  await expect(files).toHaveValue('Team Notes/one.md\nTeam Notes/two.md');
  await files.press('Home');
  await files.pressSequentially('  ');
  await page.getByRole('button', { name: 'Preview selection' }).click();
  expect(
    await page.evaluate(
      () =>
        (window as any).calls.find(
          (c: any) => c.command === 'workspace_scope_preview',
        ).args.request.spec.files,
    ),
  ).toEqual(['Team Notes/one.md', 'Team Notes/two.md']);
});

test('review F1 keyboard aliases and correction paths keep literal drafts until approval', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await finish(page);
  const aliases = page.getByLabel('Aliases', { exact: true });
  await aliases.pressSequentially('Ledger migration, Finance team');
  await expect(aliases).toHaveValue('Ledger migration, Finance team');
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  expect(
    await page.evaluate(
      () =>
        (window as any).calls.find(
          (c: any) => c.command === 'workspace_proposal_decide',
        ).args.request.decision.candidates[0].topic.aliases,
    ),
  ).toEqual(['Ledger migration', 'Finance team']);
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await page.getByRole('button', { name: 'Ledger', exact: true }).click();
  await page
    .getByLabel('Message about this topic')
    .fill('The launch is Tuesday.');
  await page.getByRole('button', { name: 'Preview chat sources' }).click();
  await page
    .getByLabel('Allow these unknown-sensitivity notes for this run')
    .check();
  await page.getByRole('button', { name: 'Send message' }).click();
  await page.evaluate(() => {
    const w = window as any;
    w.showCorrection();
    w.emitRun(1, { kind: 'completed' });
  });
  await openTool(page, /Inbox/);
  const paths = page.getByLabel('Apply future rule to these source paths');
  await paths.clear();
  await paths.pressSequentially('Team Notes/a.md');
  await paths.press('Enter');
  await paths.pressSequentially('Team Notes/b.md');
  await expect(paths).toHaveValue('Team Notes/a.md\nTeam Notes/b.md');
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  expect(
    await page.evaluate(
      () =>
        (window as any).calls
          .filter((c: any) => c.command === 'workspace_proposal_decide')
          .at(-1).args.request.decision.candidates[0].rule.scope.source_paths,
    ),
  ).toEqual(['Team Notes/a.md', 'Team Notes/b.md']);
});

test('review F2 root boot errors are immediately actionable without runtime-only advice', async ({
  page,
}) => {
  await page.addInitScript(() => {
    (window as any).__TAURI_INTERNALS__ = {
      invoke: async (command: string) => {
        if (command === 'workspace_root_get')
          throw {
            code: 'no_root',
            message: 'The selected notes folder is unavailable.',
            details: null,
          };
        if (command === 'workspace_runtime_status')
          return { available: true, version: '0.166.0', message: 'Installed' };
        throw new Error(command);
      },
    };
  });
  await page.goto('/');
  await expect(page.getByRole('alert')).toContainText(
    'The selected notes folder is unavailable.',
  );
  await expect(page.getByLabel('Notes folder path')).toBeVisible();
  await expect(
    page.getByRole('button', { name: 'Check runtime again' }),
  ).toHaveCount(0);
});

test('review F3 dates only filter an explicit nonblank files or folder selection', async ({
  page,
}) => {
  await workspaceFixture(page);
  await openTool(page, 'Analyse notes');
  await page.getByLabel('From date').fill('2026-11-01');
  await expect(
    page.getByRole('button', { name: 'Preview selection' }),
  ).toBeDisabled();
  await page.getByLabel('Folders', { exact: true }).fill('  \n  ');
  await expect(
    page.getByRole('button', { name: 'Preview selection' }),
  ).toBeDisabled();
  await expect(
    page.getByText('Dates only filter the files and folders you choose.', {
      exact: false,
    }),
  ).toBeVisible();
});

test('review F4 refresh of the same topic preserves chat draft and consent', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await finish(page);
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await page.getByRole('button', { name: 'Ledger', exact: true }).click();
  await page
    .getByLabel('Message about this topic')
    .fill('Keep my unfinished correction.');
  await page.getByRole('button', { name: 'Preview chat sources' }).click();
  await page
    .getByLabel('Allow these unknown-sensitivity notes for this run')
    .check();
  await openTool(page, 'New topic');
  await page.getByLabel('Topic name', { exact: true }).fill('Another topic');
  await page.getByRole('button', { name: 'Create proposal' }).click();
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await page.getByRole('button', { name: 'Knowledge graph', exact: true }).click();
  await expect(page.getByLabel('Message about this topic')).toHaveValue(
    'Keep my unfinished correction.',
  );
  await expect(
    page.getByLabel('Allow these unknown-sensitivity notes for this run'),
  ).toBeChecked();
});

test('review F4 source inventory loading is distinct from an empty folder', async ({
  page,
}) => {
  await workspaceFixture(page);
  await page.evaluate(() => {
    (window as any).defer.workspace_sources_list = true;
  });
  await page.getByRole('button', { name: 'Sources', exact: true }).click();
  await expect(
    page.getByText('Loading source notes…', { exact: true }),
  ).toBeVisible();
  await expect(
    page.getByRole('heading', { name: 'No Markdown notes found' }),
  ).toHaveCount(0);
  await page.evaluate(() =>
    (window as any).defer.workspace_sources_list.resolve({ sources: [] }),
  );
  await expect(
    page.getByRole('heading', { name: 'No Markdown notes found' }),
  ).toBeVisible();
});

test('review F1 keyboard correction paths independently preserve spaces and multiple lines', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await finish(page);
  await page.getByRole('button', { name: 'Approve selected changes' }).click();
  await start(page);
  await page.evaluate(() => {
    const w = window as any;
    w.showCorrection();
    w.emitRun(1, { kind: 'completed' });
  });
  await openTool(page, /Inbox/);
  const paths = page.getByLabel('Apply future rule to these source paths');
  await paths.clear();
  await paths.pressSequentially('Team Notes/a.md');
  await paths.press('Enter');
  await paths.pressSequentially('Team Notes/b.md');
  await expect(paths).toHaveValue('Team Notes/a.md\nTeam Notes/b.md');
});

test('review F4 include labels distinguish candidates of the same kind', async ({
  page,
}) => {
  await workspaceFixture(page);
  await start(page);
  await page.evaluate(() => {
    const w = window as any;
    w.showProposal();
    const second = structuredClone(w.proposals[0].candidates[1]);
    second.claim.value = 'Monday';
    w.proposals[0].candidates.push(second);
    w.emitRun(1, { kind: 'completed' });
  });
  await openTool(page, /Inbox/);
  await expect(
    page.getByRole('checkbox', {
      name: 'Include claim: Ledger launch: Friday (change 2)',
      exact: true,
    }),
  ).toHaveCount(1);
  await expect(
    page.getByRole('checkbox', {
      name: 'Include claim: Ledger launch: Monday (change 3)',
      exact: true,
    }),
  ).toHaveCount(1);
});
