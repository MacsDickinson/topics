# Core BDD RED

Feature baseline: `f3e1ed0` (all four topic-workspace feature files unchanged). Types-only prerequisite: `1b8ac16`.

Before any workspace production behavior, `cargo run -p topics-bdd --bin workspace-bdd` invoked Cucumber `fail_on_skipped()` using explicitly named failing step scaffolding, no catch-all. Actual execution failed with 24 failed scenarios, 24 passed background steps and 24 failed setup steps; no skipped or undefined steps. The scaffolding deliberately fails at each missing behavior and will be replaced by service assertions, with runtime/UI-only ownership mapped separately.

```text
  Scenario: Choose a fresh root from the desktop
  Scenario: Preview a bounded selection before analysis
  Scenario: Explicit confirmation allows unknown but never local-only notes
  Scenario: Changed selection must be previewed again
  Scenario: Discovery yields cited proposals without accepting knowledge
  Scenario: Same-name people stay unresolved
  Scenario: Repeated analysis does not duplicate accepted facts
  Scenario: Missing Codex is actionable
  Scenario: Rebuild accepted knowledge from Markdown
  Scenario: Conflict preserves external and proposed bytes
  Scenario: Recover an interrupted multi-file operation
  Scenario: Recovery preserves conflicting changes
  Scenario: Undo creation and update with dependency checks
  Scenario: Switching roots isolates records and pending proposals
  Scenario: Edit and approve a cited proposal
  Scenario: Reject a proposal
  Scenario: A connection cannot outlive an omitted proposed topic
  Scenario: Correction supersedes the old claim and scopes a future rule
  Scenario: Stale evidence prevents approval
  Scenario: Competing corrections are not silently merged
  Scenario: Tool boundary rejects unselected and local-only sources
  Scenario: Unexpected tools or isolation failure block a real-note run
  Scenario: Cancel and late model events cannot create a proposal
  Scenario: Malformed model output is rejected
4 features
24 scenarios (24 failed)
48 steps (24 passed, 24 failed)
24 steps failed; process exit 101
```

All worlds allocate a synthetic temporary directory; no live notes, config, auth or providers were accessed.
