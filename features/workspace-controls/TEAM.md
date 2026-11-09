# Workspace design and setup repair team

Run: `run_868273332d70`. Base: local main `dfee160` plus this authorization record.

User authorizes the integrated window design from the proposed plan, reports a folder-setup blocker, and explicitly requests actual BoardUI controls. Window chrome is implementation scope; disconnect/fresh-session features remain proposed. Setup selection/exclusion/control work repairs the approved folder-setup behavior. Large-note chunking is discovery/plan only.

Independent workstreams: window/native chrome; folder setup UI and actual BoardUI controls; read-only oversized-note architecture/discovery. Separate Orca worktrees, no concurrent source writers. The window worker owns main.tsx and window configuration/new chrome CSS; the setup worker owns FolderSetup, component/dependency integration and setup styles. Each writes its own design addendum; manager owns shared design index, integration, shared checks, packaging and personal-remote push. Discovery does not edit runtime/core implementation.

Routing: Jev selected Terra medium for all three streams (window low-confidence fallback0.12; setup0.52; discovery0.80). Every substantive implementation needs independent review; user-required Claude cross-provider review remains a requirement, with previous CLI authentication blocked and to be checked again before dispatching reviewers. Do not substitute a provider silently.

All development/verification uses synthetic notes and isolated app data, never the notes symlink or live folder. No provider calls during this UI work, no changes to source caps/consent/pinning/tools, no restarting the user's running app. Worker commits locally; manager preserves branches and integrates after review. Tests record real RED before the correction and appropriate browser/native evidence.

| Stream | Task | Dispatch | Worktree |
| --- | --- | --- | --- |
| Integrated native window | `task_c71228949474` | `ctx_1ffeda913ca4` | `topics-integrated-window` |
| Setup recovery / actual BoardUI controls | `task_10f68e2908df` | `ctx_fc00c925f91f` | `topics-setup-boardui-controls` |
| Large-note discovery / proposed scenarios | `task_8b10f40cb0c2` | `ctx_5a8103d4bb6d` | `topics-large-notes-discovery` |

All three started from `8dd1751`, confirmed attached and live. BoardUI primary sources confirm free source components built on React Aria. No relevant BoardUI connector is exposed, so official web/source references are used.

## Verification checkpoint

Integrated window changes were independently reviewed by Codex and integrated through `1595c52`. Native evidence covers owned synthetic windows, scrolling, dragging, traffic-light controls and focus-safe skip links; required Claude review remains outstanding.

Setup implementation is frozen and clean at `d6435318b380551317f94595998abac1acc3187e` after `9d3a48e`, `a932985` and `8e95e7e`. It retains actual BoardUI sources and pinned dependencies, fixes date-aware oversized-note blocking, groups exclusions, and restores native date-trigger focus after Escape. The implementer reports 124 browser tests before the final focused changes, 29 focused browser tests afterwards, build success and the synthetic native oversized/date/manifest/privacy/source-hash regression. PID-targeted synthetic pointer/key delivery distinguished a driver-generated scroll from normal pointer behavior; this is not physical hardware acceptance. See `docs/design/setup-controls.md` on the implementation branch for details.

Independent read-only review ran as `task_c3f6079a8fb9`, dispatch `ctx_98671f872482`, fresh Codex Terra medium context. Routing used low-confidence fallback (0.28). A final test-only commit `e9c7e9f` added Cancel/Apply/Clear focus assertions during the first review. The continuation `task_356e0be3e83d`, dispatch `ctx_ca685314ca2f`, reviewed that exact tip and independently replayed the committed synthetic native regression plus four focused browser cases: no concrete findings. The earlier review ran 26 browser cases. Both reviewers and implementers released native GUI/cache ownership. The final source worktree was clean. Review report: `/tmp/topics-setup-boardui-review.txt`.

The setup commits are integrated through `4be7f66`; the manager is running the full combined check before packaging. The standard desktop check now includes the synthetic oversized/date/focus regression after rebuilding the native binary, so the integrated result does not rely on the reviewer's reused target binary. Shared design guidance documents the actual BoardUI components and warning groups. This additional Codex review does not replace the user's required Claude review, blocked by expired CLI OAuth authentication.

Large-note chunking discovery and proposed BDD scenarios are integrated as documentation only (`045f1b7`, `15cbcdc`). Implementation awaits the user's explicit approval; current eligible oversized notes still require exclusion.
