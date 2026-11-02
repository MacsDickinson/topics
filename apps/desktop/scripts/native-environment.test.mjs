import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

const desktop = new URL('../', import.meta.url);
test('native WebKit is nonpersistent while production window config is unchanged', () => {
  const production = JSON.parse(readFileSync(new URL('src-tauri/tauri.conf.json', desktop)));
  const native = JSON.parse(readFileSync(new URL('src-tauri/tauri.native-test.conf.json', desktop)));
  assert.equal(production.app.windows[0].incognito ?? false, false);
  assert.equal(native.app.windows?.[0]?.incognito, true);
  const { incognito, ...nativeWindow } = native.app.windows[0];
  assert.deepEqual(nativeWindow, production.app.windows[0]);
});

import { mkdtempSync, mkdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { prepareNativeEnvironment, assertNativeEnvironment, assertNoTopicsFallback } from './native-environment.mjs';

test('all app sessions receive an existing synthetic home and targeted fallback guard', () => {
  const sandbox = mkdtempSync(join(tmpdir(), 'topics-session-env-'));
  try {
    assert.throws(() => assertNativeEnvironment({ HOME: join(sandbox, 'missing'), XDG_DATA_HOME: join(sandbox, 'missing/xdg') }, sandbox));
    const environment = prepareNativeEnvironment(sandbox);
    assertNativeEnvironment(environment, sandbox);
    assertNoTopicsFallback(sandbox); // OS home scaffolding is allowed, Topics fallback is not.
    const unexpected = join(environment.HOME, 'Library', 'Application Support', 'com.macsdickinson.topics');
    mkdirSync(unexpected);
    assert.throws(() => assertNoTopicsFallback(sandbox), /unexpected fallback app data/);
    rmSync(unexpected, { recursive: true });
    mkdirSync(join(sandbox, 'invalid-relative-app-data'));
    assert.throws(() => assertNoTopicsFallback(sandbox), /unexpected fallback app data/);
  } finally { rmSync(sandbox, { recursive: true, force: true }); }
});
