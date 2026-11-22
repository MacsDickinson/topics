# Native watcher convergence regression

The full check at main `2f6c940` failed after the native search test accepted a generic `h2` as proof of convergence. The recorded failure screenshot showed the redesigned no-matches state, whose `Try another word or phrase` heading satisfied that predicate even though no search result existed.

`native-search-predicate-red.txt` records the old predicate returning true for a deterministic no-matches DOM fixture. The shared predicate now requires `Converged ledger` and `new.md` within the same `.search-result`; `native-search-predicate-green.txt` covers no matches, wrong title, wrong path and the exact result. `native-search-predicate-native.txt` records the focused real macOS native run: both Phase 0 tests and both isolated invalid-app-data startup checks passed. The watcher wait remains 5 seconds, and the external note edit is still restored in `finally`.

Inspection of the other native specs found no other generic heading-existence convergence wait. One separate incompatibility is visible by inspection: the opt-in `live-workspace.ts` expects `No accepted topics yet` immediately after connecting a nonempty new root, but the current root flow enters folder setup; it was not run because this task excludes provider calls and does not modify that separate genuine-runtime spec.
