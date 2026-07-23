<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Test plan: `.warp/` diagnostics surface

Companion to [`README.md`](README.md). Requirement IDs are stable within
this file; do not renumber on edit, append instead.

| ID | Behavior | Oracle | Evidence | Status |
| --- | --- | --- | --- | --- |
| DIAG-01 | Reading `/.warp/stats` does not increment `read_count` | `scripts/acceptance-g3.sh` / `acceptance-g3-echo.sh`: two consecutive reads, `read_count` unchanged | Acceptance script | Implemented |
| DIAG-02 | `/.warp/stats` size and byte content stay in sync across separate opens (`FOPEN_DIRECT_IO`) | Acceptance script: second open's `open_count` strictly increases | Acceptance script | Implemented |
| DIAG-03 | `lookup_count` increases by at least *n* after *n* distinct guaranteed-missing lookups | Acceptance script: 8-probe bulk delta assertion | Acceptance script | Implemented |
| DIAG-04 | `getattr_count` increases by at least *n* after *n* forced attribute refreshes | Acceptance script: `stat --cached=never` × 8, bulk delta assertion | Acceptance script | Implemented |
| DIAG-05 | `read_count`, `readdir_count`, `readlink_count` each increase on the first real operation of their kind | Acceptance script: first `cat`/`ls`/`readlink` after mount | Acceptance script | Implemented |
| DIAG-06 | `runtime_observe_count` is exactly `0` for `in-memory`, exactly `2` for `echo-rlib` G3 | Acceptance script: exact numeric `assert_eq` | Acceptance script | Implemented |
| DIAG-07 | `runtime_observe_error_count` is exactly `0` in any observable (successfully mounted) run | Acceptance script: exact numeric `assert_eq` | Acceptance script | Implemented |
| DIAG-08 | `/.warp/stats` reported size (`stat`) matches bytes actually read (not `st_size`-optimized) | Acceptance script: `fresh_stat_size` vs. a piped (non-optimizable) read | Acceptance script | Implemented |
| DIAG-09 | `.warp/stats`/`.warp/runtime` snapshot JSON has constant byte length across counter magnitudes (0, 9, 10, `u64::MAX`) | Unit test | `crates/warp-drive-fuse/src/stats.rs::snapshot_byte_length_is_stable_across_decimal_boundaries` | Implemented |
| DIAG-10 | `MountStats` snapshot contains all 11 required keys with correct labels | Unit test | `crates/warp-drive-fuse/src/stats.rs::snapshot_has_all_eleven_keys`, `fresh_snapshot_is_all_zero_with_correct_labels` | Implemented |
| DIAG-11 | `open_reply_flags` grants `FOPEN_DIRECT_IO` only for the live-stats inode | Unit test | `crates/warp-drive-fuse/src/adapter.rs::open_reply_flags_grants_direct_io_only_for_live_stats` | Implemented |
| DIAG-12 | `.warp/coordinate`, `.warp/runtime`, `.warp/stats` agree on `"gate"` for the active mount; `/echo/head.json`'s own `"gate"` is untouched payload provenance | Acceptance script: explicit four-way assertion in `acceptance-g3-echo.sh` | Acceptance script | Implemented |
| DIAG-13 | `runtime_observe_count`/`_error_count` reflect only mount-startup observations, no live post-mount refresh | — | — | Gap. No live refresh exists; nothing to test yet. Tracked informally in `docs/method/backlog/cool-ideas/` (historical location — new tracking should be a GitHub issue). |
| DIAG-14 | `/.warp/stats` reads within one `open()` are mutually coherent (per-open snapshot, not per-read) | — | — | Gap, by design (see README "Known gaps"). Not required by any current gate. |
| DIAG-15 | A guessed `WARP_STATS_INO` value only yields live content if the fixture tree actually backs it with a `RegularFile` node — the "right inode, wrong kind" branch specifically | Unit test | `crates/warp-drive-fuse/src/adapter.rs::is_live_stats_requires_both_stats_ino_and_regular_file_kind` | Implemented (2026-07-22, closed issue #19 — previously untested; see [`fuse-method-surface`](../fuse-method-surface/README.md) FUSEM-03 for the full picture) |

## Coverage note

DIAG-01 through DIAG-12 are proven by the same acceptance scripts and unit
tests the G3 gate itself uses — this test plan doesn't introduce new
executable evidence, it indexes evidence that already exists so a reader
doesn't have to reconstruct the mapping from the gate record or the source
tree by hand. DIAG-13/14 are honestly marked as gaps rather than silently
omitted. DIAG-15 was added alongside the 2026-07-22 backlog crunch (see
the new [`fuse-method-surface`](../fuse-method-surface/README.md) topic
for the broader FUSE-method context this one detail sits inside).
