<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# WARP DRIVE — Testing strategy

> This document answers: what does "tested" mean for a POSIX⇄causal membrane,
> how do we structure the test infrastructure, and what is the roadmap from
> the current state to a multi-backend conformance suite.

---

## Table of contents

1. [Philosophy](#1-philosophy)
2. [Test taxonomy](#2-test-taxonomy)
3. [Current state](#3-current-state)
4. [Near-term roadmap](#4-near-term-roadmap)
5. [Fixtures library](#5-fixtures-library)
6. [Integration test harness](#6-integration-test-harness)
7. [Safe-to-execute definition](#7-safe-to-execute-definition)
8. [Backend progression](#8-backend-progression)
9. [GitHub Actions CI](#9-github-actions-ci)

---

## 1. Philosophy

WARP DRIVE is a correctness-critical translation layer. A bug here surfaces
as data corruption or silent semantic divergence — not a crash. The testing
philosophy follows directly from the engineering standard:

```
correctness > portability > observability > performance > cleverness
```

Three principles govern the test infrastructure:

**Honest.** Tests MUST exercise what the code actually does. Mocking the FUSE
kernel interface is not an option — a mocked test that passes while a real
mount fails is worse than no test at all. Integration tests mount a real
FUSE filesystem and assert against it with real OS tools.

**Safe.** A test MUST NOT be capable of corrupting production Continuum state.
Safety is structural: in-memory backends cannot reach production, and the
FUSE mount is read-only by default. A test that could corrupt real state is
a defect in the test harness, not an acceptable risk.

**Composable.** Fixture definitions are data, not code. A fixture for a gate
acceptance test is also usable in a targeted unit test for a single FUSE
method. The same 29-assertion acceptance script runs against every backend.
Only the mount setup changes.

---

## 2. Test taxonomy

Four tiers, from fastest to most complete:

| Tier | Command | What it covers | Platform |
|------|---------|----------------|----------|
| **Unit** | `cargo test` | Domain logic only — `FixtureTree`, node types, inode lookups | Any |
| **Integration** | `cargo test` (with FUSE) | Real FUSE mount, Rust assertions, per-test mount guard | Linux |
| **Gate acceptance** | `cargo xtask acceptance` | CLI tools against a full mount — `ls`, `cat`, `rg`, `stat`, `readlink`, write rejection | Linux (Docker) |
| **Conformance** | `cargo xtask acceptance --backend <name>` | Same gate assertions run against different backends | Linux (Docker) |

Each tier tests a different boundary:

- **Unit** tests the domain model in isolation.
- **Integration** tests the FUSE translation layer (lookup, getattr, readlink, read, readdir) through the kernel interface.
- **Gate acceptance** tests the full user-facing contract: does a real `cat` return the right bytes? Does `rg` find the right lines? Does `echo > file` return EROFS?
- **Conformance** tests that the contract holds regardless of which backend provides the projection.

---

## 3. Current state

### What exists

| Component | Status |
|-----------|--------|
| `crates/warp-drive-core` | In-memory `FixtureTree` — 13 hardcoded nodes |
| `crates/warp-drive-fuse` | FUSE binary — `adapter.rs` + `main.rs` |
| `scripts/acceptance.sh` | 29-assertion shell script (Linux / GNU userland) |
| `cargo xtask acceptance` | Docker build + run, exits 0 on pass |
| Gate G1 | **PASSED** (29/29 assertions, 2026-05-30) |

### What is missing

- Unit tests: none. Fixture data and tree logic are mixed in `FixtureTree::new`,
  making targeted tests difficult.
- Integration tests: none. No Rust-level FUSE mount/assert/unmount harness.
- CI: none. No GitHub Actions workflow.
- Fixtures crate: none. No separation between "the shape of a tree" and "the
  G1 acceptance fixture."
- In-memory debug backend: none. Everything is static fixture data; no backend
  abstraction exists yet.

---

## 4. Near-term roadmap

Ordered by dependency:

### Step 1 — Fix g0-spike workspace poison ✅ done

Removed `crates/warp-drive-g0-spike` from `[workspace] members` and the
`echo-wasm-abi` / `warp-wasm` path deps from `[workspace.dependencies]`.
Dockerfile `sed` hacks deleted. Workspace is now portable.

### Step 2 — GitHub Actions CI

**Depends on:** Step 1 ✅.

One workflow, three jobs. See [§9](#9-github-actions-ci) for the full spec.
Add non-blocking first; protect `main` once it passes twice.

### Step 3 — Fixtures crate (`warp-drive-fixtures`)

**Depends on:** nothing (pure domain, no FUSE).

Split `FixtureTree::new()` per the bad-code card. Two crates, not one —
see [§5](#5-fixtures-library) for the boundary rationale.

See: [`docs/backlog/bad-code/fixture-data-mixed-with-tree-logic.md`](backlog/bad-code/fixture-data-mixed-with-tree-logic.md)

### Step 4 — Integration test harness (`warp-drive-test-harness`)

**Depends on:** Step 3 (fixture crate).

`MountGuard` RAII type and assertion helpers for Rust `#[test]` functions
that mount a fixture tree, assert through the real kernel interface, and
unmount on drop.

See: [§6](#6-integration-test-harness).

### Step 5 — In-memory debug Continuum backend

**Target gate:** G2 support (not the G2 gate itself).

A scriptable fake Continuum backend for testing the projection adapter layer.
The G2 gate requires a real Echo rlib coordinate + observe cycle, not just the
fake. The debug backend enables projection-adapter unit tests without that
requirement.

See: [§8](#8-backend-progression).

---

## 5. Fixtures library

The fixture infrastructure is two crates with a clean boundary.

### Crate 1: `warp-drive-fixtures` (pure domain)

No FUSE dependency. No `tempfile`. No platform assumptions. Usable anywhere.

```rust
/// A declarative definition of a virtual filesystem tree.
pub struct FixtureTreeDef {
    pub nodes: Vec<NodeDef>,
}

pub struct NodeDef {
    pub ino: Ino,
    pub parent_ino: Ino,
    pub kind: NodeKind,
    pub content: NodeContentDef,
    pub name: &'static [u8],
}

pub enum NodeContentDef {
    File(&'static [u8]),
    Dir,          // children derived from other nodes' parent_ino
    Symlink(&'static [u8]),
}
```

Named fixtures:

| Name | Description | When to use |
|------|-------------|-------------|
| `FixtureTreeDef::g1_acceptance()` | The 13-node G1 gate tree | Gate tests, smoke tests |
| `FixtureTreeDef::minimal()` | Root + 1 file + 1 dir | Targeted unit tests for a single FUSE op |
| `FixtureTreeDef::deep_symlinks()` | Chain of symlinks | Readlink / symlink-resolution edge cases |
| `FixtureTreeDef::empty_dirs()` | Nested empty directories | Readdir edge cases |

Named fixtures grow as gate requirements grow. G2 fixtures exercise coordinate
switching and frontier advance. G3 fixtures add large files, binary content,
long paths.

### `FixtureTree::from_def`

`warp-drive-core` gains a validated constructor:

```rust
impl FixtureTree {
    pub fn from_def(def: FixtureTreeDef) -> Result<Self, FixtureError> { ... }
}

pub enum FixtureError {
    DuplicateIno(Ino),
    MissingParent { child: Ino, parent: Ino },
    RootNotFound,
}
```

Construction validates the tree and returns a typed error rather than
panicking — fixture bugs surface in tests, not at mount time.

### Crate 2: `warp-drive-test-harness` (Linux / FUSE)

Depends on `warp-drive-fixtures` + platform bits (`fuser`, `tempfile`).
Linux-only (`#[cfg(target_os = "linux")]`). Never compiled into production
binaries.

See [§6](#6-integration-test-harness) for the `MountGuard` design.

---

## 6. Integration test harness

### `MountGuard`

```rust
pub struct MountGuard {
    mount_point: TempDir,
}

impl MountGuard {
    /// Mount `tree` at a fresh temp directory.
    /// Polls until the FUSE process confirms the mount is live.
    pub fn mount(tree: FixtureTree) -> Result<Self, MountError> { ... }

    pub fn path(&self) -> &Path { ... }
}

impl Drop for MountGuard {
    fn drop(&mut self) {
        // fusermount3 -u self.mount_point (best-effort; logs on failure)
    }
}
```

### Assertion helpers

```rust
pub fn assert_file(guard: &MountGuard, rel: &str, expected: &[u8]);
pub fn assert_dir(guard: &MountGuard, rel: &str);
pub fn assert_symlink(guard: &MountGuard, rel: &str, target: &str);
pub fn assert_readonly(guard: &MountGuard, rel: &str);
pub fn assert_inode(guard: &MountGuard, rel: &str, expected_ino: u64);
```

### Example tests

```rust
#[test]
#[cfg(target_os = "linux")]
fn readme_content_is_correct() {
    let guard = MountGuard::mount(
        FixtureTree::from_def(FixtureTreeDef::g1_acceptance()).unwrap()
    ).unwrap();
    assert_file(&guard, "README.md", b"# WARP DRIVE G1 Fixture\n...");
}

#[test]
#[cfg(target_os = "linux")]
fn symlink_resolves_through_kernel() {
    let guard = MountGuard::mount(
        FixtureTree::from_def(FixtureTreeDef::g1_acceptance()).unwrap()
    ).unwrap();
    assert_symlink(&guard, "links/readme", "../README.md");
    assert_file(&guard, "links/readme", b"# WARP DRIVE G1 Fixture\n...");
}

#[test]
#[cfg(target_os = "linux")]
fn write_is_rejected_with_erofs() {
    let guard = MountGuard::mount(
        FixtureTree::from_def(FixtureTreeDef::g1_acceptance()).unwrap()
    ).unwrap();
    assert_readonly(&guard, "README.md");
    assert_readonly(&guard, "src/main.ts");
}
```

Each test gets its own `TempDir` — tests are fully isolated and run in parallel.

---

## 7. Safe-to-execute definition

A test is **safe to execute** if all five conditions hold:

1. **No production writes.** The test backend is in-memory. Even at G3+, tests
   never target a live Continuum cluster unless explicitly opted in with a
   flag that is not set in CI.

2. **Isolated.** Each test gets its own mount point (`TempDir`) and its own
   backend instance. Tests MUST NOT share state.

3. **Self-cleaning.** `MountGuard::drop` unmounts. If a test panics, the drop
   runs anyway (Rust guarantees). Leaked mounts are a test harness defect, not
   an accepted outcome.

4. **Idempotent.** Running a test twice in the same environment produces the
   same result. Tests MUST NOT depend on execution order or prior state.

5. **Network-free (unit and integration).** Unit and integration tests resolve
   all dependencies in-process. Gate acceptance tests (`cargo xtask acceptance`)
   MAY pull Docker images — that is the only permitted network dependency.

A test that violates any of these conditions MUST NOT land on `main`.

---

## 8. Backend progression

The same acceptance assertions run against every backend. Only the mount setup
changes. The long-term target is a conformance model where "passing G1
acceptance" means the same thing regardless of which backend serves the
projection — the empirical proof of the substrate-agnostic claim.

| Backend | Role | How it's wired | What it proves |
|---------|------|----------------|----------------|
| **In-memory fixture** | G1 gate ✅ | `FixtureTree::from_def(...)` | POSIX translation layer is correct |
| **In-memory debug Continuum** | G2 support | `DebugBackend::new()` — scriptable fake | Projection adapter handles coordinate switching, frontier advance, obstructions |
| **Embedded Echo rlib** | G2 gate | `EchoEmbedded::init()` over the `warp-wasm` native rlib surface | Real coordinate, real `observe` cycle, no network |
| **Live Echo / Continuum** | G4+ | `EchoRemote::connect(addr)` | Production integration; network-dependent, never in CI default |

**Critical distinction:** The G2 gate requires a real Echo rlib observe cycle —
not the debug backend. The debug backend is a G2 *support tool* that enables
projection-adapter unit tests without a live Echo runtime. These are different
things. Do not let the fake backend stand in for the real gate.

**On embedding:** G0 established the embedding path. The artifact is a native
Rust **rlib** linked directly into the binary — not a WASM module loaded via
wasmtime. G3's `EchoEmbedded` calls into that rlib surface. There is no
wasmtime in this stack.

### In-memory debug Continuum backend (G2 support)

The debug backend implements the Continuum protocol surface the projection
adapter requires. It is:

- **Scriptable:** `backend.set_file("src/main.ts", b"...")`, `backend.advance_frontier()`
- **Inspectable:** `backend.observed_ops()` returns the ops the adapter issued
- **Error-injectable:** `backend.fail_next_lookup(ObstructionReason::NotFound)` lets
  tests verify the adapter's error handling

Lives in `warp-drive-test-harness`. MUST NOT be compiled into production binaries.

### Conformance

At G3+, `cargo xtask acceptance --backend in-memory` and
`cargo xtask acceptance --backend echo-rlib` MUST both pass against the same
`scripts/acceptance.sh`. The script is backend-agnostic; only mount setup
differs.

Backend conformance is mechanical: write the backend, run the script, see if
it passes.

---

## 9. GitHub Actions CI

### Workflow: `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  acceptance:
    name: G1 gate acceptance (Linux / Docker)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.90.0"
      - name: Run G1 acceptance
        run: cargo xtask acceptance

  unit:
    name: Unit tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.90.0"
      - run: cargo test --workspace

  lint:
    name: Clippy + fmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: "1.90.0"
          components: clippy,rustfmt
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace -- -D warnings
```

### Toolchain pinning

The workspace specifies `rust-version = "1.90.0"`. CI pins to the same
version explicitly. Floating `stable` is how CI becomes haunted on a Tuesday.

### Branch protection

Add protection once the workflow passes twice on `ubuntu-latest`. Gate merges
to `main` on all three jobs. The `acceptance` job is the load-bearing one.

### Future jobs (added as gates advance)

| Job | Added at | What it runs |
|-----|----------|--------------|
| `integration` | Harness complete | `cargo test` with Linux FUSE harness |
| `acceptance-g2` | G2 gate | `cargo xtask acceptance --gate g2` |
| `acceptance-echo-rlib` | G3 gate | `cargo xtask acceptance --backend echo-rlib` |
