import { resolve } from "node:path";
import { mkdirSync, writeFileSync } from "node:fs";
import { release } from "node:os";
import { assertNativeEnvironment } from "./scripts/native-environment.mjs";
if (!process.env.TOPICS_TEST_SANDBOX)
  throw new Error(
    "Run npm run test:native to create isolated synthetic fixtures.",
  );
const sessionEnvironment = {
  HOME: process.env.TOPICS_TEST_HOME,
  XDG_DATA_HOME: process.env.TOPICS_TEST_XDG_DATA,
};
assertNativeEnvironment(sessionEnvironment, process.env.TOPICS_TEST_SANDBOX);
const live = process.env.TOPICS_TEST_LIVE === "1";
const setup = process.env.TOPICS_TEST_SETUP === "1";
const liveSetup = process.env.TOPICS_TEST_LIVE_SETUP === "1";
const session = liveSetup
  ? process.env.TOPICS_TEST_REHEARSAL === "1"
    ? process.env.TOPICS_TEST_SETUP_RESTART === "1" ? "rehearsal-restart" : "rehearsal"
    : process.env.TOPICS_TEST_SETUP_RESTART === "1" ? "live-setup-restart" : "live-setup"
  : setup
  ? process.env.TOPICS_TEST_SETUP_FAILURE === "1"
    ? "setup-failure"
    : process.env.TOPICS_TEST_SETUP_RECONSENT === "1"
      ? "setup-reconsent"
      : process.env.TOPICS_TEST_SETUP_RESTART === "1"
        ? "setup-restart"
        : "setup"
  : process.env.TOPICS_TEST_STARTUP === "1"
    ? process.env.TOPICS_APP_DATA_DIR
      ? "relative"
      : "empty"
    : live
      ? process.env.TOPICS_TEST_LIVE_RESTART === "1"
        ? "live-restart"
        : "live"
      : process.env.TOPICS_TEST_WORKSPACE === "1"
        ? "workspace"
        : "normal";
const outputDir = resolve("evidence/native-diagnostics", session);
mkdirSync(outputDir, { recursive: true });
// Deliberately no full environment dump: credentials may be present in CI.
writeFileSync(
  resolve(outputDir, "session.json"),
  JSON.stringify(
    {
      session,
      platform: process.platform,
      osRelease: release(),
      node: process.version,
      homePrepared: true,
      webkitIncognito: true,
      driverProvider: "embedded",
    },
    null,
    2,
  ),
);
const appBinaryPath = process.env.TOPICS_TEST_APP_LAUNCHER!;
const appEnvironment = {
  ...sessionEnvironment,
  ...(process.env.TOPICS_TEST_CODEX_HOME
    ? { CODEX_HOME: process.env.TOPICS_TEST_CODEX_HOME }
    : {}),
  TOPICS_TEST_PROCESS_LOG: resolve(outputDir, "app-process.jsonl"),
};
export const config = {
  outputDir,
  runner: "local",
  specs: [
    liveSetup
      ? process.env.TOPICS_TEST_SETUP_RESTART === "1" ? "./tests/native/live-setup-restart.ts" : "./tests/native/live-setup.ts"
      : setup
      ? process.env.TOPICS_TEST_SETUP_FAILURE === "1"
        ? "./tests/native/setup-failure.ts"
        : process.env.TOPICS_TEST_SETUP_RECONSENT === "1"
          ? "./tests/native/setup-reconsent.ts"
          : process.env.TOPICS_TEST_SETUP_RESTART === "1"
            ? "./tests/native/setup-restart.ts"
            : "./tests/native/setup.ts"
      : live
        ? process.env.TOPICS_TEST_LIVE_RESTART === "1"
          ? "./tests/native/live-restart.ts"
          : "./tests/native/live-workspace.ts"
        : process.env.TOPICS_TEST_STARTUP === "1"
          ? "./tests/native/startup.ts"
          : process.env.TOPICS_TEST_WORKSPACE === "1"
            ? "./tests/native/workspace.ts"
            : "./tests/native/search.ts",
  ],
  maxInstances: 1,
  capabilities: [
    { browserName: "tauri", "tauri:options": { application: appBinaryPath } },
  ],
  services: [
    [
      "@wdio/tauri-service",
      {
        driverProvider: "embedded",
        appBinaryPath: appBinaryPath,
        env: appEnvironment,
        captureBackendLogs: !liveSetup,
        captureFrontendLogs: !liveSetup,
      },
    ],
  ],
  afterTest: async (
    _test: unknown,
    _context: unknown,
    result: { passed: boolean },
  ) => {
    if (!result.passed) {
      const { browser } = await import("@wdio/globals");
      await browser
        .saveScreenshot(resolve(`evidence/native-${session}-failure.png`))
        .catch(() => undefined);
    }
  },
  framework: "mocha",
  reporters: ["spec"],
  logLevel: "warn",
  waitforTimeout: 10000,
  connectionRetryCount: 0,
  mochaOpts: { timeout: setup || live || liveSetup ? 240000 : 30000 },
};
