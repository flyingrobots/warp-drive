---
title: "GitHub Actions CI for G1 acceptance"
legend: INFRA
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# GitHub Actions CI for G1 acceptance

**Status:** cool idea. Everything needed is already in the repo — this is
just wiring.

## The idea

`cargo xtask acceptance` is now a self-contained Docker build + run. GitHub
Actions `ubuntu-latest` runners have Docker installed. The workflow is
essentially:

```yaml
jobs:
  acceptance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: G1 acceptance
        run: |
          docker build -t warp-drive-g1 .
          docker run --rm --device /dev/fuse --cap-add SYS_ADMIN warp-drive-g1
```

No Rust toolchain install needed on the runner — everything is inside the
Docker image.

## Why it's blocked (sort of)

The Dockerfile currently patches `Cargo.toml` at build time with `sed` to
exclude `warp-drive-g0-spike` and its `../echo-warp-drive` path deps. This
works but is fragile. Resolving the bad-code card
`GATE_silent-fuse-default-noop-methods` would let us delete the sed hacks,
making the Dockerfile clean and the CI workflow straightforward.

## Surface when

- Resolving workspace path-poisoning from the g0-spike crate (natural follow-on)
- Or whenever CI is a priority regardless
