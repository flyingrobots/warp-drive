# Contributing to WARP DRIVE

WARP DRIVE is a POSIX⇄causal membrane. Contributions are welcome.

## Before you start

Read [`docs/ENGINEERING_STANDARDS.md`](docs/ENGINEERING_STANDARDS.md) and
[`docs/PROCESS.md`](docs/PROCESS.md). The standards are strict by design.
A patch that weakens them is not valid unless the weakening is explicitly
requested by the maintainer.

## Process

WARP DRIVE uses [METHOD](https://github.com/flyingrobots/method).
Work flows through gate branches (`gate/gN`) with PRs to `main`.
Each gate has a design doc, a passing acceptance run, and a gate record
before it merges.

## Gate conditions

| Gate | Condition |
|------|-----------|
| G0 | `warp-wasm` embeds as native rlib; `observe_cbor` round-trips |
| G1 | In-memory FUSE mount; `ls`, `cat`, `rg`, `stat`, `readlink`, EROFS all pass |
| G2 | Real Echo coordinate; real `observe`; live projection via rlib |

## Running tests

```bash
# Full G1 acceptance (Linux Docker required)
cargo xtask acceptance

# Unit tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

## Code style

- Zero warnings policy — all clippy lints are errors.
- No `unwrap`, `expect`, `panic`, `todo`, `unimplemented` in production code.
- No `println!` / `eprintln!` outside `xtask`.
- `missing_docs` is a hard error — document public items.

## License

By contributing you agree that your contributions are licensed under
[Apache 2.0](LICENSE).
