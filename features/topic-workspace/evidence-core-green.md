# Core initial verification and integration mapping

This records the initial 85e9b77 checks; the subsequent independent review found acceptance blockers. See [the review-fix evidence](evidence-core-review-fixes.md) for current results and limitations.

Scenario baseline: `f3e1ed0`; all four `.feature` files and the complete Phase 0 scenario/step/snapshot suite are unchanged. Initial types `1b8ac16`, actual pre-production RED `ed401eb`, agreed human request types `cc402a6`, implementation `b91a0db`, scoped validator/statement/integrity follow-up `cbce881`. See [the actual RED record](evidence-core-red.md).

All tests use synthetic temporary roots and absolute application-data paths; CLI subprocesses receive absolute `TOPICS_APP_DATA_DIR`. No live notes, personal configuration/authentication, provider calls, notes Git, UI or live runtime were accessed in this stream. A disk-full build interruption was resolved with package-scoped Cargo cleanup in this worktree only; final builds disabled incremental artifacts to limit disk use.

## Final actual GREEN commands

```text
cargo fmt -p topics-core -p topics-store -p topics-cli -p topics-bdd --check
  exit 0
CARGO_INCREMENTAL=0 cargo clippy -p topics-core -p topics-store -p topics-cli -p topics-bdd --all-targets -- -D warnings
  Finished dev profile; exit 0; no warnings
CARGO_INCREMENTAL=0 cargo test -p topics-core -p topics-store -p topics-cli -p topics-bdd
  65 tests passed; 0 failed; 0 ignored
CARGO_INCREMENTAL=0 cargo run -p topics-bdd -- --tags 'not @ui'
  3 features; 15 scenarios passed; 78 steps passed; 0 skipped/undefined
CARGO_INCREMENTAL=0 cargo run -p topics-bdd --bin workspace-bdd
  4 features; 20 scenarios passed; 110 steps passed; 0 skipped/undefined
```

Both Cucumber entrypoints use `fail_on_skipped()`. `workspace-bdd` explicitly filters exactly the four presentation/protocol-owned scenarios below; they are mapped, not counted as passing core evidence. Its steps exercise real `WorkspaceService` methods, with completed-turn scripted adapters for domain discovery/correction and a deterministic synthetic interruption seam. No catch-all/fake steps remain. The manager must attach the four owners' actual evidence before claiming the complete 24-scenario integration green.

Test breakdown: unchanged Phase 0 BDD snapshot unit tests 2; unchanged real CLI regressions 8; workspace CLI recovery test 1; core strict serde/date/ID tests 2; store unit tests 11 (including new indexed-date boundary test and two commit-visibility/recovery conflict tests); unchanged coherence 2, review regressions 10 and safety tests 5; workspace service regressions 24. The 24 service regressions cover per-run consent/root invalidation, source/topic taint, cancelled/completed capabilities, UTF-8 citations, edited-subset idempotence, IDs/same names, durable fingerprint reconstruction, endpoint identity/privacy, compensation order/tombstones, path/symlink rejection, shared OS lock, bounded selections/dates, validate-only no publication, persisted statement retrieval, human-edit provenance and explicit stale source repair.

## All 24 scenarios

| Scenario | Evidence/owner |
|---|---|
| Choose a fresh root from the desktop | Desktop root/empty-state acceptance; not executed here |
| Preview a bounded selection before analysis | Core service BDD: passed |
| Explicit confirmation allows unknown but never local-only notes | Core service BDD: passed |
| Changed selection must be previewed again | Core service BDD: passed |
| Discovery yields cited proposals without accepting knowledge | Core service BDD: passed |
| Same-name people stay unresolved | Core service BDD: passed |
| Repeated analysis does not duplicate accepted facts | Core service BDD: passed |
| Missing Codex is actionable | Desktop unavailable-runtime/manual fallback acceptance; not executed here |
| Rebuild accepted knowledge from Markdown | Core service BDD: passed |
| Conflict preserves external and proposed bytes | Core service BDD: passed |
| Recover an interrupted multi-file operation | Core service BDD: passed |
| Recovery preserves conflicting changes | Core service BDD: passed |
| Undo creation and update with dependency checks | Core service BDD: passed |
| Switching roots isolates records and pending proposals | Core service BDD: passed |
| Edit and approve a cited proposal | Core service BDD: passed |
| Reject a proposal | Core service BDD: passed |
| A connection cannot outlive an omitted proposed topic | Core service BDD: passed |
| Correction supersedes the old claim and scopes a future rule | Core service BDD: passed |
| Stale evidence prevents approval | Core service BDD: passed |
| Competing corrections are not silently merged | Core service BDD: passed |
| Tool boundary rejects unselected and local-only sources | Core service BDD: passed |
| Unexpected tools or isolation failure block a real-note run | Runtime negative isolation/preflight tests; not executed here |
| Cancel and late model events cannot create a proposal | Runtime event lifecycle tests; not executed here (core cancellation capability regression passes) |
| Malformed model output is rejected | Core service BDD: passed |

## Integration/API guidance and limits

Cherry-pick this stream in commit order; the manager owns root `Cargo.toml`, `Cargo.lock` and `scripts/check.sh`. Root workspace dependencies need no new entries. Crate-local additions are `topics-core`: existing `serde_json.workspace`, `uuid = "1"`, `chrono = {version="0.4",default-features=false,features=["std"]}`; `topics-store`: `uuid` features `v4,v5`, `chrono` features `clock,std`; BDD: `topics-core.workspace`, `serde_json.workspace`, uuid `v4`, and store `test-support`. Cargo resolves `sha1_smol` for UUID v5. The generated root lockfile is deliberately not committed by this worker. Manager should regenerate it and run locked workspace/check-script integration.

`WorkspaceService` methods mirror canonical core requests/results; `validate_proposal(&self, StageProposalRequest) -> WorkspaceResult<()>` is non-publishing and does not consume a run; only application-owned successful completion invokes `stage_proposal`, which consumes a capability even for duplicates/malformed completions. Human-only `statement_get(&self, StatementGetRequest) -> WorkspaceResult<UserStatement>` exposes persisted correction/edit evidence without changing `TopicView`. Human-only `source_identity_repair(&self, SourceRepairRequest) -> WorkspaceResult<SourceSummary>` supports bounded explicit mapping repair, preserves restrictive labels and leaves old citations stale. The coordinator approved these additions in orchestration messages and the CONTRACT addendum.

Incoming mutation values reject unknown properties and require explicit nullable fields. Never expose human inventory/source/statement/repair, proposal approval, undo or legacy all-vault search as model tools. Scope/source/context methods validate root, immutable hashes and sensitivity on each call. The runtime's successful-terminal buffering, protocol lifecycle, actual confinement/preflight and IPC/event wiring remain integration responsibilities.

Markdown operation markers define visibility and survive derived DB deletion; snapshots/journals remain root-partitioned in application data. This provides cooperative locking and recoverable per-file replacement, without cross-file atomic rename or uncooperative-writer guarantees. Conflicts preserve versions for manual resolution; there is no conflict auto-merge. Undo refuses live dependencies and compensating-undo reversal, retaining history. The v1 parser accepts the documented JSON-in-YAML frontmatter format, excludes malformed records with warnings and does not silently import arbitrary hand-written YAML. Source selection is bounded to 100 files and 2 MiB of UTF-8 bytes; source identity repair does not repair old claim evidence automatically. File scans/source registries remain deliberately simple for this prototype.
