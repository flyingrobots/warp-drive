// SPDX-License-Identifier: Apache-2.0
// © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>

//! Domain types and hardcoded fixture tree for the G1 gate.
//!
//! **What this crate owns:** virtual filesystem node model; the G1 fixture tree.
//!
//! **What this crate must not know:** FUSE, libc, OS paths, Echo, sockets,
//! environment variables, wall-clock time, or async runtimes.
//!
//! **Layer:** core.
//!
//! **Introduced at:** G1.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// A stable inode number.
///
/// FUSE reserves inode `1` for the root directory. All other inodes in the
/// G1 fixture are statically assigned and will never change for the same
/// logical node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ino(pub u64);

/// Root inode — always 1 per FUSE convention.
pub const ROOT_INO: Ino = Ino(1);

/// Kind of virtual filesystem node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// A regular file with byte content.
    RegularFile,
    /// A directory containing named child nodes.
    Directory,
    /// A symbolic link with a byte-string target.
    Symlink,
}

/// Content payload for a virtual node.
pub enum NodeContent {
    /// Byte content (regular files). Owned to allow dynamic content at G2+.
    Bytes(Vec<u8>),
    /// Named children of a directory: `(child_ino, name_bytes)`.
    Children(Vec<(Ino, Vec<u8>)>),
    /// Symlink target as raw bytes (not assumed to be UTF-8).
    Link(Vec<u8>),
}

impl fmt::Debug for NodeContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const DEBUG_PREVIEW_BYTES: usize = 64;

        match self {
            Self::Bytes(bytes) => {
                let preview_len = bytes.len().min(DEBUG_PREVIEW_BYTES);
                f.debug_struct("Bytes")
                    .field("len", &bytes.len())
                    .field("preview", &&bytes[..preview_len])
                    .field("truncated", &(preview_len < bytes.len()))
                    .finish()
            }
            Self::Children(children) => f.debug_tuple("Children").field(children).finish(),
            Self::Link(target) => f.debug_tuple("Link").field(target).finish(),
        }
    }
}

/// A single node in the virtual filesystem.
#[derive(Debug)]
pub struct VirtualNode {
    /// Stable inode number.
    pub ino: Ino,
    /// Inode of the parent directory. For the root, parent is itself.
    pub parent_ino: Ino,
    /// Node type.
    pub kind: NodeKind,
    /// Node content.
    pub content: NodeContent,
}

impl VirtualNode {
    /// Byte size of this node's content.
    ///
    /// For directories this returns `0`; FUSE computes directory size from
    /// the kernel's readdir results, not from a size field.
    #[must_use]
    pub fn size(&self) -> u64 {
        match &self.content {
            NodeContent::Bytes(b) => b.len() as u64,
            NodeContent::Children(_) => 0,
            NodeContent::Link(l) => l.len() as u64,
        }
    }
}

/// The complete virtual filesystem tree for the G1 gate.
///
/// Inode assignment:
///
/// ```text
///  1 = /                      (root directory)
///  2 = /README.md
///  3 = /package.json
///  4 = /src/
///  5 = /src/main.ts
///  6 = /src/lib.ts
///  7 = /empty/
///  8 = /links/
///  9 = /links/readme          (symlink → ../README.md)
/// 10 = /.warp/
/// 11 = /.warp/coordinate
/// 12 = /.warp/runtime
/// 13 = /.warp/stats
/// ```
///
/// G2b projected-file extension:
///
/// ```text
/// 14 = /echo/
/// 15 = /echo/head.json
/// ```
pub struct FixtureTree {
    nodes: HashMap<Ino, VirtualNode>,
}

/// Errors produced while constructing fixed fixture variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureTreeError {
    /// A required metadata inode is missing from the hardcoded fixture.
    MissingMetadataInode(Ino),
    /// A required metadata inode exists but is not a regular file.
    MetadataInodeNotFile(Ino),
    /// A required directory inode is missing from the hardcoded fixture.
    MissingDirectoryInode(Ino),
    /// A required directory inode exists but is not a directory.
    DirectoryInodeNotDirectory(Ino),
    /// A projected fixture extension tried to reuse an existing inode.
    DuplicateFixtureInode(Ino),
    /// A projected fixture extension tried to reuse an existing child name.
    DuplicateDirectoryEntry {
        /// Parent directory that already contains the child name.
        parent: Ino,
        /// Duplicate child name.
        name: &'static str,
    },
}

