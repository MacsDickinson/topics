#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ $# -gt 1 || ( $# -eq 1 && "$1" != "--core-only" ) ]]; then
  echo 'Usage: scripts/check.sh [--core-only]' >&2
  exit 2
fi
scope=(--locked --workspace)
if [[ "${1:-}" == "--core-only" ]]; then
  scope+=(--exclude topics-desktop)
fi
cargo fmt --all --check
cargo clippy "${scope[@]}" --all-targets -- -D warnings
cargo test "${scope[@]}"
cargo run --locked -p topics-bdd --bin topics-bdd -- --tags 'not @ui'
cargo run --locked -p topics-bdd --bin workspace-bdd
if [[ "${1:-}" != "--core-only" ]]; then
  bash scripts/check-desktop.sh
fi
