# Topics design system

Status: structurally redesigned on 2026-11-09. The implementation uses [`tokens.css`](../apps/desktop/src/tokens.css), [`ui.tsx`](../apps/desktop/src/ui.tsx), [`ui.css`](../apps/desktop/src/ui.css) and the consolidated [`style.css`](../apps/desktop/src/style.css). See the [redesign plan and integration contract](design/redesign.md), [reference review](design/reference-review.md) and [screen specifications](design/screens.md). Browser evidence does not establish native or screen-reader acceptance.

## Principles

Topics is a local-first reading and review workspace. Give notes and evidence room to breathe; make provenance, uncertainty, privacy and the human decision visible. Use calm neutral layers, thin boundaries, restrained elevation, compact navigation and blue for actions or selection. Preserve native/offline operation, system fonts and existing behavior; presentation must not imply cloud consent, acceptance or capabilities that do not exist.

## Implemented foundations

`tokens.css` maps semantic roles directly to colors for light and, under `prefers-color-scheme: dark`, dark system appearance. There is no user theme override. The legacy aliases `--line`, `--muted` and `--blue` point to semantic roles. Prefer semantic roles in new work; the full values and both mappings live in the CSS.

| Family | Implemented roles (prefix shown once) |
| --- | --- |
| `--color-text-` | `primary`, `secondary`, `muted`, `placeholder`, `accent`, `on-action`, `info`, `warning`, `success`, `danger` |
| `--color-background-` | `app`, `sidebar`, `panel`, `subtle`, `hover`, `selected`, `action`, `action-hover`, `info`, `warning`, `success`, `danger`, `highlight` |
| `--color-border-` | `subtle`, `default`, `strong`, `focus`, `info`, `warning`, `danger` |

Spacing is `--space-1` through `--space-6` (4, 8, 12, 16, 24, 32px). Radii are `--radius-control` 8px, `--radius-card` 10px, `--radius-panel` 14px and `--radius-pill` 999px. `--motion-fast` is 120ms ease; `--elevation-panel` supplies a theme-specific subtle shadow. Reduced-motion CSS removes transitions and animation. Existing CSS still contains literal spacing and dimensions; these tokens are available, not a claim of complete migration.

The local system font stack requires no network. Composite `--type-page`, `--type-section`, `--type-body` and `--type-meta` define 26px/1.25/650, 17px/1.4/600, 14px/1.55/400 and 12px/1.5/400 respectively. `--type-reading` adds 16px/1.7/400 for original notes; `--type-control` adds 13px/1.4/500 for controls and labels. Accepted claim values enlarge the reading role to 20px/1.5/500 to make the understanding dominant. These roles share the local stack in both themes; color remains independent.

## Implemented React primitives and patterns

Shared components in `ui.tsx` are used across the shell and screens. They contain no IPC, consent or business state:

| Component | Contract and use |
| --- | --- |
| `Icon` | Small local SVG set, always decorative and hidden from assistive technology. Pair it with visible text or a named containing control. |
| `Button` | Native button props/ref, `primary`, `secondary`, `quiet`, `danger`, optional icon and `busy`. Busy exposes `aria-busy` and disables the native control; screens retain request guards. Supply `type="button"` inside forms unless submitting. |
| `PageHeader` | One page heading, description and optional action slot. `level={2}` supports proposal headers without introducing extra page headings. |
| `Badge`, `PrivacyBadge` | Non-interactive named status/privacy chips; neutral/info/warning/success/danger semantic tones. Privacy never implies analysis consent. |
| `EmptyState` | Heading, small identity icon and useful next step. Loading and failures remain separate states. |
| `Section` | Named semantic section with heading, description and optional actions. Keep related reading or decision content together. |
| `Field` | A stable id, visible associated label and one native control. Optional hint/error are linked through `aria-describedby`; errors set `aria-invalid`. It does not reset drafts or perform validation requests. |

```tsx
<PageHeader title="Sources" description="Your local Markdown library." />
<Field id="source-sensitivity" label="Source sensitivity"
  hint="Local-only notes are always excluded from analysis.">
  <select value={sensitivity} onChange={onChange}>
    <option value="local_only">Local-only</option>
  </select>
</Field>
<PrivacyBadge sensitivity="local_only" />
<Button variant="primary" disabled={!confirmed} onClick={onSave}>Save starting map</Button>
```

Global native controls provide default, hover, pressed, focus-visible and disabled states. Fields use the contrast-checked strong boundary and full-opacity placeholder role; primary controls use on-action text in both themes. Navigation uses `aria-current` and an inset selection bar; topic/source buttons use `aria-pressed`, a boundary and the same selection cue. The shell's actual Cmd/Ctrl+K handler opens Search and focuses/selects its input, while disconnected setup remains visible.

