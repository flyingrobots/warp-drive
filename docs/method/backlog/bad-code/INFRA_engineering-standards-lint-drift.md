---
title: "Engineering standards doc drifts from actual Cargo lint policy"
legend: INFRA
lane: bad-code
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Engineering standards doc drifts from actual Cargo lint policy

**File:** `docs/ENGINEERING_STANDARDS.md`, root `Cargo.toml`

**Status:** docs lie. Future agents will act on the lie.

## The smell

The standards doc originally wanted maximum strictness: `pedantic`, `nursery`,
maybe `cargo` lints. Reality pushed the workspace toward targeted safety lints
because pedantic/nursery produced noise and contradictory pressure. That is
probably the correct practical call — but the doc must not keep preaching a
policy the repo does not enforce.

## Why it matters

Standards that lie are worse than no standards. Future agents will "fix" the
repo *back* toward lint theater: flattening modules, making pointless
`const fn` cargo-cult changes, renaming things because a nursery lint says so.
We already saw that nonsense. The doc is an attack surface for wasted churn.

## Resolution

Update standards to a tiered policy:

**Required (workspace-deny):**
- correctness lints
- `unwrap`/`expect`/`panic`/`todo`/`unimplemented`/`dbg`/`print` in production code
- missing docs for public API
- `unsafe` forbidden outside explicit adapter exceptions

**Experimental (run manually, never workspace-deny):**
- `pedantic`
- `nursery`
- `cargo`

The project's own standards already say correctness outranks cleverness.
Enforce that spirit, not lint maximalism cosplay.
