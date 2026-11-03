Feature: Recover durable knowledge and preserve concurrent edits
  Background:
    Given a synthetic notes root with absolute isolated application data

  Scenario: Rebuild accepted knowledge from Markdown
    Given an accepted topic, claim, connection and correction rule
    When the derived database is deleted and the app restarts
    Then the same stable IDs, current claim, history and citation are recovered

  Scenario: Conflict preserves external and proposed bytes
    Given a pending update with an expected record hash
    When another program changes that record before approval
    Then approval reports a conflict and preserves the external bytes and proposed bytes
    And no dependent record becomes visible

  Scenario: Recover an interrupted multi-file operation
    Given approval of a topic and dependent claim is interrupted after the first file
    When the app restarts
    Then recovery completes the operation if remaining expected hashes match
    And the topic and claim become visible together

  Scenario: Recovery preserves conflicting changes
    Given approval is interrupted after its first file
    And another program changes a remaining target
    When the app restarts
    Then recovery reports a conflict and preserves old, staged and current versions
    And incomplete records are not visible as accepted

  Scenario: Undo creation and update with dependency checks
    Given a topic was created and a later claim depends on it
    When I undo the topic creation
    Then undo is refused with the dependent claim identified
    When I undo the claim and then undo the topic creation
    Then both are absent from current knowledge and remain visible in history

  Scenario: Switching roots isolates records and pending proposals
    Given root "A" has an accepted topic and pending proposal
    When I switch to root "B"
    Then neither root A's topic nor its proposal appears
    And an old root A approval request is rejected
