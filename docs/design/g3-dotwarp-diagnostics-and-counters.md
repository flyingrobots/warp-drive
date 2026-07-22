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
  "schema_version": 1,
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

`schema_version` is `warp_drive_core::WARP_DIAGNOSTICS_SCHEMA_VERSION`, a
single constant shared with `/.warp/runtime` below — never an independently
hardcoded literal in more than one place.

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
  "stats": "live",
  "schema_version": 1
}
```

Minimum Echo shape:

```json
{
  "gate": "G3",
  "runtime": "echo-rlib",
  "driver": "warp-wasm",
  "build_mode": "debug",
  "stats": "live",
  "schema_version": 1
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
9. `mmap` of `/.warp/stats` — direct I/O on that inode disables shared
   `mmap` by default.

Those surfaces remain important, but they are not required to prove that the
membrane can tell the truth about its current read-only behavior.

## Resolved decisions

These resolve the open questions this doc originally posed, plus the
additional semantics the implementation depends on. This section is
authoritative; an implementation plan living elsewhere must not contradict
it.

1. **`runtime_observe_count` scope.** Precisely: counts successful calls to
   `warp_wasm::observe_cbor` initiated by `EchoBackend` — not "observations
   performed during startup" generically, which could be misread to include
   `init_embedded()`'s own internal work. This is real accounting owned by
   the backend that performs the calls (`RuntimeObservationStats`, returned
   from `EchoBackend::into_parts()`), never a constant hardcoded downstream
   in a binary. G3 has no live refresh, so nothing increments this after
   mount.
2. **`runtime_observe_error_count`** is always `0` for any successfully
   constructed backend: an observation error aborts `init_*()` via `?`
   before a backend is ever returned. This is not a durable count of failed
   startup attempts — it only ever reports on the path that reached a live
   mount.
3. **FUSE callback counts, not userspace syscall counts.** One `cat` may
   produce several `read()` callbacks, or — without a data-cache policy —
   none at all if the kernel serves it from cache. Acceptance must assert
   with `>` or bulk lower-bound deltas, never "+1 per command". A single
   before/after probe pair is invalid on its own: the probe's own diagnostic
   reads (the `cat`s used to fetch the two snapshots) also trigger
   lookups/getattrs and would contaminate a causal single-probe claim. Use
   bulk probes with a known minimum count instead.
4. **Per-mount, process-global counters.** Equivalent in this design — one
   mount per process. A future multi-mount design should revisit this.
5. **Per-read snapshot consistency, not per-open.** Every `/.warp/stats`
   `read()` serializes a fresh snapshot independently. Coherent snapshots
   across multiple reads within one `open()` are an explicit non-goal for
   G3 (a per-open frozen snapshot is a documented future idea, not built
   now).
6. **Data-cache policy.** Every successful read-only `open()` of the stats
   inode returns `FOPEN_DIRECT_IO`, so the kernel page cache cannot serve a
   stale `read()` on repeat reads within one open — this is the mechanism
   that makes the counters live, not merely correctly-sized. Normal fixture
   files keep cached I/O. Scope note: an acceptance test that opens the file
   twice (two separate `cat` invocations) and observes fresh content each
   time proves *freshness across ordinary tool usage*, not specifically that
   `FOPEN_DIRECT_IO` is the cause — Linux already invalidates page cache on
   open by default when `FOPEN_KEEP_CACHE` is absent. The flag choice itself
   is proven by a unit test on the adapter's `open_reply_flags` policy
   function. Direct I/O disables shared `mmap` by default for this inode;
   `mmap` of `/.warp/stats` is out of scope for G3 (see Non-goals).
7. **Attribute-cache policy.** The stats inode gets a zero-duration TTL for
   both `lookup()`'s entry reply and `getattr()`'s attribute reply (one
   `fuser` TTL argument governs both caches). Normal fixture files keep the
   existing 1s TTL. This is belt-and-suspenders with direct I/O and
   constant-width JSON, not a substitute for either.
8. **Active-mount gate vs. payload-provenance gate.** `/.warp/coordinate`,
   `/.warp/runtime`, and `/.warp/stats`'s `"gate"` field identifies which CLI
   `--gate`/fixture-construction path built this mount. `/echo/head.json`'s
   own `"gate"` field is Echo-side payload content G3 does not touch — under
   `--gate g3 --runtime echo-rlib` it legitimately still reads `"G2b"`,
   because G3 reuses G2b's exact projection call unmodified. Acceptance
   must assert this distinction explicitly and must never mechanically
   relabel every `G2b` string to `G3` when adapting the G2b script.
9. **Schema versioning, single source of truth.** One constant,
   `warp_drive_core::WARP_DIAGNOSTICS_SCHEMA_VERSION = 1`, used by
   `MountStats::snapshot_json`, the in-memory G3 runtime-JSON builder, and
   the Echo-backend G3 runtime-JSON builder — never three independent
   literal `1`s. Both `/.warp/runtime` and `/.warp/stats` carry
   `"schema_version":1` under G3. Legacy G1/G2 runtime payloads (served by
   `init()`/`init_g2b()`, untouched by G3) keep their existing unversioned
   shape — their frozen acceptance scripts don't expect the field.
10. **Acceptance JSON parsing.** Plain bash + `sed`. No `jq`, Python, or
    Rust helper added to either Docker image. Numeric fields (counters,
    `schema_version`) are parsed and compared numerically — never
    substring-matched, since `"schema_version":1` would also match
    `10`/`11`/`123`.
11. **Every currently-exposed gate CLI combination keeps working on `main`**,
    including bare historical defaults: `cargo xtask acceptance` (→ G1,
    in-memory) and `cargo xtask acceptance --runtime echo-rlib` (→ G2a), not
    just their explicit `--gate` spellings. All ten `(runtime, gate)`
    combinations (2 runtimes × {none, G1, G2a, G2b, G3}) are enumerated
    explicitly and individually tested; invalid combinations produce
    explicit, matched errors, never a silent wildcard fallback.
12. **Diagnostic self-reads.** No separate `diagnostic_read_count` in G3 —
    deferred; `read_count`'s exemption for the stats inode is sufficient for
    this gate's proof.
