// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! G4a RED: write-path admission and obstruction through a real FUSE mount.
//!
//! Mounts in-process — not via the subprocess-based
//! `support::MountGuard` that `mount_smoke.rs` uses — so the test can
//! advance the runtime's basis directly through a Rust handle. That handle
//! is never exposed through a `.warp/` file, a CLI flag, or any FUSE
//! method, so no real POSIX client can ever reach it. See
//! `docs/design/g4a-intent-admission-receipts.md`'s "Resolved decisions"
//! (item 2) for why this is a second, separate harness rather than a
//! change to the existing one.
//!
//! **Fails to compile at RED.** `warp_drive_fuse::testing::spawn_with_basis_control`
//! does not exist yet, and neither does real write/fsync/flush handling in
//! `FuseAdapter`. GREEN is: make this compile, then make it pass, without
//! weakening either assertion below to get there.

#![allow(
    clippy::print_stderr,
    reason = "reporting a skipped integration test's reason to a human, matching mount_smoke.rs's existing allowance"
)]

use std::fs::OpenOptions;
use std::io::{Read, Write};

use warp_drive_fuse::testing::spawn_with_basis_control;

#[test]
fn fresh_basis_write_is_admitted_and_the_new_bytes_are_visible() {
    let Some(mount) = spawn_with_basis_control("g1") else {
        eprintln!("skipping: FUSE not available in this environment");
        return;
    };

    let path = mount.path().join("README.md");
    let new_content: &[u8] = b"fresh-basis replacement bytes\n";

    let wrote_and_synced = OpenOptions::new()
        .write(true)
        .open(&path)
        .and_then(|mut file| file.write_all(new_content).and_then(|()| file.sync_all()))
        .is_ok();
    assert!(
        wrote_and_synced,
        "fresh-basis write+fsync should be admitted"
    );

    let mut readback = Vec::new();
    let _ = OpenOptions::new()
        .read(true)
        .open(&path)
        .map(|mut file| file.read_to_end(&mut readback));
    assert_eq!(readback, new_content);

    let receipt = mount.read_intents_last().unwrap_or_default();
    assert!(receipt.contains("\"receipt_kind\":\"admitted\""));
}

#[test]
fn stale_basis_write_is_obstructed_and_the_file_is_unchanged() {
    let Some(mount) = spawn_with_basis_control("g1") else {
        eprintln!("skipping: FUSE not available in this environment");
        return;
    };

    let path = mount.path().join("README.md");
    let original = std::fs::read(&path).unwrap_or_default();

    let Ok(mut file) = OpenOptions::new().write(true).open(&path) else {
        eprintln!("skipping: could not open {} for write", path.display());
        return;
    };

    // The gate-only control seam (decision 2): advances the runtime's
    // basis out from under this still-open handle. Only reachable because
    // this test mounted in-process.
    mount.basis_control().advance();

    let _ = file.write_all(b"stale write attempt\n");
    let fsync_result = file.sync_all();
    assert!(fsync_result.is_err(), "stale-basis fsync should be refused");

    let unchanged = std::fs::read(&path).unwrap_or_default();
    assert_eq!(
        unchanged, original,
        "an obstructed write must not mutate the projected file"
    );

    let receipt = mount.read_intents_last().unwrap_or_default();
    assert!(receipt.contains("\"receipt_kind\":\"obstructed\""));
    assert!(receipt.contains("\"obstruction_code\":\"stale_basis\""));
}
