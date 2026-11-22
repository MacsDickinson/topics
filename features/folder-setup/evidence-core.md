# Folder setup core/service evidence

Workstream result: canonical Rust contracts, focused core/store modules, executable
service BDD and synthetic regressions are implemented. This is ready for the
coordinator's fresh independent Claude review; it is not final product approval.
Approved baseline is `f9b6508` (2026-11-09). Approved scenario and plan files were
not edited. No runtime/job/desktop source, root manifest/lock or prior test assertion
was changed by this worker. Manager-added optional TS dependency changes in the
working tree are intentionally excluded from these commits.

## Commits and ordering evidence

- `bfe9f65`: frozen canonical contract, compilable core/store stubs, explicit step
  registrations and actual runner/scaffold RED (20 failed scenarios at inventory).
  This was before behavior, but it is **not semantic RED for every approved scenario**.
- `4815346`: runnable inventory/manifests/jobs/guarded draft publication and grouped
  review. The coordinator released it to independent job adapter tests.
- `b57be61`: stronger service/BDD behavior, bounded decisions/history/authority
  fixes, additive reconsent/connection/context/classification contract and five
  genuine semantic RED tests committed before those additions' behavior.
- `7488504`: five additive cases GREEN; preserved original consent/batch provenance,
  outstanding-only capabilities, connection confirmation and supplemental context.
- `7154524`: bounded draft context and the actual 100/100 selected coverage RED.
  It demonstrated `selected=100, processed=100, excluded=1`, while the old predicate
  incorrectly reported incomplete coverage.
- `ddc2c4d`: coordinator-authorized selected-coverage correction and same test GREEN;
  excluded paths remain disclosed and never transmitted/count as processed.
- Final evidence commit records the completed checks and explicit scenario mapping.

Later intermediate logs are retained for audit; they are not relabelled as
pre-implementation evidence. The initial runner-only RED is an explicit process
limitation. [The scenario map](scenario-evidence.md) covers all 21 approved scenarios
and identifies presentation/runtime acceptance still owned by other workers.

## Verified checks

Rust: `rustc 1.98.1 (48a229cea 2026-09-01)`. Each Cargo build/test used this
worker's `/tmp/topics-setup-core-target`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`. Tests use synthetic
throwaway notes and explicit absolute per-fixture application data; the command
environment also sets `TOPICS_APP_DATA_DIR=/tmp/topics-setup-core-isolated-tests`.
No global environment, services, live notes, notes symlinks, user auth/config or
providers were used. No git operation ran inside a notes fixture; no source note
was rewritten. Disk was checked before building and remained above 9 GiB at final
checks. No sleeps acknowledge processing.

```sh
cargo fmt --all --check
cargo clippy --locked -p topics-core -p topics-store -p topics-cli -p topics-bdd \
  --all-targets --features topics-store/test-support -- -D warnings
cargo test --locked -p topics-core -p topics-store -p topics-cli -p topics-bdd \
  --features topics-store/test-support
cargo test --locked -p topics-store --test setup_service
cargo run --locked -p topics-bdd --bin setup-bdd
cargo run --locked -p topics-bdd --bin workspace-bdd
cargo run --locked -p topics-bdd --bin topics-bdd -- --tags 'not @ui'
git diff --check
```

The commands above have the environment prefixes described above. Results:

| Check | Result / evidence |
|---|---|
| Relevant unit/integration/CLI regressions | **235 passed, 0 failed, 0 ignored**; `evidence/regressions-green.txt` |
| Folder setup boundary regressions | **23 passed**, including actual fault-injection partial recovery under test-support; included in the 235 |
| Additive semantic/budget/coverage cases | **7 passed**; included in the 235; original 5-case RED and selected-coverage RED are retained separately |
| Default package target without test-support | **22 passed**, fault-only test is compile-gated; `evidence/package-default-green.txt` |
| Folder setup Cucumber | **20 scenarios / 142 steps**, fail_on_skipped; `evidence/bdd-green.txt`; one graph/list presentation scenario excluded explicitly |
| Prior workspace Cucumber | **20 scenarios / 110 steps**; `evidence/workspace-bdd-green.txt` |
| Phase 0 service Cucumber | **15 scenarios / 78 steps**, not @ui; `evidence/phase0-bdd-green.txt` |
| Format / warning-denied clippy | exit 0; `evidence/fmt-green.txt`, `evidence/clippy-green.txt` |

The existing final-confirmation (39), privacy re-review (31), original review
probes (47), reanalysis (21), review regressions (10), workspace service (24) and
safety/coherence tests all remain present and passed. No substring/instruction
value-authority grant, private identity oracle or journal/undo assertion was weakened.

The 100-document BDD fixture records eligible bytes, actual scoped bytes read,
100 selected/processed paths, five bounded service/model attempts, one manifest
consent and one human save. Its measured fixture was **3,090 bytes**, with the final recorded journey at
**23,312 ms**; the exact run line is in `evidence/bdd-green.txt`. These are synthetic service attempts, not provider usage
or real-model extraction-quality evidence; timing includes concurrent Cucumber
fixtures and is not an import-duration promise. A separate focused 100-document
coverage case preserves the app-owned exclusion while verifying selected completion.

