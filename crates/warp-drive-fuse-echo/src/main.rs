// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Local-only Echo rlib FUSE mount binary for G2/G3 gates.
//!
//! This package is excluded from the default workspace because it requires a
//! sibling `../echo-warp-drive` checkout. It intentionally builds a binary named
//! `warp-drive-fuse` so the acceptance script can exercise the same command
//! shape as the default G1 binary.

use std::path::PathBuf;

use clap::Parser;
use warp_drive_core::FixtureTree;
use warp_drive_fuse::{GateLabel, MountStats, RuntimeLabel};

/// Supported runtime back-ends for the local Echo gate binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Runtime {
    /// Hardcoded in-memory fixture tree.
    #[value(name = "in-memory")]
    InMemory,
    /// Embedded Echo rlib backend.
    #[value(name = "echo-rlib")]
    EchoRlib,
}

/// Echo gate behavior for the local-only binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Gate {
    /// Echo coordinate metadata over G1 fixture bytes.
    #[value(name = "g2a")]
    G2a,
    /// First Echo-projected regular-file bytes.
    #[value(name = "g2b")]
    G2b,
    /// Live `/.warp/` diagnostics and operation counters.
    #[value(name = "g3")]
    G3,
}

/// Local-only WARP DRIVE FUSE mount with Echo rlib support.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Existing directory to use as the mount point.
    #[arg(long)]
    mount: PathBuf,

    /// Runtime back-end.
    #[arg(long, value_enum, default_value = "echo-rlib")]
    runtime: Runtime,

    /// Echo gate behavior to mount.
    #[arg(long, value_enum, default_value = "g2a")]
    gate: Gate,
}

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    tracing::info!(
        mount = %cli.mount.display(),
        runtime = ?cli.runtime,
        gate = ?cli.gate,
        "WARP DRIVE local Echo gate mounting"
    );
    let (tree, stats) = fixture_tree(cli.runtime, cli.gate)?;

    warp_drive_fuse::mount_tree(tree, stats, &cli.mount)?;

    tracing::info!("unmounted");
    Ok(())
}

/// Build the fixture tree and live-diagnostics seed for `(runtime, gate)`.
///
/// Exhaustive on the pair — this binary only ever serves Echo-backed
/// mounts, so any `Runtime::InMemory` request is a routing error, not a
/// silently-ignored `gate` value.
fn fixture_tree(runtime: Runtime, gate: Gate) -> std::io::Result<(FixtureTree, MountStats)> {
    match (runtime, gate) {
        (Runtime::EchoRlib, Gate::G2a) => {
            let backend =
                warp_drive_echo_backend::EchoBackend::init().map_err(std::io::Error::other)?;
            let (tree, obs) = backend.into_parts();
            Ok((
                tree,
                MountStats::new(
                    GateLabel::G2a,
                    RuntimeLabel::EchoRlib,
                    obs.observe_count(),
                    obs.observe_error_count(),
                ),
            ))
        }
        (Runtime::EchoRlib, Gate::G2b) => {
            let backend =
                warp_drive_echo_backend::EchoBackend::init_g2b().map_err(std::io::Error::other)?;
            let (tree, obs) = backend.into_parts();
            Ok((
                tree,
                MountStats::new(
                    GateLabel::G2b,
                    RuntimeLabel::EchoRlib,
                    obs.observe_count(),
                    obs.observe_error_count(),
                ),
            ))
        }
        (Runtime::EchoRlib, Gate::G3) => {
            let backend =
                warp_drive_echo_backend::EchoBackend::init_g3().map_err(std::io::Error::other)?;
            let (tree, obs) = backend.into_parts();
            Ok((
                tree,
                MountStats::new(
                    GateLabel::G3,
                    RuntimeLabel::EchoRlib,
                    obs.observe_count(),
                    obs.observe_error_count(),
                ),
            ))
        }
        (Runtime::InMemory, _) => Err(std::io::Error::other(
            "the Echo gate binary does not serve in-memory mounts; use the workspace warp-drive-fuse binary",
        )),
    }
}
