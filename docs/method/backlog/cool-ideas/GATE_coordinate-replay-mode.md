---
title: "Coordinate replay mode — time-travel debugging at the shell"
legend: GATE
lane: cool-ideas
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Coordinate replay mode — time-travel debugging at the shell

**Status:** cool idea. Future magic.

## The idea

```
warp-drive mount --replay @main --from fr:old --to fr:new ~/replay
```

Mounts a filesystem that can be stepped through historical frontiers.
`cat ~/replay/src/main.ts` shows the file at `fr:old`. Step forward,
and the mount updates in place. Time-travel debugging at the shell level.

## Why it matters

Debugging "how did this file get here" today requires querying Echo's
suffix chain manually and reconstructing the projection in your head.
A replay mount makes the history walkable with ordinary shell tools:
`cat`, `diff`, `rg`, `stat`. The filesystem is the debugger.

## Open questions

- What is the stepping interface? A virtual `/.warp/frontier` file you
  write to? A `--step` flag on `warp-drive`? An interactive TUI?
- How do non-monotonic suffix chains behave (concurrent writes, retractions)?
- What is the performance story for large suffix chains?

## Surface when

Designing the G3+ projection adapter or the first debugging/observability
tooling sprint.
