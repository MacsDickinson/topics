# Native source snapshot regression

Reproduced on reviewed main `6da723c` with an isolated synthetic fixture and instrumented runner. Both native search/coherence tests passed, then the outer whole-tree comparison failed. The original `memory/projects/ledger-migration.md` had the same SHA-256 before and after: `0740071434042793c1c99fee7355b7f27bbe0e6707f538296e404ec0a586a4d3`.

The only added entries were root-level `topics/operations/<id>.md` and `topics/sources/<id>.md`. The operation identifies itself as `Register setup source inventory` and the source record references the unchanged original hash. The app-owned root `topics` directory therefore changed, not the synthetic source note.

The source guard now hashes all other files, including nested directories named `topics`, and still rejects symlinks including at the reserved root path. `native-source-snapshot-red.txt` records the pre-fix regression; `native-source-snapshot-green.txt` verifies the reserved directory exception, a changed source, a new source, nested `topics`, and a reserved-path symlink. `native-source-snapshot-native-green.txt` records the real native rerun after the fix.

## Debug report

- Symptom: native Phase 0 tests passed, but the outer source snapshot failed.
- Root cause: automatic inventory journalled app-owned records under the root `topics` directory; the previous guard included that directory.
- Fix: snapshot source files separately from that exact root-owned directory while rejecting symlinks and guarding all other paths.
- Evidence: preserved native before/after hashes and actual journal above; the native Phase 0 and startup rerun passed.
- Regression: `scripts/native-source-snapshot.test.mjs` exercises reserved records, modified and added notes, nested names, and symlinks.
- Status: DONE; integration and independent review remain coordinator-owned.
