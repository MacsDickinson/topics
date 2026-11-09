Feature: Launch an isolated fresh session for testing
  A fresh session can demonstrate onboarding without resetting normal data.

  Scenario: Launch a fresh session alongside the normal instance
    Given my normal test instance is connected to a synthetic folder
    When I run the fresh-session launcher
    Then a separate instance opens at the unconnected first-run screen
    And it uses new absolute app-data and synthetic sample-note directories
    And the launcher identifies the sample folder I can connect
    And the normal instance's configuration and note hashes are unchanged
    And no model run starts automatically

  Scenario: Repeated launches do not reuse initialized demo state
    Given I initialized one fresh sample session
    When I run the fresh-session launcher again
    Then the new instance has different app-data and sample-note directories
    And it starts without a connected folder or prebuilt topic records
    And the earlier instance and its data remain intact

  Scenario: Fresh app data does not erase an existing folder's Markdown knowledge
    Given a synthetic notes folder already contains accepted topic records
    And a fresh unconnected session is open
    When I explicitly connect that existing folder
    Then its saved knowledge is restored
    And the folder is not treated as an empty knowledge store
    And the generated sample folder remains available for new-onboarding tests
