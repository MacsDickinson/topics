import test from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { mkdtempSync, mkdirSync, existsSync, rmSync } from 'node:fs';
import { join, isAbsolute } from 'node:path';
import { tmpdir } from 'node:os';

test('proxy dispatch exits before GUI, root discovery or personal configuration', () => {
  const binary = process.env.TOPICS_TEST_REAL_APP_BINARY;
  assert.ok(
    binary && isAbsolute(binary),
    'Test requires the built native test binary',
  );
  const sandbox = mkdtempSync(join(tmpdir(), 'topics-proxy-entry-'));
  const home = join(sandbox, 'home');
  mkdirSync(home);
  const appData = join(sandbox, 'unused-app-data');
  try {
    const result = spawnSync(binary, ['--topics-mcp-proxy'], {
      env: { HOME: home, PATH: '/usr/bin:/bin', TOPICS_APP_DATA_DIR: appData },
      input: '',
      encoding: 'utf8',
      timeout: 3000,
    });
    assert.equal(
      result.error,
      undefined,
      `Proxy must terminate promptly: ${result.error}`,
    );
    assert.notEqual(
      result.status,
      0,
      'Missing scoped proxy credentials must fail closed',
    );
    assert.equal(
      existsSync(appData),
      false,
      'Proxy never initializes app data',
    );
    assert.equal(
      existsSync(
        join(home, 'Library/Application Support/com.macsdickinson.topics'),
      ),
      false,
    );
    assert.equal(result.stdout.includes('<html'), false);
  } finally {
    rmSync(sandbox, { recursive: true, force: true });
  }
});
