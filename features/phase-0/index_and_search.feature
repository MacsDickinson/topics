Feature: Index a notes folder read-only and search it
  The index is derived from the files. It never changes them.

  Background:
    Given a notes root with these files:
      | path                                         | content                                              |
      | memory/people/alex-example.md                | # Alex Example\n- **Known as:** Alex\n- **Pronouns:** they/them\n- **Role:** Engineering Manager\n- **Team:** Payments |
      | memory/glossary.md                           | ## Nicknames\n\| Nickname \| Person \|\n\| AJ \| Alex Example \| |
      | memory/projects/ledger-migration.md          | # Ledger migration\nMove accounts to the new ledger. |
      | Meetings/2026.10/2026-10-28-16-05-alex-121.md | # Alex 121\nWe discussed the ledger migration cutover. |
      | Log/2026.10/2026-10-26.md                    | # Actions — week of 26 October 2026\n- Chase ledger vendor |

  Scenario: Index classifies documents by folder
    When the index is built
    Then document "memory/people/alex-example.md" has kind "person"
    And document "memory/projects/ledger-migration.md" has kind "project"
    And document "Meetings/2026.10/2026-10-28-16-05-alex-121.md" has kind "meeting" dated "2026-10-28"
    And document "Log/2026.10/2026-10-26.md" has kind "log" dated "2026-10-26"

  Scenario: People are parsed from profiles with aliases
    When the index is built
    Then there is a person "Alex Example" known as "Alex" with pronouns "they/them"
    And "Alex Example" has the alias "AJ"
    And "Alex Example" has role "Engineering Manager" in team "Payments"

  Scenario: Full-text search finds matches across document kinds
    When the index is built
    And I search for "ledger"
    Then the results include "memory/projects/ledger-migration.md"
    And the results include "Meetings/2026.10/2026-10-28-16-05-alex-121.md"
    And the results include "Log/2026.10/2026-10-26.md"
    And each result shows a snippet with "ledger" highlighted

  Scenario: Indexing never modifies the notes folder
    When the index is built
    Then every file in the notes root has the same bytes and modified time as before

  Scenario: A file that cannot be parsed does not stop the index
    Given the file "memory/people/broken.md" contains invalid UTF-8
    When the index is built
    Then the index contains 6 documents
    And there is 1 warning naming "memory/people/broken.md"
