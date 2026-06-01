---
title: "BEARING"
generated_at: "2026-06-01"
provenance_level: artifact_history
---

# BEARING

This signpost summarizes direction. It does not create commitments or
replace backlog items, design docs, gate records, or acceptance runs.

## Where are we going?

**Current gate:** G3 — `.warp/` diagnostics + perf counters.

G2b passed and merged: WARP DRIVE now proves one normal POSIX-readable file,
`/echo/head.json`, whose bytes come back through Echo's
`ObservationProjection::Query -> ObservationPayload::QueryBytes` path. This is
the first Echo-projected regular-file proof, not a full Echo filesystem
projection.

Active branch: `ship-sync/post-g2b` for post-merge documentation sync. The next
feature branch should start from fresh `main` as `gate/g3`.

## What just shipped?

- **G2b gate** (`49f96ac`, merged via PR #2 at `60829e1`, 2026-06-01):
  Echo-projected regular-file bytes, 60/60 assertions. `/echo/head.json` is a
  normal read-only file whose bytes come from Echo query projection payloads.
  The copy-in Docker runner avoids live repo bind mounts and strips Git
  metadata before acceptance. Only `/echo/head.json` is Echo-projected; G1
  fixture files and directories remain fixture-backed.
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
- G3 has not yet made the membrane observable. `/.warp/stats` is still a static
  placeholder; live counters and runtime diagnostics are the next gate.
