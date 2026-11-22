# Native workspace selector reconciliation

The assembled-main native run failed at workspace.ts:39: `select[aria-label="Source sensitivity"]` was not displayed after 10000ms (/tmp/topics-folder-setup-remaining-native.log). The redesigned Field associates the visible Source sensitivity label with `select#source-sensitivity`; the focused browser fixture checks that this exact selector has the accessible name and persists the local-only choice.

The native workspace journey retains source-content and source-byte checks, the local-only privacy assertion, approval, and dependent undo protection. The opt-in live-workspace selectors were checked against current components, including the folder-setup entry screen, topic sensitivity, proposal badges, and explicitly opened correction provenance; the provider-backed flow was not run.

The real native setup suite uses synthetic notes and the scripted local child, with no live provider calls. Full command output is in the adjacent reconciliation evidence files; the source snapshot guards remain enabled.
