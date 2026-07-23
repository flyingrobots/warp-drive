// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! WARP DRIVE FUSE mount binary (in-memory gate runner: G1, G3).
//!
//! Mounts a read-only POSIX filesystem backed by a cached [`FixtureTree`],
//! plus live `/.warp/stats` diagnostics as of G3. Implements the 7 syscalls
//! required by the gate acceptance scripts: LOOKUP, GETATTR, READDIR, OPEN,
//! READ, READLINK, RELEASE.
//!
//! G2a/G2b/G3-echo Echo metadata acceptance uses the excluded
//! `warp-drive-fuse-echo` binary through `cargo xtask acceptance --runtime
//! echo-rlib`.
//!
//! **Layer:** platform (FUSE binary; wraps `warp-drive-core` fixture tree).
//!
//! **Introduced at:** G1. Gate selection added at G3.
//!
//! **Requirements:**
//! - Linux: FUSE kernel module (shipped with the kernel on most distros).
//! - macOS: macFUSE — install with `cargo xtask install-deps`.
//!   Until macFUSE is installed the binary compiles (with the
//!   `compile-without-macfuse` feature) but `mount2` returns an error.

use std::path::PathBuf;

use clap::Parser;
use warp_drive_core::FixtureTree;
use warp_drive_fuse::{GateLabel, MountStats, RuntimeLabel};

// ── CLI ─────────────────────────────────────────────────────────────────────

/// Gates this binary can serve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Gate {
    /// POSIX translation over the static in-memory fixture tree.
    #[value(name = "g1")]
    G1,
    /// Live `/.warp/` diagnostics and operation counters.
    #[value(name = "g3")]
    G3,
}

/// WARP DRIVE FUSE mount.
///
/// Mounts a read-only in-memory POSIX filesystem. Unmount with
/// `cargo xtask unmount --path <dir>` (or `umount` / `fusermount -u`).
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Existing directory to use as the mount point.
    #[arg(long)]
    mount: PathBuf,

    /// Gate to serve.
    #[arg(long, value_enum, default_value = "g1")]
    gate: Gate,
}

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    tracing::info!(
        mount = %cli.mount.display(),
        gate = ?cli.gate,
        "WARP DRIVE mounting"
    );
    let (tree, stats) = fixture_tree(cli.gate)?;

    warp_drive_fuse::mount_tree(tree, stats, &cli.mount)?;

    tracing::info!("unmounted");
    Ok(())
}

fn fixture_tree(gate: Gate) -> std::io::Result<(FixtureTree, MountStats)> {
    match gate {
        Gate::G1 => Ok((
            FixtureTree::new(),
            MountStats::new(GateLabel::G1, RuntimeLabel::InMemory, 0, 0),
        )),
        Gate::G3 => {
            let tree = FixtureTree::with_warp_metadata(
                g3_coordinate_json().into_bytes(),
                g3_runtime_json().into_bytes(),
            )
            .map_err(std::io::Error::other)?;
            Ok((
                tree,
                MountStats::new(GateLabel::G3, RuntimeLabel::InMemory, 0, 0),
            ))
        }
    }
}

/// Placeholder `/.warp/coordinate` content for the in-memory G3 mount — no
/// Echo backend is involved, so this is a genesis placeholder labeled for
/// the active gate, matching the shape the G1 fixture already used.
fn g3_coordinate_json() -> String {
    "{\"worldline\":\"00000000-0000-0000-0000-000000000001\",\
      \"frontier\":\"genesis\",\
      \"gate\":\"G3\"}\n"
        .to_owned()
}

/// `/.warp/runtime` content for the in-memory G3 mount, per the G3 design
/// doc's required shape.
fn g3_runtime_json() -> String {
    format!(
        "{{\"gate\":\"G3\",\"runtime\":\"in-memory\",\"driver\":\"warp-drive-driver-memory\",\
          \"build_mode\":\"{}\",\"stats\":\"live\",\"schema_version\":{}}}\n",
        warp_drive_core::build_mode(),
        warp_drive_core::WARP_DIAGNOSTICS_SCHEMA_VERSION
    )
}
