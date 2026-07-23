<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Test plan: FUSE integration test harness

Companion to [`README.md`](README.md). Requirement IDs are stable within
this file; do not renumber on edit, append instead.

| ID | Behavior | Oracle | Evidence | Status |
| --- | --- | --- | --- | --- |
| HARNESS-01 | `MountGuard` mounts the real binary and `mount_or_decide` returns a live guard whose `path()` serves real fixture bytes | CI log inspection (not just pass/fail color) | `mount_smoke.rs`'s CI run, directly inspected: no skip message, explicit `mounted G1 through FUSE and read README.md successfully` line present | Implemented |
| HARNESS-02 | `Unavailable` is a graceful, printed skip when `WARP_DRIVE_REQUIRE_FUSE` is unset | Manual local run | Confirmed by hand before PR #27 merged: `cargo test --test mount_smoke -- --show-output` with no env var set prints the skip reason and passes | Implemented (manually verified, not automated as a repeatable test-of-the-test) |
| HARNESS-03 | The same `Unavailable` case becomes a hard failure when `WARP_DRIVE_REQUIRE_FUSE=1` is set | Manual local run | Confirmed by hand before PR #27 merged: `WARP_DRIVE_REQUIRE_FUSE=1 cargo test --test mount_smoke -- --show-output` fails with the classification message | Implemented (manually verified, not automated) |
| HARNESS-04 | `Failed` (mkdir/spawn/timeout) is always a hard failure, regardless of `WARP_DRIVE_REQUIRE_FUSE` | — | — | Gap: no test exercises this path at all today — doing so would mean deliberately breaking mount setup (e.g. pointing at a bad binary path) inside a test, which nothing here does yet |
| HARNESS-05 | `Drop` kills the child (not just attempts unmount) when both `fusermount3` and `fusermount` fail, so `wait()` cannot hang | — | — | Gap: no test simulates both unmount commands failing; this is a code-review-verified fix (see PR #27), not a regression-tested one |
| HARNESS-06 | CI (`ubuntu-latest`, `Unit tests` job) genuinely has working FUSE support, not just a passing skip | CI log inspection | Directly inspected twice: once for the initial `mount_smoke` merge (PR #26), again after adding `WARP_DRIVE_REQUIRE_FUSE=1` (PR #27) — both show the positive success line, not a skip | Implemented |

## Coverage note

HARNESS-01 and HARNESS-06 are the two claims this harness's own value
depends on, and both were verified by reading the actual CI log text, not
inferred from a green checkmark or a passing exit code — the whole reason
this topic exists is that a passing test here can otherwise mean three
different things (skip, real success, or a silently-degraded regression).
HARNESS-02/03 were verified by hand locally, once, before merging; nobody
re-runs that verification automatically. HARNESS-04/05 are real,
code-reviewed fixes with no test of their own — regressing either would
currently only be caught by a human reading the `Drop`/`mount_or_decide`
implementation again, not by CI. Worth a follow-up if this harness grows
enough consumers that the untested paths start to matter more.
