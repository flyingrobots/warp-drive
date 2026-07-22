<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# The `.warp/` diagnostics surface

**Status:** living reference, current as of `main`.
**Established by:** [`docs/gates/G3.md`](../../gates/G3.md). Original design:
[`docs/design/g3-dotwarp-diagnostics-and-counters.md`](../../design/g3-dotwarp-diagnostics-and-counters.md)
(historical — describes the plan, not necessarily today's exact shape).
**Test plan:** [`test-plan.md`](test-plan.md).

Every WARP DRIVE mount, regardless of gate or runtime, serves two live
diagnostic files: `/.warp/stats` and `/.warp/runtime`. Both are synthesized
on read from in-process state — nothing under `/.warp/` today is backed by
persisted bytes on disk.

## `/.warp/stats`

Returns a fresh JSON snapshot of `warp_drive_fuse::MountStats` on every
`read()`:

```json
{"gate":"G3","runtime":"in-memory","schema_version":1,"lookup_count":9,"getattr_count":11,"readdir_count":3,"open_count":3,"read_count":2,"readlink_count":2,"runtime_observe_count":0,"runtime_observe_error_count":0}
```

Each `*_count` field is an independent, per-process `AtomicU64` owned by
the `FuseAdapter`, incremented with `Ordering::Relaxed`. That's a real
constraint on what the snapshot means: reading two fields "at once" is not
transactionally coherent — a fast concurrent operation could land between
them. Nothing in this repo needs cross-field coherence yet; if something
eventually does, that's new work, not an implicit guarantee this file
already gives you.

Counter semantics:

| Field | Bumped by |
| --- | --- |
| `lookup_count` | Every `LOOKUP` the adapter handles, hits and misses alike. |
| `getattr_count` | Every `GETATTR`. |
| `readdir_count` | Every `READDIR`. |
| `open_count` | Every `OPEN`, both accepted read-only opens and rejected write opens. |
| `read_count` | Every `READ` **except** a read of `/.warp/stats` itself. |
| `readlink_count` | Every `READLINK`. |
| `runtime_observe_count` | Successful `warp_wasm::observe_cbor()` calls the Echo backend made during mount startup. Always `0` for `in-memory`; `2` for the current `echo-rlib` gates (one head observation, one query-projected `/echo/head.json` observation). Does not increment after mount — there is no live refresh yet. |
| `runtime_observe_error_count` | Always `0` for any mount you can observe: an Echo observation error aborts startup before a mount exists to report from. Not a durable record of failed attempts. |

The `read_count` exemption is deliberate, not an oversight: if reading
`/.warp/stats` counted as an ordinary read, the acceptance proof could pass
by observing its own counter instead of proving that unrelated reads are
counted. `lookup_count`/`getattr_count` are intentionally not exact
acceptance targets — kernel and FUSE caching can change their totals
without the membrane contract changing — but every operational field is a
genuine lower bound: watch it, perform *n* distinct operations of that
kind, and it moves by at least *n*.

### Why the stats file is actually live, not just correctly-sized

Three mechanisms work together, and each is load-bearing on its own:

1. **Constant-width JSON.** Every counter is right-aligned in a fixed
   20-character field, so the document's byte length never changes as
   counters grow — no race between a cached `stat()` size and a freshly
   grown `read()` body.
2. **Zero attribute-cache TTL**, for this inode only. Ordinary fixture
   files keep a normal 1-second TTL; `/.warp/stats` gets `Duration::ZERO`
   on both its `lookup()` entry reply and `getattr()` attribute reply, so
   the kernel always re-fetches rather than trusting a cached size.
3. **`FOPEN_DIRECT_IO`** on open, for this inode only. Without it, the
   kernel's ordinary page cache could serve a second `read()` from the
   first `open()`'s cached content. This is what actually makes repeated
   reads within one open hit the adapter every time, not just what makes
   the reported size trustworthy.

All three are centralized behind one seam in the adapter
(`FuseAdapter::live_stats_node`), which also refuses to synthesize
anything for this inode unless the underlying fixture tree still backs it
as a regular file — a guessed magic inode number can't get live content it
isn't entitled to.

## `/.warp/runtime`

A point-in-time identity snapshot — gate, runtime backend, driver, build
profile — not a counter file. Minimum shape, both runtimes:

```json
{"gate":"G3","runtime":"in-memory","driver":"warp-drive-driver-memory","build_mode":"debug","stats":"live","schema_version":1}
```

```json
{"gate":"G3","runtime":"echo-rlib","driver":"warp-wasm","build_mode":"debug","stats":"live","schema_version":1,"worldline":"<64 hex>"}
```

`schema_version` is `warp_drive_core::WARP_DIAGNOSTICS_SCHEMA_VERSION` — one
constant shared by both files. There is exactly one place that number is
defined; nothing hardcodes it independently.

### The `"gate"` field names the active mount, not payload provenance

This is the one thing worth being careful about. `/.warp/coordinate`,
`/.warp/runtime`, and `/.warp/stats`'s `"gate"` field all identify *which
CLI `--gate` path built this mount* — under `--gate g3 --runtime
echo-rlib`, all three say `"G3"`. `/echo/head.json`'s own `"gate"` field is
a different thing entirely: Echo-side payload content from its query
response, unrelated to which gate is currently mounting. It legitimately
still says `"G2b"` under a G3 mount, because G3 reuses G2b's exact
projection call unmodified. Don't expect that field to track the mount's
gate — it's answering a different question (whose payload is this?, not
who's mounting right now?).

## Known gaps

- Counters are per-mount-process and reset on remount; there is no
  persisted or cross-mount history.
- `runtime_observe_count`/`runtime_observe_error_count` reflect startup
  only — no live refresh exists, so a long-running mount's Echo activity
  after startup is invisible here. See
  [`docs/method/backlog/cool-ideas/GATE_dotwarp-stats-schema-file.md`](../../method/backlog/cool-ideas/GATE_dotwarp-stats-schema-file.md)
  and [`GATE_stats-snapshot-seq.md`](../../method/backlog/cool-ideas/GATE_stats-snapshot-seq.md)
  for related deferred ideas (historical backlog location; new ideas go to
  GitHub issues).
- `mmap` of `/.warp/stats` is unsupported — direct I/O disables shared
  `mmap` by default for that inode.
- Snapshots are per-read, not per-open: two reads within the same open can
  observe different instants. See
  [`GATE_stats-per-open-frozen-snapshot.md`](../../method/backlog/cool-ideas/GATE_stats-per-open-frozen-snapshot.md).
