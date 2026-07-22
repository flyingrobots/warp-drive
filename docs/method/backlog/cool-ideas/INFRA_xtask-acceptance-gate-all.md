---
title: "cargo xtask acceptance --gate all — run every in-memory gate from one build"
legend: GATE
lane: cool-ideas
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `cargo xtask acceptance --gate all --runtime in-memory`

**Status:** cool idea, deferred at G3.

## The idea

A convenience `Gate::All` value for the in-memory runtime that builds the
Docker image once and runs every in-memory gate script (`acceptance.sh`,
`acceptance-g3.sh`, and whatever comes after) against it in sequence,
instead of requiring a separate `docker build` per gate.

## Why it matters

As of G3, `cargo xtask acceptance` (G1) and `cargo xtask acceptance --gate
g3 --runtime in-memory` each rebuild the same root `Dockerfile` image from
scratch. That's correct and simple, but wasteful once there are three or
more in-memory gate scripts to keep green.

## Surface when

When a G4+ gate adds a third in-memory acceptance script and the redundant
Docker builds start meaningfully slowing down local iteration or CI.
