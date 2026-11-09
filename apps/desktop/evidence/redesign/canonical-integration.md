# Canonical setup integration into the accepted redesign

This follow-up preserves accepted structural redesign `7407b50be3cdb576137b5eb56c1755b71b5d3105`
and integrates authoritative backend main `85ae348aa9a329dcdb5dbfc0ec0655bfb6dea3b6`.
It is confined to the isolated redesign worktree. No main/other worktree edits,
push, user-app restart, native suite or genuine model invocation occurred.

## Integration map

| Authorized source | Local integration | Resolution |
| --- | --- | --- |
| `85ae348aa9a329dcdb5dbfc0ec0655bfb6dea3b6` | Merge `ae159f99227ae1d6299f2868adc2cf99ac80e7da` | Exact two-parent history merge, not duplicate cherry-picks; accepted JSX/CSS/primitives retained on presentation collisions, authoritative backend/native/generated content retained. |
| `28926a7` | `61a71fd31e47ca22662a5d33a2c142132084d564` | Harness prerequisite was absent and applied before its dependent journey. |
| `1d1c37b` | `412cbaeaec38426a921877f1ae0efe43596ea18b` | Native synthetic hundred-note/restart harness and feature evidence imported unchanged. |
| `a041a968cc0512515d0a5f0c91df18bce1bf74f3` | `07fd8081e3f4d42718f3be6a20b68e0111d79869` | Canonical generated assertions, reconsent, connection receipts, context and named diffs integrated; shared controls/sections/fields reapplied without changing requests or state logic. |
| `41ed1c085a45db3f1ea0a40ffff70784d1ae0a3c` | `02047d105cd9eabb429152ae4994d5fb3e6da730` | Selected coverage/exclusions, size guard, readable units and native reconsent journey retained. |

Earlier feature equivalents remain documented in [README](README.md); the backend
merge brings their original history into ancestry. The main owner should merge
this branch/checkpoint and avoid replaying all equivalent feature commits again.
No generated TypeScript file was manually authored. Relative to backend `85ae348`,
the only pre-budget Rust differences are the authorized a041 command registrations,
adapters and type-export additions.

## Behavior parity and presentation

[`canonical-parity.json`](canonical-parity.json) records structural AST equality
against `41ed1c0` for both inventory/watch effects, job filtering/cleanup, every
FolderSetup action/state/request function and all IPC call expressions throughout
FolderSetup and ContextReview, including inline connection/context requests.
This comparison ignores source formatting through AST structure; it is stronger
than a visual diff but does not replace native service acceptance.

The new states reuse the accepted semantic palette, system font and shared React
primitives. Updated scope retains coverage context, names changed/removed/excluded
paths and requires fresh checkbox consent. Connections use a separate receipt;
supplemental context is shown as the user's words alongside the understanding.
Named old/new comparisons use `.topic-diff`, and group save receipts retain exact
candidate/dependency scope. The design guide documents these patterns and a
worked reconsent example in the same change.

## Verification and new-state evidence

Pre-budget checkpoint: **98 browser tests passed**, including the full accepted
83-check scope plus canonical feature and new themed interaction cases. Production
TypeScript/Vite build passed with the project-local **Node 24.22.2**. See
[`canonical-full-browser.txt`](canonical-full-browser.txt),
[`canonical-targeted.txt`](canonical-targeted.txt) and
[`canonical-build.txt`](canonical-build.txt).

Tests use strict isolated port **14337** and absolute synthetic
`TOPICS_APP_DATA_DIR=/tmp/topics-boardui-redesign-browser-0cd733`. They neither read
the repository notes symlink nor use a personal root. Browser fixtures mock IPC;
their passing requests do not independently prove persisted backend subset/ID
retention, which belongs to the authoritative core/native acceptance.

New tests exercise keyboard preview and Space consent, reset of consent after
another preview, exact root/job/revision/manifest requests without creating a
replacement job, context save without claim confirmation/correction, separate
connection ids, named earlier/current evidence, group receipts without implicit
save, and selected completion with exclusions retained. Existing contrast,
hostile-text, async/root guard, focus, long path and narrow checks remain intact.

All five new states have light/dark desktop (1440 × 960) and narrow light
(600 × 960) full-page screenshots in [`canonical/`](canonical/):

| State | Filename suffix |
| --- | --- |
| Updated scope with completed subset and removed/excluded path | `updated-scope.png` |
| Saved supplemental user context | `supplemental-context.png` |
| Separate connection confirmation receipt | `connection-receipt.png` |
| Named earlier understanding versus new evidence | `named-diff.png` |
| Exact topic-group save receipt | `topic-group-receipt.png` |

Prefix each suffix with `light-`, `dark-` or `narrow-`. The 15 screenshots and
three `canonical-*-contact.png` sheets were visually inspected. The existing
39 `after/` screenshots are regenerated by the full suite; the immutable 21
baseline screenshots remain unchanged.

## Pending final acceptance

The coordinator's final **64 KiB default budget** and **runtime packing fix** are
explicit required dependencies still awaiting supplied SHAs at this checkpoint.
The current authorized 41ed1c0 default is 256 KiB; no replacement type, handler or
unsupplied backend patch has been invented here. Final validation/completion must
follow those commits. Native fixture/package acceptance remains coordinated with
the feature manager; imported native green logs describe their runs, not a new run
performed by this worker. Screen-reader/actual WebView zoom and genuine runtime
acceptance are not established by these browser captures.
