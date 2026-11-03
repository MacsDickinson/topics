# Final confirmation: Topics core corrections R1-R5

Reviewer: Claude Opus 5.5, fresh context. Read-only review of commits; all builds ran from `git archive` copies in this scratchpad, on synthetic temp roots and app data. No live notes, personal configuration, provider, network or native tests. The worktree stayed clean.

**Reviewed commits:** final c637101, fix ee1ba22, RED f9bf23d, baseline ebfbe67. Main's 4ac9a24, 06f644d and 8174400 have no diff from c637101 in product source or `final_confirmation.rs`.

## Verdict: ACCEPT, with one narrow R2 counterexample for the manager to rule on

R1, R3, R4 and R5 hold. R2 holds for every case in the brief except values that appear as a substring of the user's statement (F1). I recommend not blocking on F1: it needs a human to approve that exact value, and the brief allows literal values. It does contradict R2's wording, so the call belongs to the manager.

| Item | Result |
|---|---|
| R1 private name/alias oracle | Holds |
| R2 value authority | Holds, except F1. Over-withholds in F2 (fails closed) |
| R3 dedupe supersession | Holds |
| R4 unedited ambiguity approval | Holds. F3 is a side effect |
| R5 preview exclusion | Holds |

## Findings

### F1 (Low, privacy). The literal-value test is an unbounded substring match
- **Where:** `scopes.rs:763`, `statement.text.contains(claim.value.trim())`.
- **Not a literal value (p2b):** transcript value "Zed", statement "ask Zedekiah in legal", approved new-predicate claim citing that statement. `source_context` for an unrelated scope exports "Zed".
- **The brief's own phrase (p2f):** transcript value "legal", chat statement "check with legal", approved metadata-only revision. The value is exported, though the user never asserted it. A control with the sentence value "acquisition target is Zed" stays withheld.
- **Exposure:** the value and its predicate binding from an unselected source. Statement text, path and citation are not exposed.
- **Conditions:** the value must appear verbatim in the statement, and a human must approve a claim showing that value and citing the statement.
- **Fix options:** token boundaries fix p2b only. p2f needs a policy decision, because a substring test cannot tell a question from an assertion.

### F2 (Low, availability; regression from ebfbe67). A metadata follow-up to a real correction withholds the corrected value
- **Where:** `scopes.rs:762` compares only the immediately superseded revision, so a same-value follow-up has no changed value.
- **Observed (p2a, p2e):** after a real correction to "Yak", a chat or manual revision that only adds dates or uncertainty removes "Yak" from model context. Both returned true at ebfbe67, false at c637101. "Zed" never appears.
- **Scope:** the correction on its own stays usable, and a UI value edit followed by a UI metadata edit keeps its value (p2g).
- **Fix direction:** walk the same-value supersedes chain back to a human-authored change.

### F3 (Low, human review completeness; changed from ebfbe67). A model-declared unresolved candidate hides the private alternative from the human
- **Where:** `proposals.rs:976` preserves declared unresolved candidates in Review mode as well as Approval mode.
- **Observed (p4b):** with a private "joe", a batch of Topic "JOE" plus a declared unresolved [JOE, visible Joe] stays exactly as declared. The human inbox omits the private alternative (present at ebfbe67). The server's fallback record is also skipped, because the declared record already references the topic.
- **Impact:** nothing reaches the model, and results stay uniform. R4 asks for declared alternatives to be preserved, so this is not an R4 failure.
- **Fix direction:** preserve exactly in Approval, and add server-derived alternatives in Review.

## Item evidence

**Retained assertions.** f9bf23d's test file is byte-identical to the original `final-probes.rs`. After rustfmt, the original is an exact prefix of the committed file: 0 removed lines, and only lines 1552-1826 (four tests) added. There are no production changes between ebfbe67 and f9bf23d. I did not re-run RED; the original review recorded the same 20 passed, 11 failed.

**R1.** The dependency closure covers logical and revision IDs, transitively, regardless of candidate order.
- p1 compared private name, private alias and no collision across 6 shapes: claim-to-claim, claim to unrelated claim, connection listed first, endpoint on a discarded duplicate, two case-variant topics, and source endpoint. `validate_proposal` was identical (`Ok`), staging stayed Pending, the human saw the private ID, no endpoint was stranded, and whole approval succeeded.
- p1b (visible Joe plus private joe, including an accepted-claim endpoint and a revision) and p1d (suppressed claim with dependents) were uniform.
- p1c: my lead that statement-only evidence could produce an empty unresolved record is closed. Analysis origin is rejected uniformly at the boundary, and chat origin is not converted.
- Malformed raw dependencies and guard denial are covered by the committed r1 test, which I read and ran: errors are uniform and contain no private ID, and denial leaves no pending record or temp file.

**R2.** b3, b6, b7 and b8 pass. p2d: a metadata chain and a passage-only value change stay withheld. p2c: a whitespace no-op correction and a chat no-op correction are rejected, and a real correction still exports without the old value or path. Validation requires the same logical assertion, predicate and unsuperseded tip, so a revision cannot gain authority by superseding an unrelated claim. Remaining exceptions: F1 and F2.

**R3.** p3 ran all 6 orders of plain, duplicate and superseding candidates, with endpoints on all 6 of their IDs. Each kept one claim with the established logical ID and supersedes list, unioned evidence and uncertainty, and collapsed to one connection. After approval there was one current value, and undo restored the old one. p3b: equivalent revisions of different assertions and in-batch self-supersession both return raw errors.

**R4.** a4 passes. p4 tested claim-plus-connection ambiguity with accepted and proposed UUIDs at both extremes: the alternatives set was exact, and approval succeeded without rewriting candidates.

**R5.** c7 and the r5 test pass; c7's own assertion is weak, but r5 is strict. p5, using a folder selection, visibly excluded the claim-tainted, connection-tainted and same-bytes-copy sources while the unrelated source stayed eligible. A tainted-only selection cannot start, and taint added after preview returns `forbidden`. An edited, freshly reviewed tainted path stays excluded until the topic is declassified, as documented.

## Checks run at c637101

| Check | Result |
|---|---|
| Expected `final_confirmation` command | 35 passed, 0 failed, exit 0 |
| `topics-core`, `topics-store`, `topics-cli`, `topics-bdd` with `topics-store/test-support` | 199 passed, 0 failed, 0 ignored, including rereview 31, review_probes 47, review_reanalysis 21, review_regressions 10 |
| `workspace-bdd` | 20 scenarios, 110 steps passed |
| `cargo fmt --check` on the four crates | exit 0 |
| Independent scratch probes | 17 passed, 0 failed. Several record observations rather than assert, so F1-F3 appear as OBS lines, not failures |

The same probes on an ebfbe67 archive classified F2 and F3. p5 and p4b fail there, as expected before the fixes.

**Not run:** clippy and Phase 0 BDD (manager's integration checks). Runtime and native behaviour are out of scope; "model-visible" here means the `source_context` and `validate_proposal` results.

## Residuals, not blockers
- Pre-agreed limits were not reopened: NFC/NFD aliases, hash-edited moved copies, opaque logical-endpoint revision variation, performance, uncooperative writers and wholesale metadata loss.
- Outside R5, unchanged from ebfbe67 (p5b): a connection from a private topic's claim does not taint its evidence source, because `topic_tainted_sources` matches only `Endpoint::Topic`. `topic_restricted` does follow claim endpoints. Noted only, not raised as a new requirement.

## Files
- Probes: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/r_probes.keep.rs
- Probe output: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/probes_final.out
- Baseline probe output: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/probes_base.out
- Full suite output: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/full.out
