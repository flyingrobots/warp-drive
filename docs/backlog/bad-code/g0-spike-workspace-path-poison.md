<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `warp-drive-g0-spike` poisons the workspace with unreachable path deps

**File:** `Cargo.toml` (workspace root), `crates/warp-drive-g0-spike/Cargo.toml`

**Status:** acceptable while only James works locally; fix before onboarding contributors or adding CI.

## The smell

`warp-drive-g0-spike` is a frozen proof-of-concept (never changes, never ships).
It depends on:

```toml
echo-wasm-abi = { path = "../echo-warp-drive/crates/echo-wasm-abi" }
warp-wasm     = { path = "../echo-warp-drive/crates/warp-wasm" }
```

These paths only exist in James's local worktree setup. Any checkout that
doesn't also have `../echo-warp-drive` present cannot fully resolve the
workspace, making CI, Docker, and fresh contributor clones all require a
workaround.

Current workaround in `Dockerfile`:

```bash
sed -i '/"crates\/warp-drive-g0-spike"/d' Cargo.toml
sed -i '/^echo-wasm-abi/d; /^warp-wasm/d' Cargo.toml
```

This is a smell: production files are being patched at build time to paper
over a structural dependency problem.

## Resolution

Remove `crates/warp-drive-g0-spike` from `[workspace] members`. It is
frozen G0 work — it doesn't need to be in the active workspace. Keep the
source in the repo for reference; just don't include it in the workspace.

Remove the `echo-wasm-abi` and `warp-wasm` entries from
`[workspace.dependencies]` at the same time (nothing in the active workspace
uses them).

The Dockerfile `sed` hacks can then be deleted entirely.
