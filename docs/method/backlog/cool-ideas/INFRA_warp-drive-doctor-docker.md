---
title: "`warp-drive doctor --docker` — acceptance infrastructure health check"
legend: INFRA
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `warp-drive doctor --docker` — acceptance infrastructure health check

**Status:** cool idea. Polish task, high contributor-experience value.

## The idea

A focused doctor mode for the exact acceptance-infrastructure failures that
have already bitten us.

Checks:
- Docker installed
- Docker daemon reachable
- `/dev/fuse` available in container
- `SYS_ADMIN` accepted
- `fuse3` installed in image
- `cargo xtask acceptance` can launch the image

Output:
```
✓ Docker daemon reachable
✓ /dev/fuse passed through
✓ SYS_ADMIN accepted
✗ FUSE mount failed: container lacks /dev/fuse

Try:
  docker run --rm --device /dev/fuse --cap-add SYS_ADMIN warp-drive-g1
```

## Why it matters

When `cargo xtask acceptance` fails, the error is usually deep in Docker
or FUSE permission setup — not in the Rust code. A focused doctor command
surfaces the exact misconfiguration with an actionable next step.

This complements the broader `warp-drive doctor` idea in the plan but is
specifically scoped to acceptance infrastructure.

## Surface when

Setting up GitHub Actions or when a contributor reports mysterious
acceptance failures.
