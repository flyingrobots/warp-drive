// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! WARP DRIVE FUSE mount binary (G1/G2a gate runner).
//!
//! Mounts a read-only POSIX filesystem backed by a cached [`FixtureTree`].
//! Implements the 7 syscalls required by the gate acceptance scripts: LOOKUP,
//! GETATTR, READDIR, OPEN, READ, READLINK, RELEASE.
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

use std::path::PathBuf;

use clap::Parser;
use warp_drive_core::FixtureTree;

// ── CLI ─────────────────────────────────────────────────────────────────────

/// Supported runtime back-ends for the WARP DRIVE mount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Runtime {
    /// Hardcoded in-memory fixture tree — no persistence. G1 gate target.
    #[value(name = "in-memory")]
    InMemory,
}

/// WARP DRIVE FUSE mount.
///
/// Mounts a read-only POSIX filesystem backed by the selected runtime.
/// Unmount with `cargo xtask unmount --path <dir>` (or `umount` / `fusermount -u`).
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Existing directory to use as the mount point.
    #[arg(long)]
    mount: PathBuf,

    /// Runtime back-end.
    #[arg(long, value_enum, default_value = "in-memory")]
    runtime: Runtime,
}

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    tracing::info!(
        mount = %cli.mount.display(),
        runtime = ?cli.runtime,
        "WARP DRIVE mounting"
    );
    let tree = fixture_tree(cli.runtime)?;

    warp_drive_fuse::mount_tree(tree, &cli.mount)?;

    tracing::info!("unmounted");
    Ok(())
}

fn fixture_tree(runtime: Runtime) -> std::io::Result<FixtureTree> {
    match runtime {
        Runtime::InMemory => Ok(FixtureTree::new()),
    }
}
