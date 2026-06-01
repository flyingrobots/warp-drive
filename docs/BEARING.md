---
title: "BEARING"
generated_at: "2026-05-30"
provenance_level: artifact_history
---

# BEARING

This signpost summarizes direction. It does not create commitments or
replace backlog items, design docs, gate records, or acceptance runs.

## Where are we going?

**Current gate:** G2b — Echo-projected file bytes.

G2a passed: real Echo coordinate metadata appears at the FUSE mount point via
the `warp-wasm` native rlib. Normal file bytes remain G1 fixture bytes until
G2b proves an Echo-backed projection.

Active branch: `gate/g2b`.

## What just shipped?

- **G2a gate** (`d32254e`, 2026-05-31): Echo coordinate metadata mount,
  38/38 assertions. `/.warp/coordinate` and `/.warp/runtime` are derived from a
  real embedded Echo observation; normal file bytes remain the G1 fixture.
- **G1 gate** (`25a5f0a`, 2026-05-30): In-memory FUSE fake tree, 29/29
  assertions. `ls`, `cat`, `find`, `rg`, `stat`, `readlink`, write
  rejection all pass against the fixture tree on Linux Docker.
- **G1 cleanup** (`55ea5e6`): g0-spike removed from active workspace;
  EROFS noise fixed; TESTING.md corrected; CI wired; METHOD signposts added.
- **G0 gate** (2026-05-29): `warp-wasm` rlib embedding + `observe_cbor`
  round-trip proved.

## What feels wrong?

- CI is not yet branch-protected. The `acceptance` job needs two clean
  runs on `ubuntu-latest` before we gate merges on it.
- macOS local mount is blocked on macFUSE kext approval under macOS 26.3.
  Not blocking G2; Linux Docker is the authoritative gate runner.
- Fixtures crate (`warp-drive-fixtures`) and integration test harness
  (`warp-drive-test-harness`) do not exist yet. Unit tests: zero.
