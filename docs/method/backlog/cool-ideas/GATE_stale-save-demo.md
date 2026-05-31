---
title: "The stale-save demo — G5 acceptance criterion"
legend: GATE
lane: cool-ideas
priority: high
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# The stale-save demo — G5 acceptance criterion

**Status:** must become a test. Promote to G5 acceptance criteria.

## The idea

Two editors open the same file from the same coordinate. Editor A saves.
Editor B saves. Editor B gets `EBUSY`. `cat .warp/intents/last` shows
whose suffix advanced the frontier and what basis B held.

This is the "holy shit" moment that makes the project's moral argument
tactile: WARP DRIVE doesn't silently last-write-wins. It obstructs stale
writers and surfaces the causal evidence.

## Why it matters

Every other filesystem demo shows that files exist and can be read. This
demo shows what WARP DRIVE actually is: a basis-disciplined write membrane.
Until this scenario can be demonstrated mechanically, the product claim is
an essay, not a proof.

## Resolution

Promote to a G5 acceptance criterion. Write it as a shellscript assertion:

```sh
# Two concurrent writers from the same starting basis
editor_a_writes && editor_b_writes_same_basis
assert_exit_code editor_b EBUSY
assert_content .warp/intents/last contains_basis_of editor_b
```

Surface when: designing G5 acceptance or the concurrent-write test harness.
