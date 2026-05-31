<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# `acceptance.sh` leaks EROFS shell error to stderr during write-rejection test

**File:** `scripts/acceptance.sh`

**Status:** cosmetic only — functionally correct. Fix whenever touching the script.

## The smell

The write-rejection test does:

```bash
if echo "nope" > "$MOUNT/README.md" 2>/dev/null; then ...
```

But bash emits the EROFS error for the output redirect (`>`) *before* it
sets up the `2>/dev/null` redirect, so the shell's own error message leaks
to stderr regardless:

```
scripts/acceptance.sh: line N: /tmp/warp-g1/README.md: Read-only file system
  PASS  write to README.md correctly rejected (EROFS)
```

The test passes correctly; the output is just messy.

## Resolution

Wrap in a subshell so `2>/dev/null` applies to the shell's own redirect
error too:

```bash
if ( echo "nope" > "$MOUNT/README.md" ) 2>/dev/null; then
    fail "write to README.md should be rejected (EROFS)"
else
    pass "write to README.md correctly rejected (EROFS)"
fi
```

Apply the same pattern to the `touch` check.
