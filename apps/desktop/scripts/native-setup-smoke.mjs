// Runs only a local scripted app-server against synthetic temporary notes.
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  readFileSync,
  rmSync,
  existsSync,
  symlinkSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  prepareNativeEnvironment,
  assertNoTopicsFallback,
} from "./native-environment.mjs";
const desktop = resolve(import.meta.dirname, ".."),
  repo = resolve(desktop, "../..");
const target = process.env.CARGO_TARGET_DIR
  ? resolve(process.env.CARGO_TARGET_DIR)
  : join(repo, "target");
const childTemplate = join(
  repo,
  "crates/topics-setup/tests/support/scripted_child.py",
);
if (!existsSync(childTemplate))
  throw new Error(
    "The frozen scripted job fixture is not present; integrate the job checkpoint first.",
  );
const cargoLocator = spawnSync("rustup", ["which", "cargo"], {
  encoding: "utf8",
});
if (cargoLocator.status !== 0)
  throw new Error("Cannot locate the installed Rust toolchain");
const toolchainBin = dirname(cargoLocator.stdout.trim());
const registrySource = join(
  process.env.CARGO_HOME || join(process.env.HOME, ".cargo"),
  "registry",
);
if (!existsSync(registrySource))
  throw new Error("Rust package cache is unavailable");
const fixture = mkdtempSync(join(tmpdir(), "topics-native-setup-"));
process.once("exit", () => rmSync(fixture, { recursive: true, force: true }));
const notes = join(fixture, "notes"),
  appData = join(fixture, "app-data");
const sources = [];
for (let i = 0; i < 100; i++) {
  const folder = i < 20 ? "profiles" : "meetings";
  mkdirSync(join(notes, folder), { recursive: true });
  const path = join(notes, folder, `${String(i).padStart(3, "0")}.md`);
  const content =
    "# Synthetic Alex A42\n\nAlex A42 leads Ledger. Alex uses he/him.\n";
  writeFileSync(path, content);
  sources.push([path, createHash("sha256").update(content).digest("hex")]);
}
mkdirSync(join(notes, "private"), { recursive: true });
const privatePath = join(notes, "private", "private.md");
const privateContent =
  "# Synthetic private marker\n\nPRIVATE_SYNTHETIC_ONLY must stay local.\n";
writeFileSync(privatePath, privateContent);
sources.push([
  privatePath,
  createHash("sha256").update(privateContent).digest("hex"),
]);
const session = prepareNativeEnvironment(fixture),
  codexHome = join(session.HOME, ".codex");
