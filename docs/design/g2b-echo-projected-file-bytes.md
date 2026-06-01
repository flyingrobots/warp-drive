<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# G2b - Echo-projected regular-file bytes

Status: ACCEPTED

Branch: `gate/g2b`

Proof record: [`docs/gates/G2b.md`](../gates/G2b.md)

## Purpose

G2a proved that WARP DRIVE can mount a read-only FUSE tree whose coordinate
metadata comes from the embedded Echo runtime.

G2b must prove a stronger read-path claim: at least one normal, non-`.warp`
regular file must serve bytes that came back through an Echo observation or
projection response path.

This gate must not pass by moving G2a metadata into a user-visible file and
calling that projection. A WARP DRIVE-rendered summary of already-cached
coordinate metadata is still metadata plumbing, not projected file content.

## Gate condition

G2b passes when:

1. `cargo xtask acceptance --gate g2b --runtime echo-rlib` passes in the Linux
   Docker FUSE runner.
2. The mounted filesystem contains `/echo/head.json` as a normal read-only
   regular file.
3. `cat /echo/head.json` returns bytes obtained through an Echo
   observation/projection response.
4. The projected file bytes are served through FUSE without calling Echo from
   FUSE worker threads.
5. `/.warp/coordinate` and `/.warp/runtime` remain Echo-derived as proven in
   G2a.
6. Existing G1 fixture files may remain fixture-backed, but that fallback must
   be documented in the gate record.

## Definition: projected file

For this gate, a projected file is a normal non-`.warp` regular file whose bytes
satisfy all of these requirements:

1. The bytes are requested from Echo after `warp_wasm::init_embedded()`.
2. The bytes are returned through an Echo ABI call, preferably
   `warp_wasm::observe_cbor()`.
3. The response payload is an Echo projection/observation payload, not a
   WARP DRIVE-local rendering of already-observed `/.warp/coordinate`
   metadata.
4. The bytes are cached before `fuser::mount2()`.
5. The FUSE adapter serves the cached bytes as a normal regular file.
6. Acceptance proves that the file is not one of the static G1 fixture files.

## Target file

The first projected file is:

```text
/echo/head.json
```

Expected JSON shape:

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

The `source` field is part of the proof boundary. If the file is produced by
WARP DRIVE from cached G2a coordinate metadata, the value must not be
`echo-observation-payload`, and the gate must not pass.

The `projection_hash` field in this file is an Echo-side projection hash
derived by the query observer from the resolved worldline, frontier, and state
root. It is not the self-hash of the final query `ObservationArtifact`; that
value cannot be embedded in its own payload before Echo computes it.

## Echo ABI path

The preferred implementation uses the existing observation query path:

1. Build an `ObservationRequest` for a query view.
2. Use `ObservationProjection::Query { query_id, vars_bytes }`.
3. Call `warp_wasm::observe_cbor()`.
4. Decode an `ObservationArtifact`.
5. Require `ObservationPayload::QueryBytes { data }`.
6. Use `data` as the exact bytes for `/echo/head.json`.

The relevant Echo ABI pieces already exist:

1. `ObservationProjection::Query { query_id, vars_bytes }`
2. `ObservationPayload::QueryBytes { data }`
3. The `warp-core` observation path dispatches query projections through a
   registered contract query observer and returns the observer bytes.

If the embedded Echo runtime does not yet register a query observer that can
produce the G2b file payload, add the smallest Echo-side query observer hook in
`../echo-warp-drive`. Do not synthesize the projected file in WARP DRIVE from
cached coordinate metadata just to avoid an Echo change.

Implementation choice:

1. Query id: `warp_wasm::experimental_warp_drive_g2b::HEAD_QUERY_ID`.
2. Vars bytes: `warp_wasm::experimental_warp_drive_g2b::HEAD_QUERY_VARS`,
   currently the canonical byte string `{"projection":"g2b-head","version":1}`.
3. Observer location: `warp-wasm`'s native engine kernel, registered only when
   the explicit `experimental-warp-drive-g2b` feature is enabled.
4. Payload source: `ObservationPayload::QueryBytes { data }`.

## Threading model

Echo calls remain on the startup thread:

1. Initialize Echo with `warp_wasm::init_embedded()`.
2. Observe coordinate metadata as in G2a.
3. Observe/project `/echo/head.json` bytes through the Echo query payload path.
4. Cache the returned metadata and file bytes.
5. Build the read-only fixture tree from cached values.
6. Start FUSE with `fuser::mount2()`.

