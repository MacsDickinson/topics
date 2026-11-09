Feature: Resume setup and preserve review decisions safely
  Background:
    Given synthetic notes and isolated absolute application data

  Scenario: Cancellation and restart retain completed draft work
    Given a setup job has completed one batch and has a second model turn running
    When I cancel the job and restart the app
    Then the completed draft batch and my saved draft edits are available
    And the cancelled turn's buffered candidates are absent
    And resuming processes only the still-authorized outstanding inputs
    And no draft has become accepted knowledge without my approval

  Scenario: Failed inputs do not masquerade as complete folder coverage
    Given one authorized batch fails while other batches finish
    When I open the resulting map
    Then successfully analysed topics remain browsable
    And coverage lists the failed, excluded and outstanding files separately
    And the job is not described as having analysed the whole folder
    And I can retry failed inputs without duplicating completed knowledge

  Scenario: A stale receipt cannot save or confirm changed knowledge
    Given I have previewed an exact group save or confirmation receipt
    When a referenced source, dependency or claim revision changes
    Then the old decision fails with an actionable conflict
    And a refreshed receipt shows the changed scope before I can approve it
    And unrelated deferred groups are preserved

  Scenario: Index rebuild preserves imported and personally confirmed distinctions
    Given I saved a starting map and personally corrected one fact
    When the app restarts and its disposable SQLite index is rebuilt
    Then the same topic IDs, graph connections, citations and review decisions return
    And the corrected assertion remains confirmed with its original statement
    And other saved imports remain "From your notes"
    And the setup checkpoint is not lost with the search database

  Scenario: Undo preserves independent later work
    Given I saved a starting-map group and later accepted independent work
    When I request undo of that group
    Then the app checks dependencies and preserves unrelated later work
    And a dependent later decision is surfaced instead of silently removed
    And original note bytes and operation history are preserved

  Scenario: Folder scale does not turn setup into a record checklist
    Given at least one hundred synthetic documents spanning several bounded model batches
    And the documents produce several topics plus some deferred issues
    When the unchanged authorized job builds a map
    Then one transmission authorization covers that exact job manifest
    And one explicit save action can keep its eligible starting map
    And no required per-document or per-fact confirmation blocks using the map
    And recorded coverage and call counts account for every selected document
