@proposed @not_approved
Feature: See what changed across my sources before my next meeting
  As someone preparing to meet the Ledger team
  I want to spot changed decisions and trace their evidence
  So I can act on current information without rereading all my notes
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
    Then Today shows the proposed Ledger update and both source-qualified citations
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
    Then Today reports a conflict and the topic change shows the unresolved attribution and cited conflict
    And deferring the question preserves the current decision and both people's identities

  Scenario: A busy import gives me two useful updates
    Given twelve notes repeat the saved Ledger date without adding evidence of a change
    And a decision moves Ledger from Friday to Tuesday for the support handover
    And two new accounts disagree about the Harbor launch
    When the authorized import finishes and I open Today
    Then I see exactly one Ledger update showing Friday to Tuesday and why
    And I see exactly one Harbor question labelled as a disagreement
    When I open the Ledger update
    Then I can reveal both dated sources on its topic page
    And its New indicator clears on Today and Ledger without approving anything
    When I save the Ledger update, defer Harbor, and restart Topics
    Then Today shows no new changes in the analysed sources and Harbor is available under For later
    And refreshing the same material does not raise either update again

  Scenario: Find a quiet topic when I need the answer again
    Given Today has no new updates and Ledger has the alias "Cutover"
    When I search for "Cutover" in Topics and open Ledger
    Then I see the saved Tuesday decision, its rationale and the cited Engineering passage
    And I can follow the earlier Friday account in its history

  Scenario: One consent covers the reviewed inputs and respects privacy
    Given Engineering contains ten eligible notes and one previously marked local-only
    When I connect it and authorize the displayed eligible selection once
    Then the analysis can proceed without requiring any per-file access decisions
    And the provider receives no content, title or path from the local-only note
    When I save a proposed topic group containing three sourced statements
    Then one approval saves the displayed group with its citations
    And those statements keep their sourced status

  Scenario: Source loss and removal have distinct consequences
    Given a saved decision cites Engineering
    When its folder becomes unavailable
    Then the desktop retains its identity and shows the evidence availability and last refresh
    When I remove Engineering and rebuild the derived index
    Then the desktop retains the decision history with its provenance
    And the removed source is no longer read on refresh
    And its previous analysis authorization cannot retrieve the removed evidence
