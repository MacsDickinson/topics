# BoardUI screen redesign verification

Implementation checkpoint: `c6965c044dd580bd39e43da712e8740d5397dffd`, based on
local main `a38b33471125225d57c8e1a257f092aed63a1f40`. Work was confined to
`/Users/macs/orca/workspaces/topics/topics-boardui-screen-redesign`.

The screen plan was written before implementation in
[`docs/design/redesign.md`](../../../../docs/design/redesign.md). The change is
a React structure and presentation redesign, with shared components adopted
across screens. The established semantic palette, local system font and native
controls remain. No remote font or production dependency was added.

## Commits and integration

| Commit | Purpose |
| --- | --- |
| `454f55a9b55f4b57b7e196c9f7e0de53e9a17f43` | Screen-level before/after plan, shared React primitives and reading/control type roles; shared early with the setup team. |
| `49dc58f` | Synthetic baseline evidence and isolated capture configuration. |
| `720f846c5d6978365aba05f39704468c6a5d6ee7` | Authorized cherry-pick of setup presentation checkpoint `cad67576a33179cc1101b070da9b5b54dff10218`. |
| `c6965c044dd580bd39e43da712e8740d5397dffd` | Full structural screen implementation, documentation and interaction/contrast coverage. |

The imported setup checkpoint includes its existing canonical/native adapters;
this worker did not independently change native or generated binding contracts.
The coordinator requested a final canonical setup delta before full acceptance.
That delta is still pending at this checkpoint; this report does **not** claim
verification of later setup/API changes or final native acceptance.

## Screen coverage

Every `after` screen below has a full-page synthetic screenshot in desktop
light, desktop dark and narrow light. Prefix filenames with `light-`, `dark-`
or `narrow-`. Desktop captures are 1440 × 960 CSS pixels; narrow is 600 × 960.
The 320px long-content cases are functional tests, not additional screenshot
claims. Full-page capture scroll is reset before capture so sticky navigation
appears at the document top.

| Surface | Substantive redesign | Screenshot basename in `after/` |
| --- | --- | --- |
| Shell | Grouped icon navigation, actual Cmd/Ctrl+K quick search, selected shapes, creation actions, full folder/runtime footer, skip link. | Present throughout |
| Topics | One workspace heading/action, identity/type/privacy rows, dominant accepted claim values, adjacent uncertainty, disclosed evidence, connections/unresolved sections and revisions. | `topics.png` |
| Sources | Basename/full-path library rows, selection, separate source identity/privacy/location controls, whitespace-preserving reading surface. | `sources.png` |
| Search | Persistent labelled/adorned field, title/path/excerpt rows, safe highlights and distinct loading/idle/empty/error feedback. | `search.png` |
| Inbox | Proposal heading/status and decision footer; candidate type/inclusion, related editable fields, full evidence and explicit approval/rejection. | `inbox.png` |
| History | Operation icon, description, time/status, identity disclosure and contextual Undo with explanation when unavailable. | `history.png` |
| Initial connection | Focused form; native chooser fills the labelled field, explicit Use notes folder connects; unrelated Search no longer appears beneath setup. | `setup.png` |
| Manual capture | Focused identity/privacy form and explicit Create proposal; review remains required. | `manual.png` |
| Analysis | Actual select → preview/authorize → Inbox sequence, grouped scope fields, included/excluded rows, consent and start boundary. | `analysis.png` |
| Folder inventory | Shared page/section/field/button primitives, hierarchical notes/roles, date filters and exclusions. | `folder-inventory.png` |
| Exact manifest | Named scope, exclusions, unchecked per-job authorization and disabled start until consent. | `folder-manifest.png` |
| Starting map | Coverage, keyboard-accessible graph/list, topic detail group, relationships and input-needed sections. | `starting-map.png` |
| Save receipt | Enumerated exact candidate/dependency/deferred context followed by explicit Save or Cancel. | `save-map.png` |
| Contextual review | Compact review rows, status, source disclosure, focused correction form, original/replacement values, user statement and future-rule scope before approval. | `context-correction.png` |

