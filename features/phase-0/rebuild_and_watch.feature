Feature: Keep the index in step with the files
  Files are the truth. The index can always be rebuilt from them.

  Background:
    Given a notes root with 3 markdown files including "memory/projects/ledger-migration.md"
    And the index is built

  Scenario: Rebuild after the database is deleted
    When the index database is deleted
    And the app starts
    Then the index contains 3 documents
    And searching for "ledger" returns "memory/projects/ledger-migration.md"

  Scenario: An external edit is picked up while the app runs
    Given the app is running and watching the notes root
    When another program appends "Vendor is Northwind" to "memory/projects/ledger-migration.md"
    Then within 2 seconds searching for "Northwind" returns "memory/projects/ledger-migration.md"

  Scenario: A new file is picked up while the app runs
    Given the app is running and watching the notes root
    When another program creates "Meetings/2026-11-02-10-00-standup.md" containing "Northwind call"
    Then within 2 seconds the index contains 4 documents

  Scenario: A renamed file keeps a single index entry
    Given the app is running and watching the notes root
    When another program renames "memory/projects/ledger-migration.md" to "memory/projects/ledger.md"
    Then within 2 seconds the index contains 3 documents
    And searching for "ledger" returns "memory/projects/ledger.md"
    And no result points to "memory/projects/ledger-migration.md"

  Scenario: A deleted file leaves the index
    Given the app is running and watching the notes root
    When another program deletes "memory/projects/ledger-migration.md"
    Then within 2 seconds the index contains 2 documents

  Scenario: Changes made while the app was closed are caught on start
    Given the app is stopped
    When another program appends "Vendor is Northwind" to "memory/projects/ledger-migration.md"
    And the app starts
    Then searching for "Northwind" returns "memory/projects/ledger-migration.md"
