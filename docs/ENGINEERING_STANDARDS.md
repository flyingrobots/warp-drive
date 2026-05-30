<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# WARP DRIVE Engineering Standards

WARP DRIVE is a POSIX-shaped membrane over causal history. The engineering
standard is therefore stricter than a normal prototype: incorrect semantics
are worse than missing features.

**An LLM-generated patch that weakens these standards is invalid unless the
user explicitly requested the weakening.**

---

## Prime directive

Prefer an explicit unsupported operation over an approximate lie.

```text
correctness > portability > observability > performance > cleverness
```

---

## 1. Language

`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, and `MAY` carry RFC-2119 meanings
throughout all project documentation and gate records.

Project terms MUST match the glossary in `TECHNICAL_DEEP_DIVE.md`. The
canonical set:

| Use | Never use |
|---|---|
| `siteId` | `nodeId`, `pathId`, `thingId` |
| `projection` | content reading, value, blob |
| `intent` | write request, mutation |
| `receipt` | result, error blob, response |
| `obstruction` | failure, conflict |
| `coordinate` | branch (except when mapping explicitly to Git) |
| `frontier` | head (except when mapping explicitly to Git) |

Every architectural term gets one definition. Aliases are permitted only
when explicitly declared in the glossary.

**Forbidden vague implementation terms** in standards documents, gate records,
and code comments:

- `probably`
- `kind of`
- `should work`
- `temporary` — unless linked to a tracked issue
- `POSIX-like` — unless the exact operations are named
- `best effort` — unless the failure mode is defined

---

## 2. Zero-warning policy

Warnings are failures. All of them. Add to `Cargo.toml` workspace manifest:

```toml
[workspace.lints.rust]
warnings          = "deny"
missing_docs      = "deny"
unsafe_code       = "forbid"
unused_must_use   = "deny"
unreachable_pub   = "deny"
rust_2018_idioms  = "deny"

[workspace.lints.clippy]
all               = "deny"
pedantic          = "deny"
nursery           = "deny"
cargo             = "deny"
unwrap_used       = "deny"
expect_used       = "deny"
panic             = "deny"
todo              = "deny"
unimplemented     = "deny"
dbg_macro         = "deny"
print_stdout      = "deny"
print_stderr      = "deny"
wildcard_imports  = "deny"
enum_glob_use     = "deny"
```

Any exception MUST be local, narrow, and commented:

```rust
#[allow(clippy::module_name_repetitions)]
// Reason: public API name intentionally repeats crate prefix for rustdoc clarity.
```

No crate-level lint amnesty. No "just for now."

---

## 3. Quality gates

Required before every commit:

```sh
cargo fmt --all --check
cargo check --workspace --all-targets --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
```

Preferred extended gate:

```sh
cargo nextest run --workspace --all-features
cargo test --workspace --doc
cargo deny check
cargo audit
cargo machete
```

Cross-compile checks (required before each gate milestone):

```sh
cargo check --target x86_64-unknown-linux-gnu
cargo check --target aarch64-apple-darwin
cargo check --target x86_64-apple-darwin
```

Miri is required for pure domain crates once they exist. Do not run Miri
against FUSE/platform crates.

---

## 4. Hexagonal architecture

Domain core MUST NOT depend on:

- FUSE, libfuse, macFUSE, WinFsp, Dokany
- `libc` (except through platform adapters)
- Echo internals
- OS paths or filesystem operations
- environment variables
- wall-clock time (inject clocks)
- async runtime (inject executors if needed)

### Crate shape

```text
crates/
  warp-drive-core/             # domain model, virtual paths, inode policy, attrs, errors
  warp-drive-membrane/         # POSIX translation, routing, cache, basis discipline
  warp-drive-driver/           # ContinuumClient trait definition
  warp-drive-driver-memory/    # in-memory fake runtime for tests (G1)
  warp-drive-driver-echo/      # Echo rlib adapter (G2)
  warp-drive-platform/         # platform-neutral port traits
  warp-drive-platform-fuse/    # Unix/macOS FUSE adapter
  warp-drive-platform-winfsp/  # Windows adapter (skeleton until testable)
  warp-drive-cli/              # argument parsing, command dispatch
