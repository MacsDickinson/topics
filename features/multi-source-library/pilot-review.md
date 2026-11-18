# Independent review of the Friday pilot

Claude Sonnet reviewed `8e4490e..9ae0862` on 18 November 2026. The reviewer inspected the journal, briefing, view receipts, weekly measures, desktop journeys and tests. The reviewer ran the Rust pilot suites against fictional inputs. The independent learning journey was still red at this revision. Browser and native checks belong to the integration run.

| Finding | Why it matters during a Friday sync |
|---|---|
| Blank workload and minutes could yield a passing comparison | An incomplete scorecard could make the pilot appear to save time. |
| The omissions fraction relied on hand-entered material judgments | Reporting an analysed miss from Today could still leave the fraction at zero out of zero. |
| Saving the same displayed briefing twice created separate snapshots | Feedback from a single meeting could be spread across two weekly tallies. |
| Project lookup required an exact spelling | Capitalisation or whitespace could hide the baseline week. |
| The service accepted contradictory feedback and material judgments | A wrong-owner report or a commitment could be counted as an omitted decision. |
| A source could change during feedback entry | The person reporting a miss needs to keep the sentence and see which briefing it will describe. |
| Historical items and damaged checkpoints affected Today | Repeated assertions could look new and one unreadable checkpoint could hide usable evidence. |

The review also called for clear labels on source-status measurements, whole-number validation, protection against losing unsaved weekly edits, and a checked batch lookup. Its frozen revision predates the later-meeting occurrence and cross-note scope changes.

## Re-review and integration

Claude independently checked `3d940b8` in a detached worktree and passed all 24 focused Rust service tests plus 15 library tests. That review confirmed the one-in-three workflow, blank workload handling, case-insensitive project lookup, compatible feedback kinds, stale-view protection, scoped correction behaviour, duplicate suppression and partial-history reporting. The desktop runs are separate integration evidence. Cargo's standalone store package needs the `test-support` feature to compile older fault-injection test targets. The repository check runs those targets with workspace features.

Claude also found two ways the pilot could lose evidence. A feedback-only save advanced the next meeting's cutoff, and separate feedback captures could leave reports out of the weekly tally. The integrated fix gives meeting and feedback snapshots distinct purposes, retains the original evidence for each report, and associates the interval's reports with its kept meeting. An unchanged feedback view reuses its snapshot across refresh and restart. The [interval tests](../../crates/topics-store/tests/pilot_feedback_cutoffs.rs) verify that reports are editable, count once and stay out of the next meeting. A [multi-day briefing test](../../crates/topics-store/tests/pilot_briefing.rs) captures feedback between two analyses and checks that both decisions reach Friday.

Claude re-reviewed production revision `d04e37a` and ran the focused Rust suites. Both meeting-interval findings passed review. Reports entered on a fresh Today after a meeting go toward the next meeting. Reopening the dated meeting before entering a late report attaches it to the intended week. The work-Mac instructions now cover that distinction.
