---
title: "Vestigial single-variant Runtime enum"
legend: GATE
lane: bad-code
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Vestigial single-variant Runtime enum

**File:** `crates/warp-drive-fuse/src/main.rs`

**Status:** low priority. Delete before G3 adds a real second runtime.

## The smell

`Runtime::InMemory` is the only variant. A single-variant enum is dead
ceremony — it adds pattern matching without adding any meaningful
dispatch. The `--runtime` CLI flag has one valid value.

## Resolution

Delete the enum and the `--runtime` flag until G3 introduces a second
runtime (the Echo rlib backend). At that point, the enum earns its keep.

Don't design for hypothetical future requirements that are two gates away.
