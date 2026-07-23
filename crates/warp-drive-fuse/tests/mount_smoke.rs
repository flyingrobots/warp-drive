// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Smoke test for the `MountGuard` harness itself, against the existing G1
//! read-only surface — proves the harness works using behavior this repo
//! can already assert on, ahead of G4a's write-path tests needing it.
//!
//! Requires a real FUSE mount (`/dev/fuse` plus `fusermount3`/`fusermount`
//! on `PATH`). Skips gracefully, printing why, when that's not available —
//! unless `WARP_DRIVE_REQUIRE_FUSE=1` is set, in which case an absent mount
//! is a hard failure instead of a quiet skip (CI sets this once a runner
//! is known to support FUSE, so a regression there can't hide behind a
//! green skip). `cargo xtask acceptance` remains the actual gate-acceptance
//! proof; this only proves the harness that G4a's tests will build on.

#![allow(
    clippy::print_stdout,
    reason = "printing positive success evidence for a real mount, not just inferring it from the absence of a skip message"
)]

mod support;

use std::fs;

use support::MountOutcome;

#[test]
fn mounts_g1_and_reads_the_real_readme_bytes() -> Result<(), String> {
    let guard = match support::mount_or_decide("g1") {
        Ok(guard) => guard,
        Err(MountOutcome::Skip(reason)) => {
            println!(
                "skipping: {reason} (set WARP_DRIVE_REQUIRE_FUSE=1 to make this a hard failure)"
            );
            return Ok(());
        }
        Err(MountOutcome::Fail(reason)) => return Err(reason),
    };

    let path = guard.path().join("README.md");
    let readme =
        fs::read_to_string(&path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    assert!(
        readme.contains("WARP DRIVE G1 Fixture"),
        "README.md content did not match the expected G1 fixture marker; got: {readme:?}"
    );

    println!("mounted G1 through FUSE and read README.md successfully");
    Ok(())
}
