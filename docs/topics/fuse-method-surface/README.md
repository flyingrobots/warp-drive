<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# The `fuser::Filesystem` method surface

**Status:** living reference, current as of `main`.
**Established by:** [`docs/gates/G1.md`](../../gates/G1.md) (the original
seven read-path methods). Extended by the 2026-07-22 backlog-crunch PR
(closing issue #23) to make every genuinely-silent `fuser` default
explicit. Will be extended again by G4a — see
[`docs/design/g4a-intent-admission-receipts.md`](../../design/g4a-intent-admission-receipts.md).
**Test plan:** [`test-plan.md`](test-plan.md).

`FuseAdapter` (`crates/warp-drive-fuse/src/adapter.rs`) implements
`fuser::Filesystem`. That trait has ~30 methods with default
implementations; this page tracks which ones this adapter overrides,
which ones it deliberately leaves at their `fuser` default, and why — so
nobody has to re-derive that split by reading `fuser`'s source every time
a new gate needs to touch write-adjacent behavior.

## Overridden with real, read-path logic

Established at G1, unchanged in shape since (G3 added the live-stats
branch inside several of these, not a new method):

| Method | What it does |
| --- | --- |
| `lookup` | Resolves a child by name against `FixtureTree`; `ENOENT` on miss. |
| `getattr` | Returns real `FileAttr` for an existing inode; `ENOENT` on miss. |
| `readlink` | Returns symlink target bytes; `EINVAL` if the inode isn't a symlink. |
| `open` | Rejects any non-read-only open with `EROFS`; otherwise grants a stateless handle. |
| `read` | Returns real byte content (or a live `/.warp/stats` snapshot — see the [`.warp/` diagnostics topic](../dotwarp-diagnostics/README.md)). |
| `readdir` | Returns real child entries plus `.`/`..`. |

## Overridden, explicit-and-documented, behaviorally a no-op today

Added by the 2026-07-22 backlog crunch. `fuser`'s own defaults for these
four already succeed silently and do nothing — that's not a bug in
`fuser`, it's the correct default for a filesystem with no stateful
handles or a real backing store, which describes this mount *today*. The
override exists so that fact is a written decision instead of an
unexamined inheritance, and so each has a place to say what changes it:

| Method | Current behavior | What would make this stop being a no-op |
| --- | --- | --- |
| `release` | `reply.ok()` — nothing to release, `open()` never allocates per-handle state. | G4a: file-handle-scoped staged-write state needs real cleanup here. |
| `opendir` | `reply.opened(FileHandle(0), …)` unconditionally — mirrors `open()`'s pattern; existence/kind validation is `readdir()`'s job, not this method's. | A future gate that gives directory handles their own state (e.g. a cursor). |
| `releasedir` | `reply.ok()` — same rationale as `release`. | Same trigger as `opendir`. |
| `statfs` | Reports the real fixture node count (`FixtureTree::node_count()`) for `files`; `blocks`/`bfree`/`bavail` are genuinely `0` — there is no write path, so there truthfully is no free space. | G4a: once writes exist, `blocks`/`bfree`/`bavail` need real values instead of an honest zero. |

## Not overridden — `fuser`'s own default applies

Everything else falls through to `fuser::Filesystem`'s default
implementation (`fuser` 0.17.0). Checked directly against `fuser`'s
source rather than assumed, because an earlier bad-code card assumed most
of these were silent no-ops and that assumption was stale:

- **Already explicit `ENOSYS` + a `warn!` log by default**, so nothing
  silent is happening: `setattr`, `mknod`, `mkdir`, `unlink`, `rmdir`,
  `rename`, `write`, `flush`, `fsync`, `fsyncdir`, `setxattr`, `getxattr`,
  `listxattr`, `removexattr`, `access`, `create`, `readdirplus`, and the
  file-locking/`ioctl`/`poll`/`bmap` family. Real POSIX tools calling any
  of these against a WARP DRIVE mount today get a real, kernel-visible
  error — not silence.
- **`symlink`/`link`** default to `EPERM`, also explicit.
- **`forget`/`batch_forget`** have no reply channel at all — `forget` is a
  fire-and-forget kernel notification in the FUSE protocol itself, so
  there is nothing to make "explicit": silence is the only valid response.

## Known gaps

- No method here has any notion of a per-file-handle basis, staged
  writes, or an Intent/Receipt — the entire write half of the membrane is
  unbuilt. See [G4a](../../design/g4a-intent-admission-receipts.md) for
  the design that will start filling this in, and its explicit "FUSE
  design constraint" section for why `fsync()`, not `release()`, has to
  be the admission point once writes exist.
- `opendir`'s no-validation behavior is only correct because the kernel
  is assumed to call `opendir()` only against inodes it already resolved
  via a prior `lookup()`. That assumption is unverified against a
  malicious or buggy FUSE client bypassing normal lookup order — not a
  concern for the read-only mount this describes today, but worth
  re-examining if this mount's threat model ever changes.