impl fmt::Display for FixtureTreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMetadataInode(ino) => {
                write!(f, "fixture metadata inode {} is missing", ino.0)
            }
            Self::MetadataInodeNotFile(ino) => {
                write!(f, "fixture metadata inode {} is not a regular file", ino.0)
            }
            Self::MissingDirectoryInode(ino) => {
                write!(f, "fixture directory inode {} is missing", ino.0)
            }
            Self::DirectoryInodeNotDirectory(ino) => {
                write!(f, "fixture directory inode {} is not a directory", ino.0)
            }
            Self::DuplicateFixtureInode(ino) => {
                write!(f, "fixture extension inode {} already exists", ino.0)
            }
            Self::DuplicateDirectoryEntry { parent, name } => {
                write!(
                    f,
                    "fixture directory inode {} already has child entry {name}",
                    parent.0
                )
            }
        }
    }
}

impl Error for FixtureTreeError {}

impl FixtureTree {
    /// Construct the hardcoded G1 fixture tree.
    ///
    /// # Why the `too_many_lines` allow
    ///
    /// The 13-node fixture requires 13 explicit `nodes.insert(…)` blocks. Splitting
    /// them into sub-functions would add abstraction with no modularity benefit —
    /// the verbosity is inherent to the static definition, not a design smell.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new() -> Self {
        let mut nodes: HashMap<Ino, VirtualNode> = HashMap::new();

        // ── /README.md (ino 2) ───────────────────────────────────────────────
        nodes.insert(
            Ino(2),
            VirtualNode {
                ino: Ino(2),
                parent_ino: ROOT_INO,
                kind: NodeKind::RegularFile,
                content: NodeContent::Bytes(
                    b"# WARP DRIVE G1 Fixture\n\
                  A minimal fake tree for proving the POSIX translation layer.\n"
                        .to_vec(),
                ),
            },
        );

        // ── /package.json (ino 3) ────────────────────────────────────────────
        nodes.insert(
            Ino(3),
            VirtualNode {
                ino: Ino(3),
                parent_ino: ROOT_INO,
                kind: NodeKind::RegularFile,
                content: NodeContent::Bytes(
                    b"{\n  \"name\": \"warp-drive-g1\",\n  \"version\": \"0.0.1\"\n}\n".to_vec(),
                ),
            },
        );

        // ── /src/main.ts (ino 5) ─────────────────────────────────────────────
        nodes.insert(
            Ino(5),
            VirtualNode {
                ino: Ino(5),
                parent_ino: Ino(4),
                kind: NodeKind::RegularFile,
                content: NodeContent::Bytes(
                    b"export function main(): void {\n\
                  \x20\x20console.log(\"hello from warp-drive G1 fixture\");\n\
                  }\n"
                    .to_vec(),
                ),
            },
        );

        // ── /src/lib.ts (ino 6) ──────────────────────────────────────────────
        nodes.insert(
            Ino(6),
            VirtualNode {
                ino: Ino(6),
                parent_ino: Ino(4),
                kind: NodeKind::RegularFile,
                content: NodeContent::Bytes(
                    b"export function identity<T>(x: T): T {\n\
                  \x20\x20return x;\n\
                  }\n"
                    .to_vec(),
                ),
            },
        );

        // ── /src/ (ino 4) ────────────────────────────────────────────────────
        nodes.insert(
            Ino(4),
            VirtualNode {
                ino: Ino(4),
                parent_ino: ROOT_INO,
                kind: NodeKind::Directory,
                content: NodeContent::Children(vec![
                    (Ino(5), b"main.ts".to_vec()),
                    (Ino(6), b"lib.ts".to_vec()),
                ]),
            },
        );

        // ── /empty/ (ino 7) ──────────────────────────────────────────────────
        nodes.insert(
            Ino(7),
            VirtualNode {
                ino: Ino(7),
                parent_ino: ROOT_INO,
                kind: NodeKind::Directory,
                content: NodeContent::Children(vec![]),
            },
        );

        // ── /links/readme (ino 9, symlink → ../README.md) ────────────────────
        nodes.insert(
            Ino(9),
            VirtualNode {
                ino: Ino(9),
                parent_ino: Ino(8),
                kind: NodeKind::Symlink,
                content: NodeContent::Link(b"../README.md".to_vec()),
            },
        );

