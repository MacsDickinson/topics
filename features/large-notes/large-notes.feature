@proposed @not_approved
Feature: Analyse a long selected Markdown source without widening disclosure
  All source files, app data, provider responses and identities in these scenarios are synthetic.
  Chunk completion means successful bounded delivery and guarded staging, not verified truth.

  Scenario: Review the exact long-source plan and consent once
    Given a selected 171418-byte UTF-8 Markdown source with headings and a local-only sibling
    When I preview the selected manifest
    Then the preview shows the selected source, bytes, chunk count, finite limits and exclusions
    And its fingerprint binds the complete source hash, every exact range and the chunk algorithm
    When I consent to that exact manifest
    Then only its unchanged approved chunks may be transmitted
    And the local-only sibling's body, path, heading and identity are absent from all model payloads

  Scenario: A chunk is an exact projection of the original
    Given a selected source containing CRLF, emoji, combining marks, a long line and a fenced heading-like line
    When each planned chunk is read through its capability
    Then primary ranges partition every original byte exactly once and overlap is bounded
    And each transmitted range starts and ends on UTF-8 boundaries and equals the original bytes
    And each cited original absolute range, excerpt, passage hash and document hash validate
    And a non-null heading passes the existing original-source heading check
    And forged headings, altered hashes and citations outside the allowed ranges are rejected

  Scenario: One chunk cannot become a read capability for its parent
    Given a long selected source whose later chunk contains a distinctive secret filename and text
    And the current capability authorizes only the first chunk and its allowed overlap
    When the model asks for scope or proposes an out-of-range source endpoint or citation
    Then the response contains no later text, heading, filename, path or identifying hint
    And the out-of-range proposal is rejected even when its full-document hash is correct
    And no out-of-window passage may be returned or newly cited

  Scenario: Progress waits for all primary chunks
    Given a selected source with three chunks and a second selected small source
    When the first and third chunks and the small source finish successfully
    And the second chunk fails
    Then chunk coverage shows two of three complete and the failure
    And file coverage marks the long source failed or outstanding, not processed
    And the job reports gaps and does not claim selected notes analysed
    When I retry the outstanding chunk successfully within the approved budget
    Then the long source is processed exactly once and selected coverage can complete

  Scenario: Cancel, restart and rebuild preserve committed work only
    Given one committed chunk and a second chunk with buffered proposals
    When cancellation wins the publication fence and the app restarts
    And the disposable SQLite index is deleted and rebuilt
    Then the committed chunk, user edits and original citations remain
    And the cancelled buffer is absent and its chunk remains outstanding
    And resume transmits only outstanding approved chunks

  Scenario: Changed source requires new exact consent
    Given one completed chunk of a selected source and an unchanged completed other source
    When the selected source changes before the next chunk is sent
    Then the next send and stale publication are blocked
    And old and new versions are not combined as completed coverage
    And the unchanged source remains complete and human edits are preserved
    When I review and consent to a new exact manifest for the changed outstanding source
    Then only chunks authorized by the new consent can run
    And citations to the old bytes remain stale or reviewable, never silently rewritten

  Scenario: Overlap and copies do not manufacture corroboration
    Given the same assertion spans two overlapping chunks of one source
    And a different source copies the same assertion verbatim
    When the job stages both chunks and the copied source
    Then the assertion is not duplicated due to the overlap
    And citations retain distinct original source provenance
    And the overlap and copied account are not represented as independent confirmation

  Scenario: Corrections and ambiguous identities survive consolidation
    Given an eligible earlier correction and two different people with the same name
    When later chunks repeat a suppressed assertion and mention both people
    Then the correction remains effective and the suppressed assertion is not revived
    And the people are not merged by name or by generic date or source-type hint
    And unsupported identity or conflicting claims remain issues with their own evidence

  Scenario: Private context is not an identity oracle
    Given a private correction, private topic and unselected note sharing a name with a selected chunk
    When scoped accepted and draft context is prepared for the chunk
    Then no private or unselected label, path, heading, excerpt, identity or human correction is returned
    And no inference or identity decision is created from those hidden records

  Scenario: Mandatory context and wire bounds fail visibly
    Given an eligible selected chunk and accepted correction context too large for the actual host response envelope
    When the host prepares the response including candidate schema and escaped SDK and direct envelopes
    Then no oversized response is transmitted and no context or correction is silently dropped
    And the job reports an actionable bounded-context error with no completed chunk
    And optional draft context may be omitted only as whole dependency-consistent records with a partial notice

  Scenario: Size, turn and candidate exhaustion do not fake coverage
    Given a source over the approved document or chunk-count limit, or a turn reaches the candidate cap
    When setup cannot complete all required work within the reviewed limits
    Then it identifies the affected file and reason and offers an actionable selection or new-preview path
    And it does not truncate, silently increase limits, or mark that file complete

  Scenario: Approval, undo and source preservation remain intact
    Given a completed chunked draft with exact evidence and a later independent accepted change
    When I save an eligible group, restart, rebuild SQLite and undo the saved group
    Then journal receipts and dependency checks preserve the later independent change
    And source Markdown bytes are unchanged throughout
    And no model result is accepted or human-confirmed without its explicit decision
