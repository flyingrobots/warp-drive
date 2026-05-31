<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# G2a Design — Echo coordinate metadata mount

**Gate:** G2a  
**Branch:** `gate/g2`  
**Status:** implementation  
**Design date:** 2026-05-31

---

## 1. Gate condition

> `cargo xtask acceptance --runtime=echo-rlib` exits 0 against a FUSE mount backed by a real
> embedded Echo engine. The FUSE binary calls `init_embedded()` and
> `observe_cbor()` from the `warp-wasm` rlib. `ls`, `cat`, `find`, `rg`,
> `stat`, `readlink` return the same results as G1. `cat /.warp/coordinate`
> shows a real worldline UUID and real state-root hash — not the static
> genesis placeholder from G1.

The gate is mechanical: `cargo xtask acceptance --runtime=echo-rlib` exits 0.
This is not the full Echo-projected read-only mount. It is the Echo coordinate
metadata checkpoint that precedes it.

---

## 2. What G1 proved vs. what G2a requires

### G1 proved

- The POSIX translation layer works: `lookup`, `getattr`, `open`, `read`,
  `readdir`, `readlink` all translate correctly through `fuser` to a virtual
  tree.
- EROFS write rejection is correct at the kernel level.
- `cargo xtask acceptance` is the right human interface for gate verification.

### What G2a adds

Two things that G1 deliberately deferred:

1. **Real Echo initialization.** The FUSE binary calls `init_embedded()` and
   `observe_cbor()` from `warp-wasm` (with `features = ["engine"]`). The
   embedded Echo kernel runs — it is not mocked or bypassed.

2. **Real coordinate metadata.** The bytes served at `/.warp/coordinate` come
   from the `observe_cbor()` response. Specifically: the worldline UUID is the
   real UUID assigned by `init_embedded()`, not the hardcoded genesis
   placeholder `00000000-0000-0000-0000-000000000001`.

### What G2a explicitly defers

**File content from Echo.** The current `warp-wasm` API returns
`HeadObservation` which contains only:
- `worldline_tick: WorldlineTick`
- `commit_global_tick: Option<GlobalTick>`
- `state_root: Vec<u8>` (32-byte Merkle root hash)
- `commit_id: Vec<u8>` (32-byte frontier hash)

There are no file bytes in `HeadObservation`. Echo does not have an
`echo-fs-runtime` crate or a filesystem schema yet — that is W2.M1 in
the implementation plan, and it is substantial work (~3000 lines + schema).

G2a does not build `echo-fs-runtime`. File content (`README.md`, `src/main.ts`,
etc.) continues to come from the `FixtureTree` seeded into the backend at
startup. The claim G2a makes is narrower:

> The FUSE binary successfully calls into a live Echo engine, gets real
> coordinate metadata back, and surfaces it at `/.warp/coordinate`.

G2b/G3 is where file content actually comes from Echo's projection. See §8.

---

## 3. Implementation approach

### 3.1 New crate: `warp-drive-echo-backend`

Lives at `crates/warp-drive-echo-backend/`. It is excluded from the default
workspace and is only used by the local-only Echo gate binary at
`crates/warp-drive-fuse-echo/`.

```text
crates/warp-drive-echo-backend/
├── Cargo.toml
└── src/
    └── lib.rs        — EchoBackend + init_embedded()/observe_cbor() wrapper
```

```rust
/// Handle to a live embedded Echo engine.
pub struct EchoBackend {
    /// Cached fixture tree — G1 file bytes plus Echo-derived `.warp/` metadata.
    tree: FixtureTree,
}

pub struct EchoCoordinateMeta {
    pub worldline_id_hex: String,  // 32-byte worldline UUID as hex
    pub state_root_hex: String,    // 32-byte state root as hex
    pub worldline_tick: u64,
}

impl EchoBackend {
    pub fn init() -> Result<Self, EchoBackendError>;
    pub fn into_tree(self) -> FixtureTree;
}
```

### 3.2 `EchoBackend::init()` sequence

```
1. warp_wasm::init_embedded()
     → EmbeddedHandle { worldline_id, head }

2. Build ObservationRequest for worldline_id at Frontier
     (same shape as the G0 spike)

3. warp_wasm::observe_cbor(request_cbor)
     → OkEnvelope<ObservationArtifact>
     → artifact.payload = ObservationPayload::Head { head: HeadObservation }
     → extract: worldline_tick, state_root, commit_id

4. Construct EchoCoordinateMeta from the above

5. Construct FixtureTree::new() — the G1 fixture, unchanged

6. Return EchoBackend { worldline_id, tree, coordinate }
```

