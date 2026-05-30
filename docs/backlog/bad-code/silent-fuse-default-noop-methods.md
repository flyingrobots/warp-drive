<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `FuseAdapter` silently inherits fuser no-ops for release/forget/flush

**File:** `crates/warp-drive-fuse/src/adapter.rs` — `impl fuser::Filesystem for FuseAdapter`

**Status:** correct at G1 (static read-only tree), latent bug at G3+.

## The smell

`fuser::Filesystem` provides default no-op implementations for `release`,
`forget`, `flush`, `fsync`, and others. `FuseAdapter` inherits all of them
without comment. At G1 this is fine — the tree is immutable and there is
nothing to flush or forget. At G3+ the tree will be backed by live Echo
state and those no-ops become silent data-loss bugs.

## Why it matters

A future developer working on the G3 runtime adapter will see a clean
`impl Filesystem` block, assume the no-ops are intentional, and ship
without implementing `release` or `forget`. The first time the kernel
reclaims an inode the runtime will fail silently.

## Resolution

Before G3 scaffolding: add `// G1: static tree — no-op is correct` comments
to the inherited methods that matter (`release`, `forget`, `flush`, `fsync`).
This makes the intentional omission visible rather than accidental.

At G3: implement `forget` (decrement refcount in the projection cache) and
`release` (notify the runtime that a file handle is closed). These are the
two methods most likely to affect correctness under load.
