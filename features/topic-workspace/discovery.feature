Feature: Select notes and discover reviewable knowledge
  Background:
    Given a synthetic notes root with absolute isolated application data

  Scenario: Choose a fresh root from the desktop
    Given no notes root is configured
    When I create an empty notes root from the desktop
    Then the root is selected and an empty-state explains how to add notes

  Scenario: Preview a bounded selection before analysis
    Given dated notes "Meetings/one.md" on "2026-11-01" and "Meetings/two.md" on "2026-11-03"
    And an undated note "Meetings/undated.md"
    When I preview folder "Meetings" for dates "2026-11-01" through "2026-11-02"
    Then the preview includes "Meetings/one.md" and excludes "Meetings/two.md" and "Meetings/undated.md"
    And no note bytes have been sent to a model

  Scenario: Explicit confirmation allows unknown but never local-only notes
    Given "Meetings/one.md" is unknown and "Meetings/private.md" is local-only
    When I preview both files and confirm unknown sources for this run
    Then the model can receive "Meetings/one.md"
    And all model tools deny "Meetings/private.md" and its derived topic context

  Scenario: Changed selection must be previewed again
    Given I preview "Meetings/one.md"
    When another program changes "Meetings/one.md" before analysis starts
    Then analysis fails with "stale_scope"
    And no note bytes are sent to a model

  Scenario: Discovery yields cited proposals without accepting knowledge
    Given a selected synthetic meeting says "Alex leads Ledger"
    And the scripted model proposes person "Alex", project "Ledger" and a cited "leads" connection
    When analysis completes successfully
    Then the proposals show the passage and its source locator
    And no topic, claim or connection is accepted
    And the meeting bytes are unchanged

  Scenario: Same-name people stay unresolved
    Given two accepted people named "Joe" have different IDs
    And a selected meeting mentions "Joe" without distinguishing them
    When the scripted model proposes an identity for that mention
    Then the review shows both candidates as unresolved
    And no connection to either person is accepted

  Scenario: Repeated analysis does not duplicate accepted facts
    Given a cited claim from "Meetings/one.md" was accepted
    When the same unchanged meeting produces the same claim in a later analysis
    Then the existing decision is shown as a duplicate
    And there is still one accepted claim

  Scenario: Missing Codex is actionable
    Given Codex is unavailable
    When I open the workspace
    Then I see how to install or configure the required runtime
    And I can browse accepted topics and create a manual proposal
