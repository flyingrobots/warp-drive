---
title: "Unstable inode assignment across projection rebuilds"
legend: GATE
lane: bad-code
priority: high
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Unstable inode assignment across projection rebuilds

**File:** `crates/warp-drive-core/src/lib.rs`

**Status:** acceptable at G1 (static fixture, no rebuilds). Fix before G3.

## The smell

`FixtureTree::new()` assigns inodes sequentially (1–13). Sequential
assignment breaks across projection rebuilds: if the tree is rebuilt from
a changed Echo state, nodes get different inodes than they had before.

POSIX consumers expect inode stability for the lifetime of a mount.
Rename → new inode is correct; rebuild with the same path → same inode
is also required.

## Resolution

See cool-ideas card `GATE_hash-based-stable-inodes` for the proposed fix:
assign inodes by `siphash(coordinate || path)`, truncated to u64, with
collision detection at tree construction time.

Surface at G3 projection adapter design, before any inode assignment code
is written for the live projection.
