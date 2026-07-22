#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>
#
# G3 gate acceptance test (echo-rlib runtime).
#
# Mounts the echo-rlib backend under --gate=g3 and verifies:
#   - Live /.warp/stats and /.warp/runtime diagnostics, proven immediately
#     after mount (before anything else touches the tree)
#   - The G1 read/write-rejection baseline still holds
#   - The G2b Echo baseline still holds: /echo/head.json is still a normal
#     read-only file whose bytes come from Echo's QueryBytes projection
#     payload, and its own "gate" field still legitimately reads "G2b" —
#     that's Echo-side payload provenance G3 does not touch, not the active
#     mount's gate identity
#
# Linux-only. Requires the local Echo-capable warp-drive-fuse binary and GNU
# coreutils with `stat --cached=never` support.
# Usage: cargo xtask acceptance --gate g3 --runtime echo-rlib
set -euo pipefail

# shellcheck disable=SC1007 # intentional: temporarily unset CDPATH for this cd
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=scripts/acceptance-lib.sh
source "$SCRIPT_DIR/acceptance-lib.sh"

# ── Preflight ─────────────────────────────────────────────────────────────────
if ! env stat --cached=never -c '%s' /dev/null >/dev/null 2>&1; then
    red "ERROR: GNU stat with --cached=never is required for this acceptance script"
    exit 1
fi

MOUNT=${WARP_DRIVE_ACCEPTANCE_MOUNT:-}
MOUNT_CREATED=0
if [ -z "$MOUNT" ]; then
    MOUNT=$(mktemp -d "${TMPDIR:-/tmp}/warp-g3-echo.XXXXXX")
    MOUNT_CREATED=1
else
    mkdir -p "$MOUNT"
fi

# ── Cleanup trap ──────────────────────────────────────────────────────────────

FUSE_PID=""
cleanup() {
    umount "$MOUNT" 2>/dev/null || fusermount3 -u "$MOUNT" 2>/dev/null || true
    if [ -n "${FUSE_PID:-}" ]; then
        kill "$FUSE_PID" 2>/dev/null || true
        wait "$FUSE_PID" 2>/dev/null || true
    fi
    if [ "$MOUNT_CREATED" -eq 1 ]; then
        rm -rf "$MOUNT"
    fi
}
trap cleanup EXIT INT TERM

# ── Mount ─────────────────────────────────────────────────────────────────────

echo "=== WARP DRIVE G3 acceptance (echo-rlib) ==="
echo ""
echo "Mounting at $MOUNT (echo-rlib backend, G3 gate) ..."

mkdir -p "$MOUNT"
warp-drive-fuse --runtime=echo-rlib --gate=g3 --mount "$MOUNT" &
FUSE_PID=$!

MOUNTED=0
for i in $(seq 1 50); do
    if ! kill -0 "$FUSE_PID" 2>/dev/null; then
        red "ERROR: warp-drive-fuse exited prematurely (pid $FUSE_PID)"
        exit 1
    fi
    if stat -f -c '%T' "$MOUNT" 2>/dev/null | grep -qi fuse; then
        MOUNTED=1
        break
    fi
    sleep 0.1
done

if [ "$MOUNTED" -eq 0 ]; then
    red "ERROR: mount point did not become a FUSE filesystem within 5 s"
    exit 1
fi

echo "Mounted (fuse pid $FUSE_PID)."
echo ""

# ── Diagnostics: proven immediately, before anything else touches the tree ───

echo "── live diagnostics ───────────────────────────────────────────────────"

STATS_1=$(cat "$MOUNT/.warp/stats")
STATS_2=$(cat "$MOUNT/.warp/stats")

READ_1=$(json_int_value "$STATS_1" "read_count")
READ_2=$(json_int_value "$STATS_2" "read_count")
assert_eq "$READ_2" "$READ_1" "reading /.warp/stats does not increment read_count"

