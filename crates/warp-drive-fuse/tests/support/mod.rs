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
//! An absent capability is a documented, visible condition — never a
//! silent pass. Two distinct failure shapes matter here, and callers MUST
//! NOT collapse them:
//!
//! - **Unavailable**: `/dev/fuse` genuinely isn't present. Skippable on a
//!   machine that was never expected to have FUSE — but not silently: set
//!   `WARP_DRIVE_REQUIRE_FUSE=1` to turn this into a hard failure instead,
//!   which CI does once a runner is known to support FUSE for real. A
//!   plain graceful skip on every run would let a genuine regression (a
//!   FUSE install disappearing from CI, say) turn back into permanent
//!   green with nobody noticing.
//! - **Failed**: mount point creation, spawning the binary, or the mount
//!   never appearing within the timeout. Always a hard failure, in every
//!   environment, regardless of `WARP_DRIVE_REQUIRE_FUSE` — these mean
//!   something is actually broken, not merely absent.

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

/// Why a mount attempt did not produce a live mount.
#[derive(Debug)]
pub(crate) enum MountAttemptError {
    /// `/dev/fuse` genuinely isn't present in this environment.
    Unavailable(String),
    /// Something is actually broken — never influenced by
    /// `WARP_DRIVE_REQUIRE_FUSE`.
    Failed(String),
}

impl std::fmt::Display for MountAttemptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(reason) | Self::Failed(reason) => write!(f, "{reason}"),
        }
    }
}

/// What a caller should do about a [`MountAttemptError`]: skip the test, or
/// fail it with a message.
pub(crate) enum MountOutcome {
    /// A legitimate, visible skip — print the reason and return early.
    Skip(String),
    /// A hard failure — the caller should fail the test with this message.
    Fail(String),
}

/// Mount the in-memory fixture, or decide whether the caller should skip
/// or fail based on `WARP_DRIVE_REQUIRE_FUSE`.
///
/// This is the entry point integration tests should call — never
/// [`MountGuard::mount_in_memory`] directly — so the
/// `WARP_DRIVE_REQUIRE_FUSE` policy is applied in exactly one place.
pub(crate) fn mount_or_decide(gate: &str) -> Result<MountGuard, MountOutcome> {
    MountGuard::mount_in_memory(gate).map_err(|err| match err {
        MountAttemptError::Unavailable(reason) => {
            if std::env::var_os("WARP_DRIVE_REQUIRE_FUSE").is_some() {
                MountOutcome::Fail(format!(
                    "WARP_DRIVE_REQUIRE_FUSE is set, but FUSE is unavailable: {reason}"
                ))
            } else {
                MountOutcome::Skip(reason)
            }
        }
        MountAttemptError::Failed(reason) => MountOutcome::Fail(reason),
    })
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
    /// Prefer [`mount_or_decide`] over calling this directly — it applies
    /// the `WARP_DRIVE_REQUIRE_FUSE` policy consistently.
    ///
    /// # Errors
    ///
    /// See [`MountAttemptError`] for what each variant means.
    pub(crate) fn mount_in_memory(gate: &str) -> Result<Self, MountAttemptError> {
        if !Path::new("/dev/fuse").exists() {
            return Err(MountAttemptError::Unavailable(
                "/dev/fuse not present in this environment".to_owned(),
            ));
        }

        let mount_point =
            std::env::temp_dir().join(format!("warp-drive-harness-{}-{gate}", std::process::id()));
        std::fs::create_dir_all(&mount_point).map_err(|e| {
            MountAttemptError::Failed(format!(
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
            .map_err(|e| {
                MountAttemptError::Failed(format!("failed to spawn warp-drive-fuse: {e}"))
            })?;

        let guard = Self { mount_point, child };
        guard.wait_until_mounted(Duration::from_secs(5))?;
        Ok(guard)
    }

    /// Poll `/proc/mounts` until `mount_point` shows a `fuse`-family
    /// filesystem type, or the timeout elapses.
    fn wait_until_mounted(&self, timeout: Duration) -> Result<(), MountAttemptError> {
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
        Err(MountAttemptError::Failed(format!(
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
            .is_ok_and(|s| s.success())
            || Command::new("fusermount")
                .arg("-u")
                .arg(&self.mount_point)
                .status()
                .is_ok_and(|s| s.success());

        if !unmounted {
            // Both unmount attempts failed. The child may still be blocked
            // inside fuser::mount2() waiting for the kernel to report an
            // unmount that never happened — an unconditional wait() below
            // could then hang forever. Kill it directly rather than trust
            // that unmounting worked.
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
        let _ = std::fs::remove_dir(&self.mount_point);
    }
}
