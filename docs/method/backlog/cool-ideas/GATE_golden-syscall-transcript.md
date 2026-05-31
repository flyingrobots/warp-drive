---
title: "Golden syscall transcript tests"
legend: GATE
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Golden syscall transcript tests

**Status:** cool idea. The "haunted filesystem exorcism log."

## The idea

For each gate, record the expected FUSE operation sequence for real tools:
`ls -a`, `cat README.md`, `rg export`, `stat src/main.ts`, `readlink links/readme`.

Not a strict byte-for-byte syscall trace — too brittle — but an
operation-class transcript:

```
LOOKUP /
READDIR /
LOOKUP README.md
GETATTR README.md
OPEN README.md
READ README.md
RELEASE README.md
```

Assert that the actual sequence matches the expected class sequence when
those commands are run against the mount.

## Why it's cool

It catches accidental regressions where `cat` still works but only because
the adapter is doing wasteful or semantically suspect extra lookups. It also
teaches future agents what "normal" FUSE traffic looks like for each
operation — the golden transcript is executable documentation.

Detecting a surprise extra `GETATTR` loop or a redundant `READDIR` is the
kind of regression that byte-level assertion testing completely misses.

## Surface when

Designing G2+ acceptance or the FUSE adapter test harness.