OPEN_1=$(json_int_value "$STATS_1" "open_count")
OPEN_2=$(json_int_value "$STATS_2" "open_count")
assert_delta_ge "$OPEN_2" "$OPEN_1" 1 \
    "second /.warp/stats open incorporated a fresh open_count"

PROBE_COUNT=8

LOOKUP_BEFORE=$(json_int_value "$STATS_2" "lookup_count")
for i in $(seq 1 "$PROBE_COUNT"); do
    if env stat "$MOUNT/__g3_missing_probe_$i" >/dev/null 2>&1; then
        fail "missing lookup probe $i unexpectedly succeeded"
    fi
done
STATS_AFTER_LOOKUPS=$(cat "$MOUNT/.warp/stats")
LOOKUP_AFTER=$(json_int_value "$STATS_AFTER_LOOKUPS" "lookup_count")
assert_delta_ge "$LOOKUP_AFTER" "$LOOKUP_BEFORE" "$PROBE_COUNT" \
    "lookup_count records every unique miss"

GETATTR_BEFORE=$(json_int_value "$STATS_AFTER_LOOKUPS" "getattr_count")
for i in $(seq 1 "$PROBE_COUNT"); do
    fresh_stat_size "$MOUNT/.warp/stats" >/dev/null
done
STATS_AFTER_GETATTRS=$(cat "$MOUNT/.warp/stats")
GETATTR_AFTER=$(json_int_value "$STATS_AFTER_GETATTRS" "getattr_count")
assert_delta_ge "$GETATTR_AFTER" "$GETATTR_BEFORE" "$PROBE_COUNT" \
    "getattr_count records forced attribute refreshes"

READ_BEFORE_README=$(json_int_value "$STATS_AFTER_GETATTRS" "read_count")
README_FIRST=$(cat "$MOUNT/README.md")
STATS_AFTER_README=$(cat "$MOUNT/.warp/stats")
READ_AFTER_README=$(json_int_value "$STATS_AFTER_README" "read_count")
assert_delta_ge "$READ_AFTER_README" "$READ_BEFORE_README" 1 \
    "first-ever real read (README.md) increases read_count"

READDIR_BEFORE=$(json_int_value "$STATS_AFTER_README" "readdir_count")
ls -a "$MOUNT/empty" >/dev/null
STATS_AFTER_READDIR=$(cat "$MOUNT/.warp/stats")
READDIR_AFTER=$(json_int_value "$STATS_AFTER_READDIR" "readdir_count")
assert_delta_ge "$READDIR_AFTER" "$READDIR_BEFORE" 1 \
    "first-ever readdir (/empty) increases readdir_count"

READLINK_BEFORE=$(json_int_value "$STATS_AFTER_READDIR" "readlink_count")
LINK_TARGET=$(readlink "$MOUNT/links/readme")
STATS_AFTER_READLINK=$(cat "$MOUNT/.warp/stats")
READLINK_AFTER=$(json_int_value "$STATS_AFTER_READLINK" "readlink_count")
assert_delta_ge "$READLINK_AFTER" "$READLINK_BEFORE" 1 \
    "first-ever readlink increases readlink_count"
assert_eq "$LINK_TARGET" "../README.md" "links/readme -> ../README.md"

# ── Exact-value assertions ────────────────────────────────────────────────────
# G3 makes no new Echo call — init_g3() performs the same two observations as
# init_g2b() (head + query-projected /echo/head.json) — so this is 2, exactly.

FINAL_STATS=$(cat "$MOUNT/.warp/stats")
assert_eq "$(json_int_value "$FINAL_STATS" "runtime_observe_count")" "2" \
    "runtime_observe_count is exactly 2 for echo-rlib G3"
assert_eq "$(json_int_value "$FINAL_STATS" "runtime_observe_error_count")" "0" \
    "runtime_observe_error_count is exactly 0"

# ── Byte-count proof that actually reads the bytes ───────────────────────────

