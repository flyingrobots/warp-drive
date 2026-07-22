---
title: "BEARING"
generated_at: "2026-07-22"
provenance_level: artifact_history
---

# BEARING

This signpost summarizes direction. It does not create commitments or
replace backlog items, design docs, gate records, or acceptance runs.

## Where are we going?

**Current gate:** G3 passed. No `gate/gN` branch open yet for the next one —
pick the next gate from the open backlog (see "What feels wrong?" below)
before opening `gate/g4`.

G3 passed and merged: `/.warp/stats` and `/.warp/runtime` are live for both
the in-memory and echo-rlib runtimes — real per-mount FUSE-callback counters
(`FOPEN_DIRECT_IO` + zero attribute TTL + constant-width JSON so they're
genuinely live, not just correctly-sized), and real Echo startup-observation
accounting instead of a placeholder. Every prior gate's own historical CLI
command (bare G1, G2a default, G2b) was reconfirmed passing unmodified.

## What just shipped?

- **G3 gate** (`ae4c5a7`, merged via PR #14 at `94bb3d5`, 2026-07-22):
  44/44 (in-memory) + 74/74 (echo-rlib) assertions. See `docs/gates/G3.md`
  and the living reference at `docs/topics/dotwarp-diagnostics/README.md`.
- **Documentation Standards + `docs/topics/` layer** (PR #16, `7a2687e`,
  2026-07-22): new living-reference layer for cross-gate behavior, distinct
  from frozen gate/design records. `docs/method/backlog/` is now historical
  — new planning content is GitHub Issues (see `docs/PROCESS.md`).
- **Branch protection enabled on `main`** (2026-07-22, closes #7): requires
  `Clippy + fmt`, `Unit tests`, `G1 + G3 gate acceptance (Linux / Docker)`,
  strict (must be up to date). Force-push and branch deletion blocked.
  `CodeRabbit` intentionally left non-required (review bot, not a gate).
- **G2b gate** (`49f96ac`, merged via PR #2 at `60829e1`, 2026-06-01):
  Echo-projected regular-file bytes, 60/60 assertions. `/echo/head.json` is a
  normal read-only file whose bytes come from Echo query projection payloads.
- **G2a gate** (`d32254e`, 2026-05-31): Echo coordinate metadata mount,
  38/38 assertions.
- **G1 gate** (`25a5f0a`, 2026-05-30): In-memory FUSE fake tree, 29/29
  assertions.
- **G0 gate** (2026-05-29): `warp-wasm` rlib embedding + `observe_cbor`
  round-trip proved.

## What feels wrong?

- Fixtures crate (`warp-drive-fixtures`) and integration test harness
  (`warp-drive-test-harness`) still don't exist (issue #5) — G3 added real
  unit tests inside `warp-drive-core`/`warp-drive-fuse` directly, which
  covers some of this, but not the dedicated crate/harness itself.
- `FixtureTree::new()` still mixes fixture data with tree-traversal logic,
  and inode assignment is still sequential/hardcoded, not a typed builder
  (issue #6; related historical bad-code cards `GATE_fixture-data-mixed-with-tree-logic`,
  `GATE_unstable-inode-assignment`). G3 added to this pattern rather than
  refactoring it.
- `warp-drive-fuse`'s single-variant `Runtime` enum is still vestigial after
  G3 (issue #18) — the old bad-code card expected G3 to give it a second
  runtime; it didn't, because echo-rlib is served by a separate binary.
- `FuseAdapter::live_stats_node`'s own sentinel-check logic has no direct
  unit test (issue #19) — only its consumers are tested.
- Several `fuser::Filesystem` methods (`release`, `forget`, `flush`,
  `fsync`, `opendir`, etc.) still silently use `fuser`'s no-op defaults —
  historical bad-code card `GATE_silent-fuse-default-noop-methods` said
  "address before G3"; G3 shipped without addressing it.
- macOS local mount is still blocked on macFUSE kext approval. Not blocking
  any gate; Linux Docker is the authoritative gate runner.
- No gate has picked up writes, basis-aware save receipts, or a second
  Echo-projected file/directory yet — all still open (issues #8-#11, #15).
