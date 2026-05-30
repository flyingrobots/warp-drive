// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! WARP DRIVE FUSE mount binary (G1 gate: in-memory fake tree).
//!
//! Mounts a read-only POSIX filesystem backed by the hardcoded G1 fixture tree
//! from `warp-drive-core`. Implements the 7 syscalls required by the G1
//! acceptance script: LOOKUP, GETATTR, READDIR, OPEN, READ, READLINK, RELEASE.
//!
//! **Layer:** platform (FUSE binary; wraps `warp-drive-core` fixture tree).
//!
//! **Introduced at:** G1.
//!
//! **Requirements:**
//! - Linux: FUSE kernel module (shipped with the kernel on most distros).
//! - macOS: macFUSE — install with `cargo xtask install-deps`.
//!   Until macFUSE is installed the binary compiles (with the
//!   `compile-without-macfuse` feature) but `mount2` returns an error.

mod adapter;

use std::path::PathBuf;

use clap::Parser;
use warp_drive_core::FixtureTree;

use crate::adapter::FuseAdapter;

// ── CLI ─────────────────────────────────────────────────────────────────────

/// Supported runtime back-ends for the WARP DRIVE mount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Runtime {
    /// Hardcoded in-memory fixture tree — no persistence. G1 gate target.
    #[value(name = "in-memory")]
    InMemory,
}

/// WARP DRIVE FUSE mount (G1: in-memory fake tree).
///
/// Mounts a read-only POSIX filesystem backed by a hardcoded fixture tree.
/// Unmount with `cargo xtask unmount --path <dir>` (or `umount` / `fusermount -u`).
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Existing directory to use as the mount point.
    #[arg(long)]
    mount: PathBuf,

    /// Runtime back-end (`in-memory` is the only option at G1).
    #[arg(long, value_enum, default_value = "in-memory")]
    runtime: Runtime,
}

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    tracing::info!(mount = %cli.mount.display(), "WARP DRIVE G1 mounting");

    let mut config = fuser::Config::default();
    config.mount_options = vec![
        fuser::MountOption::RO,
        fuser::MountOption::FSName("warp-drive".to_owned()),
        fuser::MountOption::Subtype("warp-drive".to_owned()),
        fuser::MountOption::DefaultPermissions,
    ];

    fuser::mount2(FuseAdapter::new(FixtureTree::new()), &cli.mount, &config)?;

    tracing::info!("unmounted");
    Ok(())
}