STAT_SIZE=$(fresh_stat_size "$MOUNT/.warp/stats")
# shellcheck disable=SC2002 # deliberate: `wc -c < file` lets coreutils
# answer from st_size without reading; piping through cat forces the read.
READ_SIZE=$(cat "$MOUNT/.warp/stats" | wc -c | tr -d '[:space:]')
assert_eq "$READ_SIZE" "$STAT_SIZE" ".warp/stats reported size matches bytes actually read"

# ── Shape assertions ──────────────────────────────────────────────────────────
# Active-mount gate (coordinate/runtime/stats) vs. Echo payload provenance
# (/echo/head.json) are different things — keep these four visible together,
# not lost in copied-script noise.

RUNTIME_JSON=$(cat "$MOUNT/.warp/runtime")
COORD=$(cat "$MOUNT/.warp/coordinate")

assert_contains "$COORD" '"gate":"G3"' ".warp/coordinate gate is G3 (active mount)"
assert_contains "$RUNTIME_JSON" '"gate":"G3"' ".warp/runtime gate is G3 (active mount)"
assert_contains "$FINAL_STATS" '"gate":"G3"' ".warp/stats gate is G3 (active mount)"

assert_contains "$RUNTIME_JSON" '"runtime":"echo-rlib"' ".warp/runtime runtime is echo-rlib"
assert_contains "$RUNTIME_JSON" '"driver":"warp-wasm"' ".warp/runtime driver is warp-wasm"
assert_contains "$RUNTIME_JSON" '"stats":"live"' ".warp/runtime stats is live"
assert_eq "$(json_int_value "$RUNTIME_JSON" "schema_version")" "1" ".warp/runtime schema_version is 1"
assert_eq "$(json_int_value "$FINAL_STATS" "schema_version")" "1" ".warp/stats schema_version is 1"

# ── G2a coordinate baseline (real Echo coordinate) ───────────────────────────

echo ""
echo "── G2a coordinate baseline ─────────────────────────────────────────────"

assert_contains "$COORD" '"worldline"'     ".warp/coordinate has worldline field"
assert_contains "$COORD" '"frontier"'      ".warp/coordinate has frontier field"
assert_contains "$COORD" '"state_root"'    ".warp/coordinate has state_root field"
assert_contains "$COORD" '"artifact_hash"' ".warp/coordinate has artifact_hash field"
assert_contains "$COORD" '"backend":"echo-rlib"' ".warp/coordinate backend is echo-rlib"
assert_not_contains "$COORD" \
    '"worldline":"00000000-0000-0000-0000-000000000001"' \
    ".warp/coordinate worldline is real (not genesis placeholder)"

WORLDLINE_VALUE=$(json_hex_value "$COORD" "worldline" || true)
FRONTIER_VALUE=$(json_hex_value "$COORD" "frontier" || true)
STATE_ROOT_VALUE=$(json_hex_value "$COORD" "state_root" || true)
ARTIFACT_HASH_VALUE=$(json_hex_value "$COORD" "artifact_hash" || true)
assert_nonzero_hex64 "$WORLDLINE_VALUE" ".warp/coordinate worldline is 64-char non-zero hex"
assert_nonzero_hex64 "$FRONTIER_VALUE" ".warp/coordinate frontier is 64-char non-zero hex"
assert_nonzero_hex64 "$STATE_ROOT_VALUE" ".warp/coordinate state_root is 64-char non-zero hex"
assert_nonzero_hex64 "$ARTIFACT_HASH_VALUE" ".warp/coordinate artifact_hash is 64-char non-zero hex"

# ── G2b projected file proof (Echo payload provenance, copied unchanged) ─────
# /echo/head.json's own "gate" field legitimately still says "G2b" — G3
# reuses G2b's exact projection call unmodified. Never mechanically relabel
# this to "G3"; it would misrepresent which gate actually produced these
# bytes.