```

### The membrane owns meaning; the adapter owns ugliness

Bad:

```rust
impl fuser::Filesystem for WarpDrive {
    fn write(...) {
        // compute basis, submit intent, update receipt log — all here
    }
}
```

Good:

```rust
impl fuser::Filesystem for FuseAdapter {
    fn write(...) {
        self.port.write(HostWriteRequest::from_fuse(...))
    }
}
```

The FUSE layer translates host syscalls into domain operations. It does not
decide causal semantics.

---

## 5. Platform support tiers

| Tier | Target | Status |
|---|---|---|
| 1 — design | Linux + macOS | full semantic support |
| 1 — CI | Linux | gating |
| 1 — local | macOS | gating |
| 2 | macOS CI | when available |
| 3 | Windows | compile skeleton only until test infrastructure exists |

Windows MUST NOT block G1–G3 unless a real Windows test path exists. Design
the crate seam for Windows portability; do not pretend to support what cannot
be tested.

Platform-specific behavior lives behind adapters:

```rust
trait PlatformPolicy {
    fn case_policy(&self) -> CasePolicy;
    fn timestamp_granularity(&self) -> TimestampGranularity;
    fn supports_readlink(&self) -> bool;
    fn max_name_len(&self) -> usize;
}
```

No `#[cfg(target_os = ...)]` inside domain crates.

---

## 6. POSIX conformance policy

WARP DRIVE implements a declared subset of POSIX, not all POSIX.

Every filesystem operation MUST be classified as one of:

| Class | Meaning |
|---|---|
| `supported` | works correctly at this gate |
| `supported-with-caveats` | works with documented limitations |
| `unsupported-by-design` | intentionally not supported; returns principled errno |
| `not-yet-implemented` | planned; currently returns `ENOSYS` |

Principled errno mapping:

| Situation | errno |
|---|---|
| Shared writable mmap | `ENODEV` |
| Unsupported operation | `EOPNOTSUPP` |
| No such projected site | `ENOENT` |
| Coordinate/frontier no longer available | `ESTALE` |
| Stale basis or concurrent conflict | `EBUSY` |
| Runtime evidence failure or transport failure | `EIO` |

The membrane MUST NOT fake:

- shared writable mmap
- file locks
- atomic rename on runtimes that cannot provide atomic multi-site admission
- successful `fsync` before receipt admission or obstruction is known

---

## 7. Path handling

Host paths are bytes (Unix) or UTF-16 (Windows). Do not conflate them with
internal virtual paths.

Rules:

- Host paths enter only at platform adapters
- Internal virtual paths use a dedicated type
- Runtime addresses are `siteId`s, never paths
- `.warp/` paths are reserved and cannot collide with runtime sites

Type shapes:

```rust
struct HostPathBytes(Vec<u8>);        // Unix adapter only
struct HostPathWide(Vec<u16>);        // Windows adapter, future
struct VirtualPath(Vec<PathComponent>);
struct PathComponent(Vec<u8>);        // not String — not assumed UTF-8
struct SiteId(String);
```

Do not assume UTF-8 in path components. Do not normalize case unless the
platform policy explicitly declares case-insensitive behavior.

---

## 8. Error handling

Production code MUST NOT use `unwrap`, `expect`, `panic!`, `todo!`,
`unimplemented!`, `dbg!`, `print!`, or `println!`.

Errors are typed at the domain layer:

```rust
enum MembraneError {
    NoSuchSite { site: SiteId },
    StaleBasis { held: BasisId, current: BasisId },
    RuntimeObstruction { receipt: Receipt },
    Unsupported { operation: Operation, reason: UnsupportedReason },
    Platform { source: PlatformError },
}
```

Errno translation happens only at the platform edge:

```rust
impl From<MembraneError> for libc::c_int {
    fn from(error: MembraneError) -> Self { /* narrow mapping */ }
}
```

The domain core does not speak `errno` except through that translation layer.

---

## 9. Observability

Observability is not a debug feature. A FUSE membrane without telemetry is
an untestable system.

By G3, `/.warp/stats` MUST expose counters for:

```text
lookup_count
getattr_count
readdir_count
open_count
read_count
readlink_count
cache_hits
cache_misses
negative_lookup_hits
runtime_observe_count
avg_lookup_latency_us
avg_read_latency_us
last_error
```

Every obstruction must be inspectable. Every unsupported semantic must appear
in `.warp/stats` as a classified non-attempt, not a silent hole.

---

## 10. Testing layers

Use four distinct layers:

| Layer | Scope | No OS? |
|---|---|---|
| unit | pure functions, no OS, no mount | yes |
| contract | adapter traits + fake drivers | yes |
| golden | fixture tree expected outputs | yes |
| platform | actual mount behavior | no |

