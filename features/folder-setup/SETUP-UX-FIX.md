# Folder setup UX regression and BoardUI controls

2026-11-09. Authorized correction to the existing folder-setup feature following user testing. The user reports disabled manifest review, confusing exclusions/warnings and poor selects/date inputs, and explicitly requests the available BoardUI components. Existing selection, scope, consent, file-size limits and validation semantics remain in force.

## Verified cause

`FolderSetup.tsx` disables preview for every selected inventory document larger than 65,536 bytes, before considering dates. Core manifest preview applies inclusive document-date selection first, then rejects an eligible oversized document. The frontend therefore can block a document that core would exclude for being undated or outside the chosen date range. The screenshot also shows a genuinely large selection; its contents and date have not been inspected.

## Repair scope

Reproduce these states using synthetic notes. Match canonical date eligibility, preserve undated exclusion when dates are set, and keep the backend authoritative. Present real blocking selected notes separately from routine exclusions. Give eligible oversized rows a direct, explicit action to exclude them from this run, update selection/partial folder state and invalidate existing previews. Do not silently skip notes or alter privacy labels. Explain disabled actions beside their resolution, and allow manifest review once the remaining eligible selection is valid. No source writes or model run occurs on preview.

Group routine exclusions into understandable counted disclosures with paths/reasons available. Keep sensitive/private exclusions and unknown reasons accurate; do not claim a file is specifically a symlink or an attachment if the backend's reason only establishes the combined category. No raw dump dominating the primary action.

Use the actual available free BoardUI Select and DatePicker sources (and their accessible dependencies), with pinned provenance/license and the existing theme tokens. This user instruction supersedes the older reference-only/no-React-Aria guidance for these controls. Preserve optional independent date bounds and exact `YYYY-MM-DD` wire values; avoid time-zone conversion or silently narrowing an incomplete range. Support keyboard navigation, clear, cancel, focus return, error labels, dark theme and narrow layouts. Avoid resetting existing app styling through a global preflight.

## Regression expectations

- An oversized selected note outside the date range or undated under an active date filter does not falsely block eligible smaller notes.
- An eligible oversized note is identified by path and size with an explicit exclusion action; using it enables preview for the valid remainder without changing its source bytes or privacy.
- Empty/all-filtered selection and reversed/invalid dates explain their state and cannot transmit anything.
- Hidden/app-owned/non-Markdown/local-only exclusions stay accurate, grouped and inspectable without looking like blocking failures.
- Date/role changes invalidate old manifests and consent. The actual BoardUI controls serialize the same canonical date/role values and retain accessible interaction.

Record semantic RED/GREEN against existing approved folder-setup behavior and preserve native/core regression coverage. This is not approval to enlarge runtime budgets or implement long-note chunking. Supporting a large eligible note completely requires separate bounded-source discovery and a proposed plan/scenarios for user approval.
