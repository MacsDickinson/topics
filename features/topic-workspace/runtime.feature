Feature: Keep model access scoped and failures non-mutating
  Background:
    Given a synthetic notes root with absolute isolated application data

  Scenario: Tool boundary rejects unselected and local-only sources
    Given the run is authorized only for "Meetings/one.md"
    And "Meetings/private.md" is local-only
    When the model requests "Meetings/private.md" or an unselected source by ID
    Then the tool returns no source content or derived private context

  Scenario: Unexpected tools or isolation failure block a real-note run
    Given runtime preflight detects an unexpected tool or failed confinement check
    When I start analysis
    Then the runtime reports an actionable isolation error
    And no selected note bytes are sent

  Scenario: Cancel and late model events cannot create a proposal
    Given a model run has emitted a partial proposed change
    When I cancel the run and a late proposal event arrives
    Then the run ends cancelled and no proposal or accepted record is created

  Scenario: Malformed model output is rejected
    Given a model run returns a proposal with an invalid source locator
    When the run completes
    Then the proposal is rejected with an actionable error
    And no accepted record or source file changes
