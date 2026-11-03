# Workspace RED baseline

Scenario / contract revision: f3e1ed0b53a7a7611bbf8ad1b00aa0f521f33e91, before any workspace UI implementation.

`PATH="$PWD/node_modules/.bin:$PATH" npm test -- --workers=1`: five existing Phase 0 tests pass; seven new focused workspace tests fail on missing workspace controls. Full output: workspace-red.txt.

`TOPICS_TEST_WORKSPACE=1 PATH="$PWD/node_modules/.bin:$PATH" npm run test:native`: builds existing real Tauri shell with native-test feature, runs embedded WebDriver against synthetic temporary root and absolute isolated app data. Fails because Analyse notes is absent. Full output: workspace-native-red.txt. No runtime/provider call occurs.

Browser transport fixture is confined to tests/fixtures; native tests do not mock IPC. Tests cover root creation, date selection/exclusions/consent, discovery/review/edited approval/topic/chat/history, stale evidence, cancellation and event sequence isolation, root switching, and missing-runtime manual fallback. Further substantive tests accompany implementation seams once manager supplies canonical manual and source APIs.
