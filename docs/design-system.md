# Topics design system

Status: reconciled with the desktop implementation on 2026-11-09. The CSS in [`tokens.css`](../apps/desktop/src/tokens.css) and [`style.css`](../apps/desktop/src/style.css) is the source of truth for implemented values. The guidance below also identifies desired behavior that still needs verification or implementation. See the [reference review](design/reference-review.md) and [screen specifications](design/screens.md).

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

The local system font stack requires no network. Composite `--type-page`, `--type-section`, `--type-body` and `--type-meta` define 26px/1.25/650, 17px/1.4/600, 14px/1.55/400 and 12px/1.5/400 respectively. Headings use the first two; not all text uses a composite token. A separate reading or control type role is not implemented.

## Implemented CSS primitives

These are CSS classes and native elements, not a React component library. Global buttons, inputs, textareas and selects share control styling and focus-visible outlines; `.primary` and `.quiet` modify buttons. `.badge` has `.pending`, `.accepted` and `.conflict` variants; `.error`, `.notice`, `.empty`, `.candidate`, `.run-panel` and `.source-evidence` provide shared surfaces. Navigation uses `aria-current`, while topic and source selection use `aria-pressed`. Hover, disabled and focus-visible styles exist; distinct pressed and busy button variants are not established. For example, `<span class="badge pending">Pending</span>` is a non-interactive status label. Keep meanings in text as well as color. Native input and textarea placeholders use `--color-text-placeholder` at full opacity in both themes; for example, `<input placeholder="Meetings/one.md">` keeps its associated label and uses the shared field border and placeholder roles. `--color-border-strong` defines field boundaries against panel and subtle surfaces, and primary hover uses `--color-background-action-hover` with `--color-text-on-action`.

## Component and state guidance

- **Navigation:** named controls, current page announced with `aria-current`; selected styling has a shape or other cue as well as color. Counts supplement labels and are not the only status signal.
- **Buttons:** primary for the next main action, secondary for alternatives, quiet for low emphasis, danger for destructive actions. Use actual buttons, descriptive labels and accessible names for icon-only controls. Provide default, hover, pressed, focus-visible, disabled and busy states. Disabled controls need nearby explanation when the reason is not obvious.
- **Fields:** persistent associated label, optional hint and connected inline error; show required and invalid states in text. Preserve entered values on errors. Use native controls where suitable.
- **Badges:** short text for status, privacy or uncertainty; informational badges are not buttons. Interactive filters must use button semantics and expose selected state.
- **Panels, lists and notices:** consistent padding, borders and headings; selected rows remain identifiable. Empty, loading, no-results, error and recovery states explain what happened and a useful next step. Use polite status announcements for asynchronous updates and an appropriate alert for urgent errors without repeatedly interrupting reading.
- **Evidence and review:** separate proposed from accepted content; show source path, excerpt and uncertainty near the assertion. Preserve original versus replacement text, selection scope, consent and explicit accept/edit/reject controls. Never visually imply that an unreviewed suggestion is accepted.

## Accessibility and layout

All controls must be reachable and operable by keyboard in a logical order. Provide a visible focus indicator distinct from selection, preserve focus after updates or restore it to a sensible trigger, and do not trap it except in a correctly managed modal. Associate labels, descriptions and errors; give icon-only actions names. Use semantic headings, lists and landmarks; announce meaningful progress without flooding live regions. Check text contrast at least 4.5:1 (3:1 for large text), and essential control boundaries, icons and focus indicators at least 3:1 against adjacent colors. Do not depend on color, hover or animation to convey state. Respect `prefers-reduced-motion` and test keyboard, zoom and screen reader behavior.

At comfortable desktop widths use navigation plus a flexible content area; list/detail screens may add a second column. At narrow widths, stack list above detail and form columns into one, keep navigation reachable and actions visible, and wrap long paths and unbroken content without horizontal page overflow. Existing 1050/760/480px breakpoints are implementation facts, not universal targets; validate minimum supported window size and zoom before changing them. Reading areas should have a comfortable line length (roughly 60–80 characters), preserve note whitespace, wrap long paths and offer access to complete evidence. Do not hide provenance behind truncation. Keep long reviews navigable without losing the decision context.

## Change discipline

Every future UI feature must reuse design-system tokens, components and patterns. If a feature exposes a missing token, component, state or pattern, extend the design system and its documentation/examples in the **same change**, before or alongside the feature. Prefer extending an existing role over a one-off literal; if a genuinely new role is needed, describe its purpose and both theme mappings.

Worked gap example: a future review adds a “needs attention” state. First check existing warning and error roles and badges. If neither communicates the state, add a semantic warning text/background/border mapping and a badge or notice variant in both themes, document its use and a small example here, then use it in the feature. Verify contrast, text meaning, keyboard behavior if interactive, and reduced motion. The suggested new role is illustrative; inspect the existing warning roles and badge first.

Definition of done / review checklist:

- [ ] Existing tokens, components and screen patterns reused; any gap and example documented in this change, with proposed versus implemented status accurate.
- [ ] Loading, empty, error, selected, disabled, busy and relevant review/consent states considered; behavior and data boundaries unchanged unless explicitly designed.
- [ ] Keyboard, focus, labels, announcements, contrast, reduced motion, long text, narrow window and zoom checked in both supported themes.
- [ ] Relevant tests and manual checks recorded with limitations; screenshots use synthetic data and isolated app data. No personal notes or remote font dependency.
