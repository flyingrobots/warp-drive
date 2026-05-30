<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Hash-based stable inode assignment for G3+ live projections

**Status:** cool idea / pre-design. Surface before G3 scaffolding.

Related bad-code card: [unstable-inode-assignment](../bad-code/unstable-inode-assignment.md)

## The idea

At G3+ the `FixtureTree` is replaced by a live Echo projection that can
be refreshed or rebuilt. Sequential inode assignment breaks across rebuilds.

Proposal: assign inodes by deterministic hash:

```rust
fn stable_ino(coordinate: &CoordinateHash, path: &[u8]) -> Ino {
    // SipHash or xxhash of (coordinate || '\0' || path), truncated to u64.
    // Reserve 0 and 1 (FUSE root is always ino 1).
    let raw = siphash13(coordinate, path);
    Ino(raw.max(2))
}
```

With a collision-detection pass at tree construction time. Collisions are
astronomically rare for any realistically sized tree; detect and abort
rather than silently renumber.

## Why this is the right invariant

POSIX consumers expect that inode numbers are stable for a given file
for the lifetime of the mount. They do not expect stability across
unmount/remount, but they do expect stability across a projection refresh
within the same mount. Sequential assignment breaks even within-mount
stability if the tree is rebuilt from a changed Echo state.

Hash-based assignment gives:
- Same path → same inode, always, within a coordinate
- Rename → new inode (correct — rename is a new logical file)
- Refresh with same paths → same inodes (correct)
- No inode map to persist or checkpoint

## Open questions

- Does SipHash's output distribution cause clustering near 0/1 in degenerate
  inputs? (Probably not, but test it.)
- What is the right hash input for directories that can be renamed?
  Should it be the canonical path from root, or a parent-ino + name tuple?
  Parent-ino + name is more stable under ancestor renames but requires
  careful bootstrapping for the root.
- At what tree size does the birthday-paradox collision probability become
  non-negligible for a u64 space? (Roughly 2^32 files — not a concern.)

## Surface when

Designing the G3 Echo projection adapter, before any inode assignment
code is written.
