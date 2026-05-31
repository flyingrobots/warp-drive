---
title: "Receipt log / last receipt — `.warp/intents/log.jsonl`"
legend: GATE
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Receipt log / last receipt — `.warp/intents/log.jsonl`

**Status:** cool idea. Already in `.warp/` surface spec (§11.5); make it *good*.

## The idea

`/.warp/intents/log.jsonl` — persistent append-only obstruction history.
`/.warp/intents/last` — the most recent receipt: who wrote, what basis, what
result (admitted / obstructed / pending).

These are already specified in the `.warp/` surface design. This card is a
reminder to implement them with enough detail to be useful — not just present
but diagnostic.

## Why it matters

`cat .warp/intents/last` is the mechanical moment that makes WARP DRIVE's
moral argument tangible. Without it, "your write was rejected because stale
basis" is an error code. With it, it's a legible receipt: this is what you
held, this is what advanced past you, here is when it happened.

## Surface when

Implementing the `.warp/` surface at G3+, or when the stale-save demo is
being built. See also `GATE_stale-save-demo`.
