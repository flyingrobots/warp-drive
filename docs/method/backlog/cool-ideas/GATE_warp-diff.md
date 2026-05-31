---
title: "`warp diff @main @agent` — substrate-aware causal diff"
legend: GATE
lane: cool-ideas
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `warp diff @main @agent` — substrate-aware causal diff

**Status:** cool idea. G7 collaboration UX.

## The idea

A CLI wrapper that queries two coordinates and renders a substrate-aware
diff. Not just text diff — basis, suffixes, witnesses, receipts. Shows
causal divergence, not just byte divergence.

```
warp diff @main @agent-lane
```

Output: which suffixes diverged, from which common ancestor basis, with
full receipt chain for each side.

## Why it matters

`git diff` shows you what changed. `warp diff` shows you *why* two
coordinates look different and whose writes caused the divergence. This
is the natural tool for multi-agent workflows where concurrent writes
are expected and the interesting question is causal, not textual.

## Surface when

Designing G7 multi-agent collaboration tools.
