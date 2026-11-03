// Explicit opt-in: real installed Codex, fictional notes, isolated app/WebKit data.
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  readFileSync,
  rmSync,
  chmodSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve, isAbsolute } from 'node:path';
import { spawnSync } from 'node:child_process';
import {
  prepareNativeEnvironment,
  assertNoTopicsFallback,
} from './native-environment.mjs';
if (process.platform !== 'darwin' || process.env.TOPICS_NATIVE_LIVE !== '1')
  throw new Error(
    'Opt in with TOPICS_NATIVE_LIVE=1 on macOS; this sends only the synthetic fixture to the installed Codex runtime.',
  );
const desktop = resolve(import.meta.dirname, '..'),
  repo = resolve(desktop, '../..');
const codex = process.env.TOPICS_CODEX_PATH;
if (!codex || !isAbsolute(codex))
  throw new Error('Set TOPICS_CODEX_PATH to the installed Codex executable.');
const runtimeHome = process.env.HOME;
if (!runtimeHome || !isAbsolute(runtimeHome))
  throw new Error(
    'Installed runtime home must be an absolute path; no credential files are inspected.',
  );
const fixture = mkdtempSync(join(tmpdir(), 'topics-native-live-')),
  notes = join(fixture, 'notes');
mkdirSync(notes);
const source = join(notes, 'meeting.md');
const original =
  '---\ndate: 2026-11-01\n---\n# Synthetic project meeting\n\nAlex is a person. Alex uses he/him pronouns. Alex leads the Ledger project.\n';
writeFileSync(source, original);
const session = prepareNativeEnvironment(fixture);
const quote = (value) => "'" + value.replaceAll("'", "'\\''") + "'";
const wrapper = join(fixture, 'installed-codex');
// Only the installed runtime keeps its own HOME, so it can own its existing
// subscription. The GUI and all Topics configuration retain synthetic HOME.
writeFileSync(
  wrapper,
  `#!/bin/sh\nHOME=${quote(runtimeHome)} exec ${quote(codex)} "$@"\n`,
);
chmodSync(wrapper, 0o700);
const env = {
  ...process.env,
  TOPICS_APP_DATA_DIR: join(fixture, 'app-data'),
  TOPICS_TEST_HOME: session.HOME,
  TOPICS_TEST_XDG_DATA: session.XDG_DATA_HOME,
  TOPICS_TEST_SANDBOX: fixture,
  TOPICS_TEST_WORKSPACE_ROOT: notes,
  TOPICS_TEST_CODEX: wrapper,
  TOPICS_TEST_LIVE: '1',
  TOPICS_TEST_EXPECTED: join(fixture, 'expected.json'),
  VITE_NATIVE_TEST: '1',
  TOPICS_TEST_REAL_APP_BINARY: join(repo, 'target/debug/topics-desktop'),
  TOPICS_TEST_APP_LAUNCHER: join(desktop, 'scripts/native-app-launcher.mjs'),
  PATH: `${join(desktop, 'node_modules/.bin')}:${process.env.PATH}`,
};
function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: desktop,
    env,
    stdio: 'inherit',
    timeout: 600000,
  });
  if (result.error) throw result.error;
  if (result.status !== 0)
    throw new Error(`${command} exited ${result.status}`);
}
try {
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
  run('npm', ['exec', '--', 'wdio', 'run', 'wdio.conf.ts']);
  // Separate WDIO invocation launches a fresh native process against the same data.
  for (const file of ['index.sqlite3', 'index.sqlite3-wal', 'index.sqlite3-shm', 'index.sqlite3-journal']) rmSync(join(env.TOPICS_APP_DATA_DIR, file), { force: true });
  env.TOPICS_TEST_LIVE_RESTART = '1';
  run('npm', ['exec', '--', 'wdio', 'run', 'wdio.conf.ts']);
  if (readFileSync(source, 'utf8') !== original)
    throw new Error('Synthetic source was changed');
  assertNoTopicsFallback(fixture);
} finally {
  rmSync(fixture, { recursive: true, force: true });
}
