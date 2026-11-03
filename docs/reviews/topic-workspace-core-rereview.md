# Independent re-review: Topics core fixes, 85e9b77..2c7883a

Reviewer: Claude Opus 5.5. Read-only; no repo edits, commits or real notes. Commits: ab7d4f6, f56fb37, d3202eb, cf40616, 2c7883a.

## Verdict: CONDITIONAL

All 27 original findings (H1-H7, M1-M15, L1-L5) are closed. My unedited original probes all pass, with only the permitted M8 label added. The final concurrency cases hold, and accepted-knowledge recovery behaves correctly.

The condition is N1-N4 below. These are new privacy gaps, mostly in code added by the fixes or in paths the original probes did not reach. Each looks like a small local fix, though I implemented none. N5 and N6 should also be fixed, but I would not hold acceptance for them. Nothing here accepts knowledge without human approval.

## Must fix before acceptance

**N1 (High). A same-name local-only topic fails the whole analysis and reveals itself through the tool result.**
- Where: proposals.rs:763-783 and 831-837 put the local-only topic ID into a server-built unresolved candidate; proposals.rs:934-940 then runs the model boundary check on it; proposals.rs:558-561 returns the error.
- Repro: n20, n21, n60. With local-only topic "SecretName", validate_proposal for a new referenced topic "SecretName" returns forbidden "Identity includes restricted context"; "Nobody" returns ok. In n21, stage for Joe plus an unrelated Ledger topic also fails with forbidden.
- Impact: the plan says local-only knowledge must not leak through errors. As read in main (not run), desktop workspace.rs:371 and bridge.rs:355-378 suggest the model sees a denial and the run is poisoned, so the leak is one bit per run. The bigger cost is availability: a meeting naming a person flagged sensitive cannot be analysed at all.
- Fix direction: check the boundary only on model-supplied IDs. Keep server-derived ambiguity for human review, without exposing it to the model.

**N2 (Medium). Redaction exports a transcript-derived value from an unselected source.**
- Where: scopes.rs:672-674 treats any attached statement as human support; proposals.rs:1187-1190 attaches one for an uncertainty-only or subject-only edit.
- Repro: n22. Claim "acquisition target is Zed" cites unselected M/secret.md. A human edits only its uncertainty. source_context scoped to M/ok.md returns the value "Zed" with evidence redacted.
- Fix direction: only treat the value as human-supported when a human actually supplied or changed it.

**N3 (Medium). Local-only bytes moved onto a reviewed path become CloudAllowed.**
- Where: scopes.rs:232-234 returns the destination's label before the same-content check at 236-242.
- Repro: n25. M/p.md is local-only and M/q.md is labelled CloudAllowed. After `mv p.md q.md`, preview lists q.md as CloudAllowed. analysis_start only asks for confirmation on Unknown sources (scopes.rs:571-576), so by reading no confirmation is needed.
- Fix direction: tie the review to path plus hash, or check same-content history first.

**N4 (Medium). Connection context exports an unselected source's ID and heading.**
- Where: scopes.rs:678 and 740-743 check the source endpoint's label but not whether it is selected.
- Repro: n24. A connection cites selected M/ok.md and has endpoint M/secret.md with heading "Secret Merger Heading". source_context returns the source ID, locator and heading. The contract says no out-of-scope metadata.
- Fix direction: drop or redact source endpoints outside the selected scope.

## Should fix

**N5 (Medium). Name or kind variants escape identity matching, so the rule does not apply.**
- Where: proposals.rs:763-770 uses exact name and kind.
- Repro: n09 stops at its first failure, "alex", which keeps the he/him claim as pending under a new topic; "Alex " and "ALEX" go through the same exact match but were not reached. n10 does the same with kind Thread.
- Limits: the original topic stays they/them and a human must approve. Case and whitespace normalisation for the ambiguity check would cover most of it. It should only ever make the identity unresolved, never merge it.

**N6 (Low-Medium). Duplicates inside one model proposal are accepted.**
- Where: proposals.rs:896-913 compares against accepted state only.
- Repro: n50. Two identical claims with different IDs produce 2 candidates and 2 current claims. By reading, connections behave the same. Both are visible to the reviewer.

## Low

- **N7. Topic-only repeats accumulate.** Each analysis proposing only a topic "Alex" adds a new pending plain Topic with no same-name marker (proposals.rs:771-784, n51). This follows from p55, which I asked for. The UI could flag the same name.
- **N8. Nested `topics` folders vanish silently.** `Projects/topics/note.md` is missing from inventory and folder preview with no exclusion row (scopes.rs:397-401, 759-765, n53), though the reserved folder is root-only and FTS excludes only depth 1. Explicit file selection was not probed.
- **N9. Inventory cost.** 1500 notes in one folder, debug build: first inventory 19.7s, repeat 3.3s, preview of 100 files 0.8s, topics_list 0.4s, under the store lock (n54). canonical_relative stats directory entries until it finds a match (scopes.rs:29-48), which I suspect dominates but did not profile. No baseline was measured at 85e9b77, so the size of any regression is unknown. Fine for the prototype.
- **N10. Newest marker plus all app data lost.** he/him becomes current again with no warning while orphan correction and rule files sit on disk (n35). The evidence documents wholesale erasure as a limit, but this needs only one lost marker and the orphans are detectable.
- **N11. Misleading conflict text.** All markers plus app data lost: preview fails closed with conflict, but the message says "since proposal review" and the whole selection is blocked (n33).
- **N12. Redacted claims keep recorded_at**, plus claim IDs and predicate (n23). The contract's redaction list doesn't name them. A design call rather than a defect.

