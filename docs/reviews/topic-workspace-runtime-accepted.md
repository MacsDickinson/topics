# Targeted confirmation: R2b Codex dotenv guard (source 8ae44f9, evidence 2f5268d)

Reviewer: Claude Opus 5.5, read-only. No repo edits, no provider calls, no real notes, and no real .env/auth/config contents read.

## Verdict: ACCEPT

There is no remaining R2b counterexample. R1 and R3-R8 were not reopened.

## Guard (environment.rs:43-62)
- `apply_from` validates HOME and CODEX_HOME, then calls `symlink_metadata` on the effective home's `.env` (explicit canonical CODEX_HOME, else canonical HOME/.codex).
- Only `ErrorKind::NotFound` permits launch. Every other result returns an error before any child environment is applied.
- Every production spawn goes through it: `check_version` (lib.rs:746), `command()` for discovery and the run (lib.rs:325, called at 275 and 291), and `bounded_probe` (isolation.rs:34). Each caller propagates the error before spawning. `check_version` is the first step in `run` (lib.rs:253).
- 1430eba..8ae44f9 changes only environment.rs in src/tests/manifest. The module is private, so tool, catalog and public API surfaces are unchanged.

## Checks actually run
1. **Source identity.** I recomputed the source-scope digest from git objects. It is 707dc53f... at both 8ae44f9 and 2f5268d, matching the chronology.
2. **Chronology.** Latest source mtime 07:02:50, source commit 07:04:30, executables built 07:04:45 (run 07:04:42-07:05:48), evidence commit 07:06:45.
3. **Executables.** The three recorded SHA256/mtime pairs match files still in target/debug/deps. They were not re-executed.
4. **Evidence-only follow-up.** 8ae44f9..2f5268d touches only evidence markdown and new dotenv-final-* files. The rereview-final-* logs are unchanged since 09b484e.
5. **Deterministic tests.** 49/49 pass (26+5+2+1+4+11), run with `env -i` and a throwaway HOME. fmt and all-target/all-feature clippy with -D warnings are clean.
6. **Guard matrix.** 37 cases through the real `Runtime::status`, linked against the crate rlib, with a fake codex that logs each spawn. All behaved as expected:
   - Rejected with no spawn: file, directory, live and dangling symlink, FIFO (no hang, so not opened), socket, mode-000 file, empty file, `.ENV` on the case-insensitive filesystem, explicit-home variants, ENOTDIR, ELOOP, EACCES on .codex and on explicit CODEX_HOME.
   - Rejected through noncanonical paths: symlinked HOME, symlinked CODEX_HOME, `..` paths, trailing slash, .codex symlink to a directory holding .env.
   - Accepted: absent default and explicit homes, no .codex directory (not created), .codex symlink to a clean directory, dangling .codex.
   - Other fail-closed launch inputs: empty, relative or missing CODEX_HOME, and empty, relative or unset HOME, all with their existing non-dotenv errors.
   - Separate check: a rejected .env keeps its atime, mtime and size, and no entry is added.
7. **Offline repro on installed Codex 0.166.0.** Plain `codex app-server` (not the production argv), throwaway homes, no auth, network denied except loopback, real-home .env/auth/config denied by literal path. The dotenv held only `RUST_LOG=trace`, and stderr bytes were the signal.

| Scenario | Guard | Codex stderr bytes |
|---|---|---|
| no dotenv; explicit clean home | accept | 0 |
| HOME/.codex/.env | reject | ~18K (loaded) |
| CODEX_HOME/.env | reject | ~18K (loaded) |
| .codex symlink to directory with .env | reject | ~18K (loaded) |
| live symlink .env | reject | ~18K (loaded) |
| HOME/.codex/.env + explicit clean CODEX_HOME | accept | 0 (not read) |
| HOME/.env only | accept | 0 (not read) |
| cwd/.env only | accept | 0 (not read) |
| dangling .codex symlink | accept | 487, no dotenv present |

No case had the guard accept while a dotenv loaded. In the dangling case the bytes are Codex's own sqlite-init failure: it exits with no init reply, so this is not a bypass. My script's `loaded/bypass=True` for that row was a false positive of the stderr-bytes heuristic. This run also closes the prior report's "not tested" item: Codex does not read HOME/.codex/.env when CODEX_HOME points elsewhere.

## Not verified independently
- Native and live results are from the implementer's logs. Per the brief I did not rerun them.
- "One live attempt, no retries" is self-reported. Git cannot prove it, though the harness hashes and logs are consistent with it.
- The actual runtime home's dotenv absence is the manager's reported metadata check. I did not check it. The evidence live run passing through the guard is consistent with absence at that time.

## Residual limits (no action required for R2b)
- Same-user replacement after the metadata check is the documented threat-model limit.
- INFO: `dotenv-remediation` wording is generic for metadata errors. EACCES or ENOTDIR on .codex says "does not support Codex dotenv configuration". This fails closed and has no security effect.
- INFO, test-only: scripted and acceptance tests that reach the guard use ambient process HOME, so on a developer machine with a real ~/.codex/.env they would fail closed. The guard unit tests inject HOME. There is no bypass, and CI or local runs with that file would see a clear failure.
