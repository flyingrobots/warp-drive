# Changelog

All notable changes to WARP DRIVE are documented here.

Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

---

## [Unreleased]

### Added

- G2b passed and merged: `/echo/head.json` proves the first Echo-projected
  normal regular-file bytes through
  `ObservationProjection::Query -> ObservationPayload::QueryBytes`.
- G2b acceptance passed 60 / 60 assertions in the copy-in Docker runner.
- Echo PR #389 merged the temporary `warp-wasm`
  `experimental-warp-drive-g2b` scaffold behind a non-default feature.
- WARP DRIVE PR #2 merged the G2b FUSE integration, gate record, and sanitized
  copy-in Docker acceptance path.
- Copy-in Docker acceptance now stages sanitized source copies instead of
  bind-mounting live host repositories, excludes Git metadata before Docker
  build context creation, and refuses to run if Git metadata appears inside the
  container.
- G2a passed: Echo coordinate metadata is surfaced through a real FUSE mount
  via the local Echo rlib path.

### Caveats

- G2b proves only `/echo/head.json` as Echo-projected normal file content. G1
  fixture files and directories remain mostly fixture-backed.

### Next

- G3 is the next gate: trustworthy `.warp/` diagnostics and live operation
  counters, especially `/.warp/stats`, so future projection/write bugs are
  observable instead of guessed.

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
