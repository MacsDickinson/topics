# Topics: macOS stack and behaviour-driven delivery

Research date: **1 November 2026**. Public documentation was checked on this date. This is an architectural recommendation, not a compiled framework benchmark. Neither `cargo` nor `rustc` is installed on this worker's PATH, so no Rust spike was attempted. No private notes were read or transmitted.

## Recommendation

Build a **Rust core with a thin SwiftUI macOS shell, using UniFFI**. Use AppKit where the text system or macOS integration needs it. Keep domain rules, persistence, retrieval, task migration and LLM orchestration in Rust. Use cucumber-rs for user-approved behavioural contracts and XCUITest for a small set of real macOS journeys.

This best fits a Mac-only personal tool whose value depends on trustworthy state and comfortable daily use. Rust preference need not mean every button must be written in Rust. Native text selection, keyboard focus, menus and accessibility matter more here than maximising language uniformity. SwiftUI and AppKit avoid recreating those conventions; the core remains independently testable by an agent.

**Tauri 2 with TypeScript and a conventional web UI is the runner-up.** Choose it if rapid markdown/chat UI iteration and browser-based agent inspection outweigh using native controls. Its macOS UI-testing position has improved materially; the earlier blanket claim that macOS E2E is unsupported is no longer correct. Do not choose GPUI merely because Zed feels excellent: its application framework and editor are different integration commitments.

Before committing the shell, prove a small vertical slice: stream a synthetic chat reply, select/copy text, correct a fact, reopen the app, and carry a task to tomorrow. Include keyboard-only interaction, a long conversation and one automated native smoke test. If UniFFI's concurrency boundary becomes awkward, prefer Tauri over designing a bespoke bridge.

## Options compared

These are fit assessments for Topics, not universal quality scores. “Native” below distinguishes platform controls from custom GPU or web rendering. All eight options can display incremental chat updates; cancellation, backpressure, ordering and preserving scroll position remain application work.

| Stack | Mac feel and maturity at research date | Markdown, editing and streamed chat | Agent testability |
|---|---|---|---|
| **Rust + SwiftUI / UniFFI** | Strongest platform fit. Mature Apple UI stack; FFI adds a separate build boundary. | Native text display and input; use an AppKit text view if selection, editing or markdown presentation exceed SwiftUI's needs. Keep markdown source editing separate from rendered preview initially. | Headless Rust domain tests; Swift view-model tests; standard XCUITest accessibility queries, keyboard actions and screenshots on a Mac. Best native integration evidence, but not a headless GUI. |
| **Tauri 2** | Established distribution and plugin ecosystem. Native window and menus around a WKWebView; content requires deliberate Mac styling and behaviour. | Best access to web markdown/editor libraries. HTML/CSS suit streaming chat, virtualised lists and code blocks. Sanitize rendered markdown. | Excellent browser renderer tests plus Rust tests. Current documented WebdriverIO embedded provider supports the actual macOS app. Native chrome still needs separate smoke coverage. |
| **Dioxus desktop / native** | Desktop is Rust-driven system WebView, not platform widgets. Native/Blitz is an early-adopter renderer, not an AppKit replacement. | Desktop supports HTML/CSS; Rust runs outside the browser, so web-library integration may need JavaScript evaluation. Streaming is straightforward in Rust. | Pure state and VirtualDom tests are useful. Web-target browser tests do not prove the desktop renderer. No equivalent mature desktop E2E route was verified in this review. |
| **GPUI** | Proven in Zed; custom GPU UI with good potential for keyboard-focused workflows. Framework remains closely tied to Zed. | Excellent foundations for text-heavy apps, but Zed's full editor is not a bundled GPUI widget. Budget for input, selection, markdown and accessibility work or evaluate component libraries separately. | Strong in-process test contexts and simulated input. Test executor/state deterministically; separately prove real rendering and macOS accessibility. Less standard external automation than DOM or XCUITest. |
| **Slint** | Supported desktop framework with custom-rendered widgets and a macOS-like style. More productised tooling than many Rust GUIs. | Suitable for structured panels and simple text entry. Full markdown editing requires extra integration; rendering rich text is not equivalent to an editor. | Preliminary testing backend, mock time, introspection and newer headless/MCP facilities are promising. Licensed Python system-testing tooling exists. Pin matching versions and budget for API/tooling changes. |
| **egui / eframe** | Established Rust toolkit, especially good for tools. Immediate-mode custom controls look unlike standard Mac apps without significant styling. | Good plain text and incremental output; markdown via ecosystem crates. Rich document editing, sophisticated selection and large chat lists need scrutiny. | Excellent all-Rust UI harness option: `egui_kittest` uses AccessKit queries, simulated interaction and image snapshots. One of the strongest choices if native feel is relaxed. |
| **iced** | Established Elm-style Rust architecture; custom-rendered controls. Good separation between state, messages and effects. | Built-in rich text, markdown and text-editor widgets are useful starting points. Still validate selection, IME and long transcript behaviour. | Update functions are easy to test. Current `iced_test` explicitly supports headless widget simulation and selectors; supplement with native integration and visual tests. |
| **Makepad** | Custom GPU UI, live design and broad platform ambition. Smaller ecosystem for this kind of desktop productivity app. | Text/creative UI foundations exist; assess markdown and editing with a spike rather than assuming a complete document toolkit. | Rust logic tests remain easy. A sufficiently documented general UI automation/snapshot contract was not verified. Would require more project-owned harness work. |

