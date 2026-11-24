@ui
Feature: Search from the desktop window
  Proves the Tauri shell talks to the Rust core.

  Scenario: Search shows results from the core
    Given the desktop app is open on a synthetic notes root
    When I focus the raw note search box
    And I type "ledger"
    Then I see a result titled "Ledger migration"
    And the result shows its path "memory/projects/ledger-migration.md"

  Scenario: Empty results say so
    Given the desktop app is open on a synthetic notes root
    When I search for "zzzz-no-match"
    Then I see "No matches"
