<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# The FUSE integration test harness

**Status:** living reference, current as of `main`.
**Established by:** PR [#26](https://github.com/flyingrobots/warp-drive/pull/26)
(G4a prep, added `MountGuard`), hardened by PR
[#27](https://github.com/flyingrobots/warp-drive/pull/27) (the
`Unavailable`/`Failed` split and `WARP_DRIVE_REQUIRE_FUSE`) — not tied to a
numbered gate; this is test infrastructure, not a gate-acceptance proof.
**Test plan:** [`test-plan.md`](test-plan.md).

**This is not what `docs/TESTING.md` §5/§6 originally planned.** That
document (written at G1 time) sketched a `TempDir`-based `MountGuard::mount(tree:
FixtureTree)` plus a dedicated `warp-drive-fixtures`/`warp-drive-test-harness`
crate pair with `assert_file`/`assert_dir`/`assert_symlink`/`assert_readonly`/
`assert_inode` helpers. None of that got built. What actually exists is
smaller, lives inline in `crates/warp-drive-fuse/tests/`, and is scoped
tightly to what G4a's write-path tests actually need — see "Why this
diverges from the original plan" below.

## What exists

`crates/warp-drive-fuse/tests/support/mod.rs` — `MountGuard`, a subprocess-based
mount lifecycle helper:

- `MountGuard::mount_in_memory(gate: &str)` spawns the real, compiled
  `warp-drive-fuse` binary (via `env!("CARGO_BIN_EXE_warp-drive-fuse")`, not
  `cargo run`) against a fresh temp directory with the given `--gate` value,
  polls `/proc/mounts` for a `fuse`-family entry at that exact path, and
  returns once it appears (or times out at 5s).
- `Drop` unmounts (`fusermount3 -u`, falling back to `fusermount -u`), kills
  the child if both unmount attempts failed (so a stuck child can't make
  `wait()` hang forever), reaps it, and removes the temp directory.
- `guard.path()` gives the live mount point for opening files under it.

`crates/warp-drive-fuse/tests/mount_smoke.rs` is the one test using it today
— a smoke test proving the harness itself against the existing G1 read-only
surface.

## The `Unavailable` / `Failed` distinction

`mount_or_decide(gate: &str) -> Result<MountGuard, MountOutcome>` is the
entry point tests should call — not `MountGuard::mount_in_memory` directly.
It classifies every way a mount attempt can not produce a live mount into
exactly two buckets, and callers MUST NOT collapse them:

| Kind | Meaning | Behavior |
| --- | --- | --- |
| `Unavailable` | `/dev/fuse` genuinely isn't present | Skippable — but only when `WARP_DRIVE_REQUIRE_FUSE` is unset |
| `Failed` | mount-point creation, spawning the binary, or the mount never appearing within the timeout | Always a hard failure, in every environment, regardless of `WARP_DRIVE_REQUIRE_FUSE` |

`WARP_DRIVE_REQUIRE_FUSE=1` turns `Unavailable` into a hard failure too.
This repo's CI sets it in the `Unit tests` job, because that runner's log
was directly inspected (not assumed) to confirm it mounts FUSE for real —
see `.github/workflows/ci.yml`. Locally, on a machine that may not have
`/dev/fuse`, it stays a graceful, visibly-printed skip.

The reason this distinction exists at all: before PR #27, every one of
these cases collapsed into a single `Skip`, printed with `--nocapture` so a
human reading the log could tell them apart — but branch protection only
sees the test's pass/fail color, not its printed text. A future regression
(a missing `fusermount3`, a permission change, a broken binary) could turn
this test back into a green skip with nobody noticing. The fix wasn't
"print more" — it was "make CI's own runner unable to silently degrade,"
since that runner's FUSE support is now a known, asserted fact, not an
assumption.

## What a passing test proves, positively

`mount_smoke.rs`'s test prints `mounted G1 through FUSE and read README.md
successfully` on its real path — proof by explicit statement, not by the
absence of a skip message. CI runs with `--show-output` (not `--nocapture`)
specifically so this line — and any skip/fail reason — is always visible
without interleaving every test's output live.

## Why this diverges from the original `docs/TESTING.md` plan

`TESTING.md`'s `MountGuard::mount(tree: FixtureTree)` assumed the harness
could construct a `FixtureTree` directly and hand it to the mount function
in-process. That's not possible today: `FuseAdapter` is private to
`warp-drive-fuse` (`mod adapter;`, not `pub mod`), so an integration test
— which only sees the crate's public API — has no way to construct one
directly. Spawning the real compiled binary as a subprocess sidesteps that
entirely, at the cost of not being able to hold any in-process state (like
a directly-advanceable basis counter).

That cost is exactly why this harness doesn't try to do everything.
G4a's write-path tests need precise control over a shared basis counter
that a subprocess can't provide without inventing IPC — so they'll use a
*second*, separate harness (an in-process mount via `fuser::spawn_mount2`,
see `docs/design/g4a-intent-admission-receipts.md`'s "Resolved decisions").
The two harnesses serve different needs; neither replaces the other.

## Known gaps

- No `assert_file`/`assert_dir`/`assert_symlink`/`assert_readonly`/
  `assert_inode` helpers exist. Tests use `std::fs` directly today. Worth
  revisiting if a third or fourth integration test starts repeating the
  same boilerplate — not needed yet for one test.
- `MountGuard` only knows how to mount the in-memory (`warp-drive-fuse`)
  binary. Nothing here covers the Echo-backed (`warp-drive-fuse-echo`)
  binary; add that if a future gate needs an Echo-backed integration test.
- No dedicated `warp-drive-fixtures`/`warp-drive-test-harness` crate exists
  (see issue [#5](https://github.com/flyingrobots/warp-drive/issues/5),
  still open). This harness is intentionally smaller than that issue's
  full scope.
