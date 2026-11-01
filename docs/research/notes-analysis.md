# Notes analysis: structure, errors, tasks and a domain model for `topics`

Date: 2026-11-01. Source: read-only sampling of `/Users/macs/notes` (510 `.md` files) and the synced Claude skills that read and write it. No note content left this machine.

## Recommendation (read this first)

1. **Keep markdown as the human-facing truth. Add a local structured index (SQLite) behind it.** A plain repo is enough for reading. It is not enough for three things the user needs: stable task identity across weeks, provenance for each claim, and corrections that stick. Today those all fail in the same way: each job rewrites a page from scratch and loses history.
2. **Make the Task the first entity to move into the store.** The weekly actions page is a full rewrite every Monday. Items carry forward with no ID, no age, no owner field and no history. A persistent task record with a log of updates fixes the biggest pain at once.
3. **Make the Correction a first-class entity.** The same errors come back every week (wrong pronouns, misheard names). A correction recorded once must be applied to every future transcript import and every generated page.
4. **Keep the existing paths working.** Skills read `memory/people/*.md`, `## Active Topics`, `Log/YYYY.MM/YYYY-MM-DD.md` and `CLAUDE.md`. The app should *generate* these files from the store in the same shape, so the skills keep working while they are migrated one by one.
5. **Replace the big weekly wrap-up with a 5–10 minute daily review** of new claims and open tasks (bullet journal migration), plus a 20-minute weekly migration.

A knowledge graph product is not needed. The graph is small (about 35 people, about 10 projects, a few hundred tasks a year). Typed links in SQLite are enough. Full-text search (SQLite FTS5) covers search.

---

## 1. Inventory

### Folders and file types

| Area | Files | Role | Who writes it | Live or historic |
|---|---|---|---|---|
| `CLAUDE.md`, `AGENTS.md` (root) | 2 | Index of people, terms, projects, preferences | Hand + Claude | Live, but stale |
| `memory/glossary.md` | 1 | Acronyms, nicknames | Claude (setup) | Live, rarely updated |
| `memory/people/*.md` | 32 | Person profiles with `## Active Topics` | `121-prep` skill after approval | Live |
| `memory/projects/*.md` | 8 | Project stubs (6–10 lines each) | Claude (setup) | Mostly abandoned |
| `memory/context/company.md` | 1 | Org chart, teams, offices | Claude (setup) | Stale |
| `Meetings/YYYY.MM/*.md` | ~300 | Transcript summaries (talat) | talat AI summary | Historic |
| `Meetings/*.md` (root) | 14 | October meetings not yet filed | talat | Recent |
| `Meetings/archive/` | 9 | Older free-form notes | Hand | Historic |
| `Log/YYYY.MM/YYYY-MM-DD.md` | ~25 | Weekly actions pages (Mondays) | `monday-actions-prep`, `eng-leadership-meeting-notes`, `121-prep` | Current week live, rest historic |
| `Log/YYYY.MM/*-weekly-roundup.md` | ~25 | Friday roundups | `weekly-roundup` | Historic |
| `Log/**/*.html` | few | Newsletters | `weekly-engineering-newsletter` | Published output |
| `Interviews/<role>/` | ~45 | Interview guides and scorecards | Skills + hand | Historic |
| `Documentation/`, `projects/lhv-forge/`, `publish/` | ~50 | Essays, project docs, published sites | Mixed | Mixed |
| Root loose notes | ~8 | `Untitled.md`, `Supplier renewals.md`, `Phishy Comms.md` and similar | Hand | Unfiled inbox |

Other types: 14 HTML, a few images in `Attachments/`, Obsidian config in `.obsidian/`. The notes folder is **not a git repo**. There is no history except file timestamps.

### Naming conventions

- Meetings: `YYYY-MM-DD-HH-MM-<slug>.md`, for example `…-codie-121.md`. Older files use `YYYY-MM-DD Title.md` with spaces.
- Logs: Monday date for actions pages, `…-weekly-roundup.md` for roundups (the date is the Friday or the Monday of the week, which is not consistent).
- People: `firstname-lastname.md`, but some use one name only (`becky.md`, `edgar.md`, `jev.md`).

### Frontmatter, links and tags

- **Frontmatter: almost none.** Only one file (under `projects/lhv-forge/`) has YAML frontmatter. All metadata is inline bold labels, like `- **Role:** …` and `*Last updated: YYYY-MM-DD*`.
- **Wikilinks: almost unused.** 17 in total. Most are from a February experiment (one file per task). People and projects are not linked from meetings or logs. All joins are by name text.
- **Markdown links:** 78, nearly all inside `publish/` docs.
- **Tags:** a few `#product`, `#leadership`, and Slack channel names. No tag system.
- **Checkboxes:** used February to June. Gone since September. Current actions pages are plain bullets.