## Final concurrency: holds

| Case | Result | Probe |
|---|---|---|
| Pre-staged provisional duplicates | Second approval conflicts; 1 topic, 1 claim; status Conflict; refresh is a no-change | n01 |
| Pre-staged duplicates with reused IDs, and connections | Second approval conflicts | n02, n03 |
| Pre-staged provisional identity, correction accepted after staging | Conflict; 1 topic; they/them current | n04 |
| Pre-staged reused subject, rule accepted after staging | Conflict | n05 |
| Edited subset no longer has the reviewed meaning | Conflict, no silent change | n06 |
| Unrelated pre-staged proposals | No false conflict | n07 |
| Genuine second Joe with new evidence | Not merged; explicit unresolved candidate; 2 Joes; first Joe's claim intact | n08 |

## Other key areas

| Area | Result | Evidence |
|---|---|---|
| Case and Unicode aliases, reserved `topics` spellings | Hold, including NFD, uppercase and long-s spellings | n27, p21, p22 |
| Current labels and rename | Hold except N3; rename plus edit stays eligible as documented | p20, r15, r16, n26 |
| Tamper and marker loss | Fail closed; marker value kept with warning | n30, n31, n32, n34, n36, p10-p14 |
| Recovery, no resurrected facts | Holds | n37, p04, p03 |
| No-change result | Accepted, empty, no operation, no history entry; later reject conflicts | n43 |
| Redaction, positive case | No path, excerpt, statement text or old value; history scoped | n23 |
| Human search | Root `topics` excluded from scan and watcher snapshot, by reading; committed test passes | scan.rs:28-38, 130-140 |
| Guarded publication | Temp written and flushed before callback; denial leaves nothing and consumes the run; no action returns internal | n40, n41 |

Human label over a damaged record succeeds and replaces the external bytes (n38). The journal keeps the old bytes, so I treat this as an explicit human resolution, not a finding.

If a publisher runs the action and then returns Err, the proposal stays visible while stage returns Err (n42). That is a caller contract for the runtime permit, which I did not review.

## Test edits: no weakening found

- Original probes changed: p23, p54, r14 and r18 only. p54, r14 and r18 are strengthened with real assertions. p23 still asserts the model cannot choose CloudAllowed.
- Fixture: corrected() adds only the human CloudAllowed label, as documented.
- workspace_service.rs: the deleted-proposals test moves from topic-only to topic plus claim. It no longer covers topic-only repeats, a real behaviour change that follows from p55 (see N7), but it now asserts the accepted claim is unchanged.
- BDD: two step assertions now expect the approved empty no-change result and Conflict status. Feature text is unchanged.
- Evidence report: maps all 27 findings with no waiver. Its counts match my run.

## Checks actually run

All in a scratch copy of 2c7883a with the manager's Cargo.lock, reusing the core target; synthetic temp dirs only.
- cargo test for topics-core, topics-store, topics-cli and topics-bdd with test-support: 133 passed, 0 failed, 0 ignored.
- workspace-bdd: 20 scenarios, 110 steps. topics-bdd 'not @ui': 15 scenarios, 78 steps.
- cargo fmt --check: exit 0. clippy --all-targets -D warnings: exit 0.
- Original probes: 35/35 and 18/18 passed, with only the M8 label added to corrected().
- New probes: 36 in the first run, 26 passed and 10 failed. The 10 are n09, n10, n20, n21, n22, n24, n25, n33, n35, n50. n33 failed on my own unwrap; it fails closed as described in N11. n60 was added later and passes (prints the error message). Several pass as informational only, with no assertion: n26, n33 after its fix, n38, n42, n51, n52, n53, n54, n60.

Not run: desktop or runtime tests, `scripts/check.sh`, `--locked`, the real adapter, or any model. The runtime path in N1 is from reading main only. The four desktop/runtime scenarios stay with their owners.

## Residual risks (documented limits, preserved)

No cross-file atomicity, power-loss guarantee, uncooperative or same-user writer protection, cross-app-data coordination, or arbitrary modified-copy recognition. Rename plus edit of a local-only file becomes Unknown, by design. A hand-forged marker is outside the threat model. The publisher runs under the store lock, so a slow permit blocks the service. Manual proposals skip duplicate normalisation (n52), which fits the human-explicit design.

## Files

In this folder: keep2/orig_probes.rs, keep2/orig_reanalysis.rs, keep2/new_probes.current.rs (n33 and n60 variants are described above), rereview-new.out, rereview-baseline.out.
