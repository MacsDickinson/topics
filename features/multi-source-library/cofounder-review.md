# Topics co-founder review: should we fund the next two weeks?

Reviewed SHA: d930aa6 (HEAD at review time).
Files read: `features/multi-source-library/PLAN.md`, `library.feature`, `capture.feature`, and the "BDD process" section of `docs/plan.md`. Nothing else was read, and no repository files were changed.

## Verdict: REVISE. Do not fund the plan as written. Fund a re-scoped two weeks.

I'd fund the two weeks only if they go to a **change-detection evaluation on real history** and a **capture precision test**. Building the multi-source registry should wait until those pass.

**The wrong first bet is milestone one.** It is "connect two local Markdown folders" and a source registry. That is plumbing, and it is a poor test of what the user values.
- PLAN.md says the real "different sources" value is Confluence, Jira and GitHub ("Sources beyond my notes"). Those are deferred.
- A second Markdown folder tests overlap rules, source IDs and removal semantics. It does not test whether a proactive briefing beats what the user already does: search their notes, or ask Claude or Codex "what changed on Ledger?".
- Nothing in the plan states or measures that baseline.

I'd keep Journey 1 as the headline bet but prove its value before building the library. Journey 2 (capture) is probably the higher-frequency loop, since it fires every meeting. The plan correctly gates it, but it has the weakest evidence bar.

**Evidence versus hypothesis.**
- Evidence: the plan's design shows care about privacy, provenance and idempotency. Examples are consent on exact bytes, local-only exclusion, and "one Ledger entry, zero repeat-only entries".
- Hypothesis, with no evidence yet: the user will open Today daily, the digest will be right and complete on real notes, and one decision per group is low-friction in practice.

## Ranked findings

### Blockers (resolve before sign-off or build)

**B1. Every acceptance bar passes by construction, and none measures omission.**
- The "How we will judge" row 1 and the "A busy import…" scenario use a hand-built fixture (12 repeats, 1 decision, 1 disagreement). The answer is known and phrased cleanly.
- The pass condition is "one entry per affected topic, correct direction, zero repeat-only entries". That measures precision only.
- Counterexample: the real decision says "let's slip it past the long weekend", or lives in a note titled "sync 11/14" that never names Ledger. Today shows "You're caught up" while the date moved. Every scenario still passes.
- The user then trusts the briefing less than search, and abandons it after the first miss they notice.
- Fix: add a recall and omission measure on non-synthetic phrasing (see Delta 1).

**B2. Acceptance thresholds don't meet the project's own BDD rule.**
- `docs/plan.md` BDD step 1 requires "a labelled evaluation with a threshold agreed before the build" for relevance and extraction quality.
- PLAN.md's last paragraph says to "evaluate actual extraction on an explicitly authorized synthetic set" but gives no size, threshold or held-out split.
- It also says "for a miss… revise and rerun until the agreed outcome is met". Without a held-out set, that rule tunes the system to its fixture.
- Capture's answer key is 4 cases. That cannot support a precision claim.

**B3. Consent cost likely contradicts "low friction" in repeated use.**
- PLAN.md: "New versions require fresh consent" and "Review changed sources" for each new batch. Scenario "Refresh is incremental": "the new bytes require a new preview and authorization".
- A live notes folder changes daily. That means a daily consent prompt, which is the exact friction the user wants removed.
- No scenario covers standing consent for a source with exclusions, or what happens to a pending consent when the user ignores it for a week.
- The plan also never says when refresh happens. If the user has to press refresh, it brings back the "remember to search" habit the plan is meant to remove. If it is automatic on open, then stale-source and consent states matter more.

**B4. "You're caught up" asserts completeness it cannot know.**
- Scenario "A busy import…": "Today says I am caught up". Scenario "Source loss…" shows availability only on the Sources screen.
- Counterexample: Engineering is unreachable, or has unconsented changes pending. Today still says "caught up" and the user walks into the meeting with a stale Friday date.
- Missing: an "as of <time>, N sources current, M pending or unavailable" line on Today, and a rule that "caught up" is shown only when the sources are fresh.

