import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, symlinkSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { sourceSnapshot } from './native-source-snapshot.mjs';

test('app-owned root records do not mask changes to synthetic source notes', () => {
  const root = mkdtempSync(join(tmpdir(), 'topics-snapshot-test-'));
  try {
    mkdirSync(join(root, 'notes'));
    writeFileSync(join(root, 'notes', 'one.md'), 'original');
    const before = sourceSnapshot(root);
    mkdirSync(join(root, 'topics', 'sources'), { recursive: true });
    writeFileSync(join(root, 'topics', 'sources', 'record.md'), 'app record');
    assert.deepEqual(sourceSnapshot(root), before);
    writeFileSync(join(root, 'notes', 'one.md'), 'changed');
    assert.notDeepEqual(sourceSnapshot(root), before);
    writeFileSync(join(root, 'notes', 'one.md'), 'original');
    writeFileSync(join(root, 'notes', 'extra.md'), 'new source');
    assert.notDeepEqual(sourceSnapshot(root), before);
    rmSync(join(root, 'notes', 'extra.md'));
    mkdirSync(join(root, 'notes', 'topics'));
    writeFileSync(join(root, 'notes', 'topics', 'nested.md'), 'nested source');
    assert.notDeepEqual(sourceSnapshot(root), before);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('the reserved path cannot bypass the symlink guard', () => {
  const root = mkdtempSync(join(tmpdir(), 'topics-snapshot-link-'));
  try {
    symlinkSync(join(root, 'missing'), join(root, 'topics'));
    assert.throws(() => sourceSnapshot(root), /Unexpected symlink/);
  } finally { rmSync(root, { recursive: true, force: true }); }
});

test('a root file named topics is still guarded', () => {
  const root = mkdtempSync(join(tmpdir(), 'topics-snapshot-file-'));
  try {
    const before = sourceSnapshot(root);
    writeFileSync(join(root, 'topics'), 'unexpected file');
    assert.notDeepEqual(sourceSnapshot(root), before);
  } finally { rmSync(root, { recursive: true, force: true }); }
});
