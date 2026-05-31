<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `cargo xtask acceptance` — one-shot gate acceptance runner

**Status:** ✅ implemented at G1 (Docker-based). Extend for G2+.

## What landed (G1)

`cargo xtask acceptance` builds a Linux Docker image and runs 29 assertions
covering the full G1 acceptance sequence:

- `ls -a` (directory listing including hidden `.warp/`)
- `cat` (file contents for README, package.json, src/\*.ts, .warp/\*)
- `find` (full tree walk)
- `rg` (ripgrep search across the mount)
- `stat` (inode number, file type)
- `readlink` + symlink resolution
- write rejection (EROFS) for both overwrite and create

Gate condition is mechanical: `cargo xtask acceptance` exits 0 = passed,
non-zero = failed.

## Extend to G2+

```
cargo xtask acceptance --gate g2
cargo xtask acceptance          # runs all gates up to current
```

Each gate gets its own acceptance script (`scripts/acceptance-g2.sh`, etc.)
or the single script gains a `--gate` flag. The xtask dispatches accordingly.

The Docker image can stay shared across gates — just mount different fixture
content or connect to a real Echo projection for G3+.
