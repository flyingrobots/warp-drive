---
title: "Agent lane dashboard (`warp lanes`) — read-only TUI"
legend: GATE
lane: cool-ideas
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Agent lane dashboard (`warp lanes`) — read-only TUI

**Status:** cool idea. G7 collaboration UX.

## The idea

A read-only TUI showing all active lanes, their current frontiers, and
pending intent counts. Makes multi-lane reality visible without a GUI.

```
warp lanes
┌─────────────┬──────────────┬─────────┐
│ Lane        │ Frontier     │ Pending │
├─────────────┼──────────────┼─────────┤
│ @main       │ fr:a3b9f2    │ 0       │
│ @agent-1    │ fr:b7c4e1    │ 2       │
│ @agent-2    │ fr:a3b9f2    │ 0       │
└─────────────┴──────────────┴─────────┘
```

## Why it matters

When multiple agents and humans are writing to adjacent lanes simultaneously,
situational awareness requires knowing which lanes are ahead, which are at
parity, and which have pending intents in flight. A TUI is the natural
complement to the per-file `/.warp/why/<path>` affordance.

## Surface when

Designing G7 multi-agent collaboration tools or the first multi-lane demo.
