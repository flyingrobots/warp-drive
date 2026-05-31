---
title: "G1 acceptance asserts a magic inode number"
legend: GATE
lane: bad-code
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# G1 acceptance asserts a magic inode number

**File:** `scripts/acceptance.sh`

**Status:** acceptable at G1, brittle before G2.

## The smell

The acceptance test asserts:

```
Inode: 5
```

for `src/main.ts`. That is fine while G1 uses a fixed hardcoded fixture,
but it becomes brittle the moment the fixture builder, stable hash-based
inode scheme, or fixture library lands.

## Why it matters

The acceptance test should verify the inode stability *law*, not bless a
magic number forever. The implementation plan explicitly treats inode
stability as a law, and the bad-code backlog already has stable inode
concerns (`GATE_unstable-inode-assignment`). The current assertion is
useful but too literal.

## Resolution

Replace "inode equals 5" with one of:

1. Inode is nonzero and not root (minimal)
2. Inode remains stable across two `stat` calls during the same mount
   (tests the law directly)
3. `inode equals fixture.expected_inode("src/main.ts")` — once the
   fixture library (`GATE_fixture-data-mixed-with-tree-logic`) exists,
   this is the right answer: the fixture owns its own expected inode map

For G1, option 3 is best once the fixture library lands. Option 2 is a
reasonable interim assertion that tests the actual invariant.