echo ""
echo "── G2b projected file assertions (payload provenance, not mount gate) ──"

ECHO_LS_OUT=$(ls -a "$MOUNT/echo")
assert_contains "$ECHO_LS_OUT" "head.json" "ls /echo contains head.json"

ECHO_HEAD=$(cat "$MOUNT/echo/head.json")
assert_contains "$ECHO_HEAD" '"kind":"echo-projected-file"' "/echo/head.json kind"
assert_contains "$ECHO_HEAD" '"gate":"G2b"' "/echo/head.json gate is G2b (payload provenance, unchanged by G3)"
assert_contains "$ECHO_HEAD" '"source":"echo-observation-payload"' "/echo/head.json source"
assert_contains "$ECHO_HEAD" '"worldline"' "/echo/head.json has worldline field"
assert_contains "$ECHO_HEAD" '"frontier"' "/echo/head.json has frontier field"
assert_contains "$ECHO_HEAD" '"state_root"' "/echo/head.json has state_root field"
assert_contains "$ECHO_HEAD" '"projection_hash"' "/echo/head.json has projection_hash field"
assert_not_contains "$ECHO_HEAD" '"artifact_hash"' "/echo/head.json omits artifact_hash"
assert_not_contains "$ECHO_HEAD" '/echo/head.json' "/echo/head.json omits POSIX path literal"

ECHO_WORLDLINE_VALUE=$(json_hex_value "$ECHO_HEAD" "worldline" || true)
ECHO_FRONTIER_VALUE=$(json_hex_value "$ECHO_HEAD" "frontier" || true)
ECHO_STATE_ROOT_VALUE=$(json_hex_value "$ECHO_HEAD" "state_root" || true)
ECHO_PROJECTION_HASH_VALUE=$(json_hex_value "$ECHO_HEAD" "projection_hash" || true)
assert_nonzero_hex64 "$ECHO_WORLDLINE_VALUE" "/echo/head.json worldline is 64-char non-zero hex"
assert_nonzero_hex64 "$ECHO_FRONTIER_VALUE" "/echo/head.json frontier is 64-char non-zero hex"
assert_nonzero_hex64 "$ECHO_STATE_ROOT_VALUE" "/echo/head.json state_root is 64-char non-zero hex"
assert_nonzero_hex64 "$ECHO_PROJECTION_HASH_VALUE" "/echo/head.json projection_hash is 64-char non-zero hex"
assert_eq "$ECHO_WORLDLINE_VALUE" "$WORLDLINE_VALUE" "/echo/head.json worldline matches .warp/coordinate"
assert_eq "$ECHO_FRONTIER_VALUE" "$FRONTIER_VALUE" "/echo/head.json frontier matches .warp/coordinate"
assert_eq "$ECHO_STATE_ROOT_VALUE" "$STATE_ROOT_VALUE" "/echo/head.json state_root matches .warp/coordinate"

# ── Inherited G1 baseline ─────────────────────────────────────────────────────

echo ""
echo "── ls ──────────────────────────────────────────────────────────────────"
LS_OUT=$(ls -a "$MOUNT")
assert_contains "$LS_OUT" "README.md"    "ls / contains README.md"
assert_contains "$LS_OUT" "package.json" "ls / contains package.json"
assert_contains "$LS_OUT" "src"          "ls / contains src/"
assert_contains "$LS_OUT" "empty"        "ls / contains empty/"
assert_contains "$LS_OUT" "links"        "ls / contains links/"
assert_contains "$LS_OUT" ".warp"        "ls / contains .warp/"
assert_contains "$LS_OUT" "echo"         "ls / contains echo/"

echo ""
echo "── cat ─────────────────────────────────────────────────────────────────"
assert_contains "$README_FIRST" "WARP DRIVE G1 Fixture" "README.md first line"

PKG=$(cat "$MOUNT/package.json")
assert_contains "$PKG" '"name": "warp-drive-g1"' "package.json name field"