The stylesheet defines `.focused-form`, `.form-grid`, `.form-section`, `.review-footer`, `.source-row`, `.candidate`, `.run-panel` and `.source-evidence` for shared screen patterns. A review footer sits after scope, consent and exact before/after context, never implying automatic approval. Native `details/summary` disclosures retain complete evidence and revision identities. Opening a disclosure grants no permission. Source passages use semantic figures/captions; original note text stays whitespace-preserving, selectable React text.

## Component and state guidance

- **Navigation:** named controls, current page announced with `aria-current`; selected styling has a shape or other cue as well as color. Counts supplement labels and are not the only status signal.
- **Buttons:** primary for the next main action, secondary for alternatives, quiet for low emphasis, danger for destructive actions. Use actual buttons, descriptive labels and accessible names for icon-only controls. Provide default, hover, pressed, focus-visible, disabled and busy states. Disabled controls need nearby explanation when the reason is not obvious.
- **Fields:** persistent associated label, optional hint and connected inline error; show required and invalid states in text. Preserve entered values on errors. Use native controls where suitable.
- **Badges:** short text for status, privacy or uncertainty; informational badges are not buttons. Interactive filters must use button semantics and expose selected state.
- **Panels, lists and notices:** consistent padding, borders and headings; selected rows remain identifiable. Empty, loading, no-results, error and recovery states explain what happened and a useful next step. Use polite status announcements for asynchronous updates and an appropriate alert for urgent errors without repeatedly interrupting reading.
- **Evidence and review:** separate proposed from accepted content; show source path, excerpt and uncertainty near the assertion. Preserve original versus replacement text, selection scope, consent and explicit accept/edit/reject controls. Never visually imply that an unreviewed suggestion is accepted.

## Accessibility and layout

All controls must be reachable and operable by keyboard in a logical order. Provide a visible focus indicator distinct from selection, preserve focus after updates or restore it to a sensible trigger, and do not trap it except in a correctly managed modal. Associate labels, descriptions and errors; give icon-only actions names. Use semantic headings, lists and landmarks; announce meaningful progress without flooding live regions. Check text contrast at least 4.5:1 (3:1 for large text), and essential control boundaries, icons and focus indicators at least 3:1 against adjacent colors. Do not depend on color, hover or animation to convey state. Respect `prefers-reduced-motion` and test keyboard, zoom and screen reader behavior.

At comfortable desktop widths use a 232px floating sidebar and a flexible content area; Topics and Sources add a 252px list beside the reader. At 1180px these tighten to 208/210px; at 980px list/detail stack. At 760px the sidebar becomes a labelled top navigation strip and form columns stack; 480px further simplifies navigation grouping. These are implementation breakpoints, not universal device targets. Test long paths and unbroken content without horizontal page overflow. Reading areas use roughly 72ch, preserve note whitespace and offer complete evidence. Keep long reviews navigable without losing decision context. The skip link targets the main landmark, and cancelling manual capture or folder settings restores focus to the actual trigger.

## Change discipline

Every future UI feature must reuse design-system tokens, components and patterns. If a feature exposes a missing token, component, state or pattern, extend the design system and its documentation/examples in the **same change**, before or alongside the feature. Prefer extending an existing role over a one-off literal; if a genuinely new role is needed, describe its purpose and both theme mappings.

Worked gap example: a future review adds a “needs attention” state. First check existing warning and error roles and badges. If neither communicates the state, add a semantic warning text/background/border mapping and a badge or notice variant in both themes, document its use and a small example here, then use it in the feature. Verify contrast, text meaning, keyboard behavior if interactive, and reduced motion. The suggested new role is illustrative; inspect the existing warning roles and badge first.

Definition of done / review checklist:

- [ ] Existing tokens, components and screen patterns reused; any gap and example documented in this change, with proposed versus implemented status accurate.
- [ ] Loading, empty, error, selected, disabled, busy and relevant review/consent states considered; behavior and data boundaries unchanged unless explicitly designed.
- [ ] Keyboard, focus, labels, announcements, contrast, reduced motion, long text, narrow window and zoom checked in both supported themes.
- [ ] Relevant tests and manual checks recorded with limitations; screenshots use synthetic data and isolated app data. No personal notes or remote font dependency.

## Folder chooser

For connecting a notes root, offer a native folder chooser alongside the existing labelled path field. Selecting a folder fills the field; only the explicit “Use notes folder” action connects it. Cancelling leaves the entered path and current workspace intact. The chooser is a user-triggered native action and does not grant model access.

## Setup and review patterns

Folder setup uses a hierarchical list of native selection controls with a role selector and explanation per folder. The exact manifest and exclusions precede the single transmission authorization. The receipt lists its included candidates, dependencies and deferred counts before the save action; changing selection clears the preview. Progress names failed and outstanding inputs instead of implying complete coverage. Topic evidence uses a collapsed disclosure; status is written in text. Graph and list representations must provide equivalent keyboard access and distinguish explicit from inferred links without color alone. Contextual decisions enumerate their scope before confirmation; correction previews show the actual statement, before and after, and the future rule.
