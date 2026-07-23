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

## Open questions

1. **Basis token representation.** Opaque string (e.g. a hash) vs. a
   structured value with separate worldline/frontier/tick fields. Should
   probably mirror whatever shape `/.warp/coordinate` already uses for
   consistency, but that needs to be checked against what a file-handle-
   scoped "captured basis" actually needs to compare cheaply on every
   `fsync()`.
2. **The gate-only frontier-advance control seam.** Needs a concrete shape:
   a CLI flag, an internal test hook, or a second synthetic `.warp/` control
   file. Must be clearly fenced as test/gate-only and not reachable from a
   normal writable mount's public surface.
3. **Intent identifier generation.** Monotonic counter vs. random/hash-based
   ID. Affects whether `intent_id` values are comparable/orderable, which
   matters if a future gate adds a receipt log.
4. **Where staged-write bytes live between `write()` and `fsync()`.**
   Per-file-handle buffer scoped to the open, presumably — needs to fit
   inside whatever typed tree/synthetic-node boundary #6 introduces rather
   than becoming its own ad hoc side-structure.

These should resolve into a "Resolved decisions" section (following the G3
design doc's precedent) before or during implementation, and that section
becomes authoritative over any implementation plan living elsewhere.
