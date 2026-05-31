---
title: "Witness-attested AI agent identity"
legend: GATE
lane: cool-ideas
priority: low
---

<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Witness-attested AI agent identity

Status: cool idea.

Depends on:

- [Technical deep dive — basis discipline](../../TECHNICAL_DEEP_DIVE.md#7-anatomy-of-a-write)
- [Witness primitives in Echo](https://github.com/flyingrobots/echo) (warp-core `Witness`, existing suffix witnesses)
- Wesley-generated codec contract for the writer field

## The idea

Every write that crosses WARP DRIVE's wire today carries a basis token
and an `author` field. The author is a string — `"james"`, `"agent-1"`,
or whatever the optic client decides to set. That string is a vibe, not
evidence.

Replace the string with a **signed witness bundle** carrying:

- agent identifier (stable across sessions; key-derived, not chosen)
- model name + version (e.g., `claude-opus-4-7`)
- prompt-chain hash (the hash of the conversation/instructions the agent
  was operating under at the time of the write)
- parent reading identity (the hologram identity the agent's proposal
  was prepared against — same `holdersChainHash` the basis uses, but
  retained as part of the writer's identity)
- timestamp (substrate-issued, not wall-clock from the agent)
- signature over the whole bundle, with a key the agent controls

Humans get a degenerate version of the same shape: a personal identity
key plus an empty model/prompt-chain. The wire format is uniform; the
difference between "human wrote this" and "agent wrote this" is fields
on the witness, not types.

## Why it matters

Three forces are converging:

1. **Regulatory.** AI-generated code provenance is going to matter within
   1-3 years. The EU AI Act already gestures at training-data
   transparency; downstream pressure on "who/what wrote this line" is
   coming. Today's answer is "the human who clicked accept on the
   suggestion." That answer will not survive.

2. **Auditability for serious teams.** Companies running multiple
   coding agents in production cannot today answer "which agent run,
   which prompt, which model version produced this commit?" with
   anything stronger than a log file. The log is in a different system
   from the code; it gets rotated; it does not survive forking the repo.

3. **Trust calibration for users.** A user looking at a hunk wants to
   know: did a model write this, which model, what was it asked? Today
   the answer is "ask the human and hope they remember." The honest
   answer should be a verifiable record attached to the write itself.

WARP DRIVE is uniquely positioned because:

- The basis discipline is already there — writers already declare what
  they observed before writing
- The wire format is already structurally honest about provenance
- The witness primitive already exists in Echo
- Adding a structured writer-identity field is a schema change, not a
  fundamental architecture shift

## What it looks like in the schema

A first sketch, extending the `warpdrive.graphql` from the implementation plan:

```graphql
"""
A signed witness attesting to who or what produced a write. Replaces
the historical free-form `author` string with verifiable provenance.
"""
type WriterWitness {
    witnessId: ID!
    agentId: ID!              # stable identity (key-derived)
    agentKind: AgentKind!
    modelName: String         # null for humans
    modelVersion: String      # null for humans
    promptChainHash: String   # null for humans
    parentReadingId: ID!      # hologram identity the writer based on
    substrateTimestamp: Int!  # runtime-issued, not client wall-clock
    signature: String!        # over the canonical encoding of the bundle
}

enum AgentKind {
    HUMAN
    AI_AGENT
    BUILD_AUTOMATION
    MIGRATION_SCRIPT
}

input FsWriteContentInput {
    siteId: ID!
    basisHash: String!
    newBytes: String!
    writer: WriterWitness!   # replaces the old `author: String`
}
```

The runtime validates the signature against `agentId`'s registered key
before admitting the suffix. An Intent with an invalid signature is
obstructed with a typed `INVALID_WRITER_WITNESS` error.

## What it unlocks

- **`git blame` becomes verifiable.** Today blame says who pushed; with
  witnesses, blame says who produced, signed, and which model run did it.
- **Model accountability is mechanical.** Roll back every write produced
  by `claude-opus-4-6` between dates X and Y, regardless of which human
  later edited or merged them. Not possible today.
- **Prompt forensics.** When a bug is traced to a specific hunk, the
  prompt-chain hash on the writer witness lets the team retrieve the
  exact conversation that produced it (if the chain is retained).
- **Cross-agent collaboration becomes accountable.** Two agents writing
  to adjacent coordinates cannot impersonate each other. The substrate
  knows whose key signed what.
- **License compliance for AI-produced code.** Some licenses (e.g., GPL)
  may demand specific provenance for derivative works produced by an AI
  trained on GPL code. A witness chain that records the model identity
  is the only honest input to that conversation.

## Open questions

- **Key management.** Where does an agent's signing key live? Per-machine
  hardware-backed, or a daemon-issued ephemeral, or both? What's the
  rotation story?
- **Prompt-chain retention.** Hashes are cheap but the chains
  themselves can be large and sensitive. Retain in the runtime? Pin to
  CAS? Out-of-band with a TTL?
- **What counts as "the agent"?** A long-running daemon? A single
  Claude Code session? A single tool call within a session? The
  granularity decision affects how meaningful the identity is.
- **Backward compatibility.** Echo today has free-form `author`
  strings. Coexistence with witness-bearing writes during migration is
  its own design problem.
- **Privacy.** Some humans don't want every keystroke attributed to a
  signed identity. The schema needs a "redacted writer" option that
  proves a key signed it without revealing which key. (Group signatures?
  Anonymous credentials? This is a real cryptography choice.)

## Why this card lives in warp-drive

Echo already has the witness primitive. Wesley already can generate the
codec. The thing that doesn't exist anywhere yet is the **decision to
make writer-identity load-bearing on every write at a wire layer that
ordinary tools touch.** WARP DRIVE is the layer where that decision
becomes real and observable. If WARP DRIVE adopts witness-attested
writers from v0.0.1, the rest of the ecosystem inherits the discipline.

## Surface when

- Implementing `fsWriteContent` in `echo-fs-runtime` (W2.M2 in the
  plan). The natural moment to choose between "carry a string" and
  "carry a witness."
- Anyone bringing up the regulatory pressure on AI-generated code in a
  team context.
- When the first multi-agent demo on adjacent WARP DRIVE coordinates
  surfaces "who wrote this" as a real user question.
