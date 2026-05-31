# Security Policy

## Supported versions

WARP DRIVE is pre-release software. No stable version has shipped.
Security fixes target the current `main` branch only.

## Reporting a vulnerability

Please do **not** open a public GitHub issue for security vulnerabilities.

Report privately via GitHub's Security Advisory feature:
https://github.com/flyingrobots/warp-drive/security/advisories/new

Include:
- A description of the vulnerability and its potential impact.
- Steps to reproduce.
- Any proof-of-concept code (treated as confidential).

You will receive a response within 7 days acknowledging receipt.

## Scope

WARP DRIVE is a FUSE filesystem driver. Relevant security concerns include:

- **Path traversal** — does the FUSE adapter correctly confine reads to
  the mounted projection?
- **Write rejection** — does the read-only posture hold under all FUSE
  operations, including edge cases like `setxattr`, `mknod`, `link`?
- **Privilege escalation** — does the mount process expose kernel
  capabilities beyond what FUSE requires?

Out of scope: vulnerabilities in `warp-wasm`/Echo (report upstream),
macFUSE (report to Benjamin Fleischer), or the Linux kernel FUSE subsystem.
