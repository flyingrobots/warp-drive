<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `cargo xtask acceptance` — one-shot gate acceptance runner

**Status:** cool idea. Surface when wiring up G1 acceptance tests.

## The idea

Add an `acceptance` subcommand to the xtask that runs the full G1
acceptance script in one shot:

1. `mkdir -p /tmp/warp-drive-acceptance`
2. Spawn `warp-drive-fuse` in the background
3. Wait for the mount point to become readable
4. Run each acceptance command (`ls`, `cat`, `find`, `rg`, `stat`,
   `readlink`, write-rejection) and capture output
5. Assert expected content / expected errors
6. `umount` and report pass/fail with diff output on failure

The gate condition becomes mechanically verifiable: `cargo xtask acceptance`
exits 0 = gate passed, non-zero = gate failed.

## Why it matters

Right now "did G1 pass?" requires a human to read the output of 10 manual
commands and judge. That is fine for the first run. It is not fine for CI,
for future contributors, or for re-verifying after a refactor.

The xtask is already the natural home for this — it already has `mount` and
`unmount`. `acceptance` is just those two plus assertions between them.

## Extend to all gates

The same pattern extends to G2, G3, etc. Each gate gets an xtask command.
`cargo xtask acceptance --gate g1` selects the script. `cargo xtask acceptance`
with no flag runs all gates up to the current one.

## Surface when

- Writing G1 acceptance tests
- Setting up GitHub Actions for the first real CI run
