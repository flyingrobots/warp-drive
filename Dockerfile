# SPDX-License-Identifier: Apache-2.0
# © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>
#
# Single-stage build for G1 acceptance testing on Linux.
# Single-stage avoids GLIBC version mismatches between builder and runtime.
#
# Usage:
#   docker build -t warp-drive-g1 .
#   docker run --rm --device /dev/fuse --cap-add SYS_ADMIN \
#     --security-opt apparmor=unconfined warp-drive-g1

FROM rust:1.90.0-bookworm
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends fuse3 ripgrep && \
    rm -rf /var/lib/apt/lists/*

COPY . .

RUN rm -rf .git .gitmodules && \
    test ! -d .git && \
    test ! -e .gitmodules

RUN cargo build --package warp-drive-fuse && \
    cp target/debug/warp-drive-fuse /usr/local/bin/warp-drive-fuse

RUN chmod +x scripts/acceptance.sh

CMD ["scripts/acceptance.sh"]
