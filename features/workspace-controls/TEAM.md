# Workspace design and setup repair team

Run: `run_868273332d70`. Base: local main `dfee160` plus this authorization record.

User authorizes the integrated window design from the proposed plan, reports a folder-setup blocker, and explicitly requests actual BoardUI controls. Window chrome is implementation scope; disconnect/fresh-session features remain proposed. Setup selection/exclusion/control work repairs the approved folder-setup behavior. Large-note chunking is discovery/plan only.

Independent workstreams: window/native chrome; folder setup UI and actual BoardUI controls; read-only oversized-note architecture/discovery. Separate Orca worktrees, no concurrent source writers. The window worker owns main.tsx and window configuration/new chrome CSS; the setup worker owns FolderSetup, component/dependency integration and setup styles. Each writes its own design addendum; manager owns shared design index, integration, shared checks, packaging and personal-remote push. Discovery does not edit runtime/core implementation.

Routing: Jev selected Terra medium for all three streams (window low-confidence fallback0.12; setup0.52; discovery0.80). Every substantive implementation needs independent review; user-required Claude cross-provider review remains a requirement, with previous CLI authentication blocked and to be checked again before dispatching reviewers. Do not substitute a provider silently.

All development/verification uses synthetic notes and isolated app data, never the notes symlink or live folder. No provider calls during this UI work, no changes to source caps/consent/pinning/tools, no restarting the user's running app. Worker commits locally; manager preserves branches and integrates after review. Tests record real RED before the correction and appropriate browser/native evidence.