        // ── /links/ (ino 8) ──────────────────────────────────────────────────
        nodes.insert(
            Ino(8),
            VirtualNode {
                ino: Ino(8),
                parent_ino: ROOT_INO,
                kind: NodeKind::Directory,
                content: NodeContent::Children(vec![(Ino(9), b"readme".to_vec())]),
            },
        );

        // ── /.warp/coordinate (ino 11) ───────────────────────────────────────
        nodes.insert(
            Ino(11),
            VirtualNode {
                ino: Ino(11),
                parent_ino: Ino(10),
                kind: NodeKind::RegularFile,
                content: NodeContent::Bytes(
                    b"{\"worldline\":\"00000000-0000-0000-0000-000000000001\",\
                  \"frontier\":\"genesis\",\
                  \"gate\":\"G1\"}\n"
                        .to_vec(),
                ),
            },
        );

        // ── /.warp/runtime (ino 12) ──────────────────────────────────────────
        nodes.insert(
            Ino(12),
            VirtualNode {
                ino: Ino(12),
                parent_ino: Ino(10),
                kind: NodeKind::RegularFile,
                content: NodeContent::Bytes(
                    b"{\"kind\":\"in-memory\",\
                  \"driver\":\"warp-drive-driver-memory\",\
                  \"gate\":\"G1\"}\n"
                        .to_vec(),
                ),
            },
        );

        // ── /.warp/stats (ino 13) ────────────────────────────────────────────
        // Static placeholder at G1. Live atomic counters arrive at G3.
        nodes.insert(
            Ino(13),
            VirtualNode {
                ino: Ino(13),
                parent_ino: Ino(10),
                kind: NodeKind::RegularFile,
                content: NodeContent::Bytes(
                    b"{\
                  \"gate\":\"G1\",\
                  \"status\":\"static-placeholder\",\
                  \"note\":\"live counters arrive at G3\",\
                  \"lookup_count\":0,\
                  \"getattr_count\":0,\
                  \"readdir_count\":0,\
                  \"open_count\":0,\
                  \"read_count\":0,\
                  \"readlink_count\":0\
                  }\n"
                    .to_vec(),
                ),
            },
        );

        // ── /.warp/ (ino 10) ─────────────────────────────────────────────────
        nodes.insert(
            Ino(10),
            VirtualNode {
                ino: Ino(10),
                parent_ino: ROOT_INO,
                kind: NodeKind::Directory,
                content: NodeContent::Children(vec![
                    (Ino(11), b"coordinate".to_vec()),
                    (Ino(12), b"runtime".to_vec()),
                    (Ino(13), b"stats".to_vec()),
                ]),
            },
        );

        // ── / (ino 1) ────────────────────────────────────────────────────────
        nodes.insert(
            ROOT_INO,
            VirtualNode {
                ino: ROOT_INO,
                parent_ino: ROOT_INO,
                kind: NodeKind::Directory,
                content: NodeContent::Children(vec![
                    (Ino(2), b"README.md".to_vec()),
                    (Ino(3), b"package.json".to_vec()),
                    (Ino(4), b"src".to_vec()),
                    (Ino(7), b"empty".to_vec()),
                    (Ino(8), b"links".to_vec()),
                    (Ino(10), b".warp".to_vec()),
                ]),
            },
        );

