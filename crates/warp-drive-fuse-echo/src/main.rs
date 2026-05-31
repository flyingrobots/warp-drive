// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Local-only Echo rlib FUSE mount binary for the G2a gate.
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
    /// Embedded Echo rlib coordinate metadata over G1 fixture bytes. G2a target.
    #[value(name = "echo-rlib")]
    EchoRlib,
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
}

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    tracing::info!(
        mount = %cli.mount.display(),
        runtime = ?cli.runtime,
        "WARP DRIVE local Echo gate mounting"
    );
    let tree = fixture_tree(cli.runtime)?;

    warp_drive_fuse::mount_tree(tree, &cli.mount)?;

    tracing::info!("unmounted");
    Ok(())
}

fn fixture_tree(runtime: Runtime) -> std::io::Result<FixtureTree> {
    match runtime {
        Runtime::InMemory => Ok(FixtureTree::new()),
        Runtime::EchoRlib => {
            let backend =
                warp_drive_echo_backend::EchoBackend::init().map_err(std::io::Error::other)?;
            Ok(backend.into_tree())
        }
    }
}
