// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Local-only Echo rlib FUSE mount binary for G2 gates.
//!
//! This package is excluded from the default workspace because it requires a
//! sibling `../echo-warp-drive` checkout. It intentionally builds a binary named
//! `warp-drive-fuse` so the acceptance script can exercise the same command
//! shape as the default G1 binary.

use std::path::PathBuf;

use clap::Parser;
use warp_drive_core::FixtureTree;

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
    let tree = fixture_tree(cli.runtime, cli.gate)?;

    warp_drive_fuse::mount_tree(tree, &cli.mount)?;

    tracing::info!("unmounted");
    Ok(())
}

fn fixture_tree(runtime: Runtime, gate: Gate) -> std::io::Result<FixtureTree> {
    match runtime {
        Runtime::InMemory => Ok(FixtureTree::new()),
        Runtime::EchoRlib if gate == Gate::G2a => {
            let backend =
                warp_drive_echo_backend::EchoBackend::init().map_err(std::io::Error::other)?;
            Ok(backend.into_tree())
        }
        Runtime::EchoRlib => {
            let backend =
                warp_drive_echo_backend::EchoBackend::init_g2b().map_err(std::io::Error::other)?;
            Ok(backend.into_tree())
        }
    }
}
