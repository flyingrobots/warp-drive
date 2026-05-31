---
title: "`--dry-run` flag for `warp-drive-fuse` — CI tree validation without macFUSE"
legend: INFRA
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `--dry-run` flag for `warp-drive-fuse` — CI tree validation without macFUSE

**Status:** cool idea. Surface when setting up GitHub Actions.

## The idea

Add a `--dry-run` flag to `warp-drive-fuse` that:

1. Constructs the `FixtureTree`
2. Walks every node: validates parent pointers, inode uniqueness, symlink
   targets, file content lengths, `.warp/` surface presence
3. Prints a structured report (or exits 0 silently)
4. Does **not** call `fuser::mount2`

This lets CI verify the fixture tree is well-formed on any platform —
including Linux runners without FUSE and macOS runners without macFUSE
installed — without needing the kernel module.

## Why it matters

The GitHub Actions runner on macOS cannot run `brew install --cask macfuse`
and approve a kernel extension in the same CI job. A dry-run mode gives CI
a meaningful signal ("the tree is structurally correct") without requiring
a real mount. Real mount verification stays in the developer acceptance
workflow (`cargo xtask acceptance`).

## Extend

At G3+, `--dry-run` becomes a health-check mode for the live runtime:
connect to Echo, fetch the projection, validate it matches the expected
shape, disconnect. No mount needed. Useful as a readiness probe.

## Surface when

Setting up GitHub Actions for warp-drive-fuse.
