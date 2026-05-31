<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Gate G2a — Echo coordinate metadata mount

**Date:** 2026-05-31  
**Branch:** `gate/g2`  
**Status:** implementation pending acceptance  
**Command:** `cargo xtask acceptance --runtime=echo-rlib`  

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
