---
title: "WARP DRIVE — Executive Summary"
generated_at: "2026-05-30"
provenance_level: artifact_history
source_files:
  - README.md
  - CHANGELOG.md
  - docs/BEARING.md
  - docs/PROCESS.md
  - docs/ENGINEERING_STANDARDS.md
  - docs/IMPLEMENTATION_PLAN.md
  - docs/TECHNICAL_DEEP_DIVE.md
  - docs/TESTING.md
  - docs/gates/G0.md
  - docs/gates/G1.md
  - docs/gates/G2a.md
---

# WARP DRIVE — Executive Summary

## Identity

WARP DRIVE is a POSIX⇄causal membrane. It mounts a coordinate from a
Continuum-speaking runtime (Echo, git-warp, or any compliant backend)
as a read-only FUSE filesystem. Normal tools — `vim`, `cat`, `ls`,
`ripgrep` — read witnessed causal history as if it were a local
directory.

The core claim: the POSIX membrane is substrate-agnostic. If a runtime
speaks Continuum, WARP DRIVE can mount it without knowing anything about
that runtime's internals.

## Current state

Three gates closed, three proofs in the repo:

- **G0 (2026-05-29):** `warp-wasm` embeds as a native Rust rlib.
  `observe_cbor` round-trips from outside the echo workspace. The
  embedding path is confirmed: no wasmtime, no WASM interpreter — plain
  rlib linkage.

- **G1 (2026-05-30):** A FUSE mount backed by an in-memory fixture tree
  passes 29/29 acceptance assertions on Linux Docker. Normal tools read
  the tree correctly. Writes are rejected with EROFS. The POSIX
  translation layer works.

- **G2a (2026-05-31):** A FUSE mount initializes the embedded Echo rlib,
  performs one real `observe_cbor()` head observation, and serves the
  resulting coordinate metadata through `/.warp/coordinate` and
  `/.warp/runtime`. Normal file bytes remain the G1 fixture source.

The FUSE membrane exists in miniature. The question now shifts from
"can a POSIX membrane exist?" to "what is the smallest Echo-projected file
byte path?"

## Signposts

| Signpost | Type | Description |
|----------|------|-------------|
| `README.md` | Hand-authored | Project overview and quick start |
| `docs/BEARING.md` | Generated | Current priority and recent ships |
| `docs/VISION.md` | Generated | This document |
| `docs/PROCESS.md` | Hand-authored | Cycle doctrine, gate lifecycle, workflow |
| `docs/ENGINEERING_STANDARDS.md` | Hand-authored | Code quality, lint policy, layer rules |
| `docs/IMPLEMENTATION_PLAN.md` | Hand-authored | Operational plan v0.0.1 → v0.1 |
| `docs/TECHNICAL_DEEP_DIVE.md` | Hand-authored | Architecture reference |
| `docs/TESTING.md` | Hand-authored | Testing strategy and backend progression |
| `docs/gates/G0.md` | Hand-authored | G0 gate record |
| `docs/gates/G1.md` | Hand-authored | G1 gate record |
| `docs/gates/G2a.md` | Hand-authored | G2a gate record |

## Legends

### GATE

Gate-level work: acceptance tests, projection adapters, FUSE semantics,
domain design, inode strategy.

- **Active:** G2b design: first Echo-projected regular-file bytes.
- **Backlog:** 4 bad-code items, 4 cool-ideas.

### INFRA

Infrastructure: CI, Docker, xtask, tooling, process.

- **Active:** CI wired; branch protection pending first clean run.
- **Backlog:** 3 cool-ideas.

## Roadmap

### Active

- **G2b — Echo-projected file bytes**: keep G2a metadata, then prove at least
  one non-`.warp` regular file is served from an Echo projection instead of
  the G1 fixture tree.

### Gate sequence

| Gate | Condition | Status |
|------|-----------|--------|
| G0 | rlib embedding + `observe_cbor` | ✅ passed |
| G1 | In-memory FUSE fake tree, 29 assertions | ✅ passed |
| G2a | Echo coordinate metadata mount, 38 assertions | ✅ passed |
| G2b | Echo-projected regular-file bytes | ⏭️ active |
| G3 | `.warp/` live diagnostics + perf counters | ⏳ |
| G4+ | Write path, basis discipline, stale-save safety | ⏳ |

### Infrastructure

- CI branch protection: pending two clean runs on `ubuntu-latest`.
- `warp-drive-fixtures` crate + `warp-drive-test-harness`: planned per
  `docs/TESTING.md`; prerequisite for Rust-level integration tests.

## Open questions

- Does the G2b projection adapter need a trait, or is a single Echo rlib
  call sufficient at this gate?
- Should `MountGuard` use a thread or a subprocess for the FUSE process
  in integration tests?
- At what tree size does hash-based inode assignment need collision
  detection to be practically relevant?

## Limits

This document is a bounded synthesis over repo-visible artifacts. It is
grounded in artifact history only. It does not claim semantic provenance
or observation lineage beyond the source surfaces named in the frontmatter.
