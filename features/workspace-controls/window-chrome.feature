Feature: Integrate native macOS window controls into Topics
  The window matches the app while retaining its native controls and behavior.

  Scenario Outline: The header is integrated in each supported theme
    Given a real Topics native window uses the "<theme>" theme
    When I view the first-run and connected-workspace screens
    Then the header uses the app's surface without a separate default title strip
    And the native close, minimize and full-screen controls remain visible
    And neither a duplicate title nor imitation traffic-light buttons are drawn
    Examples:
      | theme |
      | light |
      | dark  |

  Scenario: Drag blank header space without making controls draggable
    Given a real native Topics window is open
    When I drag the blank header region
    Then the window moves
    When I click a header control or select text in an input
    Then the requested control or selection works
    And that interaction does not drag the window

  Scenario: Retain native window operations
    Given a real native Topics window is open
    When I resize, minimize, restore and enter and leave full screen
    Then those native operations work
    And the layout remains usable
    And the native close control closes the window
    And closing the window does not delete its workspace

  Scenario: Keep controls and focus usable in the narrow window
    Given Topics is at its supported minimum window size
    When I use folder connection, setup and search with the keyboard
    Then native window controls do not cover app controls or content
    And focus indicators and form labels remain visible
    And the production and isolated native-test windows use the same header layout
