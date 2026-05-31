---
title: "`/.warp/test-report` synthetic file during acceptance"
legend: GATE
lane: cool-ideas
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `/.warp/test-report` synthetic file during acceptance

**Status:** cool idea. On-brand: the filesystem should know what it did.

## The idea

When mounted in test mode, expose `/.warp/test-report` containing:

```json
{
  "fixture": "g1_acceptance",
  "backend": "in-memory",
  "gate": "G1",
  "mount_started_at": "...",
  "ops": {
    "lookup": 12,
    "getattr": 8,
    "readdir": 3,
    "open": 5,
    "read": 7,
    "readlink": 1
  }
}
```

The mount self-reports its operation histogram. An acceptance test can
`cat /.warp/test-report` and assert that the op counts are in expected
ranges — catching regressions where extra lookups or redundant readdirs
sneak in.

## Why it matters

This is halfway between `/.warp/stats` and acceptance diagnostics. It makes
the mount observable from within — a shell tool can ask "what did you do?"
and get a structured answer. Very on-brand for a filesystem whose whole
premise is legible provenance.

It also complements `GATE_golden-syscall-transcript` — one records the
expected sequence, the other exposes the actual histogram.

## Surface when

Designing the `.warp/` surface at G3+, or when building the golden syscall
transcript test infrastructure.
