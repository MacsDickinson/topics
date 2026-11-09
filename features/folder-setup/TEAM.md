# Folder setup implementation record

User approval: “plans good - LFG”, 2026-11-09. Approved scenario baseline `f9b6508` (21 scenarios). BDD behavior work must follow actual RED evidence; approval alone is not a passing result.

Orca run: `run_ee41ef4a4c2c`. Previous prototype baseline: `286443f`, including both hosted CI suites passing. Workers commit to isolated branches; manager integrates and pushes to the personal origin.

| Stream | Task / initial dispatch | Model | Ownership |
|---|---|---|---|
| Core contracts, services and BDD | `task_32a941a9c030` / `ctx_629b4894c928` | Codex Sol, high | Canonical core types, store persistence and privacy, explicit BDD bindings, group save/verification/undo |
| Bounded setup job adapter | `task_50b086b5058f` / `ctx_420c8d463718` | Codex Sol, high | Orchestration over frozen store/runtime seams, capabilities, pause/resume, progress and failed coverage |
| Desktop | `task_343fc5e8a75e` / `ctx_97a6c457d3fe` | Codex Terra, medium | Setup tree, manifest consent, map/list and topic briefings, contextual decisions, IPC, browser/native evidence |

Routing: Jev selected Sol high for core (confidence 0.94). Job routing had a low-confidence Terra fallback (0.06); the coordinator selected Sol high for consequential privacy/capability/cancellation orchestration. Desktop used the Terra medium fallback (confidence 0.40). Independent Claude reviews follow substantive implementation; reviewers start only once the relevant implementation worker is idle.

Core publishes the strict shared contract, code signatures and real RED evidence first. Job and desktop can perform discovery/design and presentation/protocol RED independently, but cannot implement guessed duplicate contracts. The manager owns root manifests/lock, shared check scripts, integration and final validation.

A separate design run `run_b6192eed0a8c` owns BoardUI presentation foundations and standards. Its coordinator agrees not to mutate main during setup integration. Setup desktop consumes its accepted tokens/patterns and extends missing patterns/docs in the same change. Design review and accepted ordered SHAs must be recorded before integration; initial design commits are not presumed accepted.

All development and tests use synthetic temporary notes and isolated absolute app data. No live notes, source symlink traversal, source-note rewriting, personal authentication inspection or notes Git repository. Model calls during final acceptance use synthetic notes only. Existing privacy, publication permits, source scopes, correction rules, snapshots, journals and undo remain required boundaries.

Final gates: approved scenarios mapped honestly to service/browser/native evidence; all prior tests plus relevant new checks; independent review closure; real native synthetic discovery/save/correction/restart acceptance; production bundle rebuilt without native-test mode and strict signature verified; pushed revision with hosted Linux/macOS checks. No worker report alone marks the feature delivered.

## Shared checkpoints and integration

- Initial canonical API/stub checkpoint `bfe9f65` integrated as `8c81790`. Its first BDD output proves the runner and inventory stub are RED; the remaining panic-only step scaffolding must be replaced with real fixtures/actions/assertions and relevant semantic RED before implementing each behavior. It is not evidence that all 20 service behaviors have already been tested meaningfully.
- Job discovery/scaffold and facade checkpoints `b995e5d`, `a633158`, `4260dfc` integrated. `SetupCoordinator` exposes canonical job DTOs with a bounded watch subscription; setup workers still implement the behavior.
- Authorized runtime seam `70970f1` RED / `285bf89` GREEN: a default-false host hook lets successful empty setup turns use the real publication permit and stable checkpoint ID. Existing hosts keep their previous empty-turn behavior. Full independent review is pending with the job stream.
- BoardUI design run accepted `279002e` after contrast repairs. Ordered `dfecf91`, `b432b57`, `b5dc06f`, `279002e` landed through main `7363a58`; the original standards commit `2e4646a` must not be applied again. Its native, real browser zoom and screen-reader limits remain acknowledged. Desktop feature work extends these tokens/patterns and examples.

Manager-owned dependency choices: `topics-setup` workspace member; `tokio` sync for bounded/coalescing job snapshots; optional pinned `ts-rs = 12.0.1` for canonical Rust-to-TypeScript generation; pinned Tauri dialog plugin `2.8.4` for a user-only native root folder chooser. Binding generation uses required nullable fields and `Config::with_large_int("number")` to match existing JSON numeric representations, not JavaScript bigint. The chooser uses a Rust adapter off the main thread; no broad JavaScript file/dialog permissions or model tool are added. Sources: [ts-rs export API](https://docs.rs/ts-rs/12.0.1/ts_rs/trait.TS.html), [integer configuration](https://docs.rs/ts-rs/12.0.1/ts_rs/struct.Config.html), [Tauri dialog plugin](https://v2.tauri.app/plugin/dialog/). Manifests/lock are owned by the manager; workers exclude manager-local modifications from their source commits unless explicitly authorized for the new crate manifest.

## Runnable integration checkpoint (in progress)

Core `4815346`, additive contract/semantic RED `b57be61`, and runnable repair `7488504` are integrated through main `0913ab2`; optional TypeScript derives were preserved. Manager's focused integrated `setup_service` and `setup_amendments` run passes **23 + 5 tests** with `test-support`. The five new cases cover reconsent/completed subsets, connection confirmation, distinct supplemental context, classification and guarded empty terminal completion; full regressions and independent review remain outstanding. A standalone first focused invocation without the synthetic fault-injection feature failed compilation; the intended feature-enabled run passed, and the fault-only test is now feature-gated.

The job stream recorded actual old-facade baseline RED at `2c507ba`, separate from its earlier upstream-stub-blocked tests. Main includes adapter retry `95e0c8c`, budget/NeedsConsent snapshot fixes `5090f0f`, and distinct synthetic cited-claim fixture/renderer `6f2436c`. The last observed protocol failures (empty completion/prior completed context) must be rerun unchanged against `7488504` before claiming complete protocol GREEN.

Desktop checkpoints `cad6757`, `e654210`, `91bce2b`, `cb12b9c`, shared primitives `454f55a` and adoption `5452e9d` are integrated. The graph-only native journey processed 100 synthetic notes in five turns under one manifest authorization and one save, then reopened after disposable index deletion. It retained source hashes; exclusions remain explicit. This does **not** establish cited-fact correction/restart acceptance. The full cited-claim journey, new API states and coherent generated-types/UI checkpoint remain pending.

A separately authorized full structural redesign continues under design run `run_b6192eed0a8c`, worker `ctx_a5c379a871bd`. Its unreviewed structural checkpoint `c6965c0` must not be blindly applied over newer setup behavior. The design coordinator consumes exact setup commits in its isolated branch, preserves state/handlers, reviews independent screens first and will target-review the final coherent setup delta. Final packaging depends on both accepted streams.

Coverage ruling: real job integration exposed a broad `complete=false` mismatch solely because inventory now always discloses app-owned `topics/` exclusions. The manager chose exact selected-manifest completion, with excluded locations reported separately and no whole-folder completeness claim. Core owns focused RED/predicate/contract fix; job tests retain their exact selected-source, call and privacy assertions; desktop replaces its earlier observed false-status workaround only once the fix lands. The approved plan retains its original text plus an explicit implementation clarification.
