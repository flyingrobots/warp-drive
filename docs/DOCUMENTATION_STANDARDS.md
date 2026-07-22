<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# Documentation Standards

**Status:** current policy for new and substantially changed documentation.
**Normative terms:** **MUST**, **SHOULD**, **MAY** indicate requirement
strength.

This governs *how documentation is written*. It does not govern the gate
cycle itself — that's `docs/PROCESS.md`, and this doc assumes it. Loosely
adapted from a sibling standard written for `colorful-language`, a CLI/LSP
tool with a large external-user surface; that shape doesn't fit a
gate-proven FUSE membrane with essentially one audience so far
(contributors, and future us), so this is reorganized around what this repo
actually has: frozen gate/design records, and a growing set of living topic
references that outlive any one gate.

## 1. The three kinds of durable doc, and which one you're writing

WARP DRIVE's documentation splits by **whether it's frozen or living**, not
by reader-task taxonomy. Get this one distinction right and the rest of
this doc is detail.

| Kind | Where | Frozen or living | Governed by |
| --- | --- | --- | --- |
| **Gate record** | `docs/gates/GN.md` | Frozen at the commit that passed it | `docs/PROCESS.md` §"Gate record" |
| **Design record** | `docs/design/<slug>.md` | Frozen once its gate passes | `docs/PROCESS.md` §"Design" |
| **Topic reference** | `docs/topics/<topic>/README.md` (+ `test-plan.md`) | **Living** — reflects `main`, updated whenever the behavior changes | This doc |

Gate and design records already have a working discipline in `docs/PROCESS.md`
— don't duplicate it here. This standard's actual job is the topic layer,
which is new, plus the writing conventions that apply everywhere.

**The rule that matters most:** never edit a gate or design record to
describe behavior that changed after it was written. A later gate gets a
new gate record, even for a surface an earlier gate already touched (see
`docs/gates/G3.md` rewriting `.warp/stats` after `docs/gates/G1.md` already
described its placeholder shape — both records stay true to their own
moment). If cross-gate behavior needs to stay current, that's what a topic
reference is for.

## 2. Topic references

Add `docs/topics/<topic>/` when a concept outlives the gate that introduced
it and someone will need to know its *current* shape without reading every
gate record that touched it — the `.warp/` diagnostics surface, the
`FixtureTree` domain model, the Echo observation seam, the copy-in Docker
acceptance runner. Don't create one preemptively; add it the first time a
second gate extends something a first gate introduced.

```text
docs/topics/<topic>/
  README.md       -- living reference: current contracts, invariants, gaps
  test-plan.md    -- requirement IDs, oracle, evidence, status
  rationale.md    -- optional: tradeoffs and rejected alternatives, if still relevant
```

`README.md` MUST:

- describe only behavior that exists on `main` right now;
- state public contracts and invariants, not just a narrative;
- distinguish current behavior from known gaps (link the gap to a GitHub
  issue if it's tracked);
- link to the gate record(s) that established the behavior, and to the
  design record(s) for history — without re-deriving them.

`test-plan.md` MUST identify, per requirement: a stable ID, the exact
behavior under test, the oracle (a unit test, an acceptance-script
assertion, a manual check), the evidence status, and — once implemented —
the concrete test or script that proves it. A planned requirement is not
evidence; mark it a gap.

Neither file MUST become a step-by-step tutorial or reproduce a frozen
gate transcript — link the gate record instead of copying it.

## 3. Backlog content lives in GitHub Issues

New cool-ideas, known code smells, and deferred work go to GitHub Issues on
this repo. `docs/method/backlog/` is historical: existing cards stay as
record, nothing new gets added there. `docs/method/legends/` (the
`GATE`/`INFRA` doctrine) is unaffected — it still defines what counts as
gate-level vs. infra-level work.

## 4. Writing conventions

Write like a competent teammate: direct, present tense for current
behavior, imperative for procedures. Label historical content (design
records, frozen gate records) as historical rather than writing it in a
tense that implies it's still true.

- Use inline code for commands, flags, paths, `.warp/` field names, and
  Rust type/crate names — `FixtureTree`, `MountStats`, `xtask`, not
  informal paraphrases of them.
- `gate` (a `GN` milestone) and `runtime` (`in-memory`/`echo-rlib`) are two
  different axes — don't conflate them in prose the way it'd be easy to.
- `worldline`, `frontier`, `coordinate` follow Echo's own vocabulary; don't
  invent synonyms here.
- Prefer this repo's own obstruction vocabulary (`obstructed`, `rejected`,
  `EROFS`) over vague alternatives ("might not work").
- Avoid `we` unless pointing at a decision actually recorded somewhere
  (a gate record, a design record, this doc).

Prefer: *"Run `cargo xtask acceptance --gate g3 --runtime in-memory`. It
exits non-zero if any assertion fails."*
Avoid: *"You might run into an issue if the mount doesn't quite come up
right."*

Sentence/paragraph length, passive voice, and bullet count are editorial
signals, not merge gates.

## 5. Examples must be real

Every command example SHOULD be one that was actually run, not typed from
memory — lift acceptance output from a real transcript. Label anything
that wasn't actually run as illustrative, and never fabricate PASS/FAIL
lines to make an example look complete. Declare fenced-block language
(`bash`, `json`, `rust`, `text`); don't mix a `$` prompt into a block meant
for copy-paste — show the command and its output separately.

This is a CLI/FUSE project — prefer a terminal transcript over a
screenshot when showing observable behavior.

## 6. Enforcement

```bash
markdownlint-cli2 "**/*.md"
git diff --check
```

CI SHOULD block on: malformed Markdown, broken internal links, a gate
record whose recorded command no longer exists, a design or gate record
edited to describe post-gate behavior (should be a topic reference or a
new record instead), and fabricated transcripts. Page length, tone, and
bullet density stay advisory — useful for editors, not universal gates.

## 7. Before calling a doc change done

- One primary reader job per page.
- Gate/design records untouched after the fact; behavior drift went into a
  topic reference instead.
- New planning content is a GitHub issue, not a new `docs/method/backlog/`
  card.
- Examples show real output.
- Release-visible changes updated `CHANGELOG.md`.
- Markdown and diff checks pass.
