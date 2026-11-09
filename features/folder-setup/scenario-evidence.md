# Approved scenario to evidence mapping

Baseline `f9b6508`; the approved PLAN and all 21 scenario texts remain unchanged.
`setup-bdd` has exact explicit Cucumber bindings and `fail_on_skipped`: **20 service
scenarios / 142 steps** pass. Scenario 7 is deliberately excluded from this service
runner because its acceptance is presentation-owned. The other rows prove service
facets, not browser clicks, native containment, model quality or product approval.

| # | Approved scenario | Core evidence | Additional owner/evidence still required |
|---|---|---|---|
| 1 | Connecting a new folder starts local setup without transmission | `setup_inventory`: editable roles, counts, frontmatter dates, links, bytes, exclusions, original bytes; BDD | Desktop automatic trigger/tree and runtime no-transmission journey |
| 2 | Empty and returning folders have useful entry points | Empty/new/returning projection, local manual proposal, restart and existing map/IDs without reimport; BDD | Desktop entry points and folder chooser |
| 3 | One consent covers an exact unchanged job manifest | Exact manifest fingerprint, per-batch subset/run binding, unknown stays unknown, private paths absent from scoped reads; BDD + privacy tests | Actual adapter/MCP capability delegation and consent presentation |
| 4 | Changed inputs require renewed consent only for the changed scope | Old bytes denied, added file absent from old manifest, explicit `setup_job_reconsent`, preserved job/graph and prior consent; BDD + regrouped-subset test | Desktop changed-manifest preview and adapter resume |
| 5 | Batches build one sourced graph instead of independent topic lists | Canonical topic/claim/connection mapping, all citations retained, exact copied passage groups, no automatic acceptance; BDD + copy/overlap tests | Actual extraction quality and runtime turns |
| 6 | Ambiguous identities and historic disagreement remain visible | Two Joe IDs stay distinct, relationship deferred, dated conflicting leader accounts retained, no accepted winner; BDD | Topic/issue presentation and real-model quality |
| 7 | A topic map is usable as a graph and as a list | **Presentation-owned exclusion, no service GREEN claim.** Service graph/citation/assertion-kind projections are tested separately | Desktop graph/list navigation, connection styles, evidence expansion and cited prose |
| 8 | Save the eligible starting map with one explicit approval | Exact eligible receipt, journal markers, FromNotes, no human value authority/privacy upgrade, issues left open, one service decision; BDD + authority tests | One visible Save action and review labels |
| 9 | Saving a topic group preserves deferred material | Ledger saved; Harbour remains Deferred, resumable and unreviewed; BDD + default-save exclusion test | Group selection/Not now interaction |
| 10 | A group save handles visible dependencies | Receipt adds bare required Alex endpoint, excludes unrelated Alex claims, defers ambiguous endpoint relation while Ledger facts remain eligible; BDD + dependency tests | Visible receipt dependency explanation |
| 11 | Confirming a section does not confirm unseen topic facts | Exactly three revision snapshots confirmed; fourth stays FromNotes, receipt retains enumerated IDs/evidence; BDD | Enumerated section count and confirmation UI |
| 12 | Correct a fact in its topic without editing storage records | Exact statement, before/after, deterministic scope, superseded history, unrelated FromNotes and unchanged source bytes; BDD + rebuild tests | Contextual correction preview and approval presentation |
| 13 | Several occurrences of a problem create one review question | Five sources, two Joe IDs and one grouped identity issue, optional defer without blocking unrelated save; BDD | Queue grouping/evidence presentation |
| 14 | One correction can cover a clearly previewed matching set | Two dated pronoun assertions corrected together for one topic, same-name other person excluded, rule prevents bad candidate revival; BDD | Exact matching-set/rule scope preview |
| 15 | New evidence shows a topic diff without overriding confirmed understanding | Ledger Monday confirmed; Friday/new citation proposed, Monday preserved; exact repeated manifest reuses issue/job; BDD + overlap/evidence tests | Topic diff presentation and incremental adapter flow |
| 16 | Cancellation and restart retain completed draft work | Completed batch/edit survives cancellation/restart, revoked late publication absent, outstanding-only resume, no acceptance; BDD + guarded closure tests | Real CommitPermit/runtime cancellation barriers, event/root sequencing |
| 17 | Failed inputs do not masquerade as complete folder coverage | Failed/processed/outstanding/excluded lists, sanitized errors, selected gaps status, failed-only retry without duplicates; BDD | Adapter failures/retry and visible coverage wording |
| 18 | A stale receipt cannot save or confirm changed knowledge | Referenced draft revision, source bytes and accepted claim/dependency changes conflict; refreshed receipt; explicitly Deferred Harbour survives; BDD + stale tests | Actionable stale-view UI |
| 19 | Index rebuild preserves imported and personally confirmed distinctions | Same IDs/citations/history/reviews after actual SQLite deletion/reindex; corrected statement and FromNotes imports retained; checkpoint remains; BDD + fresh-app-data accepted-record reconstruction | Native restart/rebuild journey |
| 20 | Undo preserves independent later work | Independent later topic survives group undo; dependent confirmation blocks undo with IDs; original bytes and history remain; BDD | History/group-undo presentation; multi-operation receipts expose individual operations |
| 21 | Folder scale does not turn setup into a record checklist | 100 synthetic documents / 5 bounded batches, one exact authorization and one save, eligible bytes/read bytes match, coverage accounts for all selected paths and open issues; BDD | Actual adapter call counts, browser-visible action counts and native genuine-model journey |

`coverage.complete` means exact **selected-manifest** completion under the explicit
coordinator clarification. Preview exclusions remain disclosed and are never
counted as processed or transmitted; Finished never means the whole vault is learned.

Additional contract evidence: `setup_amendments` tests explicit reconsent preserving
completed subsets after batch regrouping, empty terminal checkpoints while empty
model propose stays forbidden, exact connection confirmation, supplemental context
without inferred claims/rules/verification, classification separate from verification,
and bounded 200-record/1 MiB draft context. The selected-coverage counterexample has
its own committed RED/GREEN logs.

Initial `bfe9f65` RED was a runner/scaffold RED (20 failures at inventory), not a
claim of semantic RED for every scenario. Five additive semantic RED cases were
committed in `b57be61` before their implementation in `7488504`; selected-coverage
semantic RED was committed in `7154524` before `ddc2c4d`. See [core evidence](evidence-core.md)
for commands, counts, limits and remaining integration/review work.
