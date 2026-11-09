# Topics design system

Status: initial standards draft, 2026-11-09. This is the authoritative direction for UI work. The names, values and component examples below are **proposed**, not a claim that the current implementation provides them. Reconcile this document with the implementation and its actual APIs in the dependent implementation change. See the [reference review](design/reference-review.md) and [screen specifications](design/screens.md).

## Principles

Topics is a local-first reading and review workspace. Give notes and evidence room to breathe; make provenance, uncertainty, privacy and the human decision visible. Use calm neutral layers, thin boundaries, restrained elevation, compact navigation and blue for actions or selection. Preserve native/offline operation, system fonts and existing behavior; presentation must not imply cloud consent, acceptance or capabilities that do not exist.

## Foundations (proposed)

Use primitives only in the central theme mapping; components consume semantic roles. Final names and values must be reconciled with code, and contrast measured on rendered foreground/background pairs in both themes before adoption.

| Role family | Proposed names and use |
| --- | --- |
| Text | `--color-text-primary` reading and headings; `--color-text-secondary` supporting text; `--color-text-muted` metadata; `--color-text-on-action` on filled actions; `--color-text-danger` errors |
| Background | `--color-background-canvas` outer workspace; `--color-background-surface` panels and reading surface; `--color-background-subtle` grouped content; `--color-background-action` primary button; `--color-background-selected` current item; `--color-background-danger` error context |
| Border | `--color-border-default` separators and panels; `--color-border-strong` controls; `--color-border-focus` visible focus; `--color-border-danger` invalid state |
| Other | Add semantic success, warning and highlight roles as needed, with paired text/background roles. Never encode meaning in hue alone. |

Map the same roles centrally for light and dark, including selected, hover, pressed, disabled, focus and error states; preserve the OS preference when adding a theme mechanism, with any explicit user choice taking precedence. Do not implement per-component color inversions. Avoid claiming dark support until every surface and state is checked. Prefer opaque surfaces with sufficient contrast to decorative transparency.

Proposed spacing scale: 4, 8, 12, 16, 24, 32px (`--space-1` through `--space-6`); use 8–12px within a control, 16px within a group, 24–32px between sections. Proposed radii: 6px controls, 10px cards, 14px panels (`--radius-control`, `--radius-card`, `--radius-panel`). Use a subtle panel shadow (`--elevation-panel`) sparingly; borders must also establish edges when shadows are unavailable. Proposed motion: `--motion-fast: 120ms` for state feedback and `--motion-standard: 180ms` for disclosure, with reduced motion removing nonessential animation. These are starting points, not measured implementation constants.

Use the existing local system font stack. Proposed type roles specify size / line height / weight: page title 26px / 32px / 650, section 17px / 24px / 600, body 14px / 22px / 400, reading 16px / 26px / 400, metadata 12px / 18px / 400, control 14px / 20px / 500. Define roles as reusable composite styles rather than scattering metrics; keep color separate. Support zoom and user font settings, and avoid truncating content needed for a decision.

## Components and states (proposed conventions)

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

Worked gap example: a future review adds a “needs attention” state. First check existing warning and error roles and badges. If neither communicates the state, add a semantic warning text/background/border mapping and a badge or notice variant in both themes, document its use and a small example here, then use it in the feature. Verify contrast, text meaning, keyboard behavior if interactive, and reduced motion. These names and APIs remain illustrative until implemented.

Definition of done / review checklist:

- [ ] Existing tokens, components and screen patterns reused; any gap and example documented in this change, with proposed versus implemented status accurate.
- [ ] Loading, empty, error, selected, disabled, busy and relevant review/consent states considered; behavior and data boundaries unchanged unless explicitly designed.
- [ ] Keyboard, focus, labels, announcements, contrast, reduced motion, long text, narrow window and zoom checked in both supported themes.
- [ ] Relevant tests and manual checks recorded with limitations; screenshots use synthetic data and isolated app data. No personal notes or remote font dependency.
