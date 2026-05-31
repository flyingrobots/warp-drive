#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>
#
# G2a gate acceptance test.
# Mounts the echo-rlib backend and verifies:
#   - All 29 G1 assertions still pass (same file tree)
#   - /.warp/coordinate contains a real Echo worldline UUID (not genesis placeholder)
#   - /.warp/coordinate contains a real state_root hash
#   - /.warp/runtime identifies the echo-rlib backend
#
# Linux-only. Requires warp-drive-fuse built with the echo-rlib feature.
# Usage: cargo xtask acceptance --runtime echo-rlib
set -euo pipefail

MOUNT=/tmp/warp-g2
PASS=0
FAIL=0

# ── Helpers ───────────────────────────────────────────────────────────────────

green() { printf '\033[0;32m%s\033[0m\n' "$*"; }
red()   { printf '\033[0;31m%s\033[0m\n' "$*"; }

pass() {
    PASS=$((PASS + 1))
    green "  PASS  $1"
}

fail() {
    FAIL=$((FAIL + 1))
    red   "  FAIL  $1"
    if [ -n "${2-}" ]; then
        printf '         expected: %s\n' "$2"
        printf '         got:      %s\n' "${3-}"
    fi
}

assert_eq() {
    local actual="$1" expected="$2" label="$3"
    if [ "$actual" = "$expected" ]; then
        pass "$label"
    else
        fail "$label" "$expected" "$actual"
    fi
}

assert_contains() {
    local haystack="$1" needle="$2" label="$3"
    if echo "$haystack" | grep -qF "$needle"; then
        pass "$label"
    else
        fail "$label" "(contains) $needle" "$haystack"
    fi
}

assert_not_contains() {
    local haystack="$1" needle="$2" label="$3"
    if echo "$haystack" | grep -qF "$needle"; then
        fail "$label" "(must not contain) $needle" "$haystack"
    else
        pass "$label"
    fi
}

