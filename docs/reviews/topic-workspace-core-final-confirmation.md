# Final confirmation: Topics core fixes N1-N4, 2c7883a..ebfbe67

Reviewer: Claude Opus 5.5, fresh context. Read-only. HEAD ebfbe67. No repo edits, provider or native GUI calls, or real notes.

## Verdict: NOT ACCEPTED

N3, N4, N5, N7, N8 and N10 hold. N1 and N2 are each still open on a narrower path than the original report.

| Item | Result |
|---|---|
| N1 private name collision | **Open (R1)**: ordinary shapes fixed, claim-endpoint connections not |
| N2 value-authority classifier | **Open (R2)**: UI edit path fixed, raw statement path not |
| N3 moved local-only bytes | Holds |
| N4 connection endpoints | Holds |
| N5 name/kind ambiguity | Holds, one low residual (NFD) |
| N6 in-proposal dedupe | Holds for the original repro, with a new regression (R3) |
| N7 repeat flag | Holds; new approval conflict (R4) in this path |
| N8 root-only reserved `topics` | Holds |
| N10 orphan correction | Holds |

## Must fix

### R1 (High). A private same-name topic still changes the model validation result and loses the batch

- **Where:** `proposals.rs:935` replaces a claim on an ambiguous topic with an unresolved candidate. A connection using `Endpoint::Claim` on that claim stays in the set. `proposals.rs:1089` then re-validates the server-derived set, and `proposals.rs:291` fails it.
- **Batch:** `Topic{name}`, `Claim` on that topic, `Topic Unrelated`, `Connection(from Claim -> Topic Unrelated)`, all citing selected evidence.
- **Observed from `validate_proposal`:**
  - Local-only topic named "SecretName", model proposes "SecretName": `Err(invalid_input, "Missing dependency: claim")`.
  - Same batch with "Nobody": `Ok(())`.
  - A local-only alias behaves the same, case-insensitively ("hiddenalias"), and either endpoint direction fails (probes a1c, a1).
- **Visible topic variant (a1d):** a claim on an accepted, visible "Joe" with a connection to that claim validates `Ok` until a local-only "joe" exists, then returns the same error.
- **Terminal stage (a1b):** `stage_proposal` returns the same error, so no proposal is published and the unrelated topic and claim are lost.
- **Model-visible result:** read from main, not run. `crates/topics-runtime/src/bridge.rs` maps a host error to "Tool request denied" and poisons the run, against "Validated and buffered" on success. That is one bit per run, and the model controls the name and shape. No private name or ID appears in the error.
- **Related:** an accepted cloud same-name topic fails the same way (a1d), so this is also an ordinary availability bug.
- **Fix direction:** when a claim becomes unresolved, make dependents on its ID and revision ID transitively human-only too. Review-pass validation must not be able to return an error that the input pass did not.

### R2 (Medium). A raw user statement still declassifies an unchanged transcript value

- **Where:** `scopes.rs:742`. `if operation.operation.description == "Record user statement" { return true; }` grants authority with no check on the value.
- **Setup:** "acquisition target is Zed" cites M/secret.md. The later run's scope is only M/ok.md. Each revision was human-approved, and none changes the value.

| Probe | Revision | Result in source_context |
|---|---|---|
| b3 | `user_statement` "check this with legal", then a manual revision with new uncertainty | value exported |
| b6 | Chat "I'm not sure about this; maybe it only applied from March": new uncertainty and valid_from | value exported |
| b7 | Chat question "what is this deal about?": new predicate, same value, statement-only evidence | value exported |
| b8 | Chat `Correction` with unchanged value and `match_value == replacement` | value exported, and it passes validation |

- **Model-visible result (b6, observed through `chat_start` then `source_context`):** `"value":"acquisition target is Zed"` with `"uncertainty":"Human-supported current value; evidence outside selected scope is redacted"`. From reading main, chat places this JSON in the prompt.
- **Why this is a defect:** the human approved metadata or a question, not a declassification. The value came from a source the human did not select for this run.
- **Holds:**
  - A control with no statement withholds the value (b0).
  - A plain uncertainty-only chat `Claim` dedupes to a no-change, so it exports nothing (b1, b2). That chat revision is dropped without being recorded.
  - A chat correction with a changed value stays usable, without old value, path or statement text (b4).
  - UI value edits export, and UI metadata edits are withheld with the exact persisted statement intact (b5).
  - Undoing a correction withholds both values (e6).
- **Fix direction:** apply the same before/after test to statements. Grant authority only if a superseded revision has a different value, and reject a no-op correction where `match_value == replacement`. b7 has no superseded revision, so it needs a decision. The simplest safe rule is no authority unless the statement text contains the value.

