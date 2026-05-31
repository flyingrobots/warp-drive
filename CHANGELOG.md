# Changelog

All notable changes to WARP DRIVE are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Added

- `gate/g2` branch: G2 design and acceptance work in progress.

---

## G1 — 2026-05-30

### Gate: PASSED (29/29 assertions)

First gate where normal POSIX tools touch a WARP-shaped filesystem and get
coherent answers.

### Added

- `warp-drive-fuse` binary: FUSE adapter serving the G1 in-memory fixture tree.
- `warp-drive-core`: `FixtureTree` — 13-node hardcoded fixture (README, src/, links/, .warp/).
- `cargo xtask install-deps` — installs macFUSE via Homebrew (macOS).
- `cargo xtask mount` / `unmount` — mount and unmount helpers.
- `cargo xtask acceptance` — Docker build + 29-assertion acceptance run; exits 0 on pass.
- `Dockerfile` — single-stage Linux build for acceptance testing.
- `scripts/acceptance.sh` — `ls`, `cat`, `find`, `rg`, `stat`, `readlink`, write-rejection.
- `docs/gates/G1.md` — gate record with full acceptance transcript.
- `docs/TESTING.md` — testing strategy: fixture library, `MountGuard` harness, backend progression.
- `.github/workflows/ci.yml` — Linux Docker acceptance CI (non-blocking pending first run).

---

## G0 — 2026-05-29

### Gate: PASSED

Proof that `warp-wasm` embeds as a native rlib and `observe_cbor` round-trips
from outside the echo workspace.

### Added

- `crates/warp-drive-g0-spike`: spike binary proving rlib embedding.
- `docs/gates/G0.md` — gate record.
- `docs/IMPLEMENTATION_PLAN.md` — operational plan v0.0.1 → v0.1.
- `docs/TECHNICAL_DEEP_DIVE.md` — architecture reference.
- `docs/ENGINEERING_STANDARDS.md` — coding standards and lint policy.
