<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Gate G2b - Echo-projected regular-file bytes

**Status:** PASS

**Date:** 2026-06-01 (America/Los_Angeles)

**Branch:** `gate/g2b`

**WARP DRIVE validated commit:** `edd5235f6287`

**Echo validated commit:** `d8da6d0478bb`

**Command:**

```sh
cargo xtask acceptance --gate g2b --runtime echo-rlib
```

**Result:** 60 / 60 assertions passed.

## Gate Condition

G2b passes when a Linux FUSE mount exposes a normal non-`.warp` regular file at
`/echo/head.json` whose bytes are obtained through an Echo observation/projection
response path and served through POSIX reads.

The specific proof path for this gate is:

1. `warp_wasm::init_embedded()` initializes the native Echo rlib runtime.
2. WARP DRIVE requests a query observation through `warp_wasm::observe_cbor()`.
3. The request uses `ObservationFrame::QueryView`.
4. The request uses `ObservationProjection::Query { query_id, vars_bytes }`.
5. Echo dispatches that query to a registered contract query observer.
6. Echo returns `ObservationPayload::QueryBytes { data }`.
7. WARP DRIVE inserts `data` as the contents of `/echo/head.json`.
8. The FUSE adapter serves `/echo/head.json` as a normal read-only file.

This gate does not pass by rendering `/.warp/coordinate` metadata into a normal
file. The bytes come back through Echo's query-observation payload path.

## What G2b Proves

1. Echo can produce normal file bytes through an observation/projection response.
2. WARP DRIVE can cache those bytes before `fuser::mount2()`.
3. FUSE worker threads can serve the cached bytes without calling Echo.
4. Normal POSIX tools can read the projected file:
   - `ls`
   - `cat`
   - `find`
   - `rg`
   - `stat`
5. The G2a coordinate metadata baseline still holds under the G2b mount.
6. The mounted tree remains read-only, including `/echo/head.json`.
7. The copy-in Docker runner can execute the gate without bind-mounting a live
   Git checkout.

## What G2b Does Not Prove

1. The whole filesystem tree is Echo-backed.
2. Directory entries are Echo-backed.
3. G1 sample files are Echo-backed.
4. Reads are live per-FUSE-call Echo observations.
5. Writes are lawful or basis-aware.
6. The final Echo filesystem contract exists.
7. The stale-save obstruction demo exists.

G2b proves first Echo-projected regular-file bytes. It is not a full Echo
filesystem projection.

## Echo Dependency

G2b depends on an Echo-side scaffold in `warp-wasm`.

Validated Echo commit:

```text
d8da6d0478bb test(warp-wasm): harden G2b experimental scaffold invariants
```

The validated Echo branch fences the temporary WARP DRIVE G2b query observer
behind Echo's explicit, non-default `experimental-warp-drive-g2b` feature. When
that feature is enabled, native `init_embedded()` registers the observer. The
observer handles the G2b query id and returns the `/echo/head.json` payload
through `ObservationPayload::QueryBytes`.

This is a scaffold query observer, not the final filesystem contract.

## Projected File Payload

The accepted projected file path is:

```text
/echo/head.json
```

Required payload shape:

```json
{
  "kind": "echo-projected-file",
  "gate": "G2b",
  "source": "echo-observation-payload",
  "worldline": "<64 hex>",
  "frontier": "<64 hex>",
  "state_root": "<64 hex>",
  "projection_hash": "<64 hex>"
}
```

The `source` field is part of the gate proof. It distinguishes Echo-produced
query payload bytes from WARP DRIVE-local formatting of coordinate metadata.

The `projection_hash` inside `/echo/head.json` is an Echo-side projection hash
derived by the query observer from the resolved worldline, frontier, and state
root. It is not the self-hash of the final query `ObservationArtifact`, because
that value cannot be embedded in its own payload before Echo computes the
artifact.

## Copy-in Docker Safety Invariant

G2b acceptance was run with the copy-in Docker runner.

Safety rules:

1. No acceptance container receives a bind mount of the live WARP DRIVE repo.
2. No acceptance container receives a bind mount of the live Echo repo.
3. Host-side `xtask` copies both repositories into a disposable staging
   directory.
4. The staged copies exclude `.git/`, `.gitmodules`, `target/`, and `.DS_Store`.
5. The generated Dockerfile strips `.git` and `.gitmodules` again inside the
   image.
6. The in-container `xtask` refuses to run if `.git`, `.gitmodules`, `GIT_DIR`,
   or `GIT_WORK_TREE` are present.
7. The Docker image is removed after the acceptance run.

The committed run printed the in-container isolation proof before building and
mounting:

```text
Copy-in acceptance isolation:
  PASS no git metadata in copied repos
```

## Validation Commands

WARP DRIVE validation at `edd5235f6287`:

```sh
cargo fmt --all
cargo fmt --manifest-path crates/warp-drive-echo-backend/Cargo.toml
cargo fmt --manifest-path crates/warp-drive-fuse-echo/Cargo.toml
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo clippy --manifest-path crates/warp-drive-fuse-echo/Cargo.toml --target-dir target/echo-rlib -- -D warnings
```

Echo validation at `d8da6d0478bb`:

