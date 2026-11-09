# Pinned runtime discovery validation

The synthetic regression creates both a pinned retained release and the ordinary local executable as files under a temporary home. Before the fix, the focused test failed because resolution returned `.local/bin/codex` instead of `.codex/packages/standalone/releases/0.166.0-aarch64-apple-darwin/bin/codex`; the failure was captured in `/tmp/topics-runtime-discovery-red.log`.

After the fix, `cargo test -p topics-desktop` passed all 13 tests, including the temporary-file regression and recorded metadata-probe sequences for both supported macOS architectures, unsupported platforms, overrides and fallbacks. `cargo fmt --all -- --check` and `cargo clippy -p topics-desktop --all-targets -- -D warnings` passed; Rust commands used the isolated `/tmp/topics-setup-desktop-target` and disabled incremental compilation and debug information.

These tests did not launch Codex or inspect personal configuration, credentials or notes. The release package still requires the coordinator's independent focused review and final integration checks.
