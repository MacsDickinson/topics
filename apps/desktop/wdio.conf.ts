import { resolve } from 'node:path';
if (!process.env.TOPICS_TEST_SANDBOX) throw new Error('Run npm run test:native to create isolated synthetic fixtures.');
export const config = {
  runner: 'local',
  specs: [process.env.TOPICS_TEST_STARTUP === '1' ? './tests/native/startup.ts' : './tests/native/search.ts'],
  maxInstances: 1,
  capabilities: [{ browserName: 'tauri', 'tauri:options': { application: resolve(process.env.CARGO_TARGET_DIR || '../../target', 'debug/topics-desktop') } }],
  services: [['@wdio/tauri-service', { driverProvider: 'embedded', appBinaryPath: resolve(process.env.CARGO_TARGET_DIR || '../../target', 'debug/topics-desktop'), env: process.env.TOPICS_TEST_STARTUP === '1' ? { HOME: resolve(process.env.TOPICS_TEST_SANDBOX!, 'isolated-home'), XDG_DATA_HOME: resolve(process.env.TOPICS_TEST_SANDBOX!, 'isolated-home/xdg') } : {}, captureBackendLogs: false, captureFrontendLogs: false }]],
  framework: 'mocha', reporters: ['spec'], logLevel: 'warn',
  waitforTimeout: 10000, connectionRetryCount: 0,
  mochaOpts: { timeout: 30000 },
};