# ── Cleanup trap ──────────────────────────────────────────────────────────────
# Ensures the FUSE mount is cleaned up even if a command fails mid-script.
FUSE_PID=""
cleanup() {
    umount "$MOUNT" 2>/dev/null || fusermount3 -u "$MOUNT" 2>/dev/null || true
    if [ -n "${FUSE_PID:-}" ]; then
        kill "$FUSE_PID" 2>/dev/null || true
        wait "$FUSE_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT INT TERM

# ── Mount ─────────────────────────────────────────────────────────────────────

echo "=== WARP DRIVE G2a acceptance ==="
echo ""
echo "Mounting at $MOUNT (echo-rlib backend) …"

mkdir -p "$MOUNT"
warp-drive-fuse --runtime=echo-rlib --mount "$MOUNT" &
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

# ── Directory listing ─────────────────────────────────────────────────────────

echo "── ls ──────────────────────────────────────────────────────────────────"
LS_OUT=$(ls -a "$MOUNT")
assert_contains "$LS_OUT" "README.md"    "ls / contains README.md"
assert_contains "$LS_OUT" "package.json" "ls / contains package.json"
assert_contains "$LS_OUT" "src"          "ls / contains src/"
assert_contains "$LS_OUT" "empty"        "ls / contains empty/"
assert_contains "$LS_OUT" "links"        "ls / contains links/"
assert_contains "$LS_OUT" ".warp"        "ls / contains .warp/"

# ── File contents ─────────────────────────────────────────────────────────────

echo ""
echo "── cat ─────────────────────────────────────────────────────────────────"

README=$(cat "$MOUNT/README.md")
assert_contains "$README" "WARP DRIVE G1 Fixture" "README.md first line"

PKG=$(cat "$MOUNT/package.json")
assert_contains "$PKG" '"name": "warp-drive-g1"' "package.json name field"

MAIN=$(cat "$MOUNT/src/main.ts")
assert_contains "$MAIN" "export function main" "src/main.ts export"

LIB=$(cat "$MOUNT/src/lib.ts")
assert_contains "$LIB" "export function identity" "src/lib.ts export"

# ── .warp/ surface ───────────────────────────────────────────────────────────

COORD=$(cat "$MOUNT/.warp/coordinate")
assert_contains "$COORD" '"worldline"'   ".warp/coordinate has worldline field"
assert_contains "$COORD" '"frontier"'    ".warp/coordinate has frontier field"
assert_contains "$COORD" '"state_root"'  ".warp/coordinate has state_root field"
assert_contains "$COORD" '"gate":"G2a"'  ".warp/coordinate identifies gate G2a"

RUNTIME_JSON=$(cat "$MOUNT/.warp/runtime")
assert_contains "$RUNTIME_JSON" '"kind":"echo-rlib"'  ".warp/runtime kind is echo-rlib"
assert_contains "$RUNTIME_JSON" '"gate":"G2a"'         ".warp/runtime gate is G2a"

STATS=$(cat "$MOUNT/.warp/stats")
assert_contains "$STATS" '"gate":"G2a"' ".warp/stats gate is G2a"

# ── G2a-specific: real Echo coordinate ───────────────────────────────────────

echo ""
echo "── G2a coordinate assertions ───────────────────────────────────────────"

# Worldline must not be the genesis placeholder from G1
assert_not_contains "$COORD" \
    '"worldline":"00000000-0000-0000-0000-000000000001"' \
    ".warp/coordinate worldline is real (not genesis placeholder)"

# state_root must be present and non-empty
STATE_ROOT=$(echo "$COORD" | grep -o '"state_root":"[^"]*"' | head -1 || true)
if [ -n "$STATE_ROOT" ] && [ "$STATE_ROOT" != '"state_root":""' ]; then
    pass ".warp/coordinate state_root is non-empty"
else
    fail ".warp/coordinate state_root is non-empty" "non-empty state_root" "$STATE_ROOT"
fi

# backend field must identify echo-rlib
assert_contains "$COORD" '"backend":"echo-rlib"' \
    ".warp/coordinate backend is echo-rlib"

# ── find ──────────────────────────────────────────────────────────────────────

echo ""
echo "── find ────────────────────────────────────────────────────────────────"
FIND_OUT=$(find "$MOUNT" | sort)
assert_contains "$FIND_OUT" "src/main.ts"      "find sees src/main.ts"
assert_contains "$FIND_OUT" "src/lib.ts"       "find sees src/lib.ts"
assert_contains "$FIND_OUT" ".warp/coordinate" "find sees .warp/coordinate"
assert_contains "$FIND_OUT" "empty"            "find sees empty/"
assert_contains "$FIND_OUT" "links/readme"     "find sees links/readme"

# ── ripgrep ───────────────────────────────────────────────────────────────────

echo ""
echo "── rg ──────────────────────────────────────────────────────────────────"
RG_OUT=$(rg --no-heading "export" "$MOUNT" 2>/dev/null || true)
RG_COUNT=$(echo "$RG_OUT" | grep -c "export" || true)
assert_contains "$RG_OUT" "main.ts"  "rg finds export in main.ts"
assert_contains "$RG_OUT" "lib.ts"   "rg finds export in lib.ts"
if [ "$RG_COUNT" -ge 2 ]; then
    pass "rg found $RG_COUNT export hits (≥ 2)"
else
    fail "rg export hit count" "≥ 2" "$RG_COUNT"
fi

# ── stat ──────────────────────────────────────────────────────────────────────

echo ""
echo "── stat ────────────────────────────────────────────────────────────────"
STAT_OUT=$(stat "$MOUNT/src/main.ts")
assert_contains "$STAT_OUT" "Inode: 5" "stat shows inode 5 for src/main.ts"
assert_contains "$STAT_OUT" "regular file" "stat shows regular file"

# ── readlink ──────────────────────────────────────────────────────────────────

echo ""
echo "── readlink ────────────────────────────────────────────────────────────"
LINK_TARGET=$(readlink "$MOUNT/links/readme")
assert_eq "$LINK_TARGET" "../README.md" "links/readme → ../README.md"

LINK_CONTENT=$(cat "$MOUNT/links/readme")
assert_contains "$LINK_CONTENT" "WARP DRIVE G1 Fixture" "symlink resolves to README.md"

# ── write rejection (EROFS) ───────────────────────────────────────────────────

echo ""
echo "── write rejection ─────────────────────────────────────────────────────"
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
    green "G2a GATE PASSED  ($PASS / $TOTAL assertions)"
    exit 0
else
    red   "G2a GATE FAILED  ($FAIL failed, $PASS passed, $TOTAL total)"
    exit 1
fi
