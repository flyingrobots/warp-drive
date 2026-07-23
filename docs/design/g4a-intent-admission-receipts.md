<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- Copyright James Ross / FLYING ROBOTS <https://github.com/flyingrobots> -->

# G4a - Intent, admission, receipt, and obstruction semantics (in-memory)

Status: DRAFT

Branch: `gate/g4a`

Proof record: `docs/gates/G4a.md` after the gate passes.

Tracking issue: [#20](https://github.com/flyingrobots/warp-drive/issues/20).
Splits [#11](https://github.com/flyingrobots/warp-drive/issues/11)
(stale-save obstruction demo) into gate-sized units; the Echo-backed half is
[#21](https://github.com/flyingrobots/warp-drive/issues/21) (G4b).

## Purpose

G0-G3 proved the read half of the POSIX⇄causal membrane: mount, list, read,
diagnose. Every write so far has been rejected with `EROFS` unconditionally —
that is the *absence* of a write path, not a design for one. Nothing in the
repo has an `Intent` type, a notion of basis, a receipt, or a lawful
obstruction. This is the actual product thesis of the project ("every write
is an Intent against an explicit basis") and it is entirely unbuilt.

G4a proves the domain contract — admission and obstruction — without also
solving the Echo write adapter. It stays in-memory on purpose: the frontier
advance that makes a write stale is driven by a gate-only test control seam,
not a second live process. G4b (a separate, later gate) reuses G4a's FUSE
semantics unmodified and wires them to a real external Echo frontier advance.

A stale-write-only test would be insufficient: a filesystem that rejects
every write passes that trivially, with a nicer error message than plain
`EROFS`. G4a must prove **both** paths against one existing regular file:
a fresh-basis write is admitted, and a stale-basis write is refused with a
typed obstruction and an explanatory receipt.

## Gate condition

G4a passes when this command passes in the Linux acceptance runner:

```sh
cargo xtask acceptance --gate g4a --runtime in-memory
```

The command must prove both of the following against one existing fixture
file (candidate: `README.md`, already present in `FixtureTree`):

### Fresh-basis path (must admit)

1. Open the file at basis `B0` (the mount's current frontier at open time).
2. Stage replacement bytes through real FUSE `write()` callbacks.
3. Submit/flush the intent while the runtime is still at `B0` (via `fsync()`
   — see "FUSE design constraint" below).
4. Admission succeeds. The projected file now contains the new bytes.
5. `/.warp/intents/last` reports an **admitted** receipt.

### Stale-basis path (must obstruct)

1. Open the file at basis `B1`.
2. Advance the underlying in-memory frontier to `B2` through a gate-only
   control seam (test-only; not part of the writable mount's public surface).
3. Stage bytes through the still-open handle whose captured basis is `B1`.
4. Admission is refused with a stable errno and a typed obstruction.
5. The existing projection remains unchanged — no partial write, no silent
   clobber.
6. `/.warp/intents/last` reports an **obstructed** receipt.

## `/.warp/intents/last` shape

A new synthetic file under `.warp/`, following the same "live, synthesized
on read" pattern G3 established for `/.warp/stats` (see
`docs/topics/dotwarp-diagnostics/README.md`) — not a persisted log. It
reports only the most recent intent's outcome; a durable receipt log is
explicitly out of scope (see Non-goals).

Admitted shape (minimum fields):

```json
{
  "gate": "G4a",
  "schema_version": 1,
  "receipt_kind": "admitted",
  "intent_id": "<opaque identifier>",
  "site": "<path or stable site identity>",
  "submitted_basis": "<basis token>",
  "admitted_frontier": "<basis token>"
}
```

Obstructed shape (minimum fields):

```json
{
  "gate": "G4a",
  "schema_version": 1,
  "receipt_kind": "obstructed",
  "intent_id": "<opaque identifier>",
  "site": "<path or stable site identity>",
  "obstruction_code": "stale_basis",
  "attempted_basis": "<basis token>",
  "current_basis": "<basis token>",
  "explanation": "<human-readable, stable text>",
  "suggested_action": "<human-readable, stable text>"
}
```

`schema_version` reuses `warp_drive_core::WARP_DIAGNOSTICS_SCHEMA_VERSION` —
one constant, shared with `/.warp/stats` and `/.warp/runtime`, never an
independently hardcoded literal (G3 precedent, `docs/gates/G3.md`).

The exact basis-token representation (opaque string vs. structured object) is
an open question — see "Open questions" below.

## FUSE design constraint (non-negotiable)

Do not submit the intent only from `release()`. `fuser`/the kernel VFS layer
does not propagate errors returned from `release()` back to the `close()` or
`munmap()` call that triggered it — a filesystem that only decides
admission at `release()` has no way to tell the calling process a write
failed.

- `write()` stages bytes against a file-handle-scoped captured basis (the
  basis observed when the handle was opened, not the basis at write time).
- `fsync()` is the deterministic acceptance-test admission point — this is
  where staged bytes are actually submitted as an Intent and where
  admission-or-obstruction is decided.
- `flush()` must be safe to call multiple times and must not double-submit —
  the kernel may call it more than once per handle. It should report the
  same receipt as the most recent `fsync()`/admission decision, not attempt
  a second submission.
- `release()` cleans up per-handle state (the staged-write buffer, the
  captured basis). It is not the only place admission can fail, and it must
  not be relied upon as one.

This constraint is why every write-adjacent FUSE method needs deliberate,
documented behavior — see the scope perimeter below and
[#23](https://github.com/flyingrobots/warp-drive/issues/23).

## Explicit scope perimeter

Borrows the refusal-matrix framing from
[#10](https://github.com/flyingrobots/warp-drive/issues/10) (negative
compatibility/refusal suite) as the boundary this gate's positive surface is
measured against:

| Operation | G4a behavior |
|---|---|
| Write to an existing regular file, fresh basis | **Supported** — admitted |
| Write to an existing regular file, stale basis | **Supported** — typed obstruction |
| `create` / `mknod` (new file) | Unsupported — explicit documented errno |
| `unlink` | Unsupported — explicit documented errno |
| `rename` | Unsupported — explicit documented errno |
| `fallocate` | Unsupported — explicit documented errno |
| Writable shared `mmap` (`MAP_SHARED \| PROT_WRITE`) | Unsupported — refused at `mmap`/`open` time |
| Path-only runtime mutation without basis/site identity | Impossible by API — no such call exists |

Only existing-file replacement needs to work for G4a. Every operation this
gate does not implement must return a deliberate, documented refusal —
`EOPNOTSUPP`, `EROFS`, or another named errno — never a silent fallthrough
to `fuser`'s no-op default. This is the acceptance-visible form of
[#23](https://github.com/flyingrobots/warp-drive/issues/23) (silent `fuser`
default no-op methods), which is a hard prerequisite for this gate, not
parallel cleanup.

## Explicit non-goals

- **The normal-editor-save demo.** A real editor save is usually not "open
  existing file, write bytes" — it is commonly some combination of
  temp-file creation, truncation, metadata copying, `fsync`, rename-over-
  target, unlink cleanup, and directory sync. That is several filesystem
  contracts stacked into one demo. G4a proves causal admission with a tiny
  acceptance probe that has precise control over handle lifetime, write
  offsets, basis-advance timing, `fsync` calls, and returned errno. The
  editor-facing demo is a later gate/extension building on G4a+G4b, tracked
  under the still-open [#11](https://github.com/flyingrobots/warp-drive/issues/11).
- **Echo-backed admission.** That is G4b
  ([#21](https://github.com/flyingrobots/warp-drive/issues/21)). G4a's
  frontier advance is a gate-only in-memory test seam, not a second process.
- **Dynamic/live directory projection or tree rebuilds.** Not required to
  prove existing-file write against the current static mounted tree. This
  does mean G4a does not need rebuild-stable inodes — see
  [#22](https://github.com/flyingrobots/warp-drive/issues/22) (unstable
  inode assignment) for when that *does* become blocking (any gate that
  rebuilds the tree dynamically).
- **A durable receipt log.** `/.warp/intents/last` reports only the most
  recent outcome, synthesized on read like `/.warp/stats`. A receipt
  history/log is a separate, later idea (see historical cool-ideas card
  `docs/method/backlog/cool-ideas/GATE_receipt-log.md`).
- **Full conflict-resolution UI or silent auto-merge.**

## Dependencies

- [#23](https://github.com/flyingrobots/warp-drive/issues/23) (silent
  `fuser` default no-op methods) — hard prerequisite. Every write-adjacent
  method this gate touches or deliberately refuses needs real, documented
  behavior before a writable mount is enabled at all.
- [#6](https://github.com/flyingrobots/warp-drive/issues/6) (typed
  fixture/tree-definition boundary) — `/.warp/intents/last` is another
  dynamic synthetic file layered onto the fixture tree, same as
  `/.warp/stats` and `/echo/head.json` before it. It should land through
  whatever typed boundary #6 introduces, not another ad hoc constructor
  mutation on top of the existing pile.
- [#10](https://github.com/flyingrobots/warp-drive/issues/10) as the
  refusal-perimeter reference this gate's scope table above is drawn from.

## Resolved decisions

These resolve the open questions this doc originally posed. This section is
authoritative; RED tests are written against it directly, and an
implementation plan living elsewhere must not contradict it.

1. **Basis token representation: `Basis(u64)`, a monotonic tick counter.**
   Not a hash, not a structured worldline/frontier/state_root triple —
   this is the in-memory runtime only, and there is no real cryptographic
   coordinate to hash yet. Fabricating one would be exactly the kind of
   "approximate lie" the engineering standard warns against. Matches this
   project's existing convention of plain integers for counters
   (`MountStats`) rather than invented hex. G4b (Echo-backed) will need its
   own mapping from Echo's real coordinate fields to whatever comparison
   the membrane needs — G4a does not need to solve that generalization.
2. **The gate-only frontier-advance control seam: an in-process mount, not
   a subprocess, not a synthetic control file.** Resolves the open question
   `docs/TESTING.md` originally posed ("should `MountGuard` use a thread or
   a subprocess?"). G4a's write-path integration tests mount via
   `fuser::spawn_mount2` (non-blocking, runs the FUSE loop on a background
   thread in the *same process* as the test) instead of spawning the
   compiled binary as a child process. The test holds a `BasisControl`
   handle `Arc`-shared with the mounted `FuseAdapter` and advances it by
   calling a plain Rust method directly — never through a `.warp/` file,
   CLI flag, or FUSE method of any kind, so there is no path by which a
   real POSIX client could ever reach it. The existing subprocess-based
   `MountGuard` (`crates/warp-drive-fuse/tests/support/mod.rs`, merged
   ahead of G4a) is unaffected and stays in use for black-box smoke tests
   that don't need this control — the two harnesses serve different needs,
   this doesn't replace that one.
3. **Intent identifier generation: `IntentId(u64)`, a monotonic
   `AtomicU64` counter per adapter instance.** Same style as `MountStats`'s
   existing counters. Comparable and orderable, which is all a future
   receipt-log gate would need — no reason to reach for anything
   randomized or hash-based for this gate.
4. **Staged-write storage: a per-file-handle `WriteSession` behind a
   `Mutex`, owned by `FuseAdapter`, not the fixture tree.** `write()` stages
   bytes into `WriteSession { basis: Basis, buffer: Vec<u8> }` keyed by
   `FileHandle`; `fsync()` looks the session up, decides admission via a
   pure `admit()` function, and on admission writes the buffer into the
   tree; `release()` removes the entry. This intentionally does **not**
   wait for the full #6 typed-tree-definition boundary first: G4a's actual
   footprint on the tree is one new synthetic file
   (`/.warp/intents/last`, alongside the existing `/.warp/stats` and
   `/.warp/runtime` pattern established at G3) plus one existing file
   becoming genuinely writable. That's declaring two things explicitly,
   not another ad hoc mutation pile — #6's full general typed-builder
   redesign stays deferred until a gate that actually needs more than this.

## RED

Failing tests written against the decisions above, before any
implementation:

- `crates/warp-drive-core/src/lib.rs`: unit tests for a pure `admit()`
  function exercising both paths — fresh-basis submission is `Admitted`,
  stale-basis submission is `Obstructed` with `ObstructionCode::StaleBasis`
  and the correct attempted/current basis values. `Basis`, `IntentId`,
  `Receipt`, `Obstruction`, `ObstructionCode`, and `admit` do not exist yet
  at RED — this is expected to fail to compile.
- `crates/warp-drive-fuse/tests/g4a_write_path.rs`: an in-process-mount
  integration test (per decision 2 above) exercising both paths end-to-end
  through real FUSE `write`/`fsync` calls against `README.md`, reading
  `/.warp/intents/last` afterward and checking its JSON shape matches
  this doc's "Admitted shape"/"Obstructed shape" sections. Fails to
  compile at RED — the mount function it calls
  (`warp_drive_fuse::testing::spawn_with_basis_control`) does not exist
  yet, and neither does write support in `FuseAdapter`.

GREEN is: make both compile, then make both pass, in that order — without
weakening either test to get there.
