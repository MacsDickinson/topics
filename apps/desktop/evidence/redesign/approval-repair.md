# Final 64 KiB integration and P1 approval repair

Implementation is ready for independent re-review; **acceptance is not claimed**.

## Integration and repair commits

- Authoritative backend `52a066a6cfdbe920a207907a034a1ccdd55a7d39` remains merged
  through `37eb789bc53816618c313b712f26300f708c489a`; its original RED/GREEN job
  commits were not replayed.
- Final authorized UI budget `0f8cf7b3b476cefba949d5d8574adec866cb3e3e` was
  cherry-picked as `e48e2675aa42af12e02cbd1e966083a964cb0d35` without a conflict.
  The actual request now uses `max_bytes_per_batch: 65536`, and the manifest
  discloses **64 KiB of notes per batch**, with space reserved for citations and
  context. The imported request regression verifies the value, rather than only
  checking the copy.
- P1 repair `3a71a4d1ad4f2e5018a42bd62f51648aa372ece9` intentionally changes
  ContextReview's state/effect behavior from canonical `41ed1c0`; no backend,
  native adapter, generated wire type or request shape was changed by the repair.
- The earlier exact-main merge, harness and canonical UI equivalences remain in
  [canonical-integration.md](canonical-integration.md).

## P1 cause and resolution

The independent reviewer reproduced correction → connection → Approve correction
at frozen `f9705c9`: independent `receipt` and `correction` state allowed a new
connection receipt to be submitted beneath the old correction preview/CTA. The
same mismatch applied to statement confirmation. Prior AST parity with canonical
UI proved inheritance, not safety.

ContextReview now holds one discriminated `Approval`: correction plus its own
preview/receipt, statement receipt, or connection receipt. The visible heading,
records, CTA and submitted receipt id/fingerprint all derive from that same value.
Changing review kind clears the previous approval and closes its composer.
Editing a correction or cancelling clears its approval and invalidates the active
preview request; a new preview is required to approve again.

Preview completions and errors require both the current root/topic generation and
current request sequence. A response from an edited, cancelled, replaced or
unmounted intent cannot restore an old approval or alter the newer request's busy
state. Submitted human mutations use a ref-backed lock and disabled intent/field/
cancel controls until settling; the UI does not imply that cancelling can revoke
an already submitted decision. A failed/partial/conflicting decision clears its
approval and requires a fresh preview. These state rules are documented with the
shared design patterns in the same change.

## Verification

Pinned local **Node 24.22.2**, strict isolated browser port **14337**, absolute
synthetic `TOPICS_APP_DATA_DIR=/tmp/topics-boardui-redesign-browser-0cd733`:

- **112 unfiltered browser tests passed**; see
  [final-full-browser.txt](final-full-browser.txt). All accepted 83-check scope,
  later canonical cases, retained contrast/async/focus coverage and the new budget
  regression remain present.
- **TypeScript and production Vite build passed**; see [final-build.txt](final-build.txt).
- Thirteen new P1 regression cases in `tests/context-approval.spec.ts` check the
  visible preview/CTA **and actual outgoing receipt id and fingerprint**, with
  distinct mocked identifiers/fingerprints per response. They cover correction ↔
  connection and correction ↔ statements, editing displayed and in-flight
  corrections, cancellation and late responses, root/topic changes, mutation
  locking, plus light/dark/narrow switch sequences.
- Decision fixtures deliberately return a named conflict after recording the
  request; these tests verify which record the user authorized and sent without
  pretending to prove persisted native decisions. The targeted first ten cases
  passed in [approval-regression.txt](approval-regression.txt); the final full run
  includes all thirteen.
- No personal notes/symlink access, real model call, user-app restart, runtime
  repin, competing native suite, main edit or remote publication occurred.

## Visual evidence and limits

All nine [approval screenshots](approval/) were visually inspected, including the
three [approval contact sheets](approval-light-contact.png),
[dark](approval-dark-contact.png) and [narrow](approval-narrow-contact.png).
Each appearance shows connection confirmation after a correction, correction
after connection confirmation, then statement confirmation: only its matching
preview and approval action remain. Original full-size PNGs use
`{light,dark,narrow}-{connection,correction,statements}.png`.

The full suite also refreshed 39 `after/` and 15 `canonical/` images and their
comparison sheets; the final 64 KiB manifest was inspected, including its narrow
layout. The 21 original baseline images remain unchanged. These are synthetic
browser captures, not real WebView/VoiceOver/native or genuine-runtime acceptance.

The feature team owns native/package/genuine acceptance and supplied its 64 KiB
native journey evidence in the authorized commit. This worker did not repeat
those suites or change the current user executable; the retained pinned 0.166.0
path remains documented for coordinated use. The original reviewer must recheck
P1 and the final budget delta before acceptance is asserted.
