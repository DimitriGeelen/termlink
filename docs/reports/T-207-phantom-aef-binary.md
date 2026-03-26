# T-207: Fix Phantom aef Binary Name in Installer

**Status:** In progress (horizon: later)

## Problem

The installer references a binary called `aef` but the actual binaries are `fw` and `claude-fw`. When the sudo symlink step failed (T-206), guidance suggested manually creating a symlink for `aef` — which doesn't exist. This caused confusion and wasted debugging time during macOS ARM64 installation.

## Findings

- The `aef` name appears to be a legacy artifact from an earlier naming convention
- Actual binaries: `fw` (framework CLI) and `claude-fw` (Claude Code wrapper with auto-restart)
- The installer's prerequisite check and error messages reference the wrong binary name
- Fix is straightforward: update installer references from `aef` to `fw`

## Status

Parked at horizon: later. Low priority — workaround is known (use `fw` directly).
