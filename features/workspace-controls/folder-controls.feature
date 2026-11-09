Feature: Disconnect and reconnect a notes folder
  Disconnecting changes the selected workspace without deleting its knowledge.

  Scenario: Disconnect preserves notes and remains disconnected after restart
    Given an isolated workspace with synthetic notes and saved topics, corrections and context
    And I open its folder settings
    When I disconnect the folder
    Then I see "Start with your notes"
    And no previous source content, topics or search results remain visible
    And every source and app-owned Markdown file is byte-for-byte unchanged
    And snapshots and undo history are retained
    When I restart Topics with the same app data
    Then no notes folder is selected

  Scenario: Disconnect revokes in-flight setup and late results
    Given a synthetic folder has an active setup or chat run
    And a source read or proposal publication is waiting at its guard
    When I successfully disconnect the folder
    Then the runtime and setup work are stopped or revoked
    And the old watcher and progress subscription stop
    And old source reads and publication attempts are denied
    And late events cannot restore the folder, a preview or its data

  Scenario: Failure to stop active work does not pretend to disconnect
    Given a synthetic folder has active work that cannot yet be safely stopped
    When I attempt to disconnect the folder
    Then I see an actionable error
    And the folder is not shown as successfully disconnected
    And I can retry after the stopping failure is resolved

  Scenario: Reconnect restores the same saved knowledge without another model run
    Given I disconnected a synthetic folder containing saved knowledge
    When I reconnect that folder
    Then its original topic and claim revision identifiers are restored
    And its correction rules, context and history remain available
    And no model run or source transmission starts automatically
    And I can explicitly open folder setup again

  Scenario: Connect another folder without leaking the previous workspace
    Given I disconnected one synthetic folder
    When I connect a different synthetic folder
    Then only the new folder's sources and knowledge are available
    And searches and evidence reads cannot return the old folder's content
    And decisions using an old preview or root capability are rejected

  Scenario: Disconnect a folder that is no longer available
    Given the configured synthetic notes folder has become unavailable
    When I disconnect it from the recovery or folder settings screen
    Then I return to the unconnected first-run screen
    And the unavailable folder is not read, repaired or deleted
    And I can connect an accessible folder
