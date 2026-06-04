<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- Copyright James Ross / FLYING ROBOTS <https://github.com/flyingrobots> -->

# G3 - .warp diagnostics and live counters

Status: DRAFT

Branch: `gate/g3`

Proof record: `docs/gates/G3.md` after the gate passes.

## Purpose

G2b proved that WARP DRIVE can expose one normal read-only file,
`/echo/head.json`, whose bytes came back through Echo's query projection
payload path.

G3 must not prove a second projected file. The next bottleneck is
observability. The mount must be able to say what it is doing while it is
doing it.

G3 makes `/.warp/` a trustworthy diagnostic surface:

1. `/.warp/stats` reports live operation counters, not the G1/G2 static
   placeholder.
2. `/.warp/runtime` identifies the runtime, gate, and build mode clearly.
3. Known POSIX operations move the expected counters monotonically.
4. Existing read-only behavior and write rejection remain intact.

## Gate condition

G3 passes when both of these commands pass in the Linux acceptance runner:

```sh
cargo xtask acceptance --gate g3 --runtime in-memory
cargo xtask acceptance --gate g3 --runtime echo-rlib
```

The `echo-rlib` run keeps the G2b copy-in Docker invariant. It must not
bind-mount a live host repository into the acceptance container.

## Required counter surface

`/.warp/stats` must return JSON with at least these fields:

```json
{
  "gate": "G3",
  "runtime": "in-memory",
  "lookup_count": 0,
  "getattr_count": 0,
  "readdir_count": 0,
  "open_count": 0,
  "read_count": 0,
  "readlink_count": 0,
  "runtime_observe_count": 0,
  "runtime_observe_error_count": 0
}
```

The exact JSON order is not a public API, but the acceptance script may parse
or grep the committed shape for this gate. The values must be unsigned decimal
integers.

## Counter semantics

Counters are per mount process. They reset on remount.

Counters must be monotonic for the lifetime of the mount. They must never be
derived from wall-clock time, random state, log scanning, or acceptance script
side channels.

The required operation counters mean:

| Counter | Meaning |
| --- | --- |
| `lookup_count` | FUSE `LOOKUP` calls handled by the adapter. |
| `getattr_count` | FUSE `GETATTR` calls handled by the adapter. |
| `readdir_count` | FUSE `READDIR` calls handled by the adapter. |
| `open_count` | Accepted read-only `OPEN` calls plus rejected write opens. |
| `read_count` | Non-diagnostic regular-file `READ` calls. |
| `readlink_count` | FUSE `READLINK` calls handled by the adapter. |
| `runtime_observe_count` | Echo observation/projection calls made while preparing or serving the mount. |
| `runtime_observe_error_count` | Echo observation/projection calls that returned an error. |

Reading `/.warp/stats` is diagnostic self-observation. It must not increment
`read_count`; otherwise acceptance could pass by observing the counter instead
of proving that ordinary file reads are counted. Future gates may add a
separate diagnostic-read counter, but G3 keeps the minimum surface boring.

`lookup_count` and `getattr_count` are intentionally not exact acceptance
targets. Kernel and FUSE caching can change those totals without changing the
membrane contract. Acceptance may require them to be present and
non-decreasing, but must not require exact syscall totals.

## Runtime diagnostics

`/.warp/runtime` must identify the mounted runtime and gate clearly.

Minimum in-memory shape:

```json
{
  "gate": "G3",
  "runtime": "in-memory",
  "driver": "warp-drive-driver-memory",
  "build_mode": "debug",
  "stats": "live"
}
```

Minimum Echo shape:

```json
{
  "gate": "G3",
  "runtime": "echo-rlib",
  "driver": "warp-wasm",
  "build_mode": "debug",
  "stats": "live"
}
```

Additional Echo coordinate fields may remain in `/.warp/runtime` as long as
the required fields are present and unambiguous.

## Implementation shape

Keep the tree shape stable. Do not mutate `FixtureTree` on every operation.

The G3 implementation should add a small counter state owned by the FUSE
adapter:

```rust
pub struct MountStats {
    lookup_count: AtomicU64,
    getattr_count: AtomicU64,
    readdir_count: AtomicU64,
    open_count: AtomicU64,
    read_count: AtomicU64,
    readlink_count: AtomicU64,
    runtime_observe_count: AtomicU64,
    runtime_observe_error_count: AtomicU64,
}
```

The FUSE adapter should intercept the `/.warp/stats` inode for `GETATTR` and
`READ`:

1. The inode remains part of the fixture tree so `lookup`, `readdir`, `find`,
   and path shape stay stable.
2. `GETATTR` computes the current stats JSON length for that inode.
3. `READ` serializes a fresh snapshot and serves the requested byte range.
4. Ordinary fixture and projected files continue to read from `FixtureTree`.

This keeps `FixtureTree` as the shape and content scaffold while the FUSE
adapter owns live operation observations. The adapter already receives the
syscalls, so it is the narrowest layer that can count them honestly.

The core crate may expose a named stats inode constant, for example
`WARP_STATS_INO`, so the adapter does not rely on a repeated magic number.

## Acceptance requirements

G3 acceptance must keep the G1 baseline:

1. `ls`, `cat`, `find`, `rg`, `stat`, and `readlink` work.
2. Writes fail with read-only filesystem errors.
3. G1 fixture files keep their documented contents.

G3 acceptance must keep the G2b Echo baseline for `echo-rlib`:

1. `/echo/head.json` exists as a normal read-only regular file.
2. The file contains `"kind":"echo-projected-file"`.
3. The file contains `"source":"echo-observation-payload"`.
4. The copy-in Docker safety invariant remains in force.

G3 acceptance must add the diagnostics proof:

1. `cat /.warp/stats` returns valid JSON.
2. `cat /.warp/runtime` returns valid JSON.
3. Both files contain `"gate":"G3"`.
4. `/.warp/runtime` identifies the selected runtime.
5. Before/after stats snapshots show `read_count` increases after reading a
   normal non-diagnostic file such as `/README.md`.
6. Before/after stats snapshots show `readdir_count` increases after listing a
   directory.
7. Before/after stats snapshots show `readlink_count` increases after reading
   `/links/readme`.
8. `lookup_count` and `getattr_count` are present and non-decreasing.
9. `runtime_observe_error_count` is zero in accepted runs.
10. `runtime_observe_count` is present for all runtimes and greater than zero
    for `echo-rlib`.

Do not assert exact syscall totals. G3 proves monotonic, trustworthy movement,
not a kernel-specific syscall trace.

## Non-goals

G3 does not implement:

1. A second Echo-projected file.
2. Dynamic Echo-backed directories.
3. Writes.
4. Basis-aware save receipts.
5. A daemon.
6. Background frontier refresh.
7. Cache-hit or latency counters beyond the minimum required fields.
8. `.warp/holograms`, `.warp/intents`, `.warp/errors`, or `.warp/cache`.

Those surfaces remain important, but they are not required to prove that the
membrane can tell the truth about its current read-only behavior.

## Open questions

1. Should diagnostic self-reads get a separate `diagnostic_read_count` in a
   later gate?
2. Should `runtime_observe_count` count startup observations only, or only
   observations after FUSE mount begins, once live refresh exists?
3. Should `/.warp/runtime` and `/.warp/stats` share a version field before
   external tools consume them?
4. Should the G3 acceptance script use `jq`, Python, or a tiny Rust helper for
   JSON parsing inside the Docker image?