### 3.3 Updated `/.warp/coordinate` content

G1 hardcoded:
```json
{"worldline":"00000000-0000-0000-0000-000000000001","frontier":"genesis"}
```

G2a serves real data from the observe response:
```json
{
  "worldline": "<real 32-byte hex worldline UUID>",
  "frontier": "<real 32-byte hex commit_id>",
  "state_root": "<real 32-byte hex state_root>",
  "tick": <worldline_tick>,
  "backend": "echo-rlib",
  "gate": "G2a"
}
```

### 3.4 Updated `/.warp/runtime` content

G1:
```json
{"kind":"in-memory","driver":"warp-drive-driver-memory","gate":"G1"}
```

G2a (echo-rlib runtime):
```json
{"kind":"echo-rlib","driver":"warp-wasm","gate":"G2a","worldline":"<hex>"}
```

### 3.5 FUSE binary changes

The default workspace `warp-drive-fuse` binary remains the G1 in-memory
binary. The local-only `warp-drive-fuse-echo` package builds a binary with the
same executable name and adds the Echo runtime option:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum Runtime {
    /// Hardcoded in-memory fixture tree (G1 default).
    #[value(name = "in-memory")]
    InMemory,
    /// Embedded Echo rlib coordinate metadata over G1 fixture bytes.
    #[value(name = "echo-rlib")]
    EchoRlib,
}
```

The local `warp-drive-fuse-echo` binary branches into `EchoBackend::init()`
and mounts its `FixtureTree` through the shared `warp-drive-fuse::mount_tree`
helper. The FUSE adapter doesn't change — it still talks to `FixtureTree`.
Only the init path and the `.warp/` content generation change.

### 3.6 Workspace dependency management

The `warp-wasm` path dep is not available in CI or Docker (same constraint as
the g0-spike). G2a therefore keeps all Echo path dependencies out of the
default workspace:

- `Cargo.toml` excludes `warp-drive-echo-backend` and
  `warp-drive-fuse-echo` from the active workspace.
- Default `cargo check --workspace` does not resolve or lock
  `../echo-warp-drive` path dependencies.
- `cargo xtask acceptance --runtime=echo-rlib` builds
  `crates/warp-drive-fuse-echo/Cargo.toml` into `target/echo-rlib/`.
- The Docker `cargo xtask acceptance` continues to run the `in-memory` backend
  (G1 assertions, 29/29).
- A separate `cargo xtask acceptance --runtime=echo-rlib` is added as a local
  acceptance command. G2a's gate is demonstrated by this local run, not by
  Docker CI.

This mirrors G1's "macOS local mount not passed" caveat. The gate record
documents the environment and the known CI gap. A future G2b/G3 gate should
remove this caveat by either checking out Echo in CI or moving the dependency
into a published contract/runtime package.

---

## 4. Acceptance criteria

The G2a acceptance run adds the following assertions to the existing 29:

```sh
# ── G2a-specific assertions ───────────────────────────────────────────────

# .warp/coordinate must contain a real worldline (not the genesis placeholder)
COORD=$(cat "$MOUNT/.warp/coordinate")
assert_not_contains "$COORD" "00000000-0000-0000-0000-000000000001" \
    ".warp/coordinate worldline is real (not genesis placeholder)"

# .warp/coordinate must contain a non-zero state_root
assert_contains "$COORD" '"state_root"' \
    ".warp/coordinate has state_root field"

# .warp/runtime must identify the echo-rlib backend
RUNTIME=$(cat "$MOUNT/.warp/runtime")
assert_contains "$RUNTIME" '"echo-rlib"' \
    ".warp/runtime identifies echo-rlib backend"

