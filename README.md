<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# WARP DRIVE

> **A POSIX-shaped membrane over witnessed causal history.**
> Mount it like a filesystem. Edit files like normal. Every read is a lawful
> projection from a coordinate; every write is an Intent against an explicit
> basis. The bytes on disk are not the truth — they are the latest hologram.

```text
mount -t warpdrive @main:/repo  /Users/me/project
mount -t warpdrive @agent-1:/repo /Users/me/agent-lane

# vim, ripgrep, Cursor, your build script, Claude Code — they all see "files"
# WARP DRIVE turns every save into an Intent and every read into an observation
# Two mounts of the same repo on different lanes; no Git worktrees needed
```

## Status

**Vapor.** Nothing is built. This repository currently exists to hold the
design — see [`docs/TECHNICAL_DEEP_DIVE.md`](docs/TECHNICAL_DEEP_DIVE.md).

The deep dive is intentionally substrate-agnostic. WARP DRIVE will work
against any runtime that speaks the [Continuum](#origin) causal-history
protocol — Echo today, others in principle.

## What you'll learn from the deep dive

1. **The pitch in concrete mechanism** — what `cat`, `:w`, `git diff`, and
   `inotify` look like on top of a causal-history substrate.
2. **The minimum vocabulary** to talk about this honestly — WARP, Continuum,
   coordinate, optic, hologram, suffix, receipt. About ten words; each one
   earns its place.
3. **How a read becomes an observation** and how a write becomes an Intent.
4. **Why the architecture is substrate-agnostic** — same client, four
   plausible backends sketched (Echo, git-warp, Postgres-warp, in-memory dev).
5. **What multi-lane reality unlocks** — humans and AI agents on the same
   project, on different coordinates, without merge folklore.
6. **The bold moves** — what becomes possible when the filesystem itself is
   history-aware.
7. **The non-goals** — what WARP DRIVE deliberately is not.
8. **A credible v0.0.1 path** — four steps from nothing to a useful mount.

## Origin

WARP DRIVE began as a cool-ideas card in the [Echo][echo] runtime backlog
under the name _WARPDrive POSIX Materialization Optic_. The architectural
frame ("files are materialized readings, not substrate truth") comes from
[_There Is No Graph_][tinog], an Echo architecture note. The Continuum
protocol layer that makes substrate independence real is described in Echo's
[Continuum Transport][continuum-transport] note.

This repo lifts that idea into its own project so it can grow without
inheriting Echo's roadmap.

[echo]: https://github.com/flyingrobots/echo
[tinog]: https://github.com/flyingrobots/echo/blob/main/docs/architecture/there-is-no-graph.md
[continuum-transport]: https://github.com/flyingrobots/echo/blob/main/docs/architecture/continuum-transport.md

## License

Apache 2.0. See [`LICENSE`](LICENSE).
