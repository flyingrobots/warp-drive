---
title: "Gate pass is only in commit history, not a gate record"
legend: GATE
lane: bad-code
priority: high
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Gate pass is only in commit history, not a gate record

**File:** `docs/gates/G1.md` (missing)

**Status:** G1 passed 29/29; the proof artifact does not exist.

## The smell

G1 passed 29/29, but the durable project record is currently the commit
message plus a terminal transcript. G0 has a proper gate record; G1 should
too. A commit message is not a proof artifact — it is a sticky note wearing
a SHA.

## Why it matters

The gate model says nothing advances until the condition is demonstrably
true. Future agents and contributors will not have the terminal transcript.
Without a gate record, "G1 passed" is a claim, not a proof.

## Resolution

Add `docs/gates/G1.md` containing:

- Gate condition
- Command run: `cargo xtask acceptance`
- Environment: Linux Docker / Docker Desktop
- Result: 29/29
- Commit: `250cbfb` (and the earlier `55ea5e6` acceptance-pass record)
- Known caveats:
  - macOS 26 + macFUSE local mount unresolved
  - CI not wired yet
  - Acceptance currently Docker/Linux only

Then future gates get the same treatment. See also `INFRA_gate-ledger-xtask`
for making this a first-class `cargo xtask gate record` ritual.
