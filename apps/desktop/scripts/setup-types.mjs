// Generate into a fresh directory so deleted canonical types cannot linger.
import {
  mkdtempSync,
  readdirSync,
  readFileSync,
  rmSync,
  mkdirSync,
  cpSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
const desktop = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const root = resolve(desktop, "../..");
const destination = join(desktop, "src/generated");
const temp = mkdtempSync(join(tmpdir(), "topics-setup-types-"));
try {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "-p",
      "topics-core",
      "--features",
      "typescript",
      "--example",
      "export_setup_types",
      "--",
      temp,
    ],
    { cwd: root, stdio: "inherit" },
  );
  if (result.status !== 0) process.exitCode = result.status ?? 1;
  else if (process.argv.includes("--check")) {
    const files = (dir) =>
      readdirSync(dir, { recursive: true, withFileTypes: true })
        .filter((entry) => entry.isFile())
        .map((entry) =>
          join(entry.parentPath, entry.name).slice(dir.length + 1),
        )
        .sort();
    const generated = files(temp);
    const existing = files(destination);
    if (
      JSON.stringify(generated) !== JSON.stringify(existing) ||
      generated.some(
        (file) =>
          !readFileSync(join(temp, file)).equals(
            readFileSync(join(destination, file)),
          ),
      )
    ) {
      console.error(
        "Generated setup types differ; run node scripts/setup-types.mjs.",
      );
      process.exitCode = 1;
    }
  } else {
    rmSync(destination, { recursive: true, force: true });
    mkdirSync(destination, { recursive: true });
    cpSync(temp, destination, { recursive: true });
  }
} finally {
  rmSync(temp, { recursive: true, force: true });
}
