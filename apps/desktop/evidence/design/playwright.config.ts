import { defineConfig } from '@playwright/test';
export default defineConfig({ testDir: '../../tests', testMatch: '*.spec.ts', timeout: 10000, use: { baseURL: 'http://127.0.0.1:14327', screenshot: 'only-on-failure' }, webServer: { command: 'npm run dev -- --port 14327 --strictPort', url: 'http://127.0.0.1:14327', reuseExistingServer: false } });
