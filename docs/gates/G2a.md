<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Gate G2a — Echo coordinate metadata mount

**Date:** 2026-05-31
**Branch:** `gate/g2`
**Validated commit:** `093f5c5` (acceptance proof strengthened and validated)
**Gate record commit:** `d32254e` (acceptance transcript recorded)
**Status:** PASS
**Command:** `cargo xtask acceptance --runtime=echo-rlib`
**Runner:** Docker Desktop on macOS host, Linux container `rust:1.90`
with `/Users/james/git` mounted at `/work`

---

## Gate condition

> A read-only FUSE mount initializes the embedded Echo rlib on the main thread,
> performs one real `observe_cbor()` head observation, and serves the resulting
> coordinate metadata through `/.warp/coordinate` and `/.warp/runtime`.

The user-facing POSIX tree remains the G1 fixture. This gate does not claim
that file bytes are projected from Echo.

---

## What G2a Proves

1. `warp-drive-fuse --runtime=echo-rlib` can link the native `warp-wasm` rlib
   through the local `../echo-warp-drive` checkout.
2. `warp_wasm::init_embedded()` succeeds before `fuser::mount2` starts FUSE
   worker threads.
3. `warp_wasm::observe_cbor()` returns a decodable
   `OkEnvelope<ObservationArtifact>` for the default worldline frontier.
4. `/.warp/coordinate` contains Echo-derived `worldline`, `frontier`,
   `state_root`, `tick`, and `artifact_hash` fields.
   Acceptance requires `frontier`, `state_root`, and `artifact_hash` to be
   non-zero 64-character lowercase hex values.
5. `/.warp/runtime` identifies the `echo-rlib` backend and `G2a` gate.
6. The G1 POSIX read surface still works while metadata comes from Echo.

---

## What G2a Does Not Prove

- `README.md`, `package.json`, `src/main.ts`, and `src/lib.ts` are still G1
  fixture bytes.
- Directory listings still come from `FixtureTree::new()`.
- Echo does not yet provide filesystem projection handlers or file content.
- FUSE worker threads do not call into Echo; they only read the cached tree.

Full Echo-projected file bytes belong to the next gate, currently named G2b/G3
in the design notes.

---

## Acceptance Run

The gate was run inside Linux Docker with both sibling repositories visible:

```sh
docker run --rm \
  --device /dev/fuse \
  --cap-add SYS_ADMIN \
  -v /Users/james/git:/work \
  -w /work/warp-drive \
  rust:1.90 \
  bash -lc 'set -euo pipefail; export PATH=/usr/local/cargo/bin:$PATH; export DEBIAN_FRONTEND=noninteractive; apt-get update; apt-get install -y --no-install-recommends fuse3 ripgrep pkg-config libssl-dev; cargo xtask acceptance --runtime echo-rlib'
```

The first Docker attempt used `bash -lc` without explicitly prepending
`/usr/local/cargo/bin` to `PATH`; that failed before acceptance with
`cargo: command not found`. The command above is the successful gate run.

---

## Acceptance Transcript

```text
=== WARP DRIVE G2a acceptance ===

Mounting at /tmp/warp-g2 (echo-rlib backend) ...
Mounted (fuse pid 2906).

-- ls -----------------------------------------------------------------
  PASS  ls / contains README.md
  PASS  ls / contains package.json
  PASS  ls / contains src/
  PASS  ls / contains empty/
  PASS  ls / contains links/
  PASS  ls / contains .warp/

-- cat ----------------------------------------------------------------
  PASS  README.md first line
  PASS  package.json name field
  PASS  src/main.ts export
  PASS  src/lib.ts export
  PASS  .warp/coordinate has worldline field
  PASS  .warp/coordinate has frontier field
  PASS  .warp/coordinate has state_root field
  PASS  .warp/coordinate has artifact_hash field
  PASS  .warp/coordinate identifies gate G2a
  PASS  .warp/runtime kind is echo-rlib
  PASS  .warp/runtime gate is G2a
  PASS  .warp/stats gate is G2a

-- G2a coordinate assertions ------------------------------------------
  PASS  .warp/coordinate worldline is real (not genesis placeholder)
  PASS  .warp/coordinate frontier is 64-char non-zero hex
  PASS  .warp/coordinate state_root is 64-char non-zero hex
  PASS  .warp/coordinate artifact_hash is 64-char non-zero hex
  PASS  .warp/coordinate backend is echo-rlib

-- find ---------------------------------------------------------------
  PASS  find sees src/main.ts
  PASS  find sees src/lib.ts
  PASS  find sees .warp/coordinate
  PASS  find sees empty/
  PASS  find sees links/readme

-- rg -----------------------------------------------------------------
  PASS  rg finds export in main.ts
  PASS  rg finds export in lib.ts
  PASS  rg found 2 export hits (>= 2)

-- stat ---------------------------------------------------------------
  PASS  stat shows inode 5 for src/main.ts
  PASS  stat shows regular file

-- readlink -----------------------------------------------------------
  PASS  links/readme -> ../README.md
  PASS  symlink resolves to README.md

-- write rejection ----------------------------------------------------
  PASS  write to README.md correctly rejected (EROFS)
  PASS  create newfile.txt correctly rejected (EROFS)

========================================================================
G2a GATE PASSED  (37 / 37 assertions)
```

---

## Caveats

- File content is still seeded from the G1 fixture tree. G2a proves Echo
  coordinate metadata integration, not Echo-projected file bytes.
- The local Echo gate requires a sibling `../echo-warp-drive` checkout. Default
  workspace builds remain Echo-free.
