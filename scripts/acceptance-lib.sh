#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots>
#
# Shared acceptance-script helpers for the G3 gate scripts
# (scripts/acceptance-g3.sh, scripts/acceptance-g3-echo.sh).
#
# Not sourced by the frozen G1/G2 scripts (acceptance.sh, acceptance-g2.sh,
# acceptance-g2b.sh) — those stay exactly as validated at their own gates.
#
# This file is meant to be `source`d, not executed directly.

green() { printf '\033[0;32m%s\033[0m\n' "$*"; }
red()   { printf '\033[0;31m%s\033[0m\n' "$*"; }

PASS=0
FAIL=0

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

json_hex_value() {
    local json="$1" key="$2"
    echo "$json" | sed -n "s/.*\"$key\":\"\\([0-9a-f]*\\)\".*/\\1/p" | head -1
}

assert_nonzero_hex64() {
    local value="$1" label="$2"
    local zero="0000000000000000000000000000000000000000000000000000000000000000"
    if echo "$value" | grep -Eq '^[0-9a-f]{64}$' && [ "$value" != "$zero" ]; then
        pass "$label"
    else
        fail "$label" "non-zero 64-char hex" "$value"
    fi
}

# Extracts a numeric JSON field's value, tolerating the constant-width
# leading whitespace `MountStats::snapshot_json` pads every counter with
# (e.g. `"lookup_count":                   0`).
json_int_value() {
    local json="$1" key="$2"
    echo "$json" | sed -n "s/.*\"$key\":[[:space:]]*\([0-9][0-9]*\).*/\1/p" | head -1
}

# Records a failure and returns non-zero (without touching PASS/FAIL twice)
# if `value` is not an unsigned decimal integer. Callers must check the
# return value before doing shell arithmetic on `value` — malformed
# extraction reaching `$((...))` can abort the whole script under `set -e`.
require_numeric() {
    local value="$1" label="$2"
    if [[ "$value" =~ ^[0-9]+$ ]]; then
        return 0
    fi
    fail "$label" "unsigned decimal integer" "$value"
    return 1
}

# Bulk lower-bound proof: `after - before >= minimum`. A single before/after
# probe pair is invalid on its own — the probe's own diagnostic reads also
# trigger lookups/getattrs — so every counter proof in the G3 scripts uses a
# batch of probes and a minimum delta instead of an exact "+1".
assert_delta_ge() {
    local after="$1" before="$2" minimum="$3" label="$4"
    if ! require_numeric "$after" "$label after value" ||
       ! require_numeric "$before" "$label before value" ||
       ! require_numeric "$minimum" "$label minimum delta"; then
        return 0  # failure already recorded; let the script keep going
    fi
    local delta=$((after - before))
    if [ "$delta" -ge "$minimum" ]; then
        pass "$label (delta $delta >= $minimum)"
    else
        fail "$label" "delta >= $minimum" "$delta"
    fi
}

# Forces kernel attribute synchronization rather than accepting a cached
# `stat()` — required (not merely preferred) for the getattr-freshness
# probes; the preflight check at each script's top fails loudly if this
# GNU coreutils feature is unavailable rather than silently falling back to
# a plain `stat` that would weaken the proof.
fresh_stat_size() {
    env stat --cached=never -c '%s' "$1"
}
