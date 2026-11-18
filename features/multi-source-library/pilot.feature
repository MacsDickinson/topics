@proposed @not_approved
Feature: Learn from corrections I actually make in the Friday sync
  As the person preparing for the meeting
  I want to capture a missed or wrong update in a sentence
  So the next briefing can improve and I can tell whether it saved me work

  Background:
    Given a synthetic project and isolated application data

  Scenario Outline: Today tells me how current the briefing is before I rely on it
    When I open Today with zero unread items
    Then a local refresh starts without a provider call
    When the selected inputs are <state>
    Then Today displays <message> with source and last-success time or "never"
    Examples:
      | state                         | message                            |
      | fully checked and analysed    | no new changes in analysed sources |
      | changed since consent         | awaiting batch authorization       |
      | unreachable                   | source unavailable                 |
      | partially processed or failed | analysis incomplete                |
      | skipped by scope              | excluded from this briefing        |

  Scenario: Friday reveals one catchable omission and one genuinely new decision
    Given the Thursday briefing surfaced two material changes correctly
    And a third decision had evidence in the analysed source version but was absent
    When I report the third decision and a fourth first made during Friday's meeting
    And I later refresh with Friday's minutes and reopen weekly review
    Then Thursday's briefing still contains its original two changes
    And known in-scope omissions are one out of three
    And Friday's new decision appears separately
    And I can report another omission even when no briefing card exists

  Scenario: Changing the wording of one correction does not inflate the error count
    Given I report the wrong owner on one displayed item
    When I edit the correction twice and restart Topics
    Then weekly review counts one wrong item
    And the correction is attributed to me with a reference to the original briefing
    And the corrected fact requires my approval before it becomes accepted knowledge

  Scenario: A scoped correction is evaluated on a later occurrence
    Given I approved the Ledger support handover correction identifying Sam Patel
    When another Ledger handover repeats the earlier misidentification
    Then the proposed briefing respects the correction and retains the new citation
    And the weekly tally shows one eligible later occurrence and zero recurrences
    And an unrelated project's Sam is unaffected
    When I undo the correction
    Then future briefings stop applying the rule and the prior briefing stays auditable

  Scenario: Weekly review distinguishes poor capture from missing measurements
    Given action drafts are enabled and four drafts appeared
    When I accept one unchanged, fix the due date on one, dismiss one as someone else's, and leave one unread
    And I add a missed personal commitment during Friday's sync
    Then the tally shows one unchanged, one edited, one wrong, one undecided and one missed commitment
    And a week with draft capture disabled shows draft quality as unavailable