### Three generations of task logging

The Log folder shows the user has tried three systems this year:

1. **February ("MacsBot")**: daily pages, a `todo-inbox`, one file per task with `@due`, `size:` and `confidence:` fields, and wikilinks. Abandoned in weeks.
2. **March to June**: one weekly page with "Weekly Priorities", then a section per day (Focus, Notes/Log, Meetings, End of Day). Mentions a `TASKS.md` that no longer exists.
3. **September to now**: one Monday "Actions — week of …" page, grouped by theme (Priority, Monese, Engineering Management, Delivery…), with "Done since last week" and "From this week's Engineering Leadership meeting" sections.

Lesson for the app: heavy per-task files failed. Weekly theme pages stuck because they are cheap to read. The app must keep capture as cheap as one line.

### CLAUDE.md vs AGENTS.md

`AGENTS.md` is an older copy of `CLAUDE.md` (July vs September). The diff shows drift in three lines:

- `AGENTS.md` lacks the he/him note for one direct report. That is the exact fact the transcripts get wrong.
- `AGENTS.md` lacks one person row.
- `AGENTS.md` says "Codex onboarding — Anthropic Codex Enterprise procurement". This is a blind find-and-replace of "Claude" with "Codex". It produced a false fact.

Both files still end with `currentDate: 2026-04-29`. Both still list the CEO who left in April as CEO. Neither lists the new CEO who appears on the 26 October actions page.

## 2. Error taxonomy

Each pattern is described abstractly. Snippets are short.

### E1. Pronoun errors
The talat summary for a late-October 121 uses "she/her" for a direct report whose profile says "he/him (meeting transcripts often get this wrong — always use he/him)". The same summary mixes "their" and "her" in one paragraph. The fix lives only in the person file. Nothing applies it at import time.

### E2. Misheard and mangled names
In one summary: a colleague's name becomes "Lord", two team members become "Narilee, Inoj", a candidate's name becomes a phrase that makes no sense, and a city (Tallinn) becomes "Talon". The 26 October actions page has "contact Calvin/Kalmer (check name)". Older summaries list "Speaker 11, Speaker 2" as participants (61 files contain "Speaker N").

### E3. Ambiguous identity
Two people are both "Joe" (Joe Owen and Joe Lovell). The actions page says "promotion cases for Joe (to Senior)". The profile Joe is already an Engineering Manager, so the claim likely refers to the other Joe. Nothing in the notes resolves this. A shared first name needs an alias table and a "which one?" prompt.

### E4. Invented explanations
One summary reads a speaker's own leave as "likely a typo for a sabbatical", and gives the leave to the wrong person. It also says "24 weeks, close to full term". This is internally inconsistent. The model filled a gap with a guess and stated it as fact.

### E5. Unverified claims that carry forward
"Unverified." appears 67 times across six actions pages (September to October). The roundup uses "(check)". Some claims are carried week to week without resolution. For example, "Card Ordering Limits — iOS release planned 6 Oct … Confirm iOS shipped. Unverified." appears on the 12 October and 26 October pages. The tag marks doubt but has no owner, no due date and no way to close it.

### E6. Stale facts in live docs
- Person profiles: only 5 of 32 have a `Last updated` date. Four of those are from July and August. One direct report's Active Topics date from 19 August, though there have been many 121s since.
- A colleague who left in October still has a profile that describes her as active, with topics from July.
- `company.md` shows the April handover as future ("becoming Interim CEO on 1st April").
- `memory/projects/` holds 8 stubs of 6–10 lines. Several projects are done or replaced (for example "May offsite").

### E7. Contradictions between files
- CEO: `CLAUDE.md` lists the old CEO and the interim CEO. The actions page names a new CEO. `company.md` still uses future tense.
- Family facts: a profile lists children's names. A later summary says "second child". One of them is wrong, or the profile is out of date.
- CLAUDE.md vs AGENTS.md (see above).

### E8. Path and contract drift between skills
- `weekly-roundup` says to save to `~/notes/Log/yyyy-mm-dd-weekly-roundup.md` (flat). Files actually live in `Log/YYYY.MM/`.
- `monday-actions-prep` finds the latest roundup with `ls ~/notes/Log/*weekly-roundup*`. That glob does not match files in sub-folders. The 26 October page says it used "the 9 Oct weekly roundup (latest found)". No roundup exists after 9 October, so this may be correct, but the glob is still fragile.
- `weekly-roundup` reads only `~/notes/meetings/` (top level). Filed meetings in `Meetings/YYYY.MM/` are invisible to it. This works today only because October meetings are not filed yet.

