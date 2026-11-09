# BoardUI screen redesign

Plan recorded before implementation, 2026-11-09, based on local main `a38b334`.
The existing app has semantic colors but keeps a form-like hierarchy: competing
page titles, unstructured navigation, repeated evidence cards and wide inline
forms. This change rebuilds the reading and decision surfaces in React.

## Direction and reference

Independently inspected the [BoardUI homepage](https://www.boardui.com/) and its
[sidebar](https://www.boardui.com/components/sidebar),
[button](https://www.boardui.com/components/button),
[input](https://www.boardui.com/components/input),
[chip](https://www.boardui.com/components/chip),
[typography](https://www.boardui.com/components/typography) and
[color](https://www.boardui.com/components/color) examples in a real browser.
Adapt their quiet floating sidebar, compact icon-led rows, restrained blue
selection, layered neutral surfaces, labelled fields and text status chips.
Do not import their assets, remote fonts or dependencies. Reference screenshots
were viewed locally; blurred previews on some component pages limit assessment
of their interactive states, so the documented APIs also inform this adaptation.

Keep the established palette: panel `#ffffff`, sidebar `#f5f7fa`, text `#202c3a`,
secondary `#48596b`, action `#2768ae`, selected `#dceafb`; semantic dark mappings
remain centralized. Local system type: 26px page, 17px section, 14px body, 12px
metadata; add a 16px/1.7 reading role and a 13px medium control role. Left-align
content with a roughly 72ch reading measure. Spacing uses the existing 4–32px
scale. Selection and focus remain separate; icons supplement visible labels.

The distinctive element is the topic reading surface: accepted understanding
is visually dominant, with uncertainty and traceable evidence attached to each
claim. Avoid dashboard metrics and equal-weight decorative cards. Review the
plan against the brief: the shell follows BoardUI, while the reading-first
claim/provenance hierarchy belongs specifically to Topics and its trust model.

## Screen-level before and after

```text
Before                                After desktop
┌─────────┬──────────────────────┐    ┌──────────┐ ┌────────────────────────┐
│ text nav│ repeated page header │    │ Topics   │ │ page heading   actions │
│ actions │ list │ form / cards  │    │ Search ⌘K│ ├────────┬───────────────┤
│ clipped │      │ evidence      │    │ Library  │ │ topic  │ identity      │
│ path    │      │ long form     │    │ Review   │ │ rows   │ understanding │
└─────────┴──────────────────────┘    │ Create   │ │        │ provenance    │
                                      │ folder   │ │        │ relationships │
                                      │ runtime  │ │        │ ask / correct │
                                      └──────────┘ └────────┴───────────────┘
Narrow: navigation becomes a labelled top strip; page heading, list and reader
stack in order. Long paths and note text wrap; controls and full evidence stay
reachable. No icon-only navigation, nested page scroll trap or hidden privacy.
```

| Screen | Concrete change |
| --- | --- |
| Shell | Brand with consistent local SVG icons, real quick search, grouped library/review navigation, creation actions, full root and runtime context; skip link and focusable page landmark. |
| Topics | One workspace heading and action; topic rows with identity icon, name/type/privacy and selection cue; reader identity, dominant claim values, labelled uncertainty, provenance, connections, unresolved mentions and disclosed revisions. |
| Correction composer | Separate intent from reviewed context: message, preview, explicit consent and send remain in order; no auto-send, collapsed consent or implied acceptance. |
| Sources | Library heading; rows use basename, full relative path, date and privacy; reader separates document identity/privacy controls from a whitespace-preserving reading surface. Selection uses real buttons. |
| Search | Focused icon-adorned search field with persistent label and hint; status line then title/path/excerpt result rows; separate idle/loading/empty/error guidance. Cmd/Ctrl+K opens Search and focuses/selects its input from any screen. |
| History | Operation rows with history icon, description, time, named status, identity disclosure and contextual Undo; failures retain the existing recovery message. |
| Setup/manual | Focused panel with folder/topic identity, labelled fields, privacy explanation and explicit final action; native chooser remains a field-fill action only. |
| Analysis | Sequential select → preview/consent → start sections, with selection preserved; preview describes included and excluded notes beside the actual approval controls. |
| Inbox/review | Proposal identity/status and decision footer, candidate type/inclusion with editable fields, evidence and uncertainty; original/replacement and future-rule approval stay explicit. Presentation changes to ProposalEditor/Scope depend on ownership agreement. |

## Implementation and parallel integration

1. Add `src/ui.tsx`: `Icon` (decorative SVG only), `Button` (native button props,
   variants and busy state), `PageHeader`, `Badge`, `PrivacyBadge`, `EmptyState`,
   `Section`, `Field` (associated label/hint/error). Use each abstraction in more
   than one screen where useful; keep controls native and state with its owner.
2. Rebuild shell and screen React structure; replace conflicting CSS override
   layers with one organized stylesheet consuming semantic/type/spacing tokens.
3. Coordinate `ProposalEditor.tsx`, `Scope.tsx` and `main.tsx` render seams with
   the setup team. Their new setup/grouped-review logic and generated/native
   bindings stay theirs. No changed async, consent or data contracts.
4. Verify pinned Node 24.22.2 build, existing browser suite and meaningful new
   cross-screen shortcut, keyboard/focus, selection, wrapping and responsive
   cases. Retain rendered contrast assertions and test both themes.
5. Capture synthetic before/after desktop light/dark and representative narrow
   Topics, Sources, Search, Inbox, History, setup and analysis. Inspect screenshots
   and record precise coverage and limitations in the verification report.

### Integration contract for new setup and grouped review

Import primitives from `./ui`. `PageHeader` takes `title`, `description`, optional
`actions` and a heading `level`; `Field` takes a stable `id`, visible `label`,
optional `hint`/`error` and one native input/select/textarea child. `Section`
takes `title`, optional `description`/`actions` and children. `Button` accepts
native button props, `variant="primary|secondary|quiet|danger"`, optional `icon`
and `busy`; it does not perform requests. `Badge` is non-interactive and takes
`tone="neutral|info|warning|success|danger"`; `PrivacyBadge` takes `sensitivity`.

Use `.focused-form` for setup/capture, `.form-grid` for related fields,
`.form-section` for a real step, `.review-footer` for decisions, `.candidate`
for a candidate and `.source-row` for inclusion/exclusion context. Retain the
visible selection and explicit confirmation near the consequential action.
No primitive grants consent, resets selection or calls IPC. New feature screens
must be checked after integration; this branch cannot claim coverage of screens
that have not landed.

## Implementation checkpoint

Implemented the structural screen plan and integrated authorized setup checkpoint
`cad6757` in this isolated branch. All listed primitives are adopted by multiple
screens; `style.css` is organized into one control/shell/form/reading/search/review
layer rather than the previous accumulated override blocks. See the
[verification and screen coverage report](../../apps/desktop/evidence/redesign/README.md)
for exact commits, tests, before/after screenshots and pending canonical/native
integration limits. This checkpoint does not cover a later unmerged API delta.
