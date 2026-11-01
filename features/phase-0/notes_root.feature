Feature: Choose or create a notes folder
  The app works on top of any notes folder, or sets one up on a fresh install.

  Scenario: Use an existing notes folder
    Given a folder "work-notes" containing 3 markdown files
    When I set the notes root to "work-notes"
    Then the notes root setting is "work-notes"
    And the index contains 3 documents
    And no file in "work-notes" has changed

  Scenario: Create a fresh notes folder
    Given the folder "new-notes" does not exist
    When I initialise a notes root at "new-notes"
    Then "new-notes" contains the folders "memory/people", "memory/projects", "Meetings", "Log" and "topics"
    And the notes root setting is "new-notes"

  Scenario: Refuse to initialise over an existing non-empty folder
    Given a folder "work-notes" containing 3 markdown files
    When I initialise a notes root at "work-notes"
    Then I am told the folder is not empty and to use it instead
    And no file in "work-notes" has changed

  Scenario: Switch to a different notes folder
    Given the notes root is "work-notes" with 3 markdown files
    And a folder "home-notes" containing 2 markdown files
    When I set the notes root to "home-notes"
    Then the index contains 2 documents
    And no document from "work-notes" appears in search results
