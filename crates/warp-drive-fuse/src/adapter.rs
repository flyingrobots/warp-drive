// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! FUSE filesystem adapter — bridges the `fuser` kernel interface to the
//! domain [`FixtureTree`].
//!
//! This module owns the seven read-path syscall implementations required by
//! the G1 acceptance script: LOOKUP, GETATTR, READLINK, OPEN, READ, READDIR,
//! RELEASE (release is handled by `fuser`'s default no-op).
//!
//! As of G3 the adapter also owns live diagnostics: it counts every syscall
//! it handles in a [`MountStats`], and serves `/.warp/stats` from a fresh
//! snapshot of those counters instead of static fixture bytes. This is the
//! narrowest layer that can count the syscalls honestly, and the only piece
//! of interior-mutable state in this crate — the fixture tree itself stays
//! immutable.
//!
//! **Layer:** platform (FUSE glue; depends on `warp-drive-core` only).

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::time::{Duration, SystemTime};

use fuser::{
    Errno, FileAttr, FileHandle, FileType, FopenFlags, Generation, INodeNo, LockOwner, OpenAccMode,
    OpenFlags, ReplyAttr, ReplyData, ReplyDirectory, ReplyEntry, ReplyOpen, Request,
};
use warp_drive_core::{FixtureTree, Ino, NodeContent, NodeKind, VirtualNode, WARP_STATS_INO};

use crate::stats::MountStats;

/// Kernel attribute cache TTL for ordinary (non-diagnostic) inodes.
///
/// The fixture tree is static, so a long TTL is safe. 1 s is conventional for
/// read-only FUSE filesystems that want normal `stat` caching.
const ATTR_TTL: Duration = Duration::from_secs(1);

/// Attribute-cache TTL for `/.warp/stats`.
///
/// Governs both `lookup()`'s entry reply and `getattr()`'s attribute reply —
/// `fuser` takes a single TTL argument for each, which serves both caches.
/// Zero forces the kernel to re-fetch attributes for this inode rather than
/// trusting a size that may be stale (paired with constant-width JSON
/// formatting in [`MountStats`], and with `FOPEN_DIRECT_IO` on `open()` for
/// the corresponding content-cache guarantee).
const STATS_TTL: Duration = Duration::ZERO;

/// FUSE filesystem adapter over a domain [`FixtureTree`], with live mount
/// diagnostics.
pub(crate) struct FuseAdapter {
    tree: FixtureTree,
    stats: MountStats,
}

impl FuseAdapter {
    /// Wrap a [`FixtureTree`] and its [`MountStats`] in the FUSE adapter.
    pub(crate) const fn new(tree: FixtureTree, stats: MountStats) -> Self {
        Self { tree, stats }
    }

