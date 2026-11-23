#!/usr/bin/env python3
"""Report focused local checks. Existing CI gates still run in full."""
import argparse
import os
from pathlib import Path
import shlex
import subprocess


ROOT = Path(__file__).resolve().parents[1]
FULL = "bash scripts/check.sh"
BUILD = "cd apps/desktop && npm run build"
BROWSER = "cd apps/desktop && npm test --"
# These are quick feedback paths. Shared inputs fall through to the full suite.
SCREEN_CHECKS = {
    "ContextReview.tsx": "context-approval.spec.ts folder-setup.spec.ts appearance.spec.ts",
    "TopicDetail.tsx": "redesign.spec.ts context-approval.spec.ts appearance.spec.ts",
    "Appearance.tsx": "appearance.spec.ts design.spec.ts window-chrome.spec.ts",
    "FolderSetup.tsx": "folder-setup.spec.ts canonical-redesign.spec.ts",
    "Search.tsx": "search.spec.ts",
    "Pilot.tsx": "pilot.spec.ts pilot-sources.spec.ts",
    "PilotSources.tsx": "pilot-sources.spec.ts pilot.spec.ts",
}


def checks_for(path):
    file = Path(path)
    if file.parent == Path("apps/desktop/src") and file.name in SCREEN_CHECKS:
        return [BUILD, f"{BROWSER} {SCREEN_CHECKS[file.name]}"], "screen and its browser journeys"
    if file.parent == Path("apps/desktop/tests") and file.name.endswith(".spec.ts") and (ROOT / file).is_file():
        return [f"{BROWSER} {shlex.quote(file.name)}"], "changed browser spec"
    if path.startswith("docs/") or path in {"AGENTS.md", "README.md"}:
        site = ["python3 docs/site/check-links.py docs/site"] if (ROOT / "docs/site/check-links.py").is_file() else []
        return site, "review the documentation and its relative links"
    if path.startswith(("crates/", "apps/desktop/src/generated/", "apps/desktop/src-tauri/")):
        return [FULL], "shared contract or Rust change reaches downstream desktop behaviour"
    return [FULL], "shared or unclassified input"


def git_paths(*args):
    output = subprocess.check_output(["git", *args], cwd=ROOT, stderr=subprocess.DEVNULL)
    return [os.fsdecode(path) for path in output.split(b"\0") if path]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="HEAD", help="compare base with the current working tree")
    args = parser.parse_args()
    try:
        base = subprocess.check_output(
            ["git", "rev-parse", "--verify", "--end-of-options", args.base + "^{commit}"],
            cwd=ROOT, stderr=subprocess.DEVNULL, text=True,
        ).strip()
        paths = git_paths("diff", "--name-only", "--no-renames", "-z", base, "--")
        paths += git_paths("ls-files", "--others", "--exclude-standard", "-z")
    except subprocess.CalledProcessError:
        print("Comparison unavailable. Run the full suite.\n  " + FULL)
        return
    commands = set()
    for path in sorted(set(paths)):
        checks, reason = checks_for(path)
        print(f"  {ascii(path)} — {reason}")
        for check in checks:
            print("    " + check)
        commands.update(checks)
    if commands:
        print("\nSuggested commands")
        # The full suite already includes the desktop build and browser tests.
        if FULL in commands:
            commands = {check for check in commands if check != BUILD and not check.startswith(BROWSER)}
        for check in sorted(commands):
            print("  " + check)
    elif not paths:
        print("No changes against the supplied base.")


if __name__ == "__main__":
    main()
