# SPDX-License-Identifier: Apache-2.0
# © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>
#
# Single-stage build for G1 acceptance testing on Linux.
# Single-stage avoids GLIBC version mismatches between builder and runtime.
#
# Usage:
#   docker build -t warp-drive-g1 .
#   docker run --rm --device /dev/fuse --cap-add SYS_ADMIN warp-drive-g1

FROM rust:latest
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends fuse3 ripgrep && \
    rm -rf /var/lib/apt/lists/*

COPY . .

# The g0-spike crate depends on ../echo-warp-drive (not in this build context).
# Strip it from the workspace so Cargo doesn't try to resolve its path deps.
RUN sed -i '/"crates\/warp-drive-g0-spike"/d' Cargo.toml && \
    sed -i '/^echo-wasm-abi/d; /^warp-wasm/d' Cargo.toml

RUN cargo build --package warp-drive-fuse && \
    cp target/debug/warp-drive-fuse /usr/local/bin/warp-drive-fuse

RUN chmod +x scripts/acceptance.sh

CMD ["scripts/acceptance.sh"]
