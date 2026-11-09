#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../apps/desktop"
# npm ci bootstraps the pinned local LTS runtime without changing global Node.
npm ci
export PATH="$PWD/node_modules/.bin:$PATH"
test "$(node --version)" = "v24.22.2"
npm exec -- playwright install chromium
npm exec -- node scripts/setup-types.mjs --check
npm run build
npm test
# A missing macOS session or embedded provider fails this check; no skipped smoke.
npm run test:native
npm run test:native:workspace
npm run test:native:setup