These are not data errors. But they cause silent gaps, which look like missing facts.

### Root causes
1. Transcript summaries are treated as source of truth, but they are low-quality evidence.
2. Corrections live in prose ("always use he/him"), not as rules a machine applies.
3. Every generated page is a full rewrite. There is no diff, no claim history, and nothing to show what changed.
4. There are no stable IDs for people, so joins rely on fuzzy first names.

## 3. Task patterns today

### Where tasks live
1. **Weekly actions page** (`Log/YYYY.MM/YYYY-MM-DD.md`). The main backlog. About 100 bullets, grouped by theme. Owner is a prefix in prose ("Macs —", "Ryan —"). Dates are inline ("by 6 Nov"). Status is implied by section ("Done since last week") or tags ("Unverified", "Parked").
2. **Person `## Active Topics`**. `121-prep` treats this as "the canonical list of tracked items for that person — there is no separate tracker". Each topic is a paragraph mixing context, actions and feelings.
3. **Meeting summaries.** "Decisions made" and action sections. Never closed.
4. **Engineering Leadership actions.** Appended to the Monday page as one-liners by `eng-leadership-meeting-notes`.
5. **Jira and Slack.** Referenced by key (TPM-189, IT-9282). The prep job checks status with a "light" search.

### How carry-forward works
Every Monday at 9am, `monday-actions-prep` reads last week's page and the latest roundup. It treats each item not marked done as a carry-forward candidate. It runs a few Slack/Jira/Confluence searches, then **writes a fresh page unattended**. The same week, `121-prep` proposes edits to that page, and `eng-leadership-meeting-notes` appends to it.

Problems:
- An item has no identity. If the wording changes, history is lost. Duplicates appear (the 26 October page notes "Already tracked above: …" by hand).
- No age. You cannot see that an item has carried for four weeks. The April page had to say "This has carried over for 4+ weeks" in prose.
- Three writers edit one file, with no locking and no merge.
- The page asks the user to "Review the AI-generated action list; confirm done, remove invalid". That is the big correction job, moved into a task.

### What a persistent backlog needs

| Field | Why |
|---|---|
| `id` (stable) | Survives rewording and migration |
| `title` | One line, as now |
| `owner` (Person ref) | Many items belong to others ("Kool —", "Team leads —") |
| `topics` (many) | Theme sections become views, not file structure |
| `status` | open, done, cancelled, migrated, scheduled, waiting, parked |
| `kind` | task, follow-up (waiting on someone), verify (the "Unverified" case), decision |
| `due` / `scheduled_for` | "by 6 Nov", "after the decision" |
| `created_at`, `closed_at`, `times_migrated` | Shows age and drift |
| `source` | Meeting file, Slack link, Jira key, or "hand" |
| `external_ref` | Jira key, so status can sync |
| `updates` (log) | Dated comments: "Alex chasing today", "No response yet" |
| `confidence` | Verified or not, so "Unverified" becomes a field with an action |

## 4. Bullet journal mapping

### The method (source: Bullet Journal FAQ, https://bulletjournal.com/blogs/faq, fetched 2026-11-01)

- Signifiers: `•` task, `O` event, `–` note, `X` complete, `>` migrated (moved forward), `<` scheduled (moved to the Future Log), strike-through for irrelevant.
- Logs: Daily Log (log as things occur), Monthly Log (calendar plus task list), Future Log (months ahead), Collections (topic pages, found through an Index).
- Migration: review open items, mark done, strike out what no longer matters, move the rest forward. The FAQ calls it "taking time to ask what is important to keep". The cost of copying by hand is the point: it forces you to drop things.

### Mapping to current usage and app features

