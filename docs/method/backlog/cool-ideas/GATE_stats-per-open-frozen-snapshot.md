---
title: "Per-open frozen /.warp/stats snapshots"
legend: GATE
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Per-open frozen `/.warp/stats` snapshots

**Status:** cool idea, deferred at G3. Most valuable of the deferred G3
diagnostics ideas.

## The idea

G3 serves a fresh `snapshot_json()` on every individual `read()` call to
`/.warp/stats` — an explicit, documented design decision (see the design
doc's resolved decisions: "per-read snapshot consistency, not per-open").
`FOPEN_DIRECT_IO` guarantees each `open()`/`read()` reaches the adapter
freshly, but if a single `open()` results in multiple chunked `read()`
calls (unlikely for this file's size today, but not impossible for a larger
future diagnostics surface), each chunk could reflect a different instant.

A per-open frozen snapshot would give each file handle a `MountStats`
snapshot taken once at `open()` time, served identically across every
`read()` on that handle, and released in `release()`. Direct I/O guarantees
freshness *across* opens; a frozen-per-handle snapshot would additionally
guarantee coherence *within* one open.

## Why it matters

Coherent multi-chunk reads matter more as `/.warp/stats` (or future
diagnostic files) grow past whatever the kernel/FUSE choose as a single
read's max size. Pairing this with a separate genuinely-live
`/.warp/stats.ndjson` stream (a running log of samples, not a point-in-time
snapshot) would let a consumer choose between "one coherent moment" and
"see it change live."

## Surface when

When a diagnostic surface grows large enough that multi-chunk reads of a
single open become routine, or when a consumer reports observing internally
inconsistent counters within what should have been one atomic read.
