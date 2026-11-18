import { test, expect, type Page } from '@playwright/test';
import { readFileSync, mkdirSync } from 'node:fs';

const steps: Record<string, (page: Page) => Promise<unknown>> = {
  'the desktop app is open on a synthetic notes root': async (page) => {
    await page.addInitScript(() => {
      (window as any).__TAURI_INTERNALS__ = {
        invoke: async (command: string, args: { query: string }) => {
          if (command === 'workspace_root_get')
            return Promise.resolve({
              root: { root_id: 'search-root', path: '/synthetic/search' },
            });
          if (command === 'workspace_runtime_status')
            return Promise.resolve({
              available: false,
              version: null,
              message:
                'Codex unavailable; search and manual proposals remain available.',
            });
          if (command === 'workspace_topics_list')
            return Promise.resolve({ topics: [] });
          if (command === 'workspace_proposals_list')
            return Promise.resolve({ proposals: [] });
          if (command === 'workspace_history_list')
            return Promise.resolve({ operations: [] });
          if (command !== 'search_notes')
            throw new Error(`Unexpected IPC: ${command}`);
          return args.query === 'ledger'
            ? [
                {
                  title: 'Ledger migration',
                  path: 'memory/projects/ledger-migration.md',
                  snippet: 'The ledger migration is ready for review.',
                },
              ]
            : [];
        },
      };
    });
    await page.goto('/');
    await page.getByRole('button', {name: 'Search', exact: true}).click();
  },
  'I focus the search box with Cmd+K': async (page) => {
    await expect(
      page.getByRole('searchbox', { name: 'Search notes' }),
    ).toBeVisible();
    await page.keyboard.press('Meta+k');
    await expect(
      page.getByRole('searchbox', { name: 'Search notes' }),
    ).toBeFocused();
  },
  'I type "ledger"': (page) =>
    page.getByRole('searchbox', { name: 'Search notes' }).fill('ledger'),
  'I see a result titled "Ledger migration"': async (page) => {
    await expect(
      page.getByRole('heading', { name: 'Ledger migration' }),
    ).toBeVisible();
    await expect(
      page.getByText('The ledger migration is ready for review.', {
        exact: true,
      }),
    ).toBeVisible();
  },
  'the result shows its path "memory/projects/ledger-migration.md"': (page) =>
    expect(
      page.getByText('memory/projects/ledger-migration.md', { exact: true }),
    ).toBeVisible(),
  'I search for "zzzz-no-match"': (page) =>
    page.getByRole('searchbox', { name: 'Search notes' }).fill('zzzz-no-match'),
  'I see "No matches"': (page) =>
    expect(page.getByRole('status')).toHaveText('No matches'),
};
// The approved source drives registration. Any new/undefined step fails collection;
// zero scenarios and unsupported Gherkin constructs also fail rather than skip.
const source = readFileSync(
  new URL('../../../features/phase-0/desktop_search.feature', import.meta.url),
  'utf8',
);
const scenarios: { title: string; steps: string[] }[] = [];
for (const raw of source.split('\n')) {
  const line = raw.trim();
  if (
    !line ||
    line.startsWith('@') ||
    line.startsWith('#') ||
    line.startsWith('Feature:') ||
    line === 'Proves the Tauri shell talks to the Rust core.'
  )
    continue;
  if (line.startsWith('Scenario: '))
    scenarios.push({ title: line.slice(10), steps: [] });
  else {
    const match = /^(Given|When|Then|And|But) (.+)$/.exec(line);
    if (!match || !scenarios.length || !steps[match[2]])
      throw new Error(`Undefined UI Gherkin: ${line}`);
    scenarios.at(-1)!.steps.push(match[2]);
  }
}
if (scenarios.length !== 2 || scenarios.some((s) => !s.steps.length))
  throw new Error('Expected both approved desktop scenarios');
for (const scenario of scenarios)
  test(scenario.title, async ({ page }) => {
    for (const step of scenario.steps)
      await test.step(step, () => steps[step](page));
    mkdirSync('evidence', { recursive: true });
    await page.screenshot({
      path: `evidence/browser-${scenario.title === 'Empty results say so' ? 'no-matches' : 'ledger'}.png`,
    });
  });
