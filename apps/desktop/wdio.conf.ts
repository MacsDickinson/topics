import { resolve } from 'node:path';
if (!process.env.TOPICS_APP_DATA_DIR) throw new Error('Run npm run test:native to create isolated synthetic fixtures.');
export const config = {
  runner: 'local',
  specs: ['./tests/native/*.ts'],
  maxInstances: 1,
  capabilities: [{ browserName: 'tauri', 'tauri:options': { application: resolve(process.env.CARGO_TARGET_DIR || '../../target', 'debug/topics-desktop') } }],
  services: [['@wdio/tauri-service', { driverProvider: 'embedded', appBinaryPath: resolve(process.env.CARGO_TARGET_DIR || '../../target', 'debug/topics-desktop'), captureBackendLogs: false, captureFrontendLogs: false }]],
  framework: 'mocha', reporters: ['spec'], logLevel: 'warn',
  waitforTimeout: 10000, connectionRetryCount: 0,
  mochaOpts: { timeout: 30000 },
};