## Boundaries exercised

- Human inventory, role overrides, unfamiliar paths, dates/links/bytes and hidden,
  attachment, symlink and local-only exclusions. Roles are retrieval hints, never
  sensitivity permissions or truth rankings.
- Exact immutable manifest consent; current root epoch, source IDs/hashes/privacy,
  active scope/run/attempt checks; stale source and root switch failures. Explicit
  renewed consent preserves job identity, prior immutable manifests and original
  batch/scope/run provenance, user edits and unchanged completed source coverage.
- Draft graph/canonical IDs/checkpoints persist outside SQLite. Preparation,
  canonicalization, graph merge and JSON flush precede the permit callback; one
  final rename exposes graph and completed checkpoint together. Gate denial and
  late/reused attempts expose no new draft. Successful empty terminal turns use
  that same guard; model proposal schemas remain nonempty.
- Cross-batch context includes only still-eligible completed same-manifest evidence
  plus current batch context. It excludes private labels/alternatives, unrelated
  inventory and human statement text; accepted context still uses filtered
  source_context. Context is bounded to 200 records/1 MiB after filtering.
- All distinct citations remain available. Verbatim passage hashes group copied
  evidence conservatively, including passages under different document headings;
  group count never means independent or personal confirmation. No fuzzy identity
  merge or arbitrary semantic copy recognition is claimed.
- Grouped issues, exact receipt dependency closure, stale hashes/revisions,
  partial save/defer rather than reject-omitted behavior, confirmed-current change
  priority, and explicit reported/inferred/unsupported review categories. No
  briefing prose has no alternate acceptance route; source-free claims remain
  ineligible or shape-rejected, and summary rendering still needs UI evidence.
- Baselines use FromNotes and grant no human value authority or durable cloud
  permission. Confirmation applies only enumerated claim revisions/connections.
  Actual corrections preserve exact statements, before/after revisions and scoped
  rules; unrelated facts remain FromNotes. Legacy acceptance remains unverified
  unless actual human value change/literal authority is identifiable; merely
  mentioning a statement such as “check with legal” invents no verification.
- Supplemental context is an exact human note with a distinct journal purpose,
  separate from correction/edit/chat evidence. It creates no fact, rule or
  verification and reconstructs from committed Markdown.
- Approved records, receipts and review metadata reconstruct from Markdown,
  including loss of app-data/index for accepted knowledge. Normal restart/index
  rebuild preserves drafts and human edits. Bounded multi-operation decisions
  expose completed/partial/conflict results, replay committed receipts after an
  interruption and preserve independent later work on dependency-checked undo.

## Limits and remaining work

Scopes retain 100 files/2 MiB; raw model calls retain 1–200 candidates/2 MiB.
The adapter may use its stricter 64-candidate combined buffer. Jobs permit at most
1000 batches and 32 total one-turn attempts per batch (desktop default four),
counting failures/cancellations. Human confirmations enumerate at most 1000 IDs;
corrections at most 30 matching revisions. Save operations contain at most 100
records and 30 candidates, including statement history, metadata and receipt;
operation count is previewed, and a single overlarge history fails explicitly.
No cross-file atomicity or power-loss guarantee is claimed. Existing limits on
same-user/uncooperative writers, cross-app-data coordination and modified-copy
recognition remain. Supplemental notes never silently enter model context.

`coverage.complete` / Finished means exact **selected-manifest** coverage only.
Intentional exclusions remain visible and are never represented as analysed;
selected failures/outstanding inputs produce gaps. No whole-vault understanding
or correctness is claimed from source counts or model success.

The job worker owns real CommitPermit/runtime barriers, scripted/native call counts,
facade sequencing and integration races. Desktop owns commands, generated TS,
graph/list/briefing rendering and visible approval-action counts. The manager owns
shared manifests, integration, genuine-model/native acceptance, CI and the fresh
independent Claude review. This worker did not run the full desktop/runtime/native
check script or a provider. No push, merge or main reset was performed.

## Changed source paths

The complete worker path list is [evidence/changed-paths.txt](evidence/changed-paths.txt)
(45 paths including retained RED/GREEN logs). Canonical types are in
`crates/topics-core/src/setup/{mod,inventory,jobs,review,tests}.rs`, re-exported from
core. Store implementation is in `workspace/setup/{inventory,jobs,reconsent,graph,
draft,review,decisions,context}.rs`; workspace context/persistence/scopes/proposal
helpers supply locking, guarded publication and durable review projections.
Synthetic fixtures/tests are `topics-store/tests/{setup_service,setup_amendments}.rs`
and `tests/support/setup.rs`; exact Gherkin bindings are `topics-bdd/src/bin/setup-bdd.rs`.
No CLI code change was necessary; existing CLI regressions use the same store.
