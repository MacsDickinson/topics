# Screen specifications

These specifications describe existing flows and desired interaction behavior, based on source inspection on 2026-11-09. They are review targets, not proof that every state or accessibility behavior has been verified. Apply the [design system](../design-system.md); retain current data and consent semantics.

```text
Desktop shell                         Narrow window
┌────────────┬─────────────────────┐  ┌─────────┬──────────────┐
│ navigation │ page title + action │  │ labelled top navigation│
│ current    │ list  │ detail      │  ├────────────────────────┤
│ actions    │       │ evidence    │  │ title, list, reader    │
│ root/status│       │ actions     │  │ actions in reading order│
└────────────┴─────────────────────┘  └─────────┴──────────────┘
```

The shell groups Topics/Sources under Library and Inbox/History under Review, with a real quick search, Set up folder, Analyse notes and New topic actions plus full root/runtime context. Distinguish active navigation from focus. At constrained widths, preserve labels and stack inner panes before reading becomes cramped; permit document scrolling without clipping controls. Initial connection is the only content task until a root exists; Search is not rendered underneath it. See [redesign](redesign.md) for the React primitive and parallel-feature integration contract.

## Read and inspect

**Topics:** list names and supporting metadata on the left; selected topic detail on the right. Put current understanding first, with each claim's uncertainty and evidence adjacent, followed by connections and unresolved mentions. Clearly label source excerpts and paths. Ask or correct follows the reading context; a generated response is not an accepted change. Long excerpts and paths wrap and remain available in full.

**Sources:** list notes and show selected source path, sensitivity and full note in a reading pane. Distinguish loading, missing selection and failure. Preserve whitespace and a readable measure; sensitivity labels should explain their consequence without implying consent.

**Search:** keep label and query visible above results. Each result gives title, path and excerpt with highlights; distinguish loading, error and successful no matches. Highlight is supplemental to readable text. Do not render note content as trusted HTML.

## Select, review, decide

```text
Analyse notes → choose files/folders and optional dates
              → preview included/excluded scope + consent
              → start → progress / stop → Inbox
Inbox         → proposal → candidate + evidence + uncertainty
              → include/edit choices → accept or reject
Topic         → ask/correct → scope/consent → proposal → human decision
```

**Analyse:** show selection and preview before any cloud run; excluded and unknown/private sources remain understandable. Keep the consent control and start action next to the preview. Busy, stop and error feedback must preserve the selection and tell the user what happened.

**Inbox:** group candidates inside a proposal, with type, inclusion control, editable fields and provenance in reading order. Show old and replacement values distinctly in correction reviews, including any future rule being approved. Keep Accept and Reject unambiguous, reflect pending/busy/error states and never suggest that a proposal is already current. Large proposals must remain navigable with keyboard and zoom.

**History:** show operation description and time, then available undo action and any dependency or failure explanation. An undo outcome should be announced and reflected in the list; do not imply all operations are reversible.

**Setup and manual capture:** clearly identify the notes folder and local-first implications; show validation errors beside fields and retain input. Manual topics enter the existing proposal/review flow. Future setup, graph and task concepts are extension guidance only: use the same shell, semantic states and evidence/decision patterns, and define missing patterns in the same change; they are not implemented by this specification.

**Connecting a root:** the native Choose folder control fills the labelled path field, with manual entry retained as a fallback. Cancelling the native chooser changes neither field nor root; Use notes folder remains the explicit connection action.

**Folder setup and review:** show local inventory and suggested roles, then a tree selection and exact manifest consent. Keep failed, excluded and outstanding coverage visible. Draft topics are browsable before saving; a receipt shows the eligible set and dependencies before approval. Topic sections expose collapsed evidence and contextual decisions; corrections show their affected assertions and rule before approval. The graph needs a keyboard-operable list equivalent and text labels for relationship certainty.

**Changed scope and topic diffs:** when unsent inputs change, show the old versus updated manifest and exclusions before a renewed authorization. Keep completed work and its prior consent visible through coverage. For replacement or contradictory evidence, place the earlier statement and incoming cited statement together without overwriting either. Connection confirmation lists the selected named connection and sources; supplemental context is saved and displayed as the user's own text.

### Friday pilot: Today and weekly review

For one source, Today shows its status, path and last successful analysis together. For several sources, show counts by status, with unavailable or incomplete sources first. Each disclosure opens the paths and analysis times. Keep the meeting actions reachable without expanding the inventory.

Today opens for a configured workspace and refreshes the local briefing. Source status and last-success time remain visible beside the scope, with an explicit route to folder setup and exact batch authorization. A user can keep the displayed briefing at meeting time, reopen it, report a wrong item, omission or missed commitment, and classify evidence now or later. Weekly review records baseline or pilot effort, distinct material changes and later examples, and displays unknown and unavailable measures. Saved feedback is not an approval; a correction continues through the existing scoped preview, approval and history. Source and snapshot failures are shown as failures, never as a clear briefing.
