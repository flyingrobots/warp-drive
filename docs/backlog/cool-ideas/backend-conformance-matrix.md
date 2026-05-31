<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Backend conformance matrix

**Status:** cool idea. Surface when wiring G3 acceptance.

## The idea

Run every gate acceptance suite against every backend. Make substrate-independence
mechanically visible rather than just claimed:

| Backend          | G1 | G2 | G3 | G4+ | Notes                           |
|------------------|----|----|----|-----|---------------------------------|
| fixture          | ✅ | —  | —  | —   | Static tree only                |
| debug-continuum  | ✅ | 🎯 | —  | —   | Scriptable fake                 |
| echo-rlib        | ✅ | 🎯 | 🎯 | —   | Real embedded runtime           |
| git-warp         | —  | —  | —  | ⏳  | Future — Continuum over git     |

(✅ = passing, 🎯 = target, ⏳ = future, — = not applicable)

`cargo xtask acceptance --backend <name>` runs the same `scripts/acceptance.sh`
against the named backend. The matrix documents which combinations have been
verified.

## Why it matters

WARP DRIVE's product claim is that it is a POSIX⇄Continuum membrane, not an
Echo-specific toy. The conformance matrix is the empirical proof of that claim:
if the same 29 assertions pass against the in-memory fixture AND against a live
Echo rlib coordinate, the abstraction boundary is real.

Adding a new backend = filling in a row. Regressing a cell = a gate failure.

## Surface when

Designing G3 acceptance or adding the first second backend (debug-continuum
at G2).
