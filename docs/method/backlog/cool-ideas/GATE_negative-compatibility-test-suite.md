---
title: "Negative compatibility test suite — formal refusal list"
legend: GATE
lane: cool-ideas
priority: high
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Negative compatibility test suite — formal refusal list

**Status:** must become tests. Failures here are spec conformance, not bugs.

## The idea

A formal, versioned list of things WARP DRIVE intentionally rejects:

- SQLite default mode (shared-memory locking)
- `MAP_SHARED|PROT_WRITE` (mmap write-back)
- Atomic rename on non-supporting runtimes
- Path-based runtime writes (no fd, no basis)

A published refusal list increases trust more than a promise list. Users
knowing *what doesn't work* and *why* is more honest than a compatibility
matrix that only lists what does.

## Why it matters

Every "compatible with X" claim is hollow without a "not compatible with Y"
counterpart. The refusal list is what separates a principled POSIX membrane
from a leaky FUSE mount that silently corrupts data for unknown workloads.

## Resolution

Promote to a dedicated `cargo xtask` command:

```sh
cargo xtask acceptance --negative
```

Tests:
- Write to read-only G1 mount → `EROFS`
- `mkdir` on G1 → `EROFS` or `ENOSYS`, whichever is declared
- Shared writable mmap → `ENODEV` once mmap support is wired
- Unsupported rename semantics → `EOPNOTSUPP`
- Path-based runtime write attempt → impossible by type/API (compile-time proof)
- SQLite default mode → `EBUSY`

```sh
# Example assertion
sqlite3 $MOUNT/db.sqlite "CREATE TABLE t (x TEXT)" && fail "expected EBUSY"
echo "OK: SQLite default mode rejected"
```

Each entry gets a shellscript assertion. Any regression is a protocol
violation, not a compatibility decision to be revisited. Published refusal
lists are underrated.

## Surface when

G3+ acceptance, or the concurrent-write test harness. Deserves promotion
after G1 is fully recorded.
