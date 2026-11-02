#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ $# -gt 1 || ( $# -eq 1 && "$1" != "--core-only" ) ]]; then
  echo 'Usage: scripts/check.sh [--core-only]' >&2
  exit 2
fi
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p topics-bdd -- --tags 'not @ui'
if [[ "${1:-}" != "--core-only" ]]; then
  bash scripts/check-desktop.sh
fi
