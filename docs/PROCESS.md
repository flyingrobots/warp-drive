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
- Design docs for gate work live in `docs/design/`. Pull a backlog item
  before writing the design doc.
- Cross-gate living behavior (a surface that outlives the gate that
  introduced it — `.warp/` diagnostics, the `FixtureTree` model, the Echo
  observation seam) is documented in `docs/topics/<topic>/`, updated
  whenever that behavior changes, not only at ship sync. See
  `docs/DOCUMENTATION_STANDARDS.md`.
- `docs/BEARING.md` and `CHANGELOG.md` are updated on `main` after each
  gate merges (ship sync).
- An open cycle packet on `main` is repo-truth drift. Stop and repair it
  before continuing.

### Backlog: GitHub Issues, not `docs/method/backlog/`

New planning content — cool ideas, known code smells, deferred work — goes
to GitHub Issues on this repo. `docs/method/backlog/` is historical: the
lanes below and their existing cards stay as record, but nothing new gets
added there. `docs/method/legends/` (the `GATE`/`INFRA` doctrine below) is
unaffected — it still defines what counts as gate-level vs. infra-level
work, it just no longer describes a backlog's lane structure for new items.

---

## Gate lifecycle

```text
GitHub issue → design doc → RED → GREEN → acceptance run → gate record
  → PR (CI green) → merge to main → ship sync
```

### 1. Design

Pull an issue into `docs/design/<slug>.md`. The design names:

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

## Backlog lanes (historical)

`docs/method/backlog/` predates the move to GitHub Issues. Existing cards
stay in place as record; the lane meanings below still apply to reading
them, but new items are GitHub issues, not new cards.

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
- [ ] Affected `docs/topics/<topic>/README.md` pages updated to match `main`
- [ ] Resolved GitHub issues closed with a reference to the merged PR
- [ ] Open cycle packets on `main`: zero (stop if nonzero)
