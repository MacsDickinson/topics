Feature: Save and correct understanding without reviewing every record
  Background:
    Given a synthetic draft topic map and isolated absolute application data

  Scenario: Save the eligible starting map with one explicit approval
    Given the draft has sourced unambiguous topics, statements and explicit relationships
    And it also has contradictory, ambiguous, unsupported and inferred items
    When I inspect its exact save receipt and choose "Save starting map"
    Then the eligible displayed set is saved through journalled operations
    And its statements are labelled "From your notes" rather than "Confirmed by you"
    And the remaining items stay open for input
    And no checkbox per document or extracted fact was required
    And no source privacy label or human-authored value authority was upgraded

  Scenario: Saving a topic group preserves deferred material
    Given the draft has eligible groups for Ledger and Harbour
    When I save the Ledger group and choose "Not now" for Harbour
    Then Ledger becomes available in the saved map
    And Harbour remains resumable in the draft rather than rejected
    And neither Harbour nor its facts are falsely shown as reviewed

  Scenario: A group save handles visible dependencies
    Given a Ledger relationship needs a new unambiguous person topic called "Alex"
    And the draft has unrelated claims about Alex
    When I preview saving the Ledger relationship
    Then the receipt visibly includes the required Alex endpoint topic
    And it does not include the unrelated Alex claims
    When the endpoint is ambiguous instead
    Then that relationship is deferred without blocking unrelated Ledger facts

  Scenario: Confirming a section does not confirm unseen topic facts
    Given a saved topic has a displayed section containing three sourced statements
    And other facts exist outside that section
    When I explicitly confirm the enumerated three statements
    Then those exact revisions are labelled "Confirmed by you"
    And the other facts remain "From your notes"
    And the decision retains the statement IDs and evidence that I saw

  Scenario: Correct a fact in its topic without editing storage records
    Given a saved Alex topic reports "he/him" from a transcript
    When I choose "That's wrong" and enter "Alex uses they/them"
    Then I see a proposed before and after with the future rule's exact scope
    When I approve the displayed correction
    Then the topic shows the corrected assertion attributed to my exact statement
    And unrelated facts are not marked confirmed
    And the prior assertion and original transcript remain available as evidence
    And no original source note is rewritten

  Scenario: Several occurrences of a problem create one review question
    Given five notes ambiguously refer to "Joe" in the same proposed relationship
    When I open "Needs your input"
    Then I see one identity question with its five source occurrences
    And I can defer it without blocking unrelated saved topics
    And no uncertain identity is silently merged

  Scenario: One correction can cover a clearly previewed matching set
    Given multiple assertions unambiguously repeat the same wrong pronouns for Alex
    And another person has the same first name
    When I correct Alex and inspect the affected assertions and rule scope
    And I approve that displayed set once
    Then only those unambiguously matching assertions are corrected
    And ambiguous and unrelated people are excluded
    And later analysis cannot revive the suppressed assertion for Alex

  Scenario: New evidence shows a topic diff without overriding confirmed understanding
    Given I confirmed Ledger's launch date as Monday
    When a newly authorized transcript reports Friday
    Then Ledger shows a proposed change with the new citation and prior understanding
    And the confirmed Monday assertion is not silently replaced
    And repeated unchanged evidence does not create another question or duplicate fact
