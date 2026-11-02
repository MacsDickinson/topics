import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';

if (process.platform !== 'darwin') throw new Error('This smoke test requires a real macOS app session.');
const desktop = resolve(import.meta.dirname, '..');
const repo = resolve(desktop, '../..');
const fixture = mkdtempSync(join(tmpdir(), 'topics-native-'));
const notes = join(fixture, 'notes');
const note = join(notes, 'memory/projects/ledger-migration.md');
mkdirSync(join(notes, 'memory/projects'), { recursive: true });
writeFileSync(note, '# Ledger migration\n\nThe ledger migration is ready for review.\n');
function snapshot(root) {
  return readdirSync(root, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name)).map(entry => {
    const path = join(root, entry.name);
    if (entry.isSymbolicLink()) throw new Error('Unexpected symlink in synthetic notes');
    return [entry.name, entry.isDirectory() ? snapshot(path) : createHash('sha256').update(readFileSync(path)).digest('hex')];
  });
}
const before = JSON.stringify(snapshot(notes));
for (const file of ['native-ledger.png', 'native-no-matches.png']) rmSync(join(desktop, 'evidence', file), { force: true });
const env = { ...process.env, TOPICS_APP_DATA_DIR: join(fixture, 'app-data'), VITE_NATIVE_TEST: '1', PATH: `${join(desktop, 'node_modules/.bin')}:${process.env.PATH}` };
function run(command, args, cwd = desktop) {
  const result = spawnSync(command, args, { cwd, env, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} exited ${result.status}`);
}
try {
  run('cargo', ['run', '-p', 'topics-cli', '--', 'use', notes], repo);
  run('npm', ['run', 'tauri', '--', 'build', '--debug', '--no-bundle', '--features', 'native-test', '--config', 'src-tauri/tauri.native-test.conf.json']);
  run('npm', ['exec', '--', 'wdio', 'run', 'wdio.conf.ts']);
  if (JSON.stringify(snapshot(notes)) !== before) throw new Error('Native smoke modified the synthetic source note');
} finally {
  rmSync(fixture, { recursive: true, force: true });
}
