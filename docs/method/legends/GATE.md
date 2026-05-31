# Legend: GATE

Gate-level work: the acceptance tests, projection adapters, FUSE semantics,
domain design, and inode strategy that advance WARP DRIVE through its gate
sequence.

## What it covers

- Gate acceptance criteria and acceptance script development
- FUSE adapter implementation (lookup, getattr, readlink, read, readdir)
- Domain types: `FixtureTree`, `VirtualNode`, inode assignment
- Projection adapter design (G2+): the boundary between FUSE ops and
  Continuum observe/intent calls
- Fixture library and integration test harness (`warp-drive-fixtures`,
  `warp-drive-test-harness`)
- `.warp/` surface design and semantics

## What success looks like

Each gate condition is demonstrably true: `cargo xtask acceptance` exits 0,
a gate record exists, and the POSIX membrane behavior is honest — the
claims in the deep dive are backed by rerunnable acceptance transcripts.

## How you know

- `cargo xtask acceptance` exits 0 for the current gate
- The gate record in `docs/gates/GN.md` includes the full transcript
- No acceptance assertion is mocked or skipped
- The fixture tree or live projection produces coherent output for
  `ls`, `cat`, `find`, `rg`, `stat`, `readlink`, and write rejection
