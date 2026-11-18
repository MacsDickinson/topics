# The Friday sync pilot

Approved for implementation on 2026-11-18, including the [pilot scenarios](pilot.feature).

**Decision to make.** Does a dated briefing reduce the work of reconstructing and correcting what happened before a meeting? Does it catch something you would otherwise have had to discover on Friday? Start with one project in the existing work-Mac workspace. The person attending the meeting is the evaluator. Development uses fictional inputs in temporary directories. User-led field observation stays on the work Mac, including notes, evidence, corrections and the completed scorecard.

## From preparation to feedback

1. Before the first Friday sync, use your usual preparation method. Record approximate prep and correction minutes, material changes you found, and commitments you had to create or repair. This is the baseline. A blank [scorecard](scorecard.md) can be used immediately, including before the in-app pilot exists.
2. Opening Today checks local source versions. A change can trigger local inventory immediately. Model analysis still requires authorization for the exact inputs. Group changed files into one request. On the briefing, show the time and scope of the last successful analysis and any pending, inaccessible, failed or excluded evidence. Never use a clear inbox to imply that all real-world changes were found.
3. At meeting time, keep a dated record of the briefing actually available then, its item IDs and analysed source versions. During the sync or afterwards, report from the kept meeting briefing. If you have refreshed Today or restarted Topics, open **Saved briefings → Open briefing (meeting)** and select that meeting’s timestamp first. Use **Correct** on an item or **Topics missed something** even when the briefing is empty. Feedback from the live Today view goes toward the next meeting you keep. Write one sentence, with optional evidence. Save now; classification can wait until the weekly review. The user can add a missed commitment the same way. Use ordinary note-taking during the meeting.
4. On Friday, compare that snapshot with the decisions and commitments that actually mattered. Link a correction to the original item or an omission to the expected change. Distinguish evidence already analysed before the meeting, evidence present but pending/unavailable, and information first learned in the meeting. Leave uncertain cases unclassified. A later source edit cannot rewrite the earlier snapshot. Tally automatically from edits and dismissals where possible, let the user correct the tally, and avoid creating feedback for simple reads or deferrals.
5. For a repeated error, offer one **scoped correction** for human approval, such as “For Ledger support handovers, Sam means Sam Patel.” Show the affected context, retain undo and preserve the original citation. On the next independent occurrence, check whether the same mistake returned. A team-wide policy or extraction change goes through normal review and a fictional regression example created from scratch. A feedback entry alone grants no authority to publish facts, send work data or change extraction rules.

## What to measure each Friday

| Signal | How to read it |
|---|---|
| Correction effort | Minutes spent correcting the briefing and actions, plus total prep minutes including authorization and review. Track counts of distinct wrong items and user-reported misses. Repeated edits to one item count once for accuracy; the effort still counts. |
| Known in-scope misses | Of the distinct material changes identified at the sync with evidence already analysed at briefing time, how many were absent? Show numerator and denominator. Report pending/unavailable, genuinely new and unknown separately. This measures discovered omissions; even a zero cannot establish exhaustive recall. |
| Action quality | When draft capture is enabled, count distinct drafts accepted unchanged, substantively edited, dismissed as wrong, and still undecided. Separately count genuine personal commitments the user had to add. Record wrong owner, deadline or context when known. Until that delivery exists, mark draft measures unavailable and record manually created actions as baseline only. |
| Learning | For each approved correction, count later eligible occurrences and recurrences of the same error. Show “no later example” when appropriate. Keep the effort of reviewing proposed fixes in the total. |

**Pilot decision rule.** Collect one baseline sync and two comparable pilot syncs. Record the number of sources, decisions and commitments alongside minutes. Continue to the two-folder milestone if both pilot syncs take less prep-plus-correction time than baseline, the briefing surfaces at least one useful change before the meeting across the pilot, and known in-scope omissions per eligible change do not increase between pilot weeks. Record honest source status in both weeks. A zero denominator, materially different workload, or no subsequent example of a correction leaves that comparison inconclusive. Extend one week for missing evidence. For a miss, review the largest contributor to effort, change one thing with approval, and evaluate on a fresh week. Persistent misses or inconclusive results warrant revisiting the bet before connector work. These thresholds guide the field trial. A completed weekly scorecard supplies the evidence.

## Work Mac and subsequent distribution

Use the Friday pilot build for this trial. Transfer the app archive by your work-approved channel. The app stores the briefing, feedback and weekly review in the selected folder, so create the work workspace on the work Mac. The Apple silicon prototype uses ad-hoc signing. Model analysis expects Codex 0.166.0 and the work-approved account. If work permits only Claude, you can still record baseline effort and commitments in Weekly review or the blank scorecard.

For a first rehearsal, extract the archive and run the following commands from the directory containing `Topics.app`. This opens a separate test instance with an empty app-data directory and gives you a fictional folder to select.

```sh
pilot_dir=$(mktemp -d)
mkdir "$pilot_dir/notes" "$pilot_dir/app-data"
echo 'The handover is scheduled for Friday.' > "$pilot_dir/notes/meeting.md"
echo "$pilot_dir/notes"
TOPICS_APP_DATA_DIR="$pilot_dir/app-data" ./Topics.app/Contents/MacOS/topics-desktop
```

Choose the printed folder and open **Today**. Select **Topics missed something**, enter a sentence and save. Open the dated briefing again to see your feedback. For the field trial, launch the app normally and choose your approved work folder. In **Today → Weekly review**, choose a date, use the same project name each week and record the first sync as **Baseline**. Before each pilot sync use **Keep briefing for meeting**. For feedback about that sync after a restart, reopen its saved meeting briefing. In Weekly review select that briefing under **Meeting briefing**. Record total preparation time including corrections, then enter the correction minutes separately.

The first Today briefing uses the latest authorised analysis. After you keep a meeting briefing, Today shows newly identified assertions from later analyses within the current selection. A repeated assertion with the same topic, predicate and value is suppressed, so a decision that returns to an earlier value may need to be reported as a miss.

A later distribution feature should cover an approved install channel, supported Mac architectures, signing and notarization, versioned updates, and recovery with the user's knowledge and history intact. No updater is needed to compare the first Friday sessions.
