---
title: "BEARING"
generated_at: "2026-05-30"
provenance_level: artifact_history
---

# BEARING

This signpost summarizes direction. It does not create commitments or
replace backlog items, design docs, gate records, or acceptance runs.

## Where are we going?

**Current gate:** G2 — Echo read-only mount.

Gate condition: real Echo coordinate, real `observe` via the `warp-wasm`
native rlib; projected bytes appear at the FUSE mount point.

Active branch: `gate/g2`.

## What just shipped?

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
