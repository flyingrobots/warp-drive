<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Vestigial `Runtime` enum with a single variant

**File:** `crates/warp-drive-fuse/src/main.rs` — `enum Runtime`

**Status:** minor; delete or promote when a second runtime exists.

## The smell

`Runtime` has exactly one variant (`InMemory`) and the CLI default is that
variant. Clap will reject any other value. The enum exists solely to model
future extensibility that does not yet exist. It adds ceremony without
adding capability.

## Why it matters

Code that pretends to be extensible without actually being extensible is
harder to read than code that is honestly limited. A reader seeing a
`match cli.runtime { InMemory => ... }` assumes there are other arms
somewhere; there are not. It also means any future addition must keep the
illusion alive or delete the plumbing entirely.

## Resolution

Delete `Runtime` and the `--runtime` flag. When a second runtime lands
(G3: live Echo projection), introduce the enum then with real variants and
real dispatch. Two variants justify the abstraction; one does not.
