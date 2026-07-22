// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Shared FUSE mounting surface for WARP DRIVE binaries.
//!
//! The default workspace binary mounts the in-memory G1 fixture. Local-only
//! Echo gate binaries can reuse this module without adding Echo path
//! dependencies to the default workspace.

mod adapter;
mod stats;

use std::path::Path;

use warp_drive_core::FixtureTree;

use crate::adapter::FuseAdapter;
pub use crate::stats::{GateLabel, MountStats, RuntimeLabel};

/// Mount `tree` as a read-only WARP DRIVE FUSE filesystem, serving live
/// diagnostics from `stats` at `/.warp/stats`.
///
/// # Errors
///
/// Returns any error produced by `fuser::mount2`, including missing FUSE
/// support or mount permission failures.
pub fn mount_tree(tree: FixtureTree, stats: MountStats, mount: &Path) -> std::io::Result<()> {
    let mut config = fuser::Config::default();
    config.mount_options = vec![
        fuser::MountOption::RO,
        fuser::MountOption::FSName("warp-drive".to_owned()),
        fuser::MountOption::Subtype("warp-drive".to_owned()),
        fuser::MountOption::DefaultPermissions,
    ];

    fuser::mount2(FuseAdapter::new(tree, stats), mount, &config)
}
