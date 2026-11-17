@proposed @not_approved @follow_up
Feature: Leave a meeting with an actionable follow-up from my own words
  As the person taking meeting notes
  I want a suggested action with the relevant context already attached
  So I can keep the commitment without retyping it or hunting for background

  Scenario: Save once and accept one cited follow-up
    Given Ledger has one known Sam and a cited decision moving rollout to Tuesday
    And analysis of the relevant synthetic inputs is authorized
    When I save "I need to ask Sam about support cover" in today's Ledger meeting note
    Then my exact words are saved and one action draft appears beside the note
    And the draft proposes "Ask Sam about support cover" with Ledger and Sam linked
    And the Tuesday decision appears as cited context and the action has no due date
    When I accept the draft and later reopen and refresh the note
    Then Today and Ledger show the same action ID and its originating note
    And no second action or outstanding draft is created

  Scenario: Enrichment leaves ambiguous details open
    Given two people named Sam appear in the permitted sources
    When I save "I need to ask Sam about support cover" without identifying which Sam
    Then I can accept a suggested action with an unresolved person
    And neither person is assigned the action

  Scenario: Speculation is not a personal commitment
    When I save "Maybe Sam could ask about cover"
    Then the note is available in my Daily Log
    And no personal action draft or accepted action is created

  Scenario: An unavailable analysis cannot lose the note
    Given the analysis provider is unavailable
    When I save "I need to ask Sam about support cover"
    Then my words survive restart in today's Daily Log
    And the entry shows its pending analysis state
    When I retry after analysis is available and authorized
    Then exactly one action draft refers to that commitment
