<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Fixture data mixed with tree traversal logic in `warp-drive-core`

**File:** `crates/warp-drive-core/src/lib.rs`

**Status:** acceptable at G1, refactor before writing acceptance tests.

## The smell

`FixtureTree::new()` constructs both the tree structure (inode map, parent
pointers, readdir entries) and embeds the hardcoded fixture data (file
contents, symlink targets, .warp/ surface) in the same function body.
There is no boundary between "the shape of a tree" and "the specific
content of the G1 fixture."

## Why it matters

The G1 acceptance script verifies specific file contents (`cat README.md`,
`readlink links/readme`, `cat .warp/coordinate`, etc.). Writing those tests
against the current structure requires coupling test fixtures to internal
construction details. When the fixture changes (even cosmetically) the tests
break in non-obvious ways.

It also makes it impossible to run the G1 acceptance with a different
fixture tree — e.g., a minimal 3-node tree for unit tests vs. the full
acceptance fixture.

## Resolution

Split into:
- `FixtureTreeData` (or a builder) — owns the node definitions and content
- `FixtureTree::from(data: FixtureTreeData)` — builds the inode map and
  index structures from the data

The G1 acceptance fixture becomes a named constant:
`FixtureTreeData::g1_acceptance()`. Unit tests can construct minimal trees
without hardcoding internal node IDs.