FUSE worker threads must not call Echo for G2b. The current embedded Echo path
uses thread-local runtime state, so crossing into Echo from FUSE worker threads
would make the result scheduler/thread dependent.

## Mounted tree shape

G2b adds a narrow, explicit projected-file insertion point. It should not grow a
general mutable tree API yet.

Acceptable core shape:

```rust
pub fn with_warp_metadata_and_echo_head_file(
    coordinate: Vec<u8>,
    runtime: Vec<u8>,
    stats: Vec<u8>,
    echo_head_json: Vec<u8>,
) -> Result<Self, FixtureTreeError>
```

The ugly specificity is intentional. It marks the code as a gate scaffold and
avoids prematurely designing a general VFS builder before Echo-backed directory
and content contracts exist.

After G2b, a real fixture/tree definition layer can replace this with a cleaner
model.

## Acceptance requirements

The gate record must use the explicit command:

```sh
cargo xtask acceptance --gate g2b --runtime echo-rlib
```

The `echo-rlib` acceptance runner must be copy-in Docker only:

1. The host runner copies `warp-drive` and the sibling `echo-warp-drive` into a
   disposable staging directory.
2. The staged copies exclude `target/` and `.git/`.
3. The Docker image is built from the staged copies, not from a bind-mounted
   live checkout.
4. The image strips any remaining `.git`/`.gitmodules` metadata before running
   acceptance.
5. The container receives no `-v`/bind mount of the host repository.
6. The acceptance script runs only inside the container, guarded by
   `WARP_DRIVE_ACCEPTANCE_IN_CONTAINER=1`.

Acceptance must keep the G1 baseline:

1. `ls`, `cat`, `find`, `rg`, `stat`, and `readlink` continue to work.
2. Writes fail with read-only filesystem errors.
3. Fixture-backed G1 files retain their documented contents.

Acceptance must keep the G2a baseline:

1. `/.warp/coordinate` contains `worldline`, `frontier`, `state_root`, and
   `artifact_hash`.
2. Those hashes are non-zero 64-character lowercase hex strings.
3. `/.warp/runtime` identifies `echo-rlib`.

Acceptance must add the G2b proof:

1. The root directory contains `echo/`.
2. `/echo` contains `head.json`.
3. `find` sees `/echo/head.json`.
4. `stat` reports `/echo/head.json` as a regular file.
5. `cat /echo/head.json` returns valid JSON.
6. The JSON contains `"kind":"echo-projected-file"`.
7. The JSON contains `"gate":"G2b"`.
8. The JSON contains `"source":"echo-observation-payload"`.
9. The JSON contains `worldline`, `frontier`, `state_root`, and
   `projection_hash`.
10. Those values match, or are explicitly documented as consistent with,
    `/.warp/coordinate`.
11. `rg` finds `echo-projected-file` in `/echo/head.json`.
12. Writes to `/echo/head.json` fail with read-only filesystem errors.

Do not hard-code the final assertion count in prose before the Linux acceptance
transcript exists.

## What remains fixture-backed

G2b does not require the whole tree to come from Echo.

The following may remain fixture-backed:

1. G1 sample files.
2. G1 sample directories.
3. Existing symlinks.
4. Existing static fixture content used for POSIX compatibility checks.

The gate record must say this plainly. G2b proves the first Echo-projected
regular-file bytes, not a complete Echo filesystem contract.

## Non-goals

G2b does not implement:

1. Writes.
2. Basis-aware save receipts.
3. Full Echo filesystem contracts.
4. Dynamic directory enumeration from Echo.
5. Subscriptions or frontier refresh.
6. A daemon.
7. `.warp/` diagnostic expansion beyond what the proof needs.
8. The stale-save demo.

## Proof model

G2b is recorded as a local Linux proof using the copy-in Docker runner. The
runner requires a sibling `../echo-warp-drive` checkout on the host before it
builds the sanitized Docker context. CI promotion remains future dependency
work; see the gate record for the accepted transcript and caveats.

## Proof boundary

G2b is allowed to be small, but it must be honest.

Passing G2b means:

1. Echo produced at least one regular file's bytes through an observation or
   projection payload.
2. WARP DRIVE mounted those bytes as normal POSIX-readable file content.
3. The proof happened through the Linux FUSE acceptance runner.

Passing G2b does not mean:

1. All normal files are Echo-backed.
2. Directory entries are Echo-backed.
3. Reads are live per-FUSE-call observations.
4. Writes are lawful.
5. The final filesystem contract exists.

The next gate after G2b should move from "one projected file" toward an
Echo-projected directory/content contract, not more metadata polish.
