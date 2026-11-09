import { defineConfig } from '@playwright/test';
const port = Number(process.env.TOPICS_TEST_WEB_PORT || 1420);
if (!Number.isInteger(port) || port < 1 || port > 65535) throw new Error('TOPICS_TEST_WEB_PORT must be a valid TCP port');
const url = `http://127.0.0.1:${port}`;
export default defineConfig({ testDir: './tests', testMatch: '*.spec.ts', timeout: 10000, use: { baseURL: url, screenshot: 'only-on-failure' }, webServer: { command: `npm run dev -- --port ${port} --strictPort`, url, reuseExistingServer: false } });
