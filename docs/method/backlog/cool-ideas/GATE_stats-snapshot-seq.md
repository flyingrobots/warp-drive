---
title: "Monotonic snapshot_seq on /.warp/stats"
legend: GATE
lane: cool-ideas
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Monotonic snapshot_seq on `/.warp/stats`

**Status:** cool idea, deferred at G3.

## The idea

Add a monotonically increasing `"snapshot_seq"` field to `/.warp/stats`,
bumped once per `read()` of that inode. It would not make a single
`read()`'s bytes any more internally coherent (G3's counters are already
per-field independent, not transactionally coherent as a group — see the
design doc's resolved decisions), but it would make it explicit to any
consumer that two samples are genuinely different reads rather than a
coincidentally-identical cached pair.

## Why it matters

Cheap and honest: it doesn't fix anything G3 got wrong, it just makes change
visible without requiring a consumer to diff every field.

## Surface when

Whenever a later gate adds a second diagnostic-consuming tool (a `cargo
xtask diagnostics` delta probe, a dashboard, etc.) that would benefit from
knowing "this is a new sample" without comparing all 11 fields.
