# T-208: TermLink Distribution via Homebrew Tap

**Decision:** GO (2026-03-21)
**Rationale:** PTY/fork FFI is standard POSIX, low risk. Homebrew tap solves install, Gatekeeper, sudo, and auth in one move.

## Problem

Installing TermLink on macOS requires: (1) Rust/Cargo toolchain (1.6GB LLVM), (2) authenticated access to internal OneDev repo, (3) sudo for `/opt/termlink`. This creates a 6-step cascading failure that took ~15 minutes to troubleshoot.

## Key Findings

1. **Rust should be recommended for dev/debug, not required to install.** Different framing: not required *to install*, recommended *to develop/debug*.
2. **Gatekeeper quarantines unsigned pre-built binaries** — `xattr -cr` workaround is hostile. Homebrew handles this by design.
3. **cargo-binstall doesn't help** — it requires cargo already installed.
4. **OneDev auth is moot** — GitHub mirror exists and syncs on push. Distribution should use GitHub.
5. **`/opt/termlink` is a Linux FHS convention** that fights Homebrew for namespace on macOS ARM64.

## Architecture

- GitHub Actions builds for aarch64-apple-darwin and x86_64-apple-darwin
- Homebrew tap formula (`dimitri/tap/termlink`) installs pre-built binary
- `cargo install` remains as dev/debug path
- Build task: T-212
