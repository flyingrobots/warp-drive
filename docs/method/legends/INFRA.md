# Legend: INFRA

Infrastructure: CI, Docker, xtask, tooling, and the process scaffolding
that makes gate work reproducible and verifiable.

## What it covers

- GitHub Actions CI workflows and branch protection
- `Dockerfile` and Docker acceptance harness
- `cargo xtask` commands (install-deps, mount, unmount, acceptance)
- `scripts/acceptance.sh` — the Linux acceptance runner
- Workspace configuration, linter policy, hook scripts
- METHOD signpost maintenance (BEARING, VISION, CHANGELOG, PROCESS)

## What success looks like

Gate acceptance runs reliably in CI. A contributor with a fresh clone can
run `cargo xtask acceptance` and get a meaningful signal. The process is
self-documenting.

## How you know

- `cargo xtask acceptance` exits 0 in CI on `ubuntu-latest`
- Branch protection gates merges on CI passing
- A new contributor can reproduce a gate pass from a cold clone by
  following `CONTRIBUTING.md`
