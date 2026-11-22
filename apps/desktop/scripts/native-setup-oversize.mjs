// Synthetic-only, no provider execution; caller supplies an exclusive native-test build.
import { mkdtempSync, mkdirSync, writeFileSync, rmSync, chmodSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve, isAbsolute } from "node:path";
import { spawnSync } from "node:child_process";
import { prepareNativeEnvironment, assertNoTopicsFallback } from "./native-environment.mjs";
const desktop = resolve(import.meta.dirname, "..");
const target = process.env.CARGO_TARGET_DIR;
if (!target || !isAbsolute(target)) throw new Error("Set an absolute exclusive CARGO_TARGET_DIR");
const fixture = mkdtempSync(join(tmpdir(), "topics-native-oversize-"));
try {
  const notes = join(fixture, "notes"), appData = join(fixture, "app-data");
  mkdirSync(notes); mkdirSync(appData);
  const dated = (date, size) => `---\ndate: ${date}\n---\n# Synthetic\n${"x".repeat(size)}\n`;
  writeFileSync(join(notes, "small.md"), dated("2026-11-01", 20));
  writeFileSync(join(notes, "old.md"), dated("2020-01-01", 171008));
  writeFileSync(join(notes, "large.md"), dated("2026-11-01", 171008));
  writeFileSync(join(notes, "undated.md"), `# Synthetic\n${"x".repeat(171008)}\n`);
  writeFileSync(join(notes, "private.md"), dated("2026-11-01", 21));
  const session = prepareNativeEnvironment(fixture);
  const codexHome = join(session.HOME, ".codex"); mkdirSync(codexHome);
  const stub = join(fixture, "no-provider"); writeFileSync(stub, "#!/bin/sh\nexit 1\n"); chmodSync(stub, 0o755);
  const env = {...process.env, ...session, CODEX_HOME: codexHome, TOPICS_APP_DATA_DIR: appData,
    TOPICS_TEST_HOME: session.HOME, TOPICS_TEST_XDG_DATA: session.XDG_DATA_HOME, TOPICS_TEST_CODEX_HOME: codexHome,
    TOPICS_TEST_SANDBOX: fixture, TOPICS_TEST_WORKSPACE_ROOT: notes, TOPICS_TEST_SETUP: "1", TOPICS_TEST_CODEX: stub, TOPICS_CODEX_PATH: stub,
    TOPICS_TEST_REAL_APP_BINARY: join(target, "debug/topics-desktop"), TOPICS_TEST_APP_LAUNCHER: join(desktop, "scripts/native-app-launcher.mjs")};
  const result = spawnSync("npm", ["exec", "--", "wdio", "run", "wdio.conf.ts", "--spec", "./tests/native/setup-oversize.ts"], {cwd: desktop, env, stdio: "inherit", timeout: 180000});
  if (result.status !== 0) throw new Error(`Native oversize regression failed: ${result.error || result.status}`);
  assertNoTopicsFallback(fixture);
} finally { rmSync(fixture, {recursive: true, force: true}); }
