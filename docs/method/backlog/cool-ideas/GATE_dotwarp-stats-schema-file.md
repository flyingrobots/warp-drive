---
title: "/.warp/stats.schema contract file"
legend: GATE
lane: cool-ideas
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `/.warp/stats.schema` contract file

**Status:** cool idea, deferred at G3.

## The idea

A static `/.warp/stats.schema` file documenting each `/.warp/stats` field's
exact counting semantics — which FUSE callback increments it, whether misses
count, whether it's exempted from a diagnostic self-read, etc. — instead of
requiring a tool author to read this repo's design docs or source.

## Why it matters

G3 establishes real semantics (e.g. "`read_count` counts all `read()` calls
except diagnostic self-reads of `/.warp/stats` itself"; "`open_count` counts
both accepted and rejected opens") that are currently only documented in
`docs/design/g3-dotwarp-diagnostics-and-counters.md` and this crate's source.
A machine-readable (or at least self-describing) contract file would let
external tooling avoid reverse-engineering counting semantics.

## Surface when

When a second external tool (beyond the acceptance scripts) starts consuming
`/.warp/stats` and needs to know precisely what each field means.