| BuJo | Today in ~/notes | In `topics` |
|---|---|---|
| Daily Log | Missing since June. Meetings act as an implicit daily log | Daily page: one-line capture with signifiers; meeting imports land here as proposals |
| Weekly (not core BuJo, common add-on) | Monday actions page | Generated weekly view; no longer the store |
| Monthly Log | None | Month view: calendar plus open tasks |
| Future Log | "Key dates" section | Tasks with `scheduled_for` in a future month |
| Collections | `memory/people`, `memory/projects` | Topic pages (person, project, thread) |
| Index | `CLAUDE.md` tables | Generated `CLAUDE.md` / topic list |
| `•` task | Bullet with "Macs —" | Task, status open |
| `O` event | Meeting file, key dates | Log entry of kind event, links to Source |
| `–` note | Prose in Active Topics | Claim on a topic |
| `X` | "Done since last week" | status done, `closed_at` |
| `>` | Silent carry-forward by the job | status migrated, `times_migrated += 1` |
| `<` | "Parked", "after the decision" | status scheduled, `scheduled_for` set |
| strike | "Cancelled", deleted bullets | status cancelled, kept in history |
| — (new) | "Unverified" | `?` signifier: a verify task with a claim attached |

Key design point: **the app must make migration a human decision, not a silent job.** Today the Monday job migrates everything automatically. That is why the backlog grows and the "review the AI list" task exists.

### Proposed daily ritual (under 10 minutes)

1. **Inbox of proposals (3–5 min).** The app shows what it learned since yesterday from new meetings and Slack: new claims, new tasks, possible corrections. Each card is one line, with the source. Keys: accept, edit, reject. Pronoun and name rules are already applied. Low-confidence items show first.
2. **Today (2–3 min).** Show open tasks due or scheduled today, plus items waiting on others. Mark `X`, `>` or strike.
3. **Capture (1 min).** Type anything else as rapid log lines (`- ` note, `. ` task, `o ` event).

### Proposed weekly ritual (about 20 minutes, Friday or Monday)

1. **Migration.** The app lists every open task older than 7 days, sorted by `times_migrated`. The user must choose for each: done, migrate, schedule, cancel. Items migrated 3+ times are flagged ("decide or drop").
2. **Verify list.** All open "verify" items with owner and age.
3. **Topic health.** Topics with no update in 30 days, or with contradicting claims.
4. **Generate outputs.** The roundup and the actions page are rendered from the store. They are no longer the place where corrections happen.

This replaces the big wrap-up. Corrections happen daily, in small pieces, at the point of import.

## 5. Proposed domain model

### Entities

- **Topic** — `id, kind (person|project|thread|team|org), title, slug, status, summary_md, aliases[]`. A Person is a Topic with extra fields.
- **Person** — `topic_id, full_name, known_as, aliases[], pronouns, role, team, location, reports_to, slack_id, active (bool), left_on`. Aliases include known mishearings.
- **Source** — `id, kind (meeting|slack|jira|confluence|hand|chat), path_or_url, captured_at, reliability (low|medium|high)`. Talat summaries default to low. Hand edits are high.
- **Claim** — `id, topic_id, text, kind (fact|status|opinion|personal), status (proposed|accepted|rejected|superseded), confidence, source_id, quote_span, valid_from, valid_to, superseded_by`. Every sentence on a topic page is a claim with provenance.
- **Task** — fields from section 3. Links: `owner → Person`, `topics → Topic[]`, `source → Source`, `claim → Claim` (for verify tasks).
- **TaskUpdate** — `task_id, at, author (user|job name), text, status_change`.
- **LogEntry** — `id, date, signifier, text, links[]`. The daily log. A log line can create a Task, a Claim or an event.
- **Correction** — `id, scope (global|person|source), pattern, replacement, kind (pronoun|name|fact|merge), reason, created_at, created_by`. Applied to every future import. Examples: "Narilee → Marily", "Talon → Tallinn", "Codie: he/him", "'Joe' in Banking Services context → ask".
- **Relationship** — `from_topic, to_topic, kind (reports_to|member_of|owns|works_on|depends_on), valid_from, valid_to, source_id`.

### Relationships (sketch)

```
Person ─is a─ Topic ──has many── Claim ──from── Source
   │             │                   └─superseded_by─ Claim
   │             └──tagged on── Task ──has many── TaskUpdate
   └─owner of──────────────────┘ └─from── Source
Correction ──applies to── Source imports, Claim text, generated pages
LogEntry ──creates── Task | Claim | event
Topic ──Relationship (typed, dated)── Topic
```

### What stays human-readable markdown, and what lives in the index

| Stays markdown (truth) | Lives in SQLite index (truth) | Generated markdown (view) |
|---|---|---|
| Meeting files (`Meetings/`) — immutable sources | Tasks and TaskUpdates | `Log/YYYY.MM/YYYY-MM-DD.md` actions page |
| Hand-written essays, interviews, `Documentation/` | Claims, with status and provenance | `memory/people/*.md` incl. `## Active Topics` |
| Daily log pages (new: `Log/YYYY.MM/YYYY-MM-DD-daily.md`) | Corrections | `CLAUDE.md` and `AGENTS.md` (one generator, no drift) |
| Topic narrative (`summary_md`), editable by hand | Aliases, relationships, FTS index | Weekly roundup draft |

