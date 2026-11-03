# Focused confirmation: value-authority findings F1 and F2

Reviewer: Claude Opus 5.5, same context as the R1-R5 review. Read-only review of commits; all builds ran from `git archive` copies in this scratchpad, on synthetic temp roots and app data. No live notes, personal configuration, runtime, provider or native tests. The worktree stayed clean at 636fbf2.

**Reviewed commits:** source/docs 636fbf2, RED e0e7087, baseline c637101. Main's 8650d27 and 44a0154 have no diff from 636fbf2 in product source, `final_confirmation.rs` or the evidence note.

## Verdict: ACCEPT. F1 and F2 are closed

One adjacent policy observation (below) is for the manager to note. It predates this fix and matches the brief's changed-value rule, so I do not treat it as a blocker. F3 was not reopened.

## F1: closed
The literal test at `scopes.rs` is now `statement.text.trim() == claim.value.trim()`, with a non-empty value.

| Case | c637101/e0e7087 | 636fbf2 |
|---|---|---|
| p2b: value "Zed", statement "ask Zedekiah in legal" | exported (assert fails) | withheld |
| p2f: value "legal", chat statement "check with legal", metadata revision | exported (assert fails) | withheld |
| Committed `f1_` tests, same two shapes | both fail | both pass |

q1 ran a policy table through both a new-predicate claim and a metadata revision of the transcript claim:
- **Exports:** the exact value, and the value with surrounding whitespace.
- **Withheld:** "VALUE?", "VALUE.", "Confirmed: VALUE", "is VALUE right", and lower-case and upper-case variants.

## F2: closed
`human_value_supported` now walks the supersedes chain, following only revisions with the same logical ID, subject, predicate and trimmed value.

- **Positive, RED then GREEN:** p2a (chat follow-up) and p2e (manual follow-up) fail at e0e7087 and pass at 636fbf2. The committed `f2_` positive covers two follow-ups in each mode. "Zed", the path and the statement text stay absent.
- **Control:** p2g (UI value edit, then UI metadata edit) passes on both sides.
- **No manufactured authority:** p2d and the committed `f2_` negative (passage-only value change, unchanged transcript) stay withheld.
- **No crossing a different value (q3):** literal "Zed", then passage-only "Yak", then passage-only "Zed", then a metadata revision citing "check with legal", stays withheld at each step. Corrected "Yak" followed by a passage-only return to "Zed" is refused with `conflict`, and "Zed" stays withheld.
- **Per assertion (q4):** another claim with the same value but a different predicate gains no authority. Context omits ancestor revision IDs, the follow-up statement ID, statement text and the old path.
- **Undo (q5):** undoing a correction under a live follow-up returns `conflict`. After undoing the follow-up the value stays, and after undoing the correction neither value is exported.
- **Bound (q2), tested at the exact boundary:** with 99 follow-ups the correction is the 100th visited record and still exports. With 100 follow-ups the value is withheld with no panic, and "Zed" stays absent. This is the accepted fail-closed behaviour.

## Retained assertions
The rustfmt-normalized original 31-probe file is still an exact prefix of `final_confirmation.rs`: 0 removed lines, with additions only from line 1552. The only amended test is the implementer-owned r2 positive, which now uses the exact-equality fixture and is not one of the 31. It passes on both sides, as the evidence note says. `human_value_supported` has one call site, in the context path.

## Checks run
| Check | Result |
|---|---|
| `final_confirmation` at 636fbf2 | 39 passed, 0 failed, exit 0 |
| `final_confirmation` at e0e7087 | 36 passed, 3 failed: both `f1_` and the `f2_` positive, matching the evidence note |
| Four-crate suite with `topics-store/test-support` at 636fbf2 | 203 passed, 0 failed, 0 ignored |
| `workspace-bdd` | 20 scenarios, 110 steps passed |
| `cargo fmt --check` on the four crates | exit 0 |
| Focused scratch probes at 636fbf2 | 13 passed, 0 failed (p2a-p2g, q1-q6) |
| Same probes without q2 at e0e7087 | 5 passed, 7 failed: p2a, p2b, p2e, p2f, q1, q4, q5 |

**Discarded run:** my first 636fbf2 probe run shared a Cargo target directory with the older archives. It reused a stale `topics-store` build and reproduced the RED failures. I discarded it, and all 636fbf2 results above come from a separate clean target directory.

**Not run:** clippy, Phase 0 BDD, and main integration, native and release checks (manager-owned).

## Observation for the manager (not a blocker)
Any byte-level value change that a human approves while citing a raw statement counts as changed-value authority, even when the statement is only a question. q6 recorded two cases, both with the statement "check with legal":
- **Case-only change:** "acquisition target is Zed" revised to "Acquisition Target Is Zed" exports the revised value.
- **Return to the transcript value:** "Zed", then passage-only "Yak", then back to "Zed" exports "Zed".

In both, a human approved a visible value change, and the brief explicitly allows actual changed-value supersession. This is outside the F1 literal test and F2 lineage, and q6 exports the same at e0e7087. Comparing values case-insensitively would cover only the first case. Tightening this further would mean tying authority to the statement's text, which is a policy choice for the manager.

## Files
- Probes (appended to `final_confirmation.rs` to run): /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/body2.rs
- Probe output at 636fbf2: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/f_probes.out
- Probe output at e0e7087: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/f_red_probes.out
- `final_confirmation` output: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/f_focused.out and /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/f_red_repo.out
- Full suite output: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/f_full.out
- Earlier R1-R5 report: /private/tmp/claude-501/-Users-macs-orca-workspaces-topics-topics-m1-core/1cdba55d-5764-498a-93ce-3f5e90200397/scratchpad/final-confirmation-r1-r5.md
