Feature: Build a starting topic map when connecting a notes folder
  Background:
    Given a synthetic notes folder and isolated absolute application data

  Scenario: Connecting a new folder starts local setup without transmission
    Given the folder contains profiles, projects, meetings, logs and unclassified Markdown
    When I connect the folder for the first time
    Then setup shows suggested editable folder roles, document counts, dates and exclusions
    And no source content or folder metadata has been sent to a model
    And all original note bytes are unchanged

  Scenario: Empty and returning folders have useful entry points
    Given I connect an empty folder
    Then I can add notes or capture a topic without using a model
    When I reconnect a folder with an existing saved topic map
    Then its map opens with an option to resume or update setup
    And its accepted knowledge is not imported again

  Scenario: One consent covers an exact unchanged job manifest
    Given I have previewed an eligible selection spanning several analysis batches
    And private and app-owned files are visibly excluded
    When I choose "Build starting map" and authorize the displayed manifest
    Then each batch can read only its capability-bound subset of that manifest
    And the unchanged job does not require consent for every document or batch
    And no model receives local-only bytes, paths, labels or derived private context
    And no durable cloud permission is inferred from that consent

  Scenario: Changed inputs require renewed consent only for the changed scope
    Given a setup job has completed one batch and has another batch waiting
    When a waiting source changes and a new source is added to the folder
    Then the changed source is not sent under its old consent
    And the new source is not silently added to the job
    And the changed selection and exclusions can be previewed for renewed consent
    And unaffected completed draft results remain available

  Scenario: Batches build one sourced graph instead of independent topic lists
    Given synthetic notes repeat "Alex leads Ledger" across several batches
    And one repeated account is a copy of an earlier summary
    When the authorized setup job finishes
    Then its draft has one unambiguously identified Alex, one Ledger and one sourced leads relationship
    And all distinct citations remain inspectable
    And the copied summary is not presented as independent confirmation
    And none of the draft records has been accepted automatically

  Scenario: Ambiguous identities and historic disagreement remain visible
    Given the notes mention two different people named "Joe"
    And historic and recent accounts disagree about who leads Ledger
    When the authorized setup job builds its draft graph
    Then the ambiguous Joe relationship is not attached to either person as a fact
    And the competing leadership accounts retain their dates and citations
    And recency or model confidence alone does not select the accepted account

  Scenario: A topic map is usable as a graph and as a list
    Given a setup job has produced draft topics, explicit relationships and inferred associations
    When I open its map
    Then I can navigate the topics in either graph or list form
    And explicit relationships are distinguishable from inferred associations
    And opening a statement or relationship reveals its underlying source passages
    And a prose topic briefing cannot introduce an uncited factual assertion
