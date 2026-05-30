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
//! - macOS: macFUSE — install with `brew install --cask macfuse`.
//!   Until macFUSE is installed the binary compiles but `mount2` returns an error.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use clap::Parser;
use fuser::{
    Errno, FileAttr, FileHandle, FileType, FopenFlags, Generation, INodeNo,
    LockOwner, OpenAccMode, OpenFlags, ReplyAttr, ReplyData, ReplyDirectory,
    ReplyEntry, ReplyOpen, Request,
};
use warp_drive_core::{FixtureTree, Ino, NodeContent, NodeKind, VirtualNode};

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
/// Unmount with `umount <path>` (macOS) or `fusermount -u <path>` (Linux).
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

// ── FUSE adapter ─────────────────────────────────────────────────────────────

/// Kernel attribute cache TTL.
///
/// The fixture tree is static, so a long TTL is safe. 1 s is conventional for
/// read-only FUSE filesystems that want normal `stat` caching.
const ATTR_TTL: Duration = Duration::from_secs(1);

/// FUSE filesystem adapter for the G1 in-memory fixture tree.
///
/// All methods take `&self` — the fixture tree is immutable once constructed,
/// so no interior mutability is required at G1.
struct FuseAdapter {
    tree: FixtureTree,
}

impl FuseAdapter {
    const fn new(tree: FixtureTree) -> Self {
        Self { tree }
    }
}

/// Map a domain [`NodeKind`] to a fuser [`FileType`].
const fn fuse_kind(kind: NodeKind) -> FileType {
    match kind {
        NodeKind::RegularFile => FileType::RegularFile,
        NodeKind::Directory => FileType::Directory,
        NodeKind::Symlink => FileType::Symlink,
    }
}

/// Build a fuser [`FileAttr`] from a domain [`VirtualNode`].
const fn node_attr(node: &VirtualNode) -> FileAttr {
    let kind = fuse_kind(node.kind);
    let perm: u16 = match node.kind {
        NodeKind::Directory => 0o555,
        NodeKind::RegularFile => 0o444,
        NodeKind::Symlink => 0o777,
    };
    let nlink: u32 = match node.kind {
        NodeKind::Directory => 2,
        NodeKind::RegularFile | NodeKind::Symlink => 1,
    };
    FileAttr {
        ino: INodeNo(node.ino.0),
        size: node.size(),
        blocks: 0,
        atime: SystemTime::UNIX_EPOCH,
        mtime: SystemTime::UNIX_EPOCH,
        ctime: SystemTime::UNIX_EPOCH,
        crtime: SystemTime::UNIX_EPOCH,
        kind,
        perm,
        nlink,
        uid: 0,
        gid: 0,
        rdev: 0,
        blksize: 4096,
        flags: 0,
    }
}

/// Convert a 0-based entry index to the FUSE `next_offset` for that entry.
///
/// Returning `idx + 1` produces a dense 1-based offset sequence. The kernel
/// passes `next_offset` of the last consumed entry as `offset` in the following
/// `readdir` call, giving us a stable pagination cursor.
#[allow(clippy::cast_possible_truncation)]
const fn next_offset(idx: usize) -> u64 {
    // idx is bounded by the fixture size (15 entries max including . and ..);
    // truncation is impossible on any platform where usize is at least 4 bits.
    (idx + 1) as u64
}

impl fuser::Filesystem for FuseAdapter {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        if let Some(node) = self.tree.lookup(Ino(parent.0), name.as_bytes()) {
            reply.entry(&ATTR_TTL, &node_attr(node), Generation(0));
        } else {
            tracing::debug!(parent = parent.0, name = ?name, "lookup miss");
            reply.error(Errno::ENOENT);
        }
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        match self.tree.get(Ino(ino.0)) {
            Some(node) => reply.attr(&ATTR_TTL, &node_attr(node)),
            None => reply.error(Errno::ENOENT),
        }
    }

    fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
        let Some(node) = self.tree.get(Ino(ino.0)) else {
            reply.error(Errno::ENOENT);
            return;
        };
        match &node.content {
            NodeContent::Link(target) => reply.data(target),
            _ => reply.error(Errno::EINVAL),
        }
    }

    fn open(&self, _req: &Request, _ino: INodeNo, flags: OpenFlags, reply: ReplyOpen) {
        if flags.acc_mode() != OpenAccMode::O_RDONLY {
            tracing::debug!("write open rejected (read-only mount)");
            reply.error(Errno::EROFS);
            return;
        }
        reply.opened(FileHandle(0), FopenFlags::empty());
    }

    fn read(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        size: u32,
        _flags: OpenFlags,
        _lock_owner: Option<LockOwner>,
        reply: ReplyData,
    ) {
        let Some(node) = self.tree.get(Ino(ino.0)) else {
            reply.error(Errno::ENOENT);
            return;
        };
        let NodeContent::Bytes(bytes) = &node.content else {
            reply.error(Errno::EISDIR);
            return;
        };
        let start = usize::try_from(offset)
            .unwrap_or(usize::MAX)
            .min(bytes.len());
        let end = start
            .saturating_add(usize::try_from(size).unwrap_or(usize::MAX))
            .min(bytes.len());
        reply.data(&bytes[start..end]);
    }

    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        let ino = Ino(ino.0);
        let Some(node) = self.tree.get(ino) else {
            reply.error(Errno::ENOENT);
            return;
        };
        let Some(children) = self.tree.readdir_entries(ino) else {
            reply.error(Errno::ENOTDIR);
            return;
        };

        // FUSE offset 0 = start from the beginning. We assign:
        //   . → next_offset 1, .. → next_offset 2, children → 3+.
        let skip = usize::try_from(offset).unwrap_or(usize::MAX);
        let mut idx: usize = 0;

        // . (self)
        if idx >= skip {
            let full = reply.add(INodeNo(ino.0), next_offset(idx), FileType::Directory, OsStr::new("."));
            if full { reply.ok(); return; }
        }
        idx += 1;

        // .. (parent)
        if idx >= skip {
            let full = reply.add(
                INodeNo(node.parent_ino.0),
                next_offset(idx),
                FileType::Directory,
                OsStr::new(".."),
            );
            if full { reply.ok(); return; }
        }
        idx += 1;

        // Named children
        for (child_ino, child_name) in children {
            if idx >= skip {
                let kind = self.tree.get(*child_ino)
                    .map_or(FileType::RegularFile, |n| fuse_kind(n.kind));
                let full = reply.add(
                    INodeNo(child_ino.0),
                    next_offset(idx),
                    kind,
                    OsStr::from_bytes(child_name),
                );
                if full { reply.ok(); return; }
            }
            idx += 1;
        }

        reply.ok();
    }
}
