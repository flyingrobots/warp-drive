---
title: "`/.warp/why/<path>` — provenance as a filesystem-native affordance"
legend: GATE
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `/.warp/why/<path>` — provenance as a filesystem-native affordance

**Status:** cool idea. Surface at G3/G5.

## The idea

Ask why a file looks the way it does. `cat /.warp/why/src/main.ts` returns
the chain of suffixes and receipts that produced the current projection for
that path: which write advanced the frontier, what basis it held, when it
was admitted.

Provenance as a filesystem-native affordance — no separate tool, no log
aggregator, no out-of-band query. Just `cat`.

## Why it matters

Debugging a file whose content is surprising requires asking: "who wrote
this, from what basis, and when?" Today that question requires querying
Echo directly. A `.warp/why/<path>` virtual file makes the answer first-class
and accessible to any shell tool.

## Surface when

Designing G3/G5 `.warp/` surface extensions.
