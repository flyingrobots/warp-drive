#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>
#
# G1 gate acceptance test.
# Mounts the in-memory fixture tree and verifies POSIX read semantics and
# write rejection.
#
# Usage (Docker): docker run --rm --device /dev/fuse --cap-add SYS_ADMIN warp-drive-g1
# Usage (local):  bash scripts/acceptance.sh
set -euo pipefail

MOUNT=/tmp/warp-g1
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

# ── Mount ─────────────────────────────────────────────────────────────────────

echo "=== WARP DRIVE G1 acceptance ==="
echo ""
echo "Mounting at $MOUNT …"

mkdir -p "$MOUNT"
warp-drive-fuse --runtime=in-memory --mount "$MOUNT" &
FUSE_PID=$!

# Wait up to 5 s for the FUSE mount to appear.
# We check that:
#   (a) the fuse process is still alive, and
#   (b) the mountpoint's filesystem type contains "fuse"
# Checking only `ls` is insufficient — it succeeds on an empty directory
# even when nothing is mounted.
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
    kill "$FUSE_PID" 2>/dev/null || true
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

COORD=$(cat "$MOUNT/.warp/coordinate")
assert_contains "$COORD" '"worldline"'  ".warp/coordinate has worldline"
assert_contains "$COORD" '"frontier"'   ".warp/coordinate has frontier"

RUNTIME=$(cat "$MOUNT/.warp/runtime")
assert_contains "$RUNTIME" '"kind":"in-memory"' ".warp/runtime kind"
assert_contains "$RUNTIME" '"gate":"G1"'        ".warp/runtime gate"

STATS=$(cat "$MOUNT/.warp/stats")
assert_contains "$STATS" '"gate":"G1"' ".warp/stats gate"

# ── find ──────────────────────────────────────────────────────────────────────

echo ""
echo "── find ────────────────────────────────────────────────────────────────"
FIND_OUT=$(find "$MOUNT" | sort)
assert_contains "$FIND_OUT" "src/main.ts"     "find sees src/main.ts"
assert_contains "$FIND_OUT" "src/lib.ts"      "find sees src/lib.ts"
assert_contains "$FIND_OUT" ".warp/coordinate" "find sees .warp/coordinate"
assert_contains "$FIND_OUT" "empty"           "find sees empty/"
assert_contains "$FIND_OUT" "links/readme"    "find sees links/readme"

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

# ── symlink resolution ────────────────────────────────────────────────────────

LINK_CONTENT=$(cat "$MOUNT/links/readme")
assert_contains "$LINK_CONTENT" "WARP DRIVE G1 Fixture" "symlink resolves to README.md"

# ── write rejection (EROFS) ───────────────────────────────────────────────────

echo ""
echo "── write rejection ─────────────────────────────────────────────────────"
if echo "nope" > "$MOUNT/README.md" 2>/dev/null; then
    fail "write to README.md should be rejected (EROFS)"
else
    pass "write to README.md correctly rejected (EROFS)"
fi

if touch "$MOUNT/newfile.txt" 2>/dev/null; then
    fail "create newfile.txt should be rejected (EROFS)"
else
    pass "create newfile.txt correctly rejected (EROFS)"
fi

# ── Unmount ───────────────────────────────────────────────────────────────────

echo ""
echo "Unmounting…"
umount "$MOUNT" 2>/dev/null || fusermount3 -u "$MOUNT" 2>/dev/null || true
wait "$FUSE_PID" 2>/dev/null || true
echo "Unmounted."

# ── Report ────────────────────────────────────────────────────────────────────

echo ""
echo "════════════════════════════════════════════"
TOTAL=$((PASS + FAIL))
if [ "$FAIL" -eq 0 ]; then
    green "G1 GATE PASSED  ($PASS / $TOTAL assertions)"
    exit 0
else
    red   "G1 GATE FAILED  ($FAIL failed, $PASS passed, $TOTAL total)"
    exit 1
fi