MAIN=$(cat "$MOUNT/src/main.ts")
assert_contains "$MAIN" "export function main" "src/main.ts export"

LIB=$(cat "$MOUNT/src/lib.ts")
assert_contains "$LIB" "export function identity" "src/lib.ts export"

echo ""
echo "── find ────────────────────────────────────────────────────────────────"
FIND_OUT=$(find "$MOUNT" | sort)
assert_contains "$FIND_OUT" "src/main.ts"      "find sees src/main.ts"
assert_contains "$FIND_OUT" "src/lib.ts"       "find sees src/lib.ts"
assert_contains "$FIND_OUT" ".warp/coordinate" "find sees .warp/coordinate"
assert_contains "$FIND_OUT" "empty"            "find sees empty/"
assert_contains "$FIND_OUT" "links/readme"     "find sees links/readme"
assert_contains "$FIND_OUT" "echo/head.json"   "find sees echo/head.json"

echo ""
echo "── rg ──────────────────────────────────────────────────────────────────"
RG_OUT=$(rg --no-heading "export" "$MOUNT" 2>/dev/null || true)
RG_COUNT=$(echo "$RG_OUT" | grep -c "export" || true)
assert_contains "$RG_OUT" "main.ts" "rg finds export in main.ts"
assert_contains "$RG_OUT" "lib.ts"  "rg finds export in lib.ts"
if [ "$RG_COUNT" -ge 2 ]; then
    pass "rg found $RG_COUNT export hits (>= 2)"
else
    fail "rg export hit count" ">= 2" "$RG_COUNT"
fi

RG_ECHO_OUT=$(rg --no-heading "echo-projected-file" "$MOUNT/echo/head.json" 2>/dev/null || true)
assert_contains "$RG_ECHO_OUT" "echo-projected-file" "rg finds projected file marker"

echo ""
echo "── stat ────────────────────────────────────────────────────────────────"
STAT_OUT=$(stat "$MOUNT/src/main.ts")
assert_contains "$STAT_OUT" "Inode: 5" "stat shows inode 5 for src/main.ts"
assert_contains "$STAT_OUT" "regular file" "stat shows src/main.ts regular file"

ECHO_STAT_OUT=$(stat "$MOUNT/echo/head.json")
assert_contains "$ECHO_STAT_OUT" "regular file" "stat shows /echo/head.json regular file"

echo ""
echo "── readlink / symlink resolution ──────────────────────────────────────"
LINK_CONTENT=$(cat "$MOUNT/links/readme")
assert_contains "$LINK_CONTENT" "WARP DRIVE G1 Fixture" "symlink resolves to README.md"

echo ""
echo "── write rejection (EROFS) ─────────────────────────────────────────────"
if ( echo "nope" > "$MOUNT/README.md" ) 2>/dev/null; then
    fail "write to README.md should be rejected (EROFS)"
else
    pass "write to README.md correctly rejected (EROFS)"
fi

if ( echo "nope" > "$MOUNT/echo/head.json" ) 2>/dev/null; then
    fail "write to /echo/head.json should be rejected (EROFS)"
else
    pass "write to /echo/head.json correctly rejected (EROFS)"
fi

if ( touch "$MOUNT/newfile.txt" ) 2>/dev/null; then
    fail "create newfile.txt should be rejected (EROFS)"
else
    pass "create newfile.txt correctly rejected (EROFS)"
fi

# ── Report ────────────────────────────────────────────────────────────────────

echo ""
echo "════════════════════════════════════════════"
TOTAL=$((PASS + FAIL))
if [ "$FAIL" -eq 0 ]; then
    green "G3 GATE PASSED (echo-rlib)  ($PASS / $TOTAL assertions)"
    exit 0
else
    red   "G3 GATE FAILED (echo-rlib)  ($FAIL failed, $PASS passed, $TOTAL total)"
    exit 1
fi
