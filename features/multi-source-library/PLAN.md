# One library, many sources

Proposed for discussion and BDD sign-off. The existing [agreed plan](../../docs/plan.md) describes the single-root prototype. This proposal changes the boundary of a workspace. It builds on the [storage assessment](../../docs/persistence-assessment.md) and the [folder setup and review flow](../folder-setup/PLAN.md).

## What the library is for

On Monday, open Ledger and see that Friday's rollout moved to Tuesday, why it moved, who discussed it, and which follow-up still belongs to you. The decision came from a project record. A meeting transcript still says Friday. Your agent should be able to ask for the same briefing and follow the permitted citations. You should be able to discover the change by opening Today, even if you never thought to search for “rollout.”

A topic is a persistent identity with a briefing assembled from its linked records. Facts say what is currently understood. Lore preserves attributed experience, such as “the team usually waits for the support handover.” Decisions record what was chosen, when, why if known, and what they replace. Resources lead back to the original material. People, meetings and subtopics have identities of their own. A concern such as accessibility can link several projects without forcing them into the same hierarchy. Each substantive relationship has evidence. Attending a meeting is evidence of attendance, not agreement with a decision.

## Where a note goes

Give the user a **library** they own, with a visible Markdown home they can choose and back up. A note written in Topics lands there. An imported meeting stays in its source. A topic can cite both. Capture should work before the user knows what topic a thought belongs to. Daily Log can show that capture immediately and continue pointing to the same record after it is linked to a topic. An action keeps its ID across days. Weekly Review can surface changed decisions, unresolved contradictions and actions to carry forward. These are views over the same records, so weekly preparation does not require rewriting a backlog.

For a fresh library, keep authored Markdown and the existing app-owned `topics/` records together, with source registrations and versioned evidence sufficient to rebuild the derived index. Keep snapshots before writes and journalled undo. External sources are read-only. For an existing installation, offer the current root as the initial library home and preserve its record IDs, correction rules, history and citations in place. Existing source paths must resolve as before. Connecting a second source must never relocate those files or scan app-owned records as fresh evidence.

## First milestone

**Deliver one loop end to end.** Connect two local Markdown folders, see what the second contributes to an existing topic, save the change as a group, and retrieve the cited briefing from the desktop and an explicitly authorized external MCP client. This gives us an acceptance test for each subsequent connector.

1. Register a folder with a stable source ID and a human-readable label. Identify each document within its source. Identical relative paths in two folders are valid. Detect a repeated registration of the same canonical folder. Reject overlapping registrations in this slice with an explanation. Use source-version and passage identifiers for evidence.
2. On connect or refresh, inventory locally and show new, changed, missing and skipped inputs. Detect unchanged bytes and copied evidence. Apply the existing size and privacy limits. Obtain consent for the exact data before any model analysis. New versions require fresh consent. Persist a resumable proposed update per affected topic, with the previous understanding, new evidence and remaining uncertainty. Never merge people or topics solely on a shared name.
3. Surface a **Changes** list with a short explanation such as “Ledger rollout date changed” or “Two rollout dates conflict.” The user can follow a citation, save a coherent topic update, correct it, or defer the disagreement. One decision covers the displayed group. Accepting a sourced account does not personally verify every line. A correction survives future appearances of the same bad transcript.
4. Give an external MCP client a bounded read grant for a library and permitted topics/sources. Use the existing application-owned MCP surface and the same briefing projection as the desktop, filtered to the grant. Return current understanding, changes since a supplied cursor, uncertainty and versioned citations. Keep a stable cursor across restart. Hide private evidence and anything derived solely from it, including titles, counts, change hints and links. Reading a briefing confers no write or import permission.

**Recovery and removal.** A missing folder retains its identity and displays its last successful refresh. Removing a source stops future reads and invalidates its grants. Preserve accepted history and the provenance of past decisions locally, label evidence availability accurately and suppress removed-source material from agent retrieval. A later explicit purge of retained evidence is a separate user decision. Rebuilds must preserve privacy decisions and revocations.

## Next slices

Use the same ingest lifecycle for each connector. A ZIP is a point-in-time import with archive and member identities. A website starts with an explicitly selected page and a captured version. A Git source pins repository, commit and path and can preview changes between revisions. Each adapter declares what it can fetch, how refresh works and what it retains. Imported content is data. Adding a web or Git resource should never grant an agent general network access or permission to execute repository code. Each connector gets its own sign-off.

Native quick capture, stable actions, Daily Log and Weekly Review follow the first loop. The library proposal reserves a home for authored material from day one. Those later workflows deserve separate interaction scenarios. The [large-note proposal](../large-notes/PLAN.md) remains relevant across all connectors.

## Acceptance and build

Review the [proposed scenarios](library.feature) before implementation. Use temporary synthetic sources, isolated app data and a scripted provider. Exercise the normal desktop import and external MCP entry points. Include a rebuild from authoritative records and a legacy single-root fixture. After sign-off, sequence the source identity, evidence and grant contracts first, then parallelize the folder adapter, briefing/MCP and desktop changes where contracts allow. Use the project's design system, Orca workers and independent Claude review.
