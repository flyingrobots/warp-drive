<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Test plan: `fuser::Filesystem` method surface

Companion to [`README.md`](README.md). Requirement IDs are stable within
this file; do not renumber on edit, append instead.

| ID | Behavior | Oracle | Evidence | Status |
| --- | --- | --- | --- | --- |
| FUSEM-01 | `lookup`/`getattr`/`readlink`/`read`/`readdir` return real fixture data; `ENOENT`/`EINVAL` on the documented miss cases | Acceptance scripts: `ls`, `cat`, `find`, `rg`, `stat`, `readlink` sections, all gates | `scripts/acceptance*.sh` | Implemented |
| FUSEM-02 | `open` rejects any non-read-only open with `EROFS` | Acceptance scripts: write-rejection section, all gates | `scripts/acceptance*.sh` | Implemented |
| FUSEM-03 | `is_live_stats`/`live_stats_node` gate live-diagnostics content to exactly `WARP_STATS_INO` with `NodeKind::RegularFile`, including the "right inode, wrong kind" branch | Unit test | `crates/warp-drive-fuse/src/adapter.rs::is_live_stats_requires_both_stats_ino_and_regular_file_kind`, `::live_stats_node_is_none_for_any_ino_other_than_warp_stats_ino`, `::live_stats_node_is_some_for_warp_stats_ino_in_the_real_fixture` | Implemented |
| FUSEM-04 | `release`/`opendir`/`releasedir` succeed (stateless mount, nothing to release/track) | Acceptance scripts: implicit — every `cat`/`ls`/`find` invocation triggers `open`→`release` and `opendir`→`readdir`→`releasedir`; a non-`ok()` reply would fail the surrounding command | `scripts/acceptance*.sh` (indirect; no isolated named assertion) | Implemented (indirect evidence only) |
| FUSEM-05 | `statfs` succeeds and identifies the mount as a `fuse`-type filesystem | Acceptance scripts: the mount-wait poll loop in every script calls `stat -f -c '%T'` and requires it to report `fuse` | `scripts/acceptance*.sh` (indirect — proves success, not the reported values) | Implemented (indirect evidence only) |
| FUSEM-06 | `statfs`'s `files` field reflects the real fixture node count, not a fabricated constant | Unit test (indirect, via the value `statfs` reads from) | `crates/warp-drive-core/src/lib.rs::node_count_reflects_the_real_g1_fixture_size`, `::node_count_grows_with_the_g2b_echo_head_extension` | Implemented (indirect — no acceptance script asserts the exact `stat -f` files count) |
| FUSEM-07 | Every method not listed above falls through to `fuser`'s own default, and that default is `ENOSYS` + a log, not a silent success, for everything except `forget`/`batch_forget` (no reply channel by protocol design) | Manual verification against `fuser` 0.17.0 source (`fuser-0.17.0/src/lib.rs`) | Read directly during the 2026-07-22 backlog crunch; not re-verified automatically — a `fuser` version bump could change these defaults without this repo noticing | Gap: no automated regression check. Would need a acceptance-level or unit-level probe against an unimplemented method (e.g. `mkdir`) asserting `ENOSYS`, currently absent. |
| FUSEM-08 | Write-adjacent methods (`write`, `flush`, `fsync`, `release`) have real, non-default semantics once a write path exists | — | — | Gap, by design. Tracked by G4a — see `docs/design/g4a-intent-admission-receipts.md`. |

## Coverage note

FUSEM-01/02 are proven by the same acceptance scripts every gate already
runs — this test plan indexes existing evidence rather than adding new
executable checks. FUSEM-03/06 are new unit tests added alongside the
2026-07-22 backlog crunch (issues #19 and #23). FUSEM-04/05 are real but
indirect: nothing asserts on them by name, only on the surrounding
command's success — a regression there would surface as a confusing
acceptance failure elsewhere, not a clearly-labeled one. FUSEM-07 is
honestly marked as an unautomated manual check, not a gap this repo
currently has any way to re-verify short of reading `fuser`'s source again
after a version bump.