### G1 golden fixture

```text
/
  README.md
  package.json
  src/
    main.ts
    lib.ts
  empty/
  links/
    readme -> ../README.md
/.warp/
  coordinate
  runtime
  stats
```

### G1 acceptance script

```sh
MOUNT=/tmp/warp-drive-g1
cargo run --bin warp-drive-fuse -- --runtime=in-memory --mount "$MOUNT"

ls -la "$MOUNT"
find "$MOUNT"
cat "$MOUNT/README.md"
rg "export" "$MOUNT"
stat "$MOUNT/src/main.ts"
readlink "$MOUNT/links/readme"
cat "$MOUNT/.warp/stats"

# Negative: G1 is read-only; this MUST fail cleanly
echo nope > "$MOUNT/README.md" && echo "FAIL: write should have been rejected" || echo "ok: write rejected"
```

A gate CANNOT be marked complete until its acceptance commands are recorded
with their actual output.

---

## 11. Tool compatibility matrix policy

The compatibility matrix in `IMPLEMENTATION_PLAN.md §11.5` is a living
artifact. A gate cannot be marked complete until each target tool is
classified as:

- `verified` — tested, output recorded
- `unsupported-by-design` — documented, returns principled errno
- `blocked` — linked to a tracked issue

`works on my machine` is not a classification. The machine, OS, command, and
output must be recorded.

---

## 12. Dependency policy

Before adding a dependency, answer:

1. What problem does this solve?
2. Why not the standard library?
3. Why this crate over alternatives?
4. What platform support does it imply?
5. What is its license? Is it compatible with Apache 2.0?
6. What is the removal plan if it becomes wrong?

Rules:

- No platform dependency in `warp-drive-core` or `warp-drive-membrane`
- No duplicate crates performing the same job
- No async runtime in domain crates unless unavoidable
- `Cargo.lock` is committed; the lockfile is not optional for a binary workspace
- Run `cargo deny check`, `cargo audit`, `cargo machete` before each gate

---

## 13. Unsafe policy

Initial rule: `#![forbid(unsafe_code)]` in all crates.

If platform adapters eventually require `unsafe`:

- `unsafe` is permitted only in platform adapter crates
- Every `unsafe` block MUST have a `// SAFETY:` comment
- `unsafe` code MUST have focused tests
- `unsafe` MUST NOT cross into domain crates

Crates where `unsafe` is permanently forbidden:

- `warp-drive-core`
- `warp-drive-membrane`
- `warp-drive-driver`
- `warp-drive-driver-memory`

---

## 14. Concurrency policy

FUSE is concurrent. Design for it from the start.

Rules:

- No global mutable state in the membrane
- No hidden singleton handles in long-term driver APIs (the G0 spike's
  global kernel is a spike artifact, not a model for production code)
- Every cache declares its key, eviction policy, and invalidation policy
- Every file handle has explicit handle state
- Every mount has isolated mount state

---

## 15. Documentation requirements

Every crate MUST have a crate-level doc comment stating:

```rust
//! What this crate owns.
//! What this crate must not know.
//! Which layer it belongs to (core / membrane / driver / platform / cli).
//! Which gate introduced it.
```

Every public type MUST have a doc comment. `missing_docs = "deny"` enforces
this automatically.

Every platform caveat gets a doc comment, not tribal memory.

Every unsupported semantic MUST be documented with:

- the operation name
- the reason it is unsupported
- the errno or result returned
- the test that verifies the failure mode
- the user-visible documentation entry

---

## Gate sequence summary

| Gate | What it proves | Minimum acceptance |
|---|---|---|
| G0 ✅ | rlib embedding + `observe_cbor` round-trip | spike binary exits 0 |
| G1 | POSIX translation + inode + `.warp/` | fake tree, 7 syscalls, acceptance script |
| G2 | Echo read-only mount | real coordinate, real observe |
| G3 | `.warp/` diagnostics + perf counters | stats file readable, counters non-zero |
| G4 | Whole-file writes with basis tracking | basis discipline holds under concurrent write |
| G5 | Stale-basis obstruction + receipt UX | typed receipt under `/.warp/intents/` |
| G6 | `create`, `unlink`, `rename` | basis discipline applies to structure mutations |
| G7 | Two mounts at different coordinates | isolated mount state, no cross-contamination |
| G8 | Second driver passes same membrane tests | substrate-agnostic claim becomes empirical |

See `IMPLEMENTATION_PLAN.md §11` for the full gate model.
