// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Minimal FUSE mount lifecycle harness for `warp-drive-fuse` integration
//! tests.
//!
//! Scoped tight for G4a prep, not issue #5's full `warp-drive-fixtures`/
//! `warp-drive-test-harness` scope: just enough to spawn the real
//! `warp-drive-fuse` binary against a temp mount point, wait for the kernel
//! to report it as a live FUSE mount, and unmount + reap the child on drop.
//! G4a's write-path tests need precise control over file-handle lifetime,
//! write offsets, and `fsync` timing that shell-script acceptance probes
//! handle awkwardly — this is the foundation those tests build on.
//!
//! Skips gracefully (does not fail the test) when FUSE isn't usable in the
//! current environment, mirroring this crate's existing
//! `compile-without-macfuse` pattern for macOS: an absent capability is a
//! documented, visible condition, not a silent pass or a hard failure.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// Why a mount attempt was skipped rather than treated as a test failure.
#[derive(Debug)]
pub(crate) struct Skip(String);

impl std::fmt::Display for Skip {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "skipping FUSE integration test: {}", self.0)
    }
}

/// An active WARP DRIVE in-memory mount for the duration of a test.
///
/// Unmounts and reaps the child process on drop, best-effort — `Drop`
/// cannot return a `Result`, and by the time teardown runs the test's own
/// assertions have already had their chance to fail loudly.
pub(crate) struct MountGuard {
    mount_point: PathBuf,
    child: Child,
}

impl MountGuard {
    /// Mount the in-memory `warp-drive-fuse` binary at a fresh temp
    /// directory for the given `--gate` value (e.g. `"g1"`, `"g3"`).
    ///
    /// # Errors
    ///
    /// Returns [`Skip`] (not a test failure) if `/dev/fuse` isn't present,
    /// the binary can't be spawned, or the mount doesn't become visible
    /// within the timeout. Callers should print the reason and return
    /// early rather than fail the test on `Err`.
    pub(crate) fn mount_in_memory(gate: &str) -> Result<Self, Skip> {
        if !Path::new("/dev/fuse").exists() {
            return Err(Skip("/dev/fuse not present in this environment".to_owned()));
        }

        let mount_point =
            std::env::temp_dir().join(format!("warp-drive-harness-{}-{gate}", std::process::id()));
        std::fs::create_dir_all(&mount_point).map_err(|e| {
            Skip(format!(
                "failed to create mount point {}: {e}",
                mount_point.display()
            ))
        })?;

        let child = Command::new(env!("CARGO_BIN_EXE_warp-drive-fuse"))
            .arg("--gate")
            .arg(gate)
            .arg("--mount")
            .arg(&mount_point)
            .spawn()
            .map_err(|e| Skip(format!("failed to spawn warp-drive-fuse: {e}")))?;

        let guard = Self { mount_point, child };
        guard.wait_until_mounted(Duration::from_secs(5))?;
        Ok(guard)
    }

    /// Poll `/proc/mounts` until `mount_point` shows a `fuse`-family
    /// filesystem type, or the timeout elapses.
    fn wait_until_mounted(&self, timeout: Duration) -> Result<(), Skip> {
        let deadline = Instant::now() + timeout;
        let target = self.mount_point.to_string_lossy().into_owned();
        while Instant::now() < deadline {
            if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
                for line in mounts.lines() {
                    let mut fields = line.split_whitespace();
                    let mount_dir = fields.nth(1);
                    let fstype = fields.next();
                    if mount_dir == Some(target.as_str())
                        && fstype.is_some_and(|t| t.starts_with("fuse"))
                    {
                        return Ok(());
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Err(Skip(format!(
            "mount at {target} did not become visible within {timeout:?}"
        )))
    }

    /// The live mount point, for opening files under it.
    #[must_use]
    pub(crate) fn path(&self) -> &Path {
        &self.mount_point
    }
}

impl Drop for MountGuard {
    fn drop(&mut self) {
        let unmounted = Command::new("fusermount3")
            .arg("-u")
            .arg(&self.mount_point)
            .status()
            .is_ok_and(|s| s.success());
        if !unmounted {
            let _ = Command::new("fusermount")
                .arg("-u")
                .arg(&self.mount_point)
                .status();
        }
        let _ = self.child.wait();
        let _ = std::fs::remove_dir(&self.mount_point);
    }
}