## Should fix (not privacy)

- **R3 (Low-Medium). Dedupe drops a supersession (e3).** `proposals.rs:974` onward. The batch `[new plain claim, revision superseding old]` with the same value merges to one candidate with `supersedes: []`. After approval, current is `["new value", "old value"]`. At 2c7883a the old value was superseded, so the stale value staying current is new. Fix: union `supersedes` or keep the superseding candidate.
- **R4 (Low). Approving a topic-only ambiguity unedited can conflict (a4).** `proposals.rs:929` rebuilds the unresolved candidate from the first dependency in sorted order. `proposals.rs:1415` then compares fingerprints. With two accepted "Joe" topics and a third proposed, a provisional ID that sorts last gets `conflict`, while one that sorts first is accepted. Regression: at 2c7883a this approval succeeded.
- **R5 (Low). One tainted source blocks a whole selection (c7).** The `tainted` exclusion was removed from preview at `scopes.rs:460`, and a reviewed path now skips the taint check in `source_sensitivity` at `scopes.rs:246`. Preview lists both M/a.md and M/b.md with no exclusion row, then `analysis_start` returns `forbidden`. At 2c7883a, M/a.md was visibly excluded and the run started. It fails closed; nothing reaches the model.

## Confirmed holding

- **N1 ordinary shapes (a2, a3):** topic-only, claim, topic-endpoint connection and model-supplied unresolved batches, with name variants "SecretName", "secretname" and "  SECRETNAME ", match the no-collision result. Validation publishes nothing. Subset approval works. After whole approval the private ID stays out of model context and remains visible to the human.
- **N3 (c1-c5):**
  - Moved and copied bytes stay restricted, on both labelled and new destinations, and the selection has no eligible sources, so `confirm_unknown: true` cannot start a run.
  - A stale-ID decision does not release them.
  - A newer private label beats an older review of the same hash.
  - Fresh review of the current mapping releases them, and undo re-restricts.
  - Source repair cannot launder a private hash.
  - Topic-taint lineage survives a move.
- **N4 (d1):** source and claim endpoints that are unselected, in either direction, expose no source ID, heading, path, claim ID, value or relation. A selected connection stays visible, and human `topic_get` keeps all of them.
- **N5 (e1):** "alex", "Alex ", "ALEX", " alex", "ALEX" with NBSP, and kind Thread all give a pending unresolved candidate with no pending claim and no merge. They/them stays current. The second-Joe probe n08 passes in the copied suite.
- **N7, N8 (e4):** repeats are flagged and collapse to one pending proposal. Nested `Projects/topics/` and `Other/TOPICS/` are eligible; root `topics/` and `ToPiCs` are excluded.
- **N10 (e5, n35, copied orphan-rule test):** after marker and app-data loss, current is empty with a warning, the orphan value is not accepted, and the source is out of scope.

## Checks run

- `fmt --check` on the four crates: exit 0.
- From an archive of ebfbe67, because `--locked` refused the worktree's Cargo.lock: **164 passed, 0 failed, 0 ignored**. Workspace BDD 20/110 and Phase 0 BDD 15/78 passed. These match the report.
- Independent probes `tests/final.rs`: 31 tests, 20 passed, 11 failed. The 11 are 4 for R1 (a1, a1b, a1c, a1d), 4 for R2 (b3, b6, b7, b8), and 1 each for R3 (e3), R4 (a4) and R5 (c7).
- Original `new_probes.current.rs`: 35 of 36 passed. The exception, n33, is an informational probe that unwraps a preview which now correctly fails closed with `conflict`. The N11 message has improved. Not a defect.
- Copy of the repo's `rereview_privacy.rs`: 31/31.
- Selected probes against an archive of 2c7883a, for regression classification (R3, R4, R5, N1, N2).

Clippy was not re-run. The worktree stayed clean; all build output went to the existing target directory.

## Residual limits, not blockers

- **Moved and edited bytes (c6):** a moved file edited by one line, or a local-only file edited and moved with no inventory in between, inherits the destination's CloudAllowed. This is the documented bounded hash limit. If the edit was inventoried first, it stays restricted. The manager may want this called out to the user.
- **NFD names (e2):** "José" in NFD form is not matched against NFC, so a claim stays pending under a new topic. A human still reviews it.
- **Logical claim endpoints (d2):** `find` picks an arbitrary revision, so a connection to a logical claim ID with a superseded in-scope revision and a current out-of-scope one appeared in 4 to 6 of 12 runs across two executions. Only the opaque claim ID and the relation, whose evidence is selected, are exposed; the value never appears.
- N9 and N12 were not reopened. Runtime, native and provider behaviour are out of scope and were read only where stated.
