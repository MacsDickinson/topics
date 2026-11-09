# Folder controls, fresh sessions and integrated macOS window

Date: 2026-11-09. Status: **proposed for user sign-off**. No implementation is authorized by this document alone.

## Requested outcome

Make connecting and testing notes folders straightforward, and integrate the macOS title bar into Topics' visual shell, following the user's supplied reference. Preserve the existing notes, proposal, privacy and undo boundaries.

## Folder controls

Add **Disconnect folder** to the existing folder settings, reachable from the sidebar. The control explains that disconnecting preserves notes and saved understanding. It cancels active runtime/setup work, revokes the current source capabilities, stops old folder watchers/progress subscriptions, clears the selected-folder setting and visible/derived search state, and returns to “Start with your notes”. Closing and reopening the app remains disconnected.

Disconnect is not a workspace deletion or an undo operation. It does not delete source notes, app-owned Markdown, snapshots or history, and does not create a new notes operation. Reconnecting the same folder restores its saved topics, claim revisions, corrections and context. Connecting another folder cannot expose the previous folder's search results, evidence, previews or delayed events. Disconnect must also work when the previously configured folder is unavailable.

If active work cannot be safely stopped/revoked, report the error and retain the connected state rather than announcing a successful disconnect. Previously transmitted bytes cannot be recalled; after a successful disconnect, old runs cannot read more sources or publish proposals/checkpoints.

The existing **Set up folder** action remains the way to revisit analysis for a connected folder. A fresh application profile does not erase knowledge stored in that folder's Markdown.

## Fresh-session testing

Provide a documented, one-command development launcher for a separate production-app process with new absolute app-data and synthetic notes directories. It starts at the unconnected first-run screen, prints the sample folder location, and leaves the normal instance and its selected folder untouched. Repeating the command creates another independent session. No real notes are copied, reset or modified; no model runs automatically. The sample notes are clearly fictional and contain no prebuilt app records.

This is a small development launcher, not a profile-management subsystem. It retains its temporary session directories for inspection and does not delete data belonging to an active instance. Reconnecting a previously used folder in a fresh session still loads authoritative saved knowledge; use the generated sample folder to test genuinely new onboarding.

## Integrated window appearance

Use Tauri's macOS overlay title-bar style with the text title hidden, reserving space for the native close/minimize/full-screen controls. Extend the existing shell's surface and spacing tokens into a compact draggable header so there is no separate default title-bar strip. Keep native window decorations/traffic lights, resize, full-screen and system window behavior; do not draw imitation OS controls.

Blank header space drags the window. Inputs, buttons, links and text-selection areas remain interactive rather than becoming drag regions. At the minimum supported window size, controls and content cannot overlap. Verify light and dark themes and initial connection, setup and normal workspace screens. Production and native-test configurations must use the same chrome, with the test window retaining its isolated nonpersistent WebKit configuration.

Extend `docs/design-system.md` with the header, traffic-light safe area, drag/non-drag and disconnected-folder patterns alongside implementation. Use existing shared components and semantic colors. Grant only the human main window's minimal drag capability if needed; do not change model tools, source access or filesystem permissions.

## Boundaries

No “delete all knowledge”, destructive reset, global profile switcher, task/backlog work, LLM/backend change or credential handling. No automatic source transmission, notes Git repository, or modifications to live notes during development. All automated fixtures use synthetic notes and isolated absolute app data.

## BDD and execution

The adjacent three feature files contain 13 scenario definitions, expanding to 14 cases across window themes. These are proposed acceptance criteria, not implemented/passing tests. Map service behavior to Rust BDD, presentation to browser tests, and OS chrome/drag/control behavior to real native evidence; browser mocks cannot prove native traffic lights or dragging.

After approval, use Orca workers for independent streams: Rust disconnect/lifecycle, combined frontend/native chrome and folder controls (one owner for `main.tsx`), and the isolated launcher/docs. Agree and test the canonical disconnect contract before frontend implementation. Coordinate command registration rather than giving concurrent writers the same Rust adapter file. Record semantic RED before behavior, then focused checks and independent cross-provider review, followed by signed prototype packaging and hosted checks.

The preceding milestone's required Claude review is still authentication-blocked; do not represent additional Codex review as closing that user-required gate. Approval of this plan authorizes implementation, not a waiver of review.

Technical reference: [Tauri window customization](https://v2.tauri.app/learn/window-customization/), checked against the installed pinned configuration schema (`titleBarStyle`, `hiddenTitle`, `trafficLightPosition`). Native controls require decorations to remain enabled.