**B5. No comparison to the user's existing alternatives, and no repeated-use evidence.**
- All row-2 tests are single-session and use a fixture with one salient answer. "Identify Tuesday in one minute" is trivial there.
- Nothing tests day 5. Unread backlog, "For later" becoming a graveyard, and a New indicator on everything are the usual reasons these tools get abandoned.
- Nothing times the "ask Claude over the folder" baseline.

### High, but not a gate to starting the evaluation work

**H1. False action extraction and draft noise (capture).**
- Scenarios are clean first-person sentences. Real handwritten lines look like "Sam – cover?", "Priya to send deck", "told Sam I'd…" (already done), "chase Ledger dates".
- Failure modes with no scenario:
  - another person's commitment ("Priya to send") becomes my action;
  - a past-tense or completed item becomes a draft;
  - a draft appears on every save, so users learn to ignore the panel.
- The plan has no draft-rate or precision ceiling.
- Latency is also untested: save to draft while still in the meeting. The "unavailable provider" scenario covers failure but not slowness.

**H2. Enrichment can attach a confidently wrong or stale decision.**
- The capture scenario "Save once and accept…" attaches the Tuesday decision. If a newer decision exists in an unconnected source, the action carries misleading cited context.
- The citation shows its source, but not its freshness or the sources not covered.

**H3. Where the accepted action lives.**
- "Accepting creates one persistent action" inside Topics, carried by Weekly Review.
- The user likely already has a task place. A commitment locked in a new app is abandoned.
- Not a Journey 1 blocker, but it is a capture-sign-off question: export or hand-off, or an explicit bet that Topics becomes the task home.

**H4. The causal "why" is model-inferred.**
- PLAN.md: "Engineering recorded a support handover as the reason."
- The Gherkin checks that a rationale is displayed, not that it appears in the cited passage.
- Add a pass condition: any stated reason must be quoted from, or verbatim-supported by, a cited passage. Otherwise show only the change and the dates.

**H5. Plan promises with no scenario in `library.feature`.**
- Overlapping-registration rejection (PLAN item 1).
- A correction that survives a future appearance of the same bad transcript (item 3).
- A materially different update resurfacing a topic on Today.
- A resumable proposed update after a crash (item 2).
- The "Review changed sources" entry.
- Without scenarios, these are unverified claims, and some are the ones most likely to break trust.

### Later risks

- **First-import privacy.** Local-only exclusion honors prior marks. On first connect of existing notes, nothing is marked, so sensitive notes get consented in bulk. A pre-send "looks sensitive" prompt is worth considering. The bounded manifest preview partly mitigates this.
- **Connector sequencing.** Connector choice should follow the pilot's "which missing answer mattered most" result, as PLAN.md already says.
- **External MCP.** Correctly out of scope. Keep it separate until a separately launched client verifies it.
- **Cost and latency** of model analysis per refresh on large folders. The large-notes plan covers this separately.

## Minimum corrective edits (planning files only)

1. **Add a reality-based evaluation row** to "How we will judge the benefit", with thresholds fixed before building (Delta 1), and a held-out split.
2. **Reorder milestone one.** Run an evaluation harness and retrospective replay on a single root before the registry work. Take the second source from an exported snapshot (PLAN already says ZIP snapshots share the lifecycle). Defer the overlap, removal and ID machinery until the replay passes.
3. **Add a standing-consent scenario** (Delta 2), and state the refresh trigger.
4. **Add the freshness line** on Today and rule out false "caught up" (Delta 3).
5. **Expand the capture answer key** to at least 40 labelled lines with other-person, past-tense and fragment cases, plus a drafts-per-save ceiling (Delta 4).
6. **Add the missing-scenario list** from H5, or remove those claims from the plan.

### Gherkin and evaluation deltas

**Delta 1. Omission and recall on realistic notes (evaluation row).**
- Fixture: at least 30 notes in natural phrasing, written independently of the system under test. Include 8 true material changes, 6 of them with indirect wording or no topic name in the title. Hold out half and do not tune on it.
- Pass: at least 7 of the 8 true changes appear on Today as the correct topic with the correct direction (recall ≥ 85%). At least 90% of Today entries are true changes, so there is no more than 1 spurious entry per 10. Zero repeat-only entries.
- Fail: any missed change that the "caught up" state hides, or recall below 85% on the held-out half.

