---
title: "Gate ledger: `cargo xtask gate record`"
legend: INFRA
lane: cool-ideas
priority: high
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Gate ledger: `cargo xtask gate record`

**Status:** cool idea. Directly adjacent to the work that just landed.

Related bad-code card: `GATE_g1-gate-record-missing`

## The idea

Turn gate proof into a first-class ritual.

```sh
cargo xtask gate record g1 \
  --command "cargo xtask acceptance" \
  --result target/warp-drive/gates/g1.json
```

It writes:

- `docs/gates/G1.md` — human-readable gate record
- `target/warp-drive/gates/G1.json` — machine-readable artifact

The JSON contains: assertion count, environment, Docker image hash, Git
commit, tool versions, and pass/fail per assertion. The Markdown is the
prose record with known caveats.

## Why it matters

This makes the gate model mechanically honest: run gate, record gate, ship
gate. No vibes. No "trust me bro, I saw green text." The proof artifact is
committed alongside the code that created the gate condition.

Future gates follow the same pattern. Gate records become queryable: "show
me every gate that passed on a real FUSE mount vs. only under Docker."

## Surface when

G1 gate record is overdue right now. Top-three priority.
