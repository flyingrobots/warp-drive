---
title: "WARP DRIVE as an Application Kitten"
legend: GATE
lane: cool-ideas
priority: medium
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# WARP DRIVE as an Application Kitten

**Status:** cool idea, cross-repo. Depends on Echo landing session/authority
primitives that don't exist yet (see the companion note in the Echo repo).

## The idea

The Profunctor Optics constitution's Kitten taxonomy names three Kitten
kinds: Human, Agent, and Application — "Application" being "one possible
Kitten kind, when it has an active observer/intent/update loop." WARP
DRIVE's mount lifecycle (mount → observe → buffer a save → submit an Intent
→ receive a frontier advance → repeat) is exactly that loop. WARP DRIVE
should be modeled as an **Application Kitten**, not an Agent Kitten — it
doesn't propose autonomous candidates or consume bounded obligations the
way a coding agent does; its loop is materialization, not proposal
generation.

Concretely, WARP DRIVE would own its own `kitten_identity` and
`KittenSession`. That session **references** the mounting human's (or
agent's) `ObserverSession` — it does not become it. A `delegated_capability_refs`
field names exactly what aperture, authority, law, and policy coordinates
the human granted the mount; nothing wider is assumed, and nothing is
inherited by silent substitution. This mirrors the constitution's own
non-collapse rule for composite Kittens generally: an agent acting for a
human is not the human. A mount acting for a human is not the human
either.

## Why it matters

Right now, every observation WARP DRIVE makes through Echo uses the most
degenerate possible values on the fields that *would* carry this: Echo's
`ObservationRequest` already has real slots for `observer_instance`,
`budget`, and `rights`, and `warp-drive-echo-backend`'s
`ObservationRequest::builtin_one_shot()` fills every one of them with a
placeholder (`observer_instance: None`, `budget: UnboundedOneShot`,
`rights: KernelPublic`). There's no session behind a read, no real budget
enforcement, and no meaningful authority posture — just a generic
"kernel-public" default. Modeling WARP DRIVE as a Kitten with its own
session gives those slots something honest to carry instead of a
placeholder forever.

## The interesting sub-problem: authority and time travel

If WARP DRIVE ever gains a causal-navigation surface (reading an earlier
coordinate, not just the frontier), a real question falls out immediately:
if the delegating human's authority was wider at an earlier coordinate than
it is now, does reading that earlier coordinate resurrect the wider
authority?

No — and the constitution already has the vocabulary for why, even though
it wasn't spelled out for authority specifically until this discussion:
authority observed at a historical coordinate is *Then-known*, never
*Now-known*. Admission of any new proposal resolves the admitting authority
at the coordinate of admission (now, wherever the frontier is), never at
the coordinate a prior reading was taken from — the same law that already
governs stale-basis writes. A proposal built from a wider-authority
historical reading, submitted after that authority narrowed, should receive
the same typed obstruction a stale-basis write receives.

## Open question

Whether a mount's delegated authority tracks the grantor's current
authority continuously (shrinks the instant the grantor's does) or was
fixed at grant time and needs separate revocation. Both are lawful designs.
Whoever implements this needs to pick one and witness the choice, not leave
it ambient.

## Surface when

Echo lands any real `ObserverSession`/`KittenSession`-shaped primitive (see
the companion note in `echo/docs/topics/`), or when WARP DRIVE grows a
causal-navigation / time-travel read surface beyond the current frontier-only
reads.