# All 29 G1 assertions still pass (file content unchanged)
```

Total: 29 + 3 = 32 assertions. Gate passes when all 32 exit 0.

---

## 5. Crate structure

```text
warp-drive/
├── crates/
│   ├── warp-drive-core/           (unchanged — FixtureTree, VirtualNode)
│   ├── warp-drive-fuse/           (shared mount_tree + default G1 binary)
│   ├── warp-drive-echo-backend/   (excluded — EchoBackend)
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── warp-drive-fuse-echo/      (excluded — local Echo binary)
│       ├── Cargo.toml
│       └── src/main.rs
```

`warp-drive-echo-backend` depends on:
- `warp-drive-core` (FixtureTree)
- `warp-wasm` (path dep via `../echo-warp-drive`)
- `echo-wasm-abi` (ObservationRequest types)

---

## 6. Risks and mitigations

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `init_embedded()` panics on thread contention (warp-wasm uses thread_local!) | Low | High | Call init once, in main thread, before FUSE mount spawns |
| Echo kernel state leaks across tests (thread_local! is process-global) | Medium | Medium | `cargo xtask acceptance` runs a single process per Docker container; no cross-test leakage |
| `observe_cbor()` returns error at genesis (before any intents) | Low | Medium | G0 spike proved genesis observe round-trips; this is a known-good path |
| CI cannot build `warp-drive-echo-backend` (missing path dep) | High | Low | Excluded from default workspace; Docker CI only builds `in-memory` backend |
| `warp-wasm` thread_local initialization order with fuser threads | Medium | High | Initialize Echo before spawning FUSE worker threads; verify with G2a acceptance run |

---

## 7. Open questions

1. **Thread safety — RESOLVED.** `warp-wasm` uses `thread_local! { KERNEL }`. `observe_cbor()` accesses this thread-local. FUSE worker threads spawned by `fuser::mount2` are different threads from main — they will see an uninitialized kernel and `observe_cbor()` will return `NOT_INITIALIZED`. **Resolution:** call `init_embedded()` + `observe_cbor()` exactly once on the main thread before `fuser::mount2`. Cache the result in the pre-built `FixtureTree`. FUSE handler threads only touch the cached tree — they never call into `warp-wasm`.

2. **Observation caching — RESOLVED.** At G2a, the Echo state never changes after init (no writes). `observe_cbor()` is called exactly once at startup on the main thread. The result is baked into the `FixtureTree` nodes for `/.warp/coordinate` and `/.warp/runtime`. This is correct for G2a; G3 will need a background refresh mechanism.

3. **Workspace exclusion strategy — RESOLVED FOR G2a.** `warp-drive-echo-backend` and `warp-drive-fuse-echo` are excluded packages. The default workspace does not depend on them, so normal CI/Docker builds do not require `../echo-warp-drive`.

4. **Accept the `echo-rlib` binary in Docker?** The Docker image currently runs `warp-drive-fuse --runtime=in-memory`. G2a leaves `echo-rlib` as a local-only gate until CI either checks out `echo-warp-drive` or Echo ships a consumable package/contract bundle.

---

## 8. Gate G3 preview

G2b/G3 builds on G2a by replacing the seeded `FixtureTree` with a live Echo
projection. The key work in G3:

- **echo-fs-runtime (W2.M1):** New crate in `echo-warp-drive` that implements
  the `warpdrive.graphql` read handlers: `fsObserveNode`, `fsReadProjection`,
  `fsListDirectory`. File content is stored in `echo-cas` and committed as
  intents. The `observe_cbor()` response for a path query returns file bytes.

- **Projection adapter in warp-drive:** `VirtualTree` is built from
  `fsListDirectory` + `fsReadProjection` calls instead of `FixtureTree::new()`.
  File bytes come from Echo, not from embedded static slices.

- **`.warp/` live diagnostics:** `/.warp/stats` shows real counters from
  the projection adapter. `/.warp/holograms/<ino>` shows provenance.

The G2a `EchoBackend` is the scaffold for G2b/G3's projection adapter. The
`observe_cbor()` plumbing, the coordinate serialization, and the thread-safety
decisions made in G2a are all reused when projected file bytes are added.

---

## 9. Work breakdown

| Task | Where | Estimate |
|------|-------|----------|
| `warp-drive-echo-backend` crate scaffold | warp-drive | 1 h |
| `EchoBackend::init()` — `init_embedded()` + `observe_cbor()` | warp-drive | 2 h |
| `EchoCoordinateMeta` + `/.warp/coordinate` content generation | warp-drive | 1 h |
| FUSE binary `--runtime=echo-rlib` plumbing | warp-drive | 1 h |
| Workspace Cargo.toml: exclude local Echo crates | warp-drive | 1 h |
| `scripts/acceptance-g2.sh` (computed assertion count) | warp-drive | 1 h |
| `cargo xtask acceptance --runtime=echo-rlib` xtask command | warp-drive | 1 h |
| G2a gate record (`docs/gates/G2a.md`) | warp-drive | 0.5 h |
| **Total** | | **~8.5 h** |

---

## 10. Acceptance script location

`scripts/acceptance-g2.sh` — a thin G2a wrapper over `scripts/acceptance.sh` that:
1. Runs with `WARP_RUNTIME=echo-rlib` (or `--runtime=echo-rlib` flag)
2. Replays the G1 read assertions against the Echo metadata mount
3. Adds stricter G2a coordinate assertions for non-placeholder worldline,
   echo-rlib backend, and non-zero 64-character hex `frontier`, `state_root`,
   and `artifact_hash` values
4. Reports the computed assertion count on success
