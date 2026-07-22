---
title: "cargo xtask diagnostics — before/after delta probe"
legend: GATE
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `cargo xtask diagnostics --path <mount>`

**Status:** cool idea, deferred at G3.

## The idea

A developer command that reads `/.warp/stats`, performs one controlled
operation of each counted kind (a read, a readdir, a readlink, a lookup
miss, a forced getattr), reads `/.warp/stats` again, and renders named
deltas — essentially the G3 acceptance script's diagnostics section, but as
an interactive developer tool instead of a pass/fail assertion suite.

## Why it matters

This is both a developer tool (a fast way to sanity-check that a live mount
is actually counting things) and a human-readable acceptance debugger — when
`scripts/acceptance-g3.sh` fails a delta assertion, this command would let a
developer reproduce the same probe sequence by hand and see the raw before/
after numbers instead of parsing bash output.

## Surface when

When someone hits a real G3 diagnostics bug in the field and wants faster
iteration than re-running the full Docker acceptance script.
