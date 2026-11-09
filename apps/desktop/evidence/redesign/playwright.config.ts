import { defineConfig } from '@playwright/test';
export default defineConfig({
  testDir: '../../tests', testMatch: '*.spec.ts', timeout: 15000,
  use: { baseURL: 'http://127.0.0.1:14337', screenshot: 'only-on-failure' },
  webServer: {
    cwd: new URL('../../', import.meta.url).pathname,
    command: 'node_modules/node/bin/node node_modules/vite/bin/vite.js --host 127.0.0.1 --port 14337 --strictPort',
    url: 'http://127.0.0.1:14337', reuseExistingServer: false,
  },
});
