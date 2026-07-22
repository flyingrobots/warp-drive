#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>
#
# G3 gate acceptance test (in-memory runtime).
#
# Mounts the in-memory fixture tree under --gate=g3 and verifies:
#   - Live /.warp/stats and /.warp/runtime diagnostics, proven immediately
#     after mount (before anything else touches the tree, so kernel caching
#     from later commands can't contaminate the counter proof)
#   - The G1 read/write-rejection baseline still holds under the G3 gate
#
# Linux-only. Requires GNU coreutils with `stat --cached=never` support.
# Usage: cargo xtask acceptance --gate g3 --runtime in-memory
set -euo pipefail

# shellcheck disable=SC1007 # intentional: temporarily unset CDPATH for this cd
SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
# shellcheck source=scripts/acceptance-lib.sh
source "$SCRIPT_DIR/acceptance-lib.sh"

# ── Preflight ─────────────────────────────────────────────────────────────────
# The Docker environment is controlled: if the sync-forcing stat this gate
# relies on isn't available, fail loudly rather than silently accepting a
# weaker probe.
if ! env stat --cached=never -c '%s' /dev/null >/dev/null 2>&1; then
    red "ERROR: GNU stat with --cached=never is required for this acceptance script"
    exit 1
fi

MOUNT=${WARP_DRIVE_ACCEPTANCE_MOUNT:-}
MOUNT_CREATED=0
if [ -z "$MOUNT" ]; then
    MOUNT=$(mktemp -d "${TMPDIR:-/tmp}/warp-g3.XXXXXX")
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

echo "=== WARP DRIVE G3 acceptance (in-memory) ==="
echo ""
echo "Mounting at $MOUNT (in-memory backend, G3 gate) ..."

mkdir -p "$MOUNT"
warp-drive-fuse --runtime=in-memory --gate=g3 --mount "$MOUNT" &
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
    "second /.warp/stats open incorporated a fresh open_count (two ordinary cats both see live diagnostics)"

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
assert_contains "$README_FIRST" "WARP DRIVE G1 Fixture" "README.md first read is correct content"

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

FINAL_STATS=$(cat "$MOUNT/.warp/stats")
assert_eq "$(json_int_value "$FINAL_STATS" "runtime_observe_count")" "0" \
    "runtime_observe_count is exactly 0 for in-memory"
assert_eq "$(json_int_value "$FINAL_STATS" "runtime_observe_error_count")" "0" \
    "runtime_observe_error_count is exactly 0"

# ── Byte-count proof that actually reads the bytes ───────────────────────────
# `wc -c < regular-file` lets GNU coreutils answer from st_size/lseek without
# reading any bytes, which would make this comparison a tautology (stat vs.
# stat-derived). Force the bytes through a pipe so wc cannot apply that
# optimization.

STAT_SIZE=$(fresh_stat_size "$MOUNT/.warp/stats")
# shellcheck disable=SC2002 # deliberate: `wc -c < file` lets coreutils
# answer from st_size without reading; piping through cat forces the read.
READ_SIZE=$(cat "$MOUNT/.warp/stats" | wc -c | tr -d '[:space:]')
assert_eq "$READ_SIZE" "$STAT_SIZE" ".warp/stats reported size matches bytes actually read"

# ── Shape assertions ──────────────────────────────────────────────────────────

RUNTIME_JSON=$(cat "$MOUNT/.warp/runtime")
COORD_JSON=$(cat "$MOUNT/.warp/coordinate")

assert_contains "$FINAL_STATS" '"gate":"G3"' ".warp/stats gate is G3"
assert_contains "$FINAL_STATS" '"runtime":"in-memory"' ".warp/stats runtime is in-memory"
assert_eq "$(json_int_value "$FINAL_STATS" "schema_version")" "1" ".warp/stats schema_version is 1"

assert_contains "$RUNTIME_JSON" '"gate":"G3"' ".warp/runtime gate is G3"
assert_contains "$RUNTIME_JSON" '"runtime":"in-memory"' ".warp/runtime runtime is in-memory"
assert_contains "$RUNTIME_JSON" '"driver":"warp-drive-driver-memory"' ".warp/runtime driver is warp-drive-driver-memory"
assert_contains "$RUNTIME_JSON" '"build_mode":"debug"' ".warp/runtime build_mode is debug"
assert_contains "$RUNTIME_JSON" '"stats":"live"' ".warp/runtime stats is live"
assert_eq "$(json_int_value "$RUNTIME_JSON" "schema_version")" "1" ".warp/runtime schema_version is 1"

assert_contains "$COORD_JSON" '"gate":"G3"' ".warp/coordinate gate is G3 (no half-updated mount)"

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

echo ""
echo "── cat ─────────────────────────────────────────────────────────────────"
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

echo ""
echo "── stat ────────────────────────────────────────────────────────────────"
STAT_OUT=$(stat "$MOUNT/src/main.ts")
assert_contains "$STAT_OUT" "Inode: 5" "stat shows inode 5 for src/main.ts"
assert_contains "$STAT_OUT" "regular file" "stat shows regular file"

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
    green "G3 GATE PASSED (in-memory)  ($PASS / $TOTAL assertions)"
    exit 0
else
    red   "G3 GATE FAILED (in-memory)  ($FAIL failed, $PASS passed, $TOTAL total)"
    exit 1
fi
