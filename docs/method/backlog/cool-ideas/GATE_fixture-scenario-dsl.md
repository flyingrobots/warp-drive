---
title: "Fixture scenario DSL — executable fixture stories"
legend: GATE
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Fixture scenario DSL — executable fixture stories

**Status:** cool idea. Makes the stale-save demo executable.

Related cards: `GATE_stale-save-demo`, `GATE_fixture-data-mixed-with-tree-logic`

## The idea

Not just fixture trees — fixture stories.

```rust
Scenario::new("stale-save")
    .file("README.md", b"v1")
    .mount("@main")
    .open("editor-a", "README.md")
    .open("editor-b", "README.md")
    .write("editor-a", b"v2")
    .flush("editor-a").expect_admitted()
    .write("editor-b", b"v3")
    .flush("editor-b").expect_errno(EBUSY)
    .expect_receipt("STALE_BASIS");
```

A fluent builder that describes the multi-actor sequence, drives it against
a live mount, and asserts the expected outcomes — both the POSIX error codes
and the `.warp/intents/last` receipt content.

## Why it matters

The stale-save demo is the tactile "holy shit" moment that proves WARP
DRIVE's moral argument. Right now it lives as prose in a cool-ideas card.
The DSL makes it a test that can be run, regressed, and committed.

The DSL also makes it trivially easy to add new scenario variants without
writing boilerplate mount/unmount/assert scaffolding for each one.

## Surface when

Designing the concurrent-write test harness, or when the stale-save demo
is being promoted to G5 acceptance criteria.
