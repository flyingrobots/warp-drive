---
title: "`acceptance.sh` can leak a mounted FUSE filesystem on early exit"
legend: INFRA
lane: bad-code
priority: high
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `acceptance.sh` can leak a mounted FUSE filesystem on early exit

**File:** `scripts/acceptance.sh`

**Status:** live bug. Fix before G2.

## The smell

The script mounts, runs assertions, then unmounts at the end. But with
`set -euo pipefail`, any unexpected command failure before the explicit
`umount` can skip cleanup. A failed test leaves `/tmp/warp-g1` mounted.
Classic "test harness becomes crime scene."

## Why it matters

A leaked FUSE mount poisons the next run (the mount point already exists
and is busy), confuses CI (the container exits with a mounted filesystem
and Docker layer cleanup fails), and makes local debugging feel haunted —
`cargo xtask acceptance` starts failing with `EBUSY` and the reason is
invisible.

## Resolution

Add a `cleanup` trap immediately after `FUSE_PID` is assigned:

```sh
cleanup() {
    umount "$MOUNT" 2>/dev/null || fusermount3 -u "$MOUNT" 2>/dev/null || true
    if [ -n "${FUSE_PID:-}" ]; then
        kill "$FUSE_PID" 2>/dev/null || true
        wait "$FUSE_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM
```

Then the explicit end-of-script unmount becomes redundant and can be
removed — `cleanup` already handles it. The trap fires on any exit path:
normal, error, or signal.
