Feature: Review knowledge and correct it through topic chat
  Background:
    Given a synthetic notes root with absolute isolated application data

  Scenario: Edit and approve a cited proposal
    Given a pending claim says "Ledger launch is Friday" with a meeting citation
    When I edit the claim to "Ledger launch is Monday" and approve it
    Then the topic shows the accepted Monday claim with its citation and uncertainty
    And the source meeting bytes are unchanged

  Scenario: Reject a proposal
    Given a pending claim says "Ledger is cancelled"
    When I reject the proposal
    Then no accepted claim says "Ledger is cancelled"
    And the proposal is marked rejected

  Scenario: A connection cannot outlive an omitted proposed topic
    Given a proposal contains a new person and a connection to that person
    When I approve only the connection
    Then approval fails with a dependency error
    And neither record is accepted

  Scenario: Correction supersedes the old claim and scopes a future rule
    Given an accepted claim says person "Alex" uses "he/him" with a transcript citation
    And topic chat proposes "they/them" and a rule scoped to Alex's pronouns
    When I approve the correction
    Then Alex's current claim says "they/them" and cites my persisted chat statement
    And the prior claim is superseded but visible in history
    When the same transcript is analyzed again and the app restarts
    Then "he/him" is not restored as current knowledge
    And the rule does not change another person's pronouns
    And the transcript bytes are unchanged

  Scenario: Stale evidence prevents approval
    Given a pending claim cites a passage in "Meetings/one.md"
    When another program changes that passage before I approve it
    Then approval fails with "stale_evidence"
    And the proposed claim is not accepted

  Scenario: Competing corrections are not silently merged
    Given two corrections supersede the same accepted claim revision
    When the first correction is approved and the second is approved from its stale view
    Then the second approval reports a conflict
    And neither correction silently overwrites the other
