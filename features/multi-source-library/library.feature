@proposed @not_approved
Feature: A library accumulates cited knowledge across sources
  Background:
    Given an isolated library with synthetic Markdown sources

  Scenario: Existing knowledge survives adding a source
    Given a single-root installation with a saved Ledger topic and a corrected claim
    When I use that root as the library home and connect a second folder
    Then Ledger retains its identity, correction history and original citations
    And neither source's original notes change
    And app-owned records are excluded from discovery

  Scenario: Two folders contribute different evidence to one decision
    Given Meetings and Engineering each contain a different "ledger.md"
    And Ledger reports a Friday rollout from Meetings
    When I connect Engineering
    Then its local inventory is available before any provider call
    When I authorize its selected version containing a decision to move rollout to Tuesday
    Then Changes shows the proposed Ledger update and both source-qualified citations
    When I save that topic update
    Then the Ledger briefing shows Tuesday and links to the earlier Friday account
    And the other statements on Ledger retain their existing review status

  Scenario: Refresh is incremental and preserves consent
    Given I have saved an update from both sources
    When I refresh unchanged inputs or register the same canonical folder again
    Then no duplicate source, model call or proposed update is created
    When a selected document changes after preview but before analysis
    Then the new bytes require a new preview and authorization

  Scenario: Ambiguity becomes a question the user can defer
    Given two people named Alex are linked to different projects
    When an authorized transcript attributes a conflicting rollout date only to "Alex"
    Then the topic change shows the unresolved attribution and cited conflict
    And deferring the question preserves the current decision and both people's identities

  Scenario: An agent can read a permitted briefing and its changes
    Given an external MCP client has a read grant for Ledger and both allowed sources
    When it requests Ledger after a saved cursor
    Then it receives the same permitted decision change, uncertainty and citations as the desktop
    And the cursor still identifies that change boundary after restart
    And the read grant cannot approve a proposed change

  Scenario: Private evidence cannot affect a client-visible briefing
    Given an external MCP client can read a public Ledger decision
    When I add a local-only note introducing a person and a cross-project concern
    Then the private note stays out of provider analysis
    And the client's briefing, change feed and visible counts reveal neither the note nor its derived connections

  Scenario: Source loss and removal have distinct consequences
    Given a saved decision cites Engineering
    When its folder becomes unavailable
    Then the desktop retains its identity and shows the evidence availability and last refresh
    When I remove Engineering and rebuild the derived index
    Then the desktop retains the decision history with its provenance
    And the removed source is no longer read on refresh
    And the old client grant cannot retrieve its content or derived-only facts
