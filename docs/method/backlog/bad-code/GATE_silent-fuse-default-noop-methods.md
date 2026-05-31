---
title: "Silent fuser default no-op methods hide G3+ bugs"
legend: GATE
lane: bad-code
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Silent fuser default no-op methods hide G3+ bugs

**File:** `crates/warp-drive-fuse/src/adapter.rs`

**Status:** acceptable at G1 (read-only fake tree). Address before G3.

## The smell

`fuser::Filesystem` provides default no-op implementations for methods
we have not implemented: `release`, `forget`, `flush`, `fsync`, `opendir`,
`releasedir`, `fsyncdir`, `statfs`, `setxattr`, `getxattr`, `listxattr`,
`removexattr`, `access`, `create`.

At G1 this is fine — the fixture is read-only and no operation should
trigger these. At G3+ with a live projection and multiple clients, silent
no-ops on `flush` and `fsync` will cause data hazards that are impossible
to debug.

## Resolution

Add explicit `unimplemented_op!` or `ENOSYS` returns for all no-op methods
we intentionally do not support, with a comment explaining why each is
deferred. This makes G3+ gaps visible rather than silent.