Sources for the assessments: [Tauri architecture](https://v2.tauri.app/concept/architecture/) and [test support](https://v2.tauri.app/develop/tests/); [Dioxus desktop](https://dioxuslabs.com/learn/0.7/guides/platforms/desktop/), [renderer split](https://dioxuslabs.com/learn/0.7/beyond/project_structure/) and [Blitz status](https://github.com/DioxusLabs/blitz); [GPUI's own scope](https://gpui.rs/) and [input test example](https://github.com/zed-industries/zed/blob/main/crates/gpui/examples/view_example/example_tests.rs); [Slint styles](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/style/), [testing backend](https://docs.slint.dev/latest/docs/rust/i_slint_backend_testing/) and [system testing](https://testing.slint.dev/); [egui](https://github.com/emilk/egui) and [egui_kittest](https://github.com/emilk/egui/blob/main/crates/egui_kittest/README.md); [iced widget history](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md) and [headless tests](https://docs.iced.rs/iced_test/index.html); [Makepad](https://github.com/makepad/makepad). Dioxus documentation and Blitz status wording differ across pages; treat the native renderer as early adoption rather than promise a production stability level. No future release schedule is assumed.

### Desktop integration, packaging and size

| Stack | Keyboard, menu bar, global hotkey and notifications | Packaging/signing and size implications |
|---|---|---|
| SwiftUI shell | Native Commands, focus and accessibility; `MenuBarExtra`; AppKit/system APIs or small adapters for global shortcuts and notifications. | Xcode app/archive plus Rust library. No browser engine bundled. FFI and multi-architecture library builds add CI work. |
| Tauri 2 | Menus, tray, global-shortcut and notification facilities are documented. DOM keyboard handling must respect Mac shortcuts and text input. | Integrated app/DMG/signing workflow. Uses system WebView, so avoids bundling Chromium. |
| Dioxus desktop | Window/tray facilities exist; further integrations through Rust/macOS crates. Validate shortcuts and notification lifecycle. | Bundler exists; still prove signed app pipeline. System WebView gives a small baseline, but assets and dependencies dominate real size. |
| GPUI | Actions and key dispatch are a strength; application menu support. Tray/global shortcut/notification requirements need specific platform adapters. | Own app bundling/signing integration. GPU runtime, fonts and chosen components determine size. |
| Slint | Declarative keyboard/focus handling. Native-style widgets do not imply complete menu-bar/system-service integration. | Desktop deployment supported; renderer/backend and licensing choices affect shipped dependencies. |
| egui/eframe | Input is easy to handle, but Mac menus, tray, notifications and global shortcuts require additional integration. | Own bundling/signing tooling. Renderer/font choices drive footprint. |
| iced | Message/event model supports shortcuts; OS-wide integration requires adapters beyond on-window events. | Own packaging pipeline; selected graphics/text stack affects footprint. |
| Makepad | Custom keyboard/event system; verify Mac menu and OS-service adapters individually. | Repository platform tooling helps, but signing and release validation remain project responsibilities. |

The latter five integration rows are implementation estimates, not claims that all adapters are absent. Prefer thin platform adapters over domain logic in a toolkit callback. Tauri's [menu](https://v2.tauri.app/learn/window-menu/), [tray](https://v2.tauri.app/learn/system-tray/), [global shortcut](https://v2.tauri.app/plugin/global-shortcut/) and [notification](https://v2.tauri.app/plugin/notification/) docs cover the requested facilities. Apple documents [MenuBarExtra](https://developer.apple.com/documentation/swiftui/menubarextra).

Do not make a decision from hello-world megabyte claims. No comparable release binaries were measured. Tauri documents [size optimisation](https://v2.tauri.app/concept/size/); Dioxus advertises small desktop baselines, but these omit Topics' eventual assets, storage and model dependencies. Record stripped arm64 app size, universal app size, download size and idle RSS in the vertical slice. Keep local model weights outside the application bundle.

Every candidate needs a credible release check: build the app, sign nested code correctly, notarise, staple, install into a clean account and launch it. Tauri has a [macOS signing guide](https://v2.tauri.app/distribute/sign/macos/); Apple documents [Developer ID notarisation and command-line integration](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution). Start with direct distribution; App Store sandbox/bookmark requirements should be a separate decision.

## BDD as the acceptance contract

For each feature, the agent drafts concrete Gherkin examples and unresolved policy choices. Macs signs off the examples before implementation. Record approval against the feature revision; changing the meaning requires renewed sign-off. This is product agreement, not approval for every test run.

Use [cucumber-rs](https://cucumber-rs.github.io/cucumber/current/) in a Rust integration-test crate. Step definitions call the same application service as the GUI, never a second implementation of the rules. A scenario World owns a temporary repository, temporary SQLite database, frozen clock, deterministic IDs and scripted LLM transport. Use real storage for durability scenarios and in-memory adapters for narrow domain rules.

Most scenarios should run without network, a window or a real model. A scripted structured correction proposal tests orchestration and policy; it does not establish that a live model understands arbitrary prose. Maintain a separate synthetic prompt-evaluation set for that, with structured-output validation and human-reviewed examples. A model proposes changes; deterministic code validates and applies them.

Example `features/person_correction.feature` (synthetic people only):

```gherkin
Feature: Correct a fact about a person via chat
  User corrections take precedence over imported meeting claims.

  Scenario: A correction survives reimport and restart
    Given person "person-1" is named "Alex Example"
    And a meeting import claims their pronouns are "he/him"
    And the chat interpreter proposes pronouns "they/them" for "person-1"
    When I send "Alex uses they/them pronouns" in that person's chat
    Then the current pronouns for "person-1" are "they/them"
    And the correction records my message as its source
    And the earlier meeting claim is retained as superseded evidence
    When the same meeting is imported again
    And I restart the application service
    Then the current pronouns for "person-1" are "they/them"
    And there is exactly one applied correction

  Scenario: Ambiguous identity requires clarification
    Given two people are named "Alex"
    When I send "Alex uses they/them pronouns" in general chat
    Then I am asked which person I mean
    And no person's facts have changed
```

Example `features/task_carry_forward.feature`:

```gherkin
Feature: Carry forward an open task
  Daily review preserves task identity and its history.

  Scenario: Migrate an unfinished task exactly once
    Given today is "2026-11-02" in "Europe/London"
    And task "task-1" was open in the previous daily log
    And task "task-1" has the comment "Waiting for a reply"
    When I carry task "task-1" forward to today
    And I repeat the same carry-forward request
    Then today's log links to task "task-1" exactly once
    And its comment is still "Waiting for a reply"
    And the previous log records its migration to today
    And there is still exactly one backlog task with ID "task-1"

  Scenario: Completed tasks do not carry forward
    Given task "task-2" is completed
    When I request the next daily review
    Then task "task-2" is not proposed for migration
```

These encode proposed policies, including immediate application of an unambiguous explicit correction and explicit task migration. Macs should confirm those policies. Add failure examples for uncertain claims, conflicting edits, cancellation, duplicate model events, missing files and interrupted transactions. Add date-boundary cases using the user's time zone.

### Layered verification

1. **Domain and storage:** Rust unit/property tests cover precedence, task identity and idempotency. Cucumber proves agreed journeys through services and real temporary persistence. Storage tests exercise migrations, restart, reimport and recovery.
2. **Presentation state:** Test loading/streaming/cancelled/error/completed states, reordered events and preserved drafts without pixels. Swift view models consume typed fixtures; test that a stale stream cannot update another conversation. Check the generated FFI interface and run one Swift-to-real-Rust contract test.
3. **UI smoke:** XCUITest launches an isolated fixture app, sends a chat correction, checks visible result, restarts, and migrates a task using the keyboard. Give controls stable accessibility identifiers and assert semantic state. Apple documents [XCTest UI targets](https://developer.apple.com/documentation/xcode/adding-tests-to-your-xcode-project) and [identifier-based automation](https://developer.apple.com/videos/play/wwdc2025/344/). Use a logged-in macOS runner; view-model tests cannot prove focus, selection or menus.
4. **Visual and platform checks:** Capture a few fixed-size light/dark screenshots for chat, topic detail and daily review. Pin OS/font/scale fixtures; review differences rather than automatically blessing new baselines. Exercise menu bar/global shortcut, notification permission denial, Unicode/IME and large-text selection. Screenshots alone cannot verify durable state.

For the Tauri alternative, use Playwright against the frontend in a browser with mocked IPC for rapid DOM, keyboard and screenshot tests. This does **not** prove the WKWebView, Rust IPC or native menus. Current [Tauri WebDriver docs](https://v2.tauri.app/develop/tests/webdriver/) recommend WebdriverIO with `@wdio/tauri-service`: its embedded server supports macOS; direct official `tauri-driver` supports Windows/Linux only. [WebdriverIO configuration](https://webdriver.io/docs/desktop-testing/tauri/configuration) distinguishes the embedded plugin from the official and CrabNebula providers. Pin and prove the embedded route on the target Mac; disable test servers/plugins in distributable builds. Do not assume Playwright attaches to a Tauri WKWebView as if it were Chromium.

## Workspace and agent workflow

```text
Cargo.toml / Cargo.lock / rust-toolchain.toml
crates/
  topics-core/       # IDs, claims, corrections, tasks, application services
  topics-store/      # SQLite migrations, FTS, markdown import/export, watcher
  topics-llm/        # provider adapter, streamed events, schema validation
  topics-shell/      # UniFFI facade; optional CLI for diagnostic journeys
  topics-bdd/        # scenario World, steps, cucumber integration target
apps/macos/          # SwiftUI app, AppKit adapters, view models, Xcode tests
features/            # agreed Gherkin; synthetic fixtures alongside
```

Core defines ports for persistence, clock and model proposals; adapters implement them. No Swift or toolkit types cross into domain code. UniFFI generates [Swift records, errors and mockable protocols](https://mozilla.github.io/uniffi-rs/latest/swift/overview.html). Prefer coarse commands and versioned snapshots/events over exposing mutable Rust objects. Deliver streamed batches on the Swift main actor, with explicit conversation/request IDs and cancellation. UniFFI documents partial Swift 6 concurrency support: prove Sendable/async behaviour with the chosen compiler. [swift-bridge](https://github.com/chinedufn/swift-bridge) is a credible alternative with generated interop and async support, but its type-support matrix makes it a spike candidate rather than the default.

Proposed CI commands, once the workspace exists:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --lib --bins --locked
cargo test -p topics-store --test persistence --locked
cargo test -p topics-bdd --test cucumber --locked -- features
xcodebuild test -project apps/macos/Topics.xcodeproj -scheme Topics \
  -destination 'platform=macOS' -resultBundlePath artifacts/Topics.xcresult
```

Configure the Cucumber target with `harness = false`; use `World::cucumber().fail_on_skipped().run_and_exit(...)` so skipped/undefined steps cannot produce a misleading green build. These APIs are documented by [cucumber-rs](https://docs.rs/cucumber/latest/cucumber/struct.Cucumber.html). Add explicit doctests and any further integration targets as they appear; the commands above are a proposed contract, not executed checks. Pin toolchains and lockfiles. Build both target architectures only if universal distribution is required.

An agent completes a feature by linking the approved scenario revision, showing the initial failure, implementing through core services, running applicable checks and supplying scenario results plus relevant UI evidence. It must test production adapters where the feature touches persistence or FFI. CI uses synthetic notes and temporary app data, never the live notes directory. Live-model evaluations are opt-in, with no confidential corpus uploaded.

## Data boundary, risks and decisions still needed

A markdown repository is enough for source documents and portable exports; it is a poor sole representation of stable task IDs, comments, correction precedence and atomic migrations. Recommend a local SQLite store for structured authoritative task/correction state, with FTS and rebuildable document indexes. Retain source paths, hashes and provenance. Define one authority for each field and make markdown exports explicit; avoid silent bidirectional overwrites. This does not require a separate graph database: people/topic/task relationships fit relational tables initially.

Main risks are the Swift/Rust concurrency and packaging boundary, the cost of polished markdown selection/editing, and an agent confusing mocked green tests with real integration evidence. Keep full WYSIWYG editing out of the first slice unless it is essential. Review toolkit/dependency licences before adopting editor components; a successful host application does not establish that all its internal code is reusable under the same terms.

The decisions for Macs are whether native controls justify a small Swift layer, whether source editing is needed inside Topics, and whether explicit corrections and daily migration use the proposed policies. Authentication/provider choice remains independent of the UI stack; do not make any particular sign-in mechanism a prerequisite for the core or its tests. The technical acceptance gate is a signed macOS slice with durable corrections, identity-preserving task migration and reproducible automated evidence.