        Self { nodes }
    }

    /// Construct the G1 fixture tree with caller-provided `.warp/` file bytes.
    ///
    /// This is the G2a bridge: the POSIX tree stays seeded from the G1 fixture,
    /// while runtime metadata files can be produced by an external backend.
    ///
    /// # Errors
    ///
    /// Returns [`FixtureTreeError`] if the hardcoded `.warp/` metadata inodes
    /// are missing or no longer point at regular files.
    pub fn with_warp_metadata(
        coordinate: Vec<u8>,
        runtime: Vec<u8>,
        stats: Vec<u8>,
    ) -> Result<Self, FixtureTreeError> {
        let mut tree = Self::new();
        tree.replace_file_bytes(Ino(11), coordinate)?;
        tree.replace_file_bytes(Ino(12), runtime)?;
        tree.replace_file_bytes(Ino(13), stats)?;
        Ok(tree)
    }

    /// Construct the G1 fixture tree with dynamic `.warp/` metadata and one
    /// Echo-projected regular file at `/echo/head.json`.
    ///
    /// This is the G2b bridge: most POSIX-visible files remain fixture-backed,
    /// but `/echo/head.json` is supplied by the Echo projection path.
    ///
    /// # Errors
    ///
    /// Returns [`FixtureTreeError`] if required fixture inodes are missing,
    /// have the wrong kind, or the projected file extension would collide with
    /// an existing inode/name.
    pub fn with_warp_metadata_and_echo_head_file(
        coordinate: Vec<u8>,
        runtime: Vec<u8>,
        stats: Vec<u8>,
        echo_head_json: Vec<u8>,
    ) -> Result<Self, FixtureTreeError> {
        let mut tree = Self::with_warp_metadata(coordinate, runtime, stats)?;
        tree.insert_echo_head_file(echo_head_json)?;
        Ok(tree)
    }

    /// Look up a child of `parent` by raw name bytes.
    ///
    /// Returns `None` if `parent` does not exist, is not a directory, or has
    /// no child with the given name.
    #[must_use]
    pub fn lookup(&self, parent: Ino, name: &[u8]) -> Option<&VirtualNode> {
        let parent_node = self.nodes.get(&parent)?;
        let NodeContent::Children(children) = &parent_node.content else {
            return None;
        };
        for (child_ino, child_name) in children {
            if *child_name == name {
                return self.nodes.get(child_ino);
            }
        }
        None
    }

    /// Get a node by inode number.
    #[must_use]
    pub fn get(&self, ino: Ino) -> Option<&VirtualNode> {
        self.nodes.get(&ino)
    }

    /// Return the ordered child entries of a directory node.
    ///
    /// Each entry is `(child_ino, name_bytes)`. Returns `None` if `ino` does
    /// not exist or is not a directory.
    #[must_use]
    pub fn readdir_entries(&self, ino: Ino) -> Option<impl Iterator<Item = (Ino, &[u8])> + '_> {
        match &self.nodes.get(&ino)?.content {
            NodeContent::Children(c) => Some(
                c.iter()
                    .map(|(child_ino, name)| (*child_ino, name.as_slice())),
            ),
            _ => None,
        }
    }

    fn replace_file_bytes(&mut self, ino: Ino, bytes: Vec<u8>) -> Result<(), FixtureTreeError> {
        let Some(node) = self.nodes.get_mut(&ino) else {
            return Err(FixtureTreeError::MissingMetadataInode(ino));
        };
        if node.kind != NodeKind::RegularFile {
            return Err(FixtureTreeError::MetadataInodeNotFile(ino));
        }
        node.content = NodeContent::Bytes(bytes);
        Ok(())
    }

    fn insert_echo_head_file(&mut self, bytes: Vec<u8>) -> Result<(), FixtureTreeError> {
        const ECHO_DIR_INO: Ino = Ino(14);
        const ECHO_HEAD_INO: Ino = Ino(15);

        if self.nodes.contains_key(&ECHO_DIR_INO) {
            return Err(FixtureTreeError::DuplicateFixtureInode(ECHO_DIR_INO));
        }
        if self.nodes.contains_key(&ECHO_HEAD_INO) {
            return Err(FixtureTreeError::DuplicateFixtureInode(ECHO_HEAD_INO));
        }

        let Some(root) = self.nodes.get_mut(&ROOT_INO) else {
            return Err(FixtureTreeError::MissingDirectoryInode(ROOT_INO));
        };
        let NodeContent::Children(root_children) = &mut root.content else {
            return Err(FixtureTreeError::DirectoryInodeNotDirectory(ROOT_INO));
        };
        if root_children
            .iter()
            .any(|(_, name)| name.as_slice() == b"echo")
        {
            return Err(FixtureTreeError::DuplicateDirectoryEntry {
                parent: ROOT_INO,
                name: "echo",
            });
        }
        root_children.push((ECHO_DIR_INO, b"echo".to_vec()));

        self.nodes.insert(
            ECHO_DIR_INO,
            VirtualNode {
                ino: ECHO_DIR_INO,
                parent_ino: ROOT_INO,
                kind: NodeKind::Directory,
                content: NodeContent::Children(vec![(ECHO_HEAD_INO, b"head.json".to_vec())]),
            },
        );
        self.nodes.insert(
            ECHO_HEAD_INO,
            VirtualNode {
                ino: ECHO_HEAD_INO,
                parent_ino: ECHO_DIR_INO,
                kind: NodeKind::RegularFile,
                content: NodeContent::Bytes(bytes),
            },
        );

        Ok(())
    }
}

impl Default for FixtureTree {
    fn default() -> Self {
        Self::new()
    }
}
