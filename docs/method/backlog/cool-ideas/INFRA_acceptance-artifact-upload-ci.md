---
title: "Acceptance artifact upload in CI"
legend: INFRA
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Acceptance artifact upload in CI

**Status:** cool idea. Natural follow-on once GitHub Actions CI lands.

Related cards: `INFRA_gate-ledger-xtask`, `INFRA_github-actions-ci`

## The idea

Once GitHub Actions lands, upload the gate artifacts as workflow artifacts:

- `target/warp-drive/gates/G1.json`
- `docs/gates/G1.md`

```yaml
- uses: actions/upload-artifact@v4
  with:
    name: gate-g1-record
    path: |
      target/warp-drive/gates/G1.json
      docs/gates/G1.md
```

## Why it matters

Branch protection says pass/fail. Artifacts say *what* passed. If a flaky
FUSE edge case appears later, the full assertion transcript and environment
pins are retained per run — not just in the commit history, but in the CI
run itself.

This is the difference between "CI was green" and "here is exactly what
green meant, with a Docker image hash, tool versions, and a per-assertion
transcript."

## Surface when

Wiring `INFRA_github-actions-ci` and the `INFRA_gate-ledger-xtask` gate
record command.
