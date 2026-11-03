#!/usr/bin/env python3
"""One source-frozen synthetic subscription verification; no credential reads."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

REPO = Path(__file__).resolve().parents[3]
EVIDENCE = Path(__file__).resolve().parent
SCOPE = ["crates/topics-runtime/src", "crates/topics-runtime/tests", "crates/topics-runtime/Cargo.toml"]

def git(*args):
    return subprocess.check_output(["git", *args], cwd=REPO).decode().strip()

def source_identity():
    paths = subprocess.check_output(["git", "ls-files", "-z", *SCOPE], cwd=REPO).split(b"\0")
    digest = hashlib.sha256()
    latest = 0
    for raw in sorted(p for p in paths if p):
        path = REPO / os.fsdecode(raw)
        digest.update(raw + b"\0" + path.read_bytes())
        latest = max(latest, path.stat().st_mtime_ns)
    return digest.hexdigest(), latest

def safe_environment():
    env = {}
    for name in ["HOME", "CODEX_HOME", "PATH", "TMPDIR", "LANG", "LC_ALL", "LC_CTYPE"]:
        value = os.environ.get(name)
        if value is not None:
            env[name] = value
    env["RUST_LOG"] = "off"
    env["CARGO_INCREMENTAL"] = "0"
    env["TOPICS_TEST_CODEX"] = "/Users/macs/.local/bin/codex"
    return env

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-prefix", choices=["rereview-final", "dotenv-final"], default="rereview-final")
    prefix = parser.parse_args().output_prefix
    subprocess.run(["git", "diff", "--exit-code", "HEAD", "--", *SCOPE], cwd=REPO, check=True)
    source_commit = git("rev-parse", "HEAD")
    source_sha, latest = source_identity()
    env = safe_environment()
    started = datetime.datetime.now(datetime.timezone.utc).isoformat()
    build = subprocess.run(["cargo", "test", "-p", "topics-runtime", "--features", "native-live-tests", "--test", "live_codex", "--test", "schema_pin", "--test", "config_override_native", "--no-run", "--message-format=json"], cwd=REPO, env=env, capture_output=True, timeout=180)
    if build.returncode:
        raise RuntimeError("Frozen native test build failed; inspect local build output")
    binaries = {}
    for line in build.stdout.splitlines():
        item = json.loads(line)
        if item.get("reason") == "compiler-artifact" and item.get("executable") and item["target"]["name"] in ["live_codex", "schema_pin", "config_override_native"]:
            binaries[item["target"]["name"]] = Path(item["executable"])
    assert len(binaries) == 3
    records = []
    with tempfile.TemporaryDirectory(prefix="topics-final-env-") as temporary:
        scratch = Path(temporary)
        hostile = {
            "CODEX_ROLLOUT_TRACE_ROOT": str(scratch / "trace"),
            "CODEX_CA_CERTIFICATE": str(scratch / "missing-ca"),
            "SSLKEYLOGFILE": str(scratch / "keylog"),
            "CODEX_ACCESS_TOKEN": "synthetic-not-auth",
            "CODEX_APP_SERVER_CHATGPT_BASE_URL": "http://127.0.0.1:1",
            "CODEX_AUTHAPI_BASE_URL": "http://127.0.0.1:1",
            "OPENAI_ORGANIZATION": "synthetic",
            "OPENAI_PROJECT": "synthetic",
            "HTTPS_PROXY": "http://127.0.0.1:1",
            "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:1",
        }
        for name in ["schema_pin", "config_override_native", "live_codex"]:
            binary = binaries[name]
            before = hashlib.sha256(binary.read_bytes()).hexdigest()
            assert binary.stat().st_mtime_ns > latest, "Binary predates source freeze"
            test_env = dict(env)
            if name == "live_codex":
                test_env.update(hostile)
            run_start = time.monotonic()
            run = subprocess.run([str(binary), "--nocapture", "--test-threads=1"], cwd=REPO, env=test_env, capture_output=True, timeout=600)
            log = run.stdout + run.stderr
            (EVIDENCE / f"{prefix}-{name}.log").write_bytes(log)
            assert run.returncode == 0, f"Final {name} failed; no automatic live retries"
            assert hashlib.sha256(binary.read_bytes()).hexdigest() == before
            records.append({"target": name, "sha256": before, "mtime_ns": binary.stat().st_mtime_ns, "elapsed_seconds": round(time.monotonic()-run_start, 3), "exit_code": run.returncode})
        assert not (scratch / "trace").exists(), "Forbidden trace path created"
        assert not (scratch / "keylog").exists(), "Forbidden keylog created"
    assert source_identity() == (source_sha, latest), "Sources changed during final run"
    subprocess.run(["git", "diff", "--exit-code", source_commit, "--", *SCOPE], cwd=REPO, check=True)
    report = {"source_commit": source_commit, "source_scope_sha256": source_sha, "latest_source_mtime_ns": latest, "started_utc": started, "finished_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "binaries": records, "source_unchanged": True, "hostile_environment_keys_tested": sorted(hostile), "trace_path_created": False, "keylog_created": False, "live_attempts": 1, "synthetic_only": True}
    (EVIDENCE / f"{prefix}-chronology.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))

if __name__ == "__main__":
    main()
