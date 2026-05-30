<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Unstable inode assignment strategy

**File:** `crates/warp-drive-core/src/lib.rs` — `FixtureTree::new()`

**Status:** acceptable at G1 (static tree), design debt for G3+.

## The smell

Inodes are assigned sequentially at construction time in `FixtureTree::new()`.
There is no persistent inode map and no stable assignment strategy across
remounts. If the tree is rebuilt — even with identical content — inodes change.

## Why it matters

POSIX consumers (`find`, `rsync`, NFS, inotify, certain editors) key their
internal state on inode numbers. A remount that shuffles inodes causes:

- `find -inum` queries to miss or ghost
- `rsync` to re-copy files it has already transferred
- Editor undo histories to point at dead inodes
- Any daemon that watches by inode to mis-fire

At G1 the tree is hardcoded and never rebuilt, so this is silent. At G3+
the tree is derived from a live Echo projection; remounts or partial
refreshes will happen, and inode churn will be a real correctness bug.

## Resolution sketch

Assign inodes by deterministic hash of `(coordinate, path)` truncated to
a u64, with a collision-detection pass at construction. This produces the
same inode for the same logical file across remounts as long as the path
does not change — which is exactly the stability POSIX consumers expect.

Design this before G3 scaffolding, not after.