    /// Return the stats-inode's backing node iff `ino` is `WARP_STATS_INO`
    /// *and* the fixture tree actually backs it with a regular-file sentinel.
    ///
    /// Centralizing this check means `lookup()`, `getattr()`, `read()`, and
    /// `open()`'s direct-I/O policy all agree on when live diagnostics apply
    /// — a guessed magic inode number must not succeed if the tree shape
    /// doesn't actually back it.
    fn live_stats_node(&self, ino: Ino) -> Option<&VirtualNode> {
        if ino != WARP_STATS_INO {
            return None;
        }
        self.tree
            .get(ino)
            .filter(|node| node.kind == NodeKind::RegularFile)
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

/// Build a fuser [`FileAttr`] from a domain [`VirtualNode`], overriding its
/// reported size.
///
/// Used both for ordinary nodes (`size == node.size()`) and for the live
/// stats inode, whose real content length varies from its stored sentinel
/// bytes.
fn node_attr_with_size(node: &VirtualNode, size: u64) -> FileAttr {
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
        size,
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

/// Build a fuser [`FileAttr`] from a domain [`VirtualNode`] using its own
/// reported size.
fn node_attr(node: &VirtualNode) -> FileAttr {
    node_attr_with_size(node, node.size())
}

/// Open-reply flags for a successfully opened inode.
///
/// The stats inode gets `FOPEN_DIRECT_IO` so the kernel page cache cannot
/// serve a stale `read()` on a repeat read within one open — the mechanism
/// that makes live counters actually live, not merely correctly-sized.
/// Every other inode keeps normal cached I/O.
const fn open_reply_flags(is_live_stats: bool) -> FopenFlags {
    if is_live_stats {
        FopenFlags::FOPEN_DIRECT_IO
    } else {
        FopenFlags::empty()
    }
}

/// Clamp `[offset, offset + size)` against `bytes`, returning the slice FUSE
/// should serve for a `read()` at that offset/size.
fn slice_bytes(bytes: &[u8], offset: u64, size: u32) -> &[u8] {
    let start = usize::try_from(offset)
        .unwrap_or(usize::MAX)
        .min(bytes.len());
    let end = start
        .saturating_add(usize::try_from(size).unwrap_or(usize::MAX))
        .min(bytes.len());
    &bytes[start..end]
}

/// Convert a 0-based entry index to the FUSE `next_offset` for that entry.
///
/// Returning `idx + 1` produces a dense 1-based offset sequence. The kernel
/// passes `next_offset` of the last consumed entry as `offset` in the following
/// `readdir` call, giving us a stable pagination cursor.
///
/// The overflow path is defensive only: real fixture entry counts are tiny, but
/// this saturates if a platform-sized index cannot be represented as `u64`.
fn next_offset(idx: usize) -> u64 {
    u64::try_from(idx.saturating_add(1)).unwrap_or(u64::MAX)
}

impl fuser::Filesystem for FuseAdapter {
    fn lookup(&self, _req: &Request, parent: INodeNo, name: &OsStr, reply: ReplyEntry) {
        self.stats.record_lookup();
        if let Some(node) = self.tree.lookup(Ino(parent.0), name.as_bytes()) {
            if let Some(stats_node) = self.live_stats_node(node.ino) {
                let size = self.stats.snapshot_json().len() as u64;
                reply.entry(
                    &STATS_TTL,
                    &node_attr_with_size(stats_node, size),
                    Generation(0),
                );
            } else {
                reply.entry(&ATTR_TTL, &node_attr(node), Generation(0));
            }
        } else {
            tracing::debug!(parent = parent.0, name = ?name, "lookup miss");
            reply.error(Errno::ENOENT);
        }
    }

    fn getattr(&self, _req: &Request, ino: INodeNo, _fh: Option<FileHandle>, reply: ReplyAttr) {
        self.stats.record_getattr();
        let ino = Ino(ino.0);
        if let Some(stats_node) = self.live_stats_node(ino) {
            let size = self.stats.snapshot_json().len() as u64;
            reply.attr(&STATS_TTL, &node_attr_with_size(stats_node, size));
            return;
        }
        match self.tree.get(ino) {
            Some(node) => reply.attr(&ATTR_TTL, &node_attr(node)),
            None => reply.error(Errno::ENOENT),
        }
    }

    fn readlink(&self, _req: &Request, ino: INodeNo, reply: ReplyData) {
        self.stats.record_readlink();
        let Some(node) = self.tree.get(Ino(ino.0)) else {
            reply.error(Errno::ENOENT);
            return;
        };
        match &node.content {
            NodeContent::Link(target) => reply.data(target),
            _ => reply.error(Errno::EINVAL),
        }
    }

    fn open(&self, _req: &Request, ino: INodeNo, flags: OpenFlags, reply: ReplyOpen) {
        self.stats.record_open();
        if flags.acc_mode() != OpenAccMode::O_RDONLY {
            tracing::debug!("write open rejected (read-only mount)");
            reply.error(Errno::EROFS);
            return;
        }
        let is_live_stats = self.live_stats_node(Ino(ino.0)).is_some();
        reply.opened(FileHandle(0), open_reply_flags(is_live_stats));
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
        let ino = Ino(ino.0);
        if self.live_stats_node(ino).is_some() {
            // Diagnostic self-read: intentionally does not bump read_count —
            // otherwise acceptance could pass by observing the counter
            // instead of proving that ordinary file reads are counted.
            let snapshot = self.stats.snapshot_json();
            reply.data(slice_bytes(&snapshot, offset, size));
            return;
        }
        self.stats.record_read();
        let Some(node) = self.tree.get(ino) else {
            reply.error(Errno::ENOENT);
            return;
        };
        let NodeContent::Bytes(bytes) = &node.content else {
            reply.error(Errno::EISDIR);
            return;
        };
        reply.data(slice_bytes(bytes, offset, size));
    }

    fn readdir(
        &self,
        _req: &Request,
        ino: INodeNo,
        _fh: FileHandle,
        offset: u64,
        mut reply: ReplyDirectory,
    ) {
        self.stats.record_readdir();
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
            let full = reply.add(
                INodeNo(ino.0),
                next_offset(idx),
                FileType::Directory,
                OsStr::new("."),
            );
            if full {
                reply.ok();
                return;
            }
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
            if full {
                reply.ok();
                return;
            }
        }
        idx += 1;

        // Named children
        for (child_ino, child_name) in children {
            if idx >= skip {
                let kind = self
                    .tree
                    .get(child_ino)
                    .map_or(FileType::RegularFile, |n| fuse_kind(n.kind));
                let full = reply.add(
                    INodeNo(child_ino.0),
                    next_offset(idx),
                    kind,
                    OsStr::from_bytes(child_name),
                );
                if full {
                    reply.ok();
                    return;
                }
            }
            idx += 1;
        }

        reply.ok();
    }
}

#[cfg(test)]
mod tests {
    use super::{open_reply_flags, slice_bytes};
    use fuser::FopenFlags;

    #[test]
    fn open_reply_flags_grants_direct_io_only_for_live_stats() {
        assert_eq!(open_reply_flags(true), FopenFlags::FOPEN_DIRECT_IO);
        assert_eq!(open_reply_flags(false), FopenFlags::empty());
    }

    #[test]
    fn slice_bytes_clamps_offset_and_size() {
        let bytes = b"0123456789";
        assert_eq!(slice_bytes(bytes, 0, 4), b"0123");
        assert_eq!(slice_bytes(bytes, 8, 10), b"89");
        assert_eq!(slice_bytes(bytes, 20, 5), b"");
        assert_eq!(slice_bytes(bytes, 3, 0), b"");
        assert_eq!(slice_bytes(bytes, 0, u32::MAX), bytes);
    }
}
