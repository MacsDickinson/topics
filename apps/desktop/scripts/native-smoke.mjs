import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  readFileSync,
  readdirSync,
  rmSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, relative } from 'node:path';
import {
  prepareNativeEnvironment,
  assertNoTopicsFallback,
} from './native-environment.mjs';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';

if (process.platform !== 'darwin')
  throw new Error('This smoke test requires a real macOS app session.');
const startupOnly = process.argv.includes('--startup-only');
const workspace =
  process.argv.includes('--workspace') ||
  process.env.TOPICS_TEST_WORKSPACE === '1';
if (
  process.argv
    .slice(2)
    .some((arg) => !['--startup-only', '--workspace'].includes(arg))
)
  throw new Error('Usage: native-smoke.mjs [--startup-only] [--workspace]');
const desktop = resolve(import.meta.dirname, '..');
const repo = resolve(desktop, '../..');
const buildTarget = process.env.CARGO_TARGET_DIR
  ? resolve(process.env.CARGO_TARGET_DIR)
  : join(repo, 'target');
const fixture = mkdtempSync(join(tmpdir(), 'topics-native-'));
const notes = join(fixture, 'notes');
const newNotes = join(fixture, 'new-notes');
mkdirSync(newNotes);
writeFileSync(
  join(newNotes, 'new.md'),
  '# New ledger migration\n\nNewRoot ledger content.\n',
);
const newBefore = JSON.stringify(snapshot(newNotes));
const note = join(notes, 'memory/projects/ledger-migration.md');
mkdirSync(join(notes, 'memory/projects'), { recursive: true });
writeFileSync(
  note,
  '# Ledger migration\n\nThe ledger migration is ready for review.\n',
);
function snapshot(root) {
  return readdirSync(root, { withFileTypes: true })
    .sort((a, b) => a.name.localeCompare(b.name))
    .map((entry) => {
      const path = join(root, entry.name);
      if (entry.isSymbolicLink())
        throw new Error('Unexpected symlink in synthetic notes');
      return [
        entry.name,
        entry.isDirectory()
          ? snapshot(path)
          : createHash('sha256').update(readFileSync(path)).digest('hex'),
      ];
    });
}
const before = JSON.stringify(snapshot(notes));
for (const file of workspace
  ? ['native-workspace.png', 'native-workspace-empty.png']
  : [
      'native-ledger.png',
      'native-no-matches.png',
      'native-root-switch.png',
      'native-watcher-edit.png',
      'native-startup-empty.png',
      'native-startup-relative.png',
    ])
  rmSync(join(desktop, 'evidence', file), { force: true });
const sessionEnvironment = prepareNativeEnvironment(fixture);
const env = {
  ...process.env,
  CARGO_TARGET_DIR: buildTarget,
  TOPICS_TEST_HOME: sessionEnvironment.HOME,
  TOPICS_TEST_XDG_DATA: sessionEnvironment.XDG_DATA_HOME,
  TOPICS_APP_DATA_DIR: join(fixture, 'app-data'),
  VITE_NATIVE_TEST: '1',
  TOPICS_TEST_WORKSPACE: workspace ? '1' : '0',
  TOPICS_TEST_WORKSPACE_ROOT: join(fixture, 'workspace-notes'),
  TOPICS_TEST_CODEX: join(fixture, 'missing-codex'),
  TOPICS_TEST_SANDBOX: fixture,
  TOPICS_TEST_NEW_ROOT: newNotes,
  TOPICS_TEST_CLI: join(buildTarget, 'debug/topics'),
  TOPICS_TEST_REAL_APP_BINARY: join(buildTarget, 'debug/topics-desktop'),
  TOPICS_TEST_APP_LAUNCHER: join(desktop, 'scripts/native-app-launcher.mjs'),
  PATH: `${join(desktop, 'node_modules/.bin')}:${process.env.PATH}`,
};
function run(command, args, cwd = desktop) {
  const result = spawnSync(command, args, { cwd, env, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${command} exited ${result.status}`);
}
try {
  run('node', ['--test', 'scripts/native-environment.test.mjs']);
  if (!workspace)
    run(
      'cargo',
      ['run', '--locked', '-p', 'topics-cli', '--', 'use', notes],
      repo,
    );
  run('npm', [
    'run',
    'tauri',
    '--',
    'build',
    '--debug',
    '--no-bundle',
    '--features',
    'native-test',
    '--config',
    'src-tauri/tauri.native-test.conf.json',
  ]);
  run('node', ['--test', 'scripts/native-proxy.test.mjs']);
  if (!startupOnly) run('npm', ['exec', '--', 'wdio', 'run', 'wdio.conf.ts']);
  if (JSON.stringify(snapshot(newNotes)) !== newBefore)
    throw new Error('Native smoke modified synthetic new-root notes');
  if (JSON.stringify(snapshot(notes)) !== before)
    throw new Error('Native smoke modified the synthetic source note');
  for (const invalid of [
    '',
    relative(desktop, join(fixture, 'invalid-relative-app-data')),
  ]) {
    env.TOPICS_APP_DATA_DIR = invalid;
    env.TOPICS_TEST_STARTUP = '1';
    run('npm', ['exec', '--', 'wdio', 'run', 'wdio.conf.ts']);
    assertNoTopicsFallback(fixture);
  }
  assertNoTopicsFallback(fixture);
} finally {
  rmSync(fixture, { recursive: true, force: true });
}