There are 21 immutable `before/` screenshots of the original seven requested
surfaces, and 39 `after/` screenshots (13 surfaces × 3 appearances). New setup
and contextual-review screens did not exist in the baseline and have no invented
before screenshots. The `before-*-contact.png` and `after-*-contact.png` sheets
provide comparison indexes. The original full-size images remain authoritative.

All baseline screens and final screenshots were visually inspected, including
the populated reader, source policy, search results, review/consent boundaries,
operation rows, narrow stacking and the new setup/contextual states. Inspection
caught a full-page skip-link artifact; unfocused skip links now use clipping,
and screenshots reset scroll for consistent sticky-sidebar placement.

## Checks

Pinned local runtime: `apps/desktop/node_modules/node/bin/node`, **v24.22.2**.
The initial dependency bootstrap used system npm; all verification used the
project-local Node binary. Browser port **14337** is strict and isolated.
Every test server invocation received the absolute isolated
`TOPICS_APP_DATA_DIR=/tmp/topics-boardui-redesign-browser-0cd733`. Browser IPC
is mocked with synthetic notes; no personal note tree, repository `notes`
symlink, genuine model call or real user application was used.

From `apps/desktop`:

```sh
TOPICS_APP_DATA_DIR=/tmp/topics-boardui-redesign-browser-0cd733 \
  node_modules/node/bin/node node_modules/@playwright/test/cli.js test \
  --config evidence/redesign/playwright.config.ts
node_modules/node/bin/node node_modules/typescript/bin/tsc --noEmit
node_modules/node/bin/node node_modules/vite/bin/vite.js build
```

- **81 browser tests passed**, including all existing feature/browser suites,
  retained rendered contrast checks, the new interaction cases and captures.
  See [`full-browser.txt`](full-browser.txt).
- **TypeScript and production Vite build passed**; see [`build.txt`](build.txt).
- New meaningful checks exercise Cmd/Ctrl+K from a topic, query selection,
  disconnected setup guards, native disclosure Enter/Space behavior, delayed
  topic selection, manual/settings cancel focus restoration, post-proposal
  focus, exact manifest consent, graph keyboard activation, correction preview
  without implicit approval, 320px long paths/content and browser text selection.
- Existing async, stale-root/response, hostile markup, approval/dependency,
  source sensitivity and draft/consent-preservation behavior remains covered.
- Computed light/dark field boundary ratios: **3.89 / 3.48**; placeholders:
  **5.48 / 6.89**; primary hover text: **7.67 / 4.87**. Added selected navigation,
  privacy, uncertainty and icon checks meet their 4.5:1 text / 3:1 icon thresholds.
  The contrast helper now composites all ancestor backgrounds, including alpha
  during transitions, instead of incorrectly blending a dark surface onto white.
- A full run caught remounted contextual-trigger focus loss. Stable trigger ids
  now restore the actual trigger; the test also verifies folder-settings cancel
  and resulting Inbox focus after manual proposal/approval.

Some existing test locators were scoped to the named sidebar because New topic
also exists in the Topics page header. The source empty-state assertion now uses
its semantic heading. No approval or async safety assertion was removed.

## Limits and remaining work

- Final canonical setup delta and coordinator integration remain outstanding at
  this checkpoint. Preserve their request/receipt/consent authority when merging.
- Native Tauri/WebView smoke, packaged build and genuine runtime acceptance are
  owned by the setup/integration team and were not run here. No user app restart,
  push, PR or main merge occurred.
- Browser width tests include 320, 480/600, 1280 and 1440px cases, with a 600px
  200%-zoom-equivalent layout check and reduced-motion media. They do not replace
  actual WebView zoom, VoiceOver or exhaustive viewport/state testing.
- Setup job pause/resume/retry and every backend error have preserved controls,
  but the screenshot set covers the representative synthetic finished-job path;
  it is not exhaustive native state coverage.
