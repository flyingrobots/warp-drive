<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# WARP DRIVE — Process

WARP DRIVE uses [METHOD](https://github.com/flyingrobots/method) adapted
for gate-based development. This document is the cycle doctrine for this repo.

---

## Rules

- Gates are the unit of shipped work. A gate does not close until its
  acceptance condition is demonstrably true.
- Gate work happens on `gate/gN` branches and merges to `main` via PR.
- CI must be green on the PR before merge. The gate acceptance job is
  non-negotiable; lint and unit jobs must also pass.
- A gate record (`docs/gates/GN.md`) is created before the PR merges.
  It includes the acceptance transcript, the commit SHA, the runner, and
  all known caveats.
- Backlog maintenance happens at gate boundaries, not continuously.
- The backlog lives in `docs/method/backlog/`. Moving a file between lanes
  is a decision. Moving it to `graveyard/` is permanent.
- Design docs for gate work live in `docs/design/`. Pull a backlog item
  before writing the design doc.
- `docs/BEARING.md` and `CHANGELOG.md` are updated on `main` after each
  gate merges (ship sync).
- An open cycle packet on `main` is repo-truth drift. Stop and repair it
  before continuing.

---

## Gate lifecycle

```text
backlog item → design doc → RED → GREEN → acceptance run → gate record
  → PR (CI green) → merge to main → ship sync
```

### 1. Design

Pull an item from the backlog into `docs/design/<slug>.md`. The design names:
- The gate condition (exact, falsifiable).
- The implementation approach.
- Open questions and risks.
- Acceptance criteria (maps directly to the acceptance script).

### 2. Red

Write failing tests or acceptance assertions before writing the implementation.
Tests are the spec. Do not write an implementation before its tests exist.

### 3. Green

Make the tests pass. Keep CI green throughout.

### 4. Acceptance run

`cargo xtask acceptance` exits 0. If it does not, the gate is not done.

### 5. Gate record

Create `docs/gates/GN.md` with:
- Date, commit SHA, runner
- Full acceptance transcript
- Caveats (what was not tested, what platform limitations exist)

### 6. PR and merge

Open a PR from `gate/gN` to `main`. CI must pass. No merge without green CI.

### 7. Ship sync

After merge, on `main`:
- Update `docs/BEARING.md`
- Update `CHANGELOG.md`
- Refresh `docs/VISION.md` if the gate significantly changes the project shape

---

## Branch naming

| Branch type | Pattern | Example |
|-------------|---------|---------|
| Gate work | `gate/gN` | `gate/g2` |
| Backlog cycle | `cycles/LEGEND_slug` | `cycles/INFRA_ci-hardening` |
| Maintenance | `maint-slug` | `maint-fix-typos` |
| Triage | `triage-slug` | `triage-backlog-capture` |

---

## Backlog lanes

| Lane | What goes here |
|------|----------------|
| `inbox/` | Raw ideas, unprocessed observations |
| `asap/` | Pull next |
| `cool-ideas/` | Experiments, future directions |
| `bad-code/` | Tech debt, structural jank |
| `graveyard/` | Retired work (permanent) |

---

## Legends

| Legend | What it covers |
|--------|----------------|
| `GATE` | Gate-level work: acceptance tests, projection adapters, FUSE semantics, domain design |
| `INFRA` | Infrastructure: CI, Docker, xtask, tooling, process |

---

## Ship sync checklist

After a gate branch merges to `main`:

- [ ] `docs/BEARING.md` updated (current priority, recent ships)
- [ ] `CHANGELOG.md` entry added
- [ ] `docs/VISION.md` refreshed if the gate changes the project shape
- [ ] Any resolved backlog cards deleted or moved to `graveyard/`
- [ ] Open cycle packets on `main`: zero (stop if nonzero)
