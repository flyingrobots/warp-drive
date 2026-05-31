---
title: "`/.warp/` visibility semantics are conceptually ambiguous"
legend: GATE
lane: bad-code
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `/.warp/` visibility semantics are conceptually ambiguous

**File:** `scripts/acceptance.sh`, docs around `.warp/`

**Status:** docs/acceptance language is imprecise. Nail down before G3.

## The smell

The acceptance test had to switch from `ls` to `ls -a` because `.warp/` is
a hidden POSIX directory. That is correct behavior, but the docs sometimes
speak as though `.warp/` is simply "there" without distinguishing "visible
to `ls -a`" from "visible to ordinary tree walkers."

Some tools walk hidden dirs (`find`, `rg` with `--hidden`), some don't
(`ls`, `tree` by default). Some users will want `.warp/` visible for
inspection; others will want it hidden because ripgrep and IDE crawlers
get nosy.

## Why it matters

Without a declared contract, each tool integration makes its own assumption.
The plan already mentions making `.warp/` optionally hideable, which is good;
the acceptance language and docs need to be equally explicit.

## Resolution

Add a small explicit contract to the `.warp/` surface spec:

```
dotwarp=on      # default — hidden POSIX dir exposed at /.warp/
dotwarp=off     # no synthetic surface
dotwarp=debug   # extended diagnostics: cache, holograms, witnesses, errors
```

Acceptance assertions should explicitly use `ls -a` (not plain `ls`) and
comment why: "`.warp/` is a hidden dir by POSIX convention."