```sh
cargo fmt --manifest-path crates/warp-wasm/Cargo.toml
cargo check --manifest-path crates/warp-wasm/Cargo.toml --features engine
cargo clippy --manifest-path crates/warp-wasm/Cargo.toml --features engine -- -D warnings
cargo test --manifest-path crates/warp-wasm/Cargo.toml --features engine --lib default_engine_tests
cargo check --manifest-path crates/warp-wasm/Cargo.toml --features experimental-warp-drive-g2b
cargo clippy --manifest-path crates/warp-wasm/Cargo.toml --features experimental-warp-drive-g2b -- -D warnings
cargo test --manifest-path crates/warp-wasm/Cargo.toml --features experimental-warp-drive-g2b --lib experimental_warp_drive_g2b_tests
cargo test --manifest-path crates/warp-wasm/Cargo.toml --all-features
```

All validation commands passed.

## Acceptance Transcript

Command:

```sh
cargo xtask acceptance --gate g2b --runtime echo-rlib
```

Runner setup:

```text
Building Docker image `warp-drive-g2b-echo-copyin-80862-1780306575968` from sanitized copies (no bind mounts)...
Running g2b echo-rlib acceptance in copy-in Docker container...
Copy-in acceptance isolation:
  PASS no git metadata in copied repos
Building local-only warp-drive-fuse Echo binary...
Running g2b echo-rlib acceptance script...
=== WARP DRIVE G2b acceptance ===

Mounting at /tmp/warp-g2b.30tjQp (echo-rlib backend, G2b gate) ...
Mounted (fuse pid 2786).
```

Acceptance assertions:

```text
-- ls ------------------------------------------------------------------
  PASS  ls / contains README.md
  PASS  ls / contains package.json
  PASS  ls / contains src/
  PASS  ls / contains empty/
  PASS  ls / contains links/
  PASS  ls / contains .warp/
  PASS  ls / contains echo/
  PASS  ls /echo contains head.json

-- cat -----------------------------------------------------------------
  PASS  README.md first line
  PASS  package.json name field
  PASS  src/main.ts export
  PASS  src/lib.ts export
  PASS  .warp/coordinate has worldline field
  PASS  .warp/coordinate has frontier field
  PASS  .warp/coordinate has state_root field
  PASS  .warp/coordinate has artifact_hash field
  PASS  .warp/coordinate identifies gate G2b
  PASS  .warp/runtime kind is echo-rlib
  PASS  .warp/runtime gate is G2b
  PASS  .warp/stats gate is G2b

-- G2a coordinate baseline ---------------------------------------------
  PASS  .warp/coordinate worldline is real (not genesis placeholder)
  PASS  .warp/coordinate worldline is 64-char non-zero hex
  PASS  .warp/coordinate frontier is 64-char non-zero hex
  PASS  .warp/coordinate state_root is 64-char non-zero hex
  PASS  .warp/coordinate artifact_hash is 64-char non-zero hex
  PASS  .warp/coordinate backend is echo-rlib

-- G2b projected file assertions ---------------------------------------
  PASS  /echo/head.json kind
  PASS  /echo/head.json gate
  PASS  /echo/head.json source
  PASS  /echo/head.json has worldline field
  PASS  /echo/head.json has frontier field
  PASS  /echo/head.json has state_root field
  PASS  /echo/head.json has projection_hash field
  PASS  /echo/head.json omits artifact_hash
  PASS  /echo/head.json omits POSIX path literal
  PASS  /echo/head.json worldline is 64-char non-zero hex
  PASS  /echo/head.json frontier is 64-char non-zero hex
  PASS  /echo/head.json state_root is 64-char non-zero hex
  PASS  /echo/head.json projection_hash is 64-char non-zero hex
  PASS  /echo/head.json worldline matches .warp/coordinate
  PASS  /echo/head.json frontier matches .warp/coordinate
  PASS  /echo/head.json state_root matches .warp/coordinate

-- find ----------------------------------------------------------------
  PASS  find sees src/main.ts
  PASS  find sees src/lib.ts
  PASS  find sees .warp/coordinate
  PASS  find sees empty/
  PASS  find sees links/readme
  PASS  find sees echo/head.json

-- rg ------------------------------------------------------------------
  PASS  rg finds export in main.ts
  PASS  rg finds export in lib.ts
  PASS  rg found 2 export hits (>= 2)
  PASS  rg finds projected file marker

-- stat ----------------------------------------------------------------
  PASS  stat shows inode 5 for src/main.ts
  PASS  stat shows src/main.ts regular file
  PASS  stat shows /echo/head.json regular file

-- readlink ------------------------------------------------------------
  PASS  links/readme -> ../README.md
  PASS  symlink resolves to README.md

-- write rejection -----------------------------------------------------
  PASS  write to README.md correctly rejected (EROFS)
  PASS  write to /echo/head.json correctly rejected (EROFS)
  PASS  create newfile.txt correctly rejected (EROFS)

G2b GATE PASSED  (60 / 60 assertions)
```

Cleanup:

```text
Untagged: warp-drive-g2b-echo-copyin-80862-1780306575968:latest
Deleted: sha256:071f93127b1e5b7cfc2b9e35fd945a48b4d2f83e972463fe7f3d77737acf2528
```

## Caveats

1. Only `/echo/head.json` is Echo-projected.
2. G1 fixture files remain fixture-backed.
3. G1 fixture directories remain fixture-backed.
4. The Echo query observer is a G2b scaffold behind Echo's explicit
   `experimental-warp-drive-g2b` feature.
5. The final filesystem contract still belongs to a later gate.
6. The local Echo gate still requires a sibling `../echo-warp-drive` checkout
   before the copy-in Docker image is built.

## Next Gate Direction

The next gate should move from one projected file toward an Echo-projected
directory/content contract. It should not add more metadata-only surface area
unless that surface is required to prove projected reads.
