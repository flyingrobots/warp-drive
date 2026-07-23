// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Smoke test for the `MountGuard` harness itself, against the existing G1
//! read-only surface — proves the harness works using behavior this repo
//! can already assert on, ahead of G4a's write-path tests needing it.
//!
//! Requires a real FUSE mount (`/dev/fuse` plus `fusermount3`/`fusermount`
//! on `PATH`). Skips gracefully, printing why, when that's not available —
//! e.g. a plain `cargo test` on a machine or CI job with no FUSE device.
//! `cargo xtask acceptance` remains the actual gate-acceptance proof; this
//! only proves the harness that G4a's tests will build on.

#![allow(
    clippy::print_stderr,
    reason = "reporting a skipped integration test's reason to a human is the intended behavior here, matching xtask's own print allowance for the same reason"
)]

mod support;

use std::fs;

use support::MountGuard;

#[test]
fn mounts_g1_and_reads_the_real_readme_bytes() {
    let guard = match MountGuard::mount_in_memory("g1") {
        Ok(guard) => guard,
        Err(skip) => {
            eprintln!("{skip}");
            return;
        }
    };

    let readme = fs::read_to_string(guard.path().join("README.md")).unwrap_or_default();
    assert!(readme.contains("WARP DRIVE G1 Fixture"));
}