**Delta 2. Standing consent without daily friction.**
```gherkin
Scenario: A changing folder does not ask me every day
  Given I authorized Engineering with exclusions and a standing scope
  When five days of new notes within that scope are added
  Then Topics analyses them without a new consent prompt
  And Today shows one "N new notes analysed" line, not N questions
  When a note appears that is outside the scope or marked local-only
  Then it is not sent and is listed once under "Review changed sources"
```
Pass: 0 consent prompts over the 5-day replay for in-scope notes, 0 bytes of excluded notes in the provider log.
Fail: any prompt per file or per day.

**Delta 3. Honest "caught up".**
```gherkin
Scenario: Caught up never hides stale or unavailable sources
  Given Engineering is unavailable, or has changes awaiting consent
  When I open Today with no new updates
  Then Today does not say "You're caught up"
  And it says which sources are current, stale or pending, with the last refresh time
```
Pass: the state text differs correctly in all three source states.
Fail: "caught up" appears while any source is unavailable or pending.

**Delta 4. Capture precision (evaluation row).**
- Fixture: at least 40 labelled lines from realistic handwritten meeting notes. Include fragments ("Sam – cover?"), others' commitments ("Priya to send deck"), past tense ("told Sam…"), speculation and 12 true personal commitments.
- Pass: drafts for at least 11 of the 12 true commitments (recall ≥ 90%). At most 1 draft from the non-commitment lines (precision ≥ 90%). Drafts-per-save no more than 1.2 across ordinary notes. Zero invented owners or due dates. Any stated context must cite a passage.
- Fail: any other person's commitment drafted as mine, or precision below 90%.

## One falsifiable, low-cost pilot (about 3 days)

**Change-detection replay on real history, with the user as labeller.**
1. Choose a consented, bounded slice of the user's own past notes, such as the last 4 weeks of one project. The user decides this.
2. Before seeing any output, the user lists from memory the 8 to 10 changes that mattered (dates moved, decisions reversed) and writes the answer key.
3. Replay the notes in weekly batches through the current prototype, or a thin script using the same consent boundary. Record each Today digest.
4. Separately, spend 10 minutes per change asking Claude or Codex over the same folder, as the baseline.
5. Then use the prototype before meetings for 5 working days.

**Pass (all of them):**
- At least 7 of 10 key changes appear.
- At least 60% of entries are rated "I'd want to know this".
- No more than 3 entries per day.
- Digest-to-answer time is shorter than the baseline for at least half the changes.
- The user opens Today on at least 4 of the 5 days and learns or acts on at least one thing they would not otherwise have known by day 5.

**Fail on any one of those.** If it fails, stop treating the proactive digest as the headline and re-bet on capture or on search-style answers. Registry work stays unfunded in that case.

A 20-meeting capture precision run on Delta 4's thresholds can share the same week.

## What to preserve

- The persona-first framing and the user-observable benefits. They match the BDD process well.
- Consent on exact bytes before any provider call, local-only exclusion, and the "provider receives nothing from local-only notes" check.
- Group-level saving ("one decision per group") and independent New/seen state.
- Idempotency scenarios: refresh and re-save create no duplicates, and the same action ID is used.
- Never merging people or topics on a shared name. The two-Sams and two-Alexes scenarios are good, and "Maybe Sam could ask" is the right negative case.
- The clean separation of Today, capture follow-up, work connectors and external MCP, including the statement that milestone one ends at desktop retrieval.
- Read-only external sources, snapshots before writes, and provenance retained after source removal.

## Limitations

- I reviewed only the four specified planning files at d930aa6. I did not inspect code, the design system, other feature plans, the notes symlink, user notes or any external site.
- I ran nothing, so the pilot thresholds are proposals. They need the user's sign-off and are not measured results.
- I did not see the real notes or how often they change. B3's daily-prompt concern assumes an actively edited folder.
