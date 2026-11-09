# Screen specifications

These specifications describe existing flows and desired interaction behavior, based on source inspection on 2026-11-09. They are review targets, not proof that every state or accessibility behavior has been verified. Apply the [design system](../design-system.md); retain current data and consent semantics.

```text
Desktop shell                         Narrow window
┌────────────┬─────────────────────┐  ┌─────────┬──────────────┐
│ navigation │ page title + status │  │ nav     │ page title   │
│ current    │ list  │ detail      │  │         │ list         │
│ actions    │       │ evidence    │  │         │ detail       │
│ root/status│       │ actions     │  │         │ actions      │
└────────────┴─────────────────────┘  └─────────┴──────────────┘
```

The shell keeps Topics, Search, Inbox, History and Sources accessible, with Analyse notes and New topic actions and root/runtime context. Distinguish active navigation from focus. At constrained widths, preserve labels and stack inner panes before reading becomes cramped; permit scrolling without clipping controls.

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
