# Cool Ideas

Ideas surfaced during design review, 2026-05-28.

## Must become tests

- [ ] **The "stale save" demo** — two editors open the same file from the same
  coordinate. Editor A saves. Editor B saves. Editor B gets `EBUSY`.
  `cat .warp/intents/last` shows whose suffix advanced the frontier and what
  basis B held. The "holy shit" moment that makes the project's moral argument
  tactile. **Promote to G5 acceptance criteria.**

- [ ] **Negative compatibility test suite** — a formal, versioned list of things
  WARP DRIVE intentionally rejects: SQLite default mode, `MAP_SHARED|PROT_WRITE`,
  atomic rename on non-supporting runtimes, path-based runtime writes. A
  published refusal list increases trust more than a promise list.
  **Promote to test strategy; failures here are spec conformance, not bugs.**

## G3/G5 product affordances

- [ ] **`/.warp/why/<path>`** — ask why a file looks the way it does. Returns
  the chain of suffixes and receipts that produced the current projection.
  Provenance as a filesystem-native affordance.

- [ ] **Receipt log / last receipt** — `/.warp/intents/log.jsonl` +
  `/.warp/intents/last` as persistent append-only obstruction history.
  Already in the `.warp/` surface spec (§11.5); this is a reminder to
  make it *good*, not just present.

## G7 collaboration UX

- [ ] **`warp diff @main @agent`** — a CLI wrapper that queries two coordinates
  and renders a substrate-aware diff. Not just text diff — basis, suffixes,
  witnesses, receipts. Shows causal divergence, not just byte divergence.

- [ ] **Agent lane dashboard (`warp lanes`)** — a read-only TUI showing all
  active lanes, their current frontiers, and pending intent counts. Makes
  multi-lane reality visible without a GUI.

## Future magic

- [ ] **Coordinate replay mode** —
  `warp-drive mount --replay @main --from fr:old --to fr:new ~/replay`
  Mounts a filesystem that can be stepped through historical frontiers.
  Time-travel debugging at the shell level.
