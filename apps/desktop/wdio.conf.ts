import { resolve } from 'node:path';
import { mkdirSync, writeFileSync } from 'node:fs';
import { release } from 'node:os';
import { assertNativeEnvironment } from './scripts/native-environment.mjs';
if (!process.env.TOPICS_TEST_SANDBOX)
  throw new Error(
    'Run npm run test:native to create isolated synthetic fixtures.',
  );
const sessionEnvironment = {
  HOME: process.env.TOPICS_TEST_HOME,
  XDG_DATA_HOME: process.env.TOPICS_TEST_XDG_DATA,
};
assertNativeEnvironment(sessionEnvironment, process.env.TOPICS_TEST_SANDBOX);
const session =
  process.env.TOPICS_TEST_STARTUP === '1'
    ? process.env.TOPICS_APP_DATA_DIR
      ? 'relative'
      : 'empty'
    : 'normal';
const outputDir = resolve('evidence/native-diagnostics', session);
mkdirSync(outputDir, { recursive: true });
// Deliberately no full environment dump: credentials may be present in CI.
writeFileSync(
  resolve(outputDir, 'session.json'),
  JSON.stringify(
    {
      session,
      platform: process.platform,
      osRelease: release(),
      node: process.version,
      homePrepared: true,
      webkitIncognito: true,
      driverProvider: 'embedded',
    },
    null,
    2,
  ),
);
const appBinaryPath = process.env.TOPICS_TEST_APP_LAUNCHER!;
const appEnvironment = {
  ...sessionEnvironment,
  TOPICS_TEST_PROCESS_LOG: resolve(outputDir, 'app-process.jsonl'),
};
export const config = {
  outputDir,
  runner: 'local',
  specs: [
    process.env.TOPICS_TEST_STARTUP === '1'
      ? './tests/native/startup.ts'
      : process.env.TOPICS_TEST_WORKSPACE === '1'
        ? './tests/native/workspace.ts'
        : './tests/native/search.ts',
  ],
  maxInstances: 1,
  capabilities: [
    { browserName: 'tauri', 'tauri:options': { application: appBinaryPath } },
  ],
  services: [
    [
      '@wdio/tauri-service',
      {
        driverProvider: 'embedded',
        appBinaryPath: appBinaryPath,
        env: appEnvironment,
        captureBackendLogs: true,
        captureFrontendLogs: true,
      },
    ],
  ],
  framework: 'mocha',
  reporters: ['spec'],
  logLevel: 'warn',
  waitforTimeout: 10000,
  connectionRetryCount: 0,
  mochaOpts: { timeout: 30000 },
};
