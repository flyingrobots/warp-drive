<!-- SPDX-License-Identifier: Apache-2.0 OR LicenseRef-MIND-UCAL-1.0 -->
<!-- © James Ross Ω FLYING•ROBOTS <https://github.com/flyingrobots> -->

# macFUSE system extension approval check in `cargo xtask install-deps`

**Status:** cool idea. Polish task, low effort, high dev-experience value.

## The idea

`brew install --cask macfuse` installs the package but a macOS system
extension approval step is required before the kernel module loads. That
step requires the user to open System Settings → Privacy & Security and
click "Allow". Until they do, `mount2` fails with a cryptic permission
error.

`cargo xtask install-deps` should detect whether the extension is already
approved and, if not, print a clear next-step message:

```
✓ macFUSE installed.
  Action required: open System Settings → Privacy & Security and
  allow the kernel extension from "Benjamin Fleischer / macFUSE".
  Then rerun: cargo xtask mount --path /tmp/warp-drive-g1
```

Detection: `kextstat | grep macfuse` or `systemextensionsctl list` returns
the extension's activation state. A non-active state triggers the message.

## Why it matters

Without this check, a new contributor runs `cargo xtask install-deps`,
immediately tries `cargo xtask mount`, gets `Operation not permitted` or
`No such file or directory` on `/dev/osxfuse0`, and has no idea why.
The error is in macOS security policy, not in the code, but the code is
where they are looking.

## Surface when

Polishing `xtask install-deps` for the G1 acceptance pass.
