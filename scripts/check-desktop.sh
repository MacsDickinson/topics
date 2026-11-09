#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../apps/desktop"
# Native stages run serially and share Cargo's normal repository target by default.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$(cd ../.. && pwd)/target}"
# npm ci bootstraps the pinned local LTS runtime without changing global Node.
npm ci
export PATH="$PWD/node_modules/.bin:$PATH"
test "$(node --version)" = "v24.22.2"
npm exec -- playwright install chromium
npm exec -- node scripts/setup-types.mjs --check
npm run build
npm exec -- node --test scripts/live-setup-observation.test.mjs scripts/live-setup-lineage.test.mjs
npm test
# A missing macOS session or embedded provider fails this check; no skipped smoke.
npm run test:native
npm run test:native:workspace
npm run test:native:setup
TOPICS_SCRIPTED_REHEARSAL=1 npm run test:native:setup:rehearsal