Rule: **a generated file carries a header** like `<!-- generated by topics; edit in app or below the marker -->`. A hand-edited region below a marker is read back as a new hand Source. This keeps Obsidian editing possible without losing edits.

Why not markdown only: claim status, supersession, task identity and correction rules are all row-shaped data with history. Storing them in prose is the current failure. Why not a graph database: the scale is tiny and SQLite with typed link tables answers every query here. A graph view can be drawn from SQLite later.

## 6. Migration plan

Principle: **read everything, write nothing to existing files until the generated output matches today's shape.**

1. **Phase 0 — read-only index (week 1).** Parse `memory/people`, `memory/projects`, `glossary.md`, `company.md`, `CLAUDE.md`. Build Person and Topic rows, with aliases from the glossary nickname table. Import every meeting file as a Source (reliability low). Import every actions page as Sources. No writes to `~/notes`.
2. **Phase 1 — bootstrap tasks.** Parse the latest actions page (26 October) into Tasks. Use the section heading as topic, the "Name —" prefix as owner, "Unverified" as a verify task, and dated phrases as due dates. Walk back through older pages to fill `created_at` and `times_migrated` by fuzzy-matching titles. The user confirms matches in a one-off review.
3. **Phase 2 — bootstrap claims and corrections.** Split each `## Active Topics` bullet into claims linked to its 121 meeting source. Seed corrections from known facts: pronouns from profiles, the misheard names listed in section 2, the CEO change. Run them across all imported meetings and show the hit count. That measures the error rate.
4. **Phase 3 — shadow generation.** Generate `memory/people/*.md`, the actions page and `CLAUDE.md` into a staging folder inside the app's data directory. Diff against the real files. Fix the generator until the diffs are only real improvements.
5. **Phase 4 — take over writes, file by file.** Start with `CLAUDE.md` + `AGENTS.md` (one generator ends the drift). Then the actions page: the app writes it on Monday instead of `monday-actions-prep`. Skills that append (`eng-leadership-meeting-notes`, `121-prep`) keep working; the app watches the file (FSEvents) and imports appended lines as proposed tasks.
6. **Phase 5 — move skills to the app API.** Give skills a small CLI or local MCP server (`topics task add`, `topics claim propose`) so they stop editing markdown. Retire `monday-actions-prep` last.

Safety rules:
- `~/notes` is not under git. **Put it under git (or snapshot it) before the app writes anything.** This is a prerequisite, not an option.
- Never rewrite meeting files. Corrections apply on read and to generated views. The original transcript stays as evidence.
- Fix the path contracts in `weekly-roundup` and `monday-actions-prep` (flat vs `YYYY.MM` folders) before relying on them as inputs.

## Risks

- **Two-way sync with Obsidian edits.** If the user edits a generated file by hand, the app must detect and import it, or the edit is lost. The marker approach reduces this but does not remove it.
- **Fuzzy matching during bootstrap.** Task history from old pages will be imperfect. Accept a "history starts at import" fallback.
- **Over-structuring.** The February system failed because capture was heavy. Keep capture as one line with optional signifiers.
- **Personal and sensitive claims.** Profiles hold family and health details from transcripts. A `personal` claim kind should default to hidden and never go to a chat model without consent.
- **Corrections that over-apply.** A global "Talon → Tallinn" rule could break a real use of the word. Scope rules to sources and show matches before applying.

## Open questions for the user

1. Should generated files (`memory/people/*.md`, actions page) remain editable in Obsidian, or become read-only views?
2. Is it OK to put `~/notes` under git before the app writes to it?
3. Daily ritual time: morning (plan) or end of day (review)? It changes what the inbox shows first.
4. Should tasks owned by others (follow-ups) live in the same backlog, or in a separate "waiting on" list?
5. Which personal details (family, health, leave) should the app store at all?
6. Should Jira-backed tasks sync status both ways, or read-only?

## Could not verify

- Whether `monday-actions-prep` actually missed roundups because of the glob, or whether none were written after 9 October. Both fit the evidence.
- Where `TASKS.md` (mentioned in April) went.
- The exact count of pronoun errors across all meetings. I sampled; I did not run a full scan against every profile.
- The bulletjournal.com "learn" page returned 404 on 2026-11-01; signifiers are from the FAQ page instead.
