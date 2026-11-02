# Phase 0 hosted macOS native startup investigation

Status: **independent review accepted; Linux and macOS hosted verification passed at `712014f4a815980b59f59884f4e32028302a9a92`**. The original WebKit failure mechanism remains unconfirmed. The implementation branch fast-forwarded from `bb88282` to clean main/origin-main `268b4d84fdd850edaa4161aa93192da3cb452411` before changes; the coordinator subsequently integrated and pushed the reviewed commit. Production logic and approved features are unchanged.

## Failure and evidence

[Hosted macOS run 41446356300](https://github.com/MacsDickinson/topics/actions/runs/41446356300) ran on **macOS 26.6.2**, image `macos-26-arm64/20261026.0455.2`. Normal real-app search/coherence passed both tests. The next session, empty `TOPICS_APP_DATA_DIR`, reported WebKit **unsupported JavaScript result type (code 5)** from the embedded driver's `execute/sync` during plugin initialization, followed by a startup-test timeout. Relative-path startup was never reached. [Linux core run 41446356310](https://github.com/MacsDickinson/topics/actions/runs/41446356310) succeeded.

Local macOS is **15.5**. The installed packages are WDIO Tauri service/plugin 1.5.1, Tauri 2.12.5 / tauri-utils 2.10.2 and Wry 0.57.1. No dependency version changed.

## Confirmed harness defects, and causal limits

The old startup-only service environment replaced HOME with a **nonexistent** synthetic path, while the normal session retained the user's HOME. It also used a persistent default WebKit data store and asserted that any creation of the entire fake HOME meant Topics had fallen back. That combined two separate concerns: a valid GUI runtime environment and the Topics config/index fallback guard.

A safe local child-process probe loaded Foundation and compared only `NSHomeDirectory()`'s returned string to the synthetic HOME; it did not enumerate/read/create personal home or Application Support content. Result: **Foundation home did not match synthetic HOME**, and the synthetic directory still did not exist. [Probe record](../apps/desktop/evidence/ci-home-probe.txt). Therefore HOME alone is not Cocoa/WebKit storage isolation, and checking only whether that directory appeared was not evidence that WebKit had been isolated.

The installed Wry source selects `WKWebsiteDataStore::defaultDataStore` in ordinary mode and `nonPersistentDataStore` for incognito. The installed Tauri config supports that flag; it is also documented in the official [window configuration reference](https://v2.tauri.app/reference/config/#incognito), and the exact installed [Wry 0.57.1 implementation](https://github.com/tauri-apps/wry/blob/wry-v0.57.1/src/wkwebview/mod.rs) confirms the data-store selection. The new overlay compiled and ran successfully locally.

The hosted stack locates the failure in the embedded WebKit script/bootstrap path, before the test can interact with the search field. It does **not** establish that HOME caused code 5. Upstream [WDIO issue 540](https://github.com/webdriverio/desktop-mobile/issues/540) describes macOS-26 cold-WebView/occlusion problems, but that is related evidence, **not proof of the same cause here**. No assertion is made that the entire macOS framework runtime honors HOME or that its internal mechanism has been identified. Hosted rerun is required.

The coordinator explicitly approved this deterministic isolation correction with those limits, plus diagnostics, while requiring all existing native coverage and production behavior to remain unchanged.

## Focused correction

- Every real app session receives the same **pre-created, owned temporary HOME/XDG directories**. WDIO fails before launch if those values are missing, outside the expected sandbox paths or nonexistent.
- The **native-test config overlay only** sets the WebView to incognito. All other window properties exactly match production; the production config, Rust and React sources are unchanged. HOME does not serve as the Cocoa isolation mechanism: the nonpersistent WebKit store is explicit.
- Fallback checks target the known synthetic Topics app-data paths, rather than forbidding legitimate OS scaffolding in the entire HOME. The invalid relative override resolves back inside the owned temporary sandbox if a future resolver regression were to accept it, keeping even failure paths synthetic.
- A native-test process parent records app PID, exit code/signal and cleanup signal forwarding. Per-session backend/frontend and WebDriver logs plus a small OS/runtime summary go to `evidence/native-diagnostics/{normal,empty,relative}`. No environment dump or credentials are recorded. Existing CI artifact collection already includes this directory; the workflow did not need changing.
- No retry loop, additional sleep, increased timeout, skip, weakened startup assertion or mock IPC was added. Normal search/coherence and both invalid overrides remain mandatory.

## Verification, including the failed local attempt

1. Before the overlay change, `node --test scripts/native-environment.test.mjs` failed because the native WebView was not configured nonpersistent. [Regression red](../apps/desktop/evidence/ci-isolation-red.txt). This proves the isolation gap, not reproduction of hosted code 5.
2. After correction, the two deterministic Node checks passed: native-only nonpersistent window config with production properties unchanged; prepared synthetic directories with positive/negative fallback guard cases. [Regression green](../apps/desktop/evidence/ci-isolation-green.txt).
3. The first local native attempt, before process-exit instrumentation, reached the real ledger screenshot and then lost the app/server with **ECONNREFUSED**. It failed and is preserved in [first-attempt output](../apps/desktop/evidence/ci-native-first-attempt.txt) and [first-attempt driver log](../apps/desktop/evidence/native-diagnostics/first-attempt-normal/wdio.log). Its exit cause is **unconfirmed**. It differs from hosted code 5 and is not labeled explained or fixed. There was no blind rerun: process-exit diagnostics were added before the next verification.
4. `npm run test:native --prefix apps/desktop` with diagnostics: **exit 0**, **2 normal assertions + 1 empty-override + 1 relative-override**, across three real app sessions. [Native result](../apps/desktop/evidence/ci-native-check.txt). All three process logs show only expected WebDriver cleanup SIGTERM followed by app exit: [normal](../apps/desktop/evidence/native-diagnostics/normal/app-process.jsonl), [empty](../apps/desktop/evidence/native-diagnostics/empty/app-process.jsonl), [relative](../apps/desktop/evidence/native-diagnostics/relative/app-process.jsonl). The nonpersistent WebView is therefore supported locally; the first disconnect was not reproduced in this instrumented verification, which does not prove its cause.
5. `node --check` passed for the native runner, environment helper and diagnostic launcher. `npm run build --prefix apps/desktop` passed strict frontend TypeScript and the ordinary 35-module production build; [build log](../apps/desktop/evidence/ci-production-build.txt). Production assets contain none of `wdioTauri`, `__wdio` or `WDIO Plugin`.

Native source snapshots remained unchanged after their explicit synthetic editor fixture was restored, no Topics fallback directory appeared, and the temporary notes/config/index/home were removed. All instrumented child processes exited. No live notes or personal Topics configuration/defaults were probed. No personal diagnostic-report folder was inspected to explain the first disconnect.

## Unchanged product and hosted verification

`git diff --exit-code 268b4d8` is empty for crates/core services, frontend source, production Tauri Rust/config/manifest, package/Cargo manifests and lockfiles, approved features and plan. [Proof](../apps/desktop/evidence/ci-unchanged-production.txt). Test-only overlay/harness/diagnostics and this evidence are the scope.

Fresh Claude Sonnet 5.5 independently reviewed `268b4d8..712014f` and accepted it with no blocking findings. The [review record](reviews/phase-0-ci-review.md) includes its verification and nonblocking diagnostic/cleanup limitations. The reviewer did not repeat the GUI run.

The coordinator fast-forwarded main and pushed `712014f`. [Linux core CI 41467316951](https://github.com/MacsDickinson/topics/actions/runs/41467316951) and [macOS desktop CI 41467317179](https://github.com/MacsDickinson/topics/actions/runs/41467317179) both completed successfully. The macOS job passed the full Rust, BDD, browser and real native script, including both invalid-override startup sessions, and uploaded its synthetic UI evidence.

This is evidence that the revised harness passes on the hosted runner. It does not identify the original WebKit mechanism or attribute the improvement to a single setting. If the failure recurs, retain the per-session process/backend/driver artifacts; do not add retries or relax the native checks. Previously accepted nonblocking product follow-ups remain unchanged.