mkdirSync(codexHome);
const cargoHome = join(fixture, "cargo-home");
mkdirSync(cargoHome);
// Use the installed package cache without reading the user's Cargo config or credentials.
symlinkSync(registrySource, join(cargoHome, "registry"), "dir");
const fixtureOutput = join(fixture, "runtime-fixture");
mkdirSync(fixtureOutput);
const env = {
  ...process.env,
  HOME: session.HOME,
  CODEX_HOME: codexHome,
  XDG_DATA_HOME: session.XDG_DATA_HOME,
  CARGO_TARGET_DIR: target,
  CARGO_HOME: cargoHome,
  CARGO_NET_OFFLINE: "true",
  TOPICS_APP_DATA_DIR: appData,
  TOPICS_TEST_HOME: session.HOME,
  TOPICS_TEST_CODEX_HOME: codexHome,
  TOPICS_TEST_XDG_DATA: session.XDG_DATA_HOME,
  TOPICS_TEST_SANDBOX: fixture,
  TOPICS_TEST_WORKSPACE_ROOT: notes,
  TOPICS_TEST_SETUP: "1",
  TOPICS_TEST_GRAPH_ONLY:
    process.env.TOPICS_SETUP_GRAPH_ONLY === "1" ? "1" : "0",
  VITE_NATIVE_TEST: "1",
  TOPICS_TEST_REAL_APP_BINARY: join(target, "debug/topics-desktop"),
  TOPICS_TEST_APP_LAUNCHER: join(desktop, "scripts/native-app-launcher.mjs"),
  PATH: `${join(desktop, "node_modules/.bin")}:${toolchainBin}:${process.env.PATH}`,
};
async function run(command, args, capture = false) {
  return await new Promise((resolveRun, rejectRun) => {
    const child = spawn(command, args, {
      cwd: desktop,
      env,
      stdio: capture ? ["ignore", "pipe", "inherit"] : "inherit",
      detached: true,
    });
    let output = "";
    if (capture)
      child.stdout.on("data", (chunk) => {
        output += chunk;
      });
    let timedOut = false;
    const killGroup = (signal) => {
      try {
        if (child.pid) process.kill(-child.pid, signal);
      } catch (error) {
        if (error.code !== "ESRCH") throw error;
      }
    };
    const timer = setTimeout(() => {
      timedOut = true;
      killGroup("SIGTERM");
      setTimeout(() => killGroup("SIGKILL"), 2000).unref();
    }, 300000);
    child.on("error", (error) => {
      clearTimeout(timer);
      rejectRun(error);
    });
    child.on("exit", (code, signal) => {
      clearTimeout(timer);
      killGroup("SIGTERM");
      if (timedOut || code !== 0)
        rejectRun(
          new Error(
            `${command} ${timedOut ? "timed out" : `exited ${code ?? signal}`}`,
          ),
        );
      else resolveRun(output);
    });
  });
}
try {
  const mode =
    process.env.TOPICS_SETUP_GRAPH_ONLY === "1" ? "graph" : "graph_claim";
  const rendered = JSON.parse(
    await run(
      "cargo",
      [
        "run",
        "-q",
        "-p",
        "topics-setup",
        "--example",
        "render_fixture",
        "--",
        fixtureOutput,
        mode,
      ],
      true,
    ),
  );
  env.TOPICS_TEST_CODEX = rendered.executable;
  env.TOPICS_CODEX_PATH = rendered.executable;
  env.TOPICS_TEST_SETUP_LOG = rendered.log_file;
  if (process.env.TOPICS_SETUP_SKIP_BUILD !== "1")
    await run("npm", [
      "run",
      "tauri",
      "--",
      "build",
      "--debug",
      "--no-bundle",
      "--features",
      "native-test",
      "--config",
      "src-tauri/tauri.native-test.conf.json",
    ]);
  if (
    process.env.TOPICS_SETUP_RECONSENT_ONLY !== "1" &&
    process.env.TOPICS_SETUP_FAILURE_ONLY !== "1"
  ) {
    await run("npm", ["exec", "--", "wdio", "run", "wdio.conf.ts"]);
    for (const file of [
      "index.sqlite3",
      "index.sqlite3-wal",
      "index.sqlite3-shm",
      "index.sqlite3-journal",
    ])
      rmSync(join(appData, file), { force: true });
    env.TOPICS_TEST_SETUP_RESTART = "1";
    await run("npm", ["exec", "--", "wdio", "run", "wdio.conf.ts"]);
  }
  if (process.env.TOPICS_SETUP_FAILURE_ONLY !== "1") {
    const reconsentRoot = join(fixture, "reconsent-notes");
    mkdirSync(join(reconsentRoot, "meetings"), { recursive: true });
    for (let i = 0; i < 41; i++)
      writeFileSync(
        join(reconsentRoot, "meetings", `${String(i).padStart(3, "0")}.md`),
        "# Synthetic Alex A42\n\nAlex A42 leads Ledger.\n",
      );
    writeFileSync(rendered.mode_file, "hold_second");
    rmSync(rendered.log_file, { force: true });
    env.TOPICS_APP_DATA_DIR = join(fixture, "reconsent-app-data");
    env.TOPICS_TEST_WORKSPACE_ROOT = reconsentRoot;
    env.TOPICS_TEST_SETUP_RECONSENT = "1";
    env.TOPICS_TEST_SETUP_RESTART = "0";
    env.TOPICS_TEST_MODE_FILE = rendered.mode_file;
    await run("npm", ["exec", "--", "wdio", "run", "wdio.conf.ts"]);
  } else {
    const failureRoot = join(fixture, "failure-notes");
    mkdirSync(failureRoot);
    writeFileSync(
      join(failureRoot, "one.md"),
      "# Synthetic note\n\nAlex leads Ledger.\n",
    );
    writeFileSync(rendered.mode_file, "fail_first");
    rmSync(rendered.log_file, { force: true });
    env.TOPICS_APP_DATA_DIR = join(fixture, "failure-app-data");
    env.TOPICS_TEST_WORKSPACE_ROOT = failureRoot;
    env.TOPICS_TEST_SETUP_FAILURE = "1";
    env.TOPICS_TEST_MODE_FILE = rendered.mode_file;
    await run("npm", ["exec", "--", "wdio", "run", "wdio.conf.ts"]);
  }
  for (const [path, expected] of sources)
    if (
      createHash("sha256").update(readFileSync(path)).digest("hex") !== expected
    )
      throw new Error("Synthetic note changed");
  assertNoTopicsFallback(fixture);
} finally {
  rmSync(fixture, { recursive: true, force: true });
}
