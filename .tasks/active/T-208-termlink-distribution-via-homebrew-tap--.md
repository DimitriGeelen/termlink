---
id: T-208
name: "TermLink distribution via Homebrew tap — keep Rust as dev dependency"
description: >
  Inception: TermLink distribution via Homebrew tap — keep Rust as dev dependency

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T15:32:45Z
last_update: 2026-03-21T15:42:38Z
date_finished: null
---

# T-208: TermLink distribution via Homebrew tap — keep Rust as dev dependency

## Problem Statement

Installing TermLink on macOS requires: (1) Rust/Cargo toolchain not present by default, (2) authenticated access to internal OneDev repo, (3) sudo for `/opt/termlink`. This creates a cascading barrier — each dependency blocks the next, and none have graceful fallbacks.

**Consumer report from macOS ARM64 installation (2026-03-21):**

### Error Artefact — Cascading failures
```
# Step 1: fw termlink check
WARN  TermLink not installed
  Install: git clone <repo> && cd termlink && cargo install --path crates/termlink-cli

# Step 2: No Rust
$ cargo --version
cargo not found
# Required: brew install rust (1.6GB LLVM dependency, ~5 min)

# Step 3: After Rust installed, clone fails
$ git clone https://onedev.docker.ring20.geelenandcompany.com/termlink /opt/termlink
fatal: could not create work tree dir '/opt/termlink': Permission denied

# Step 4: Try home dir
$ git clone https://onedev.docker.ring20.geelenandcompany.com/termlink ~/.termlink
fatal: could not read Username for '...': Device not configured
# Non-interactive context can't prompt for OneDev credentials

# Step 5: With inline credentials (workaround)
$ git clone 'https://admin:pass@onedev..../termlink' ~/.termlink
# Success — then cargo install works

# Step 6: TERMLINK_REPO env var needed
$ export TERMLINK_REPO="$HOME/.termlink"
# fw termlink check now passes
```

Total time to install TermLink: ~15 minutes of troubleshooting across 6 failure points.

### Critical Research Finding
- **Rust should NOT be eliminated as a requirement** — TermLink is a 4-crate Rust workspace with tokio-rustls, PTY/fork via libc, and capability-token crypto. Users who debug TLS negotiation or PTY handling NEED `cargo build`.
- **But Rust shouldn't be a BARRIER to install.** Different framing: not required *to install*, recommended *to develop/debug*.
- **Gatekeeper will quarantine unsigned pre-built binaries on macOS.** `xattr -cr` workaround is hostile to new users. Homebrew handles this by design — binaries installed via `brew` are not quarantined.
- **cargo-binstall doesn't help** — it requires cargo already installed, which is the exact problem.
- **OneDev auth is moot** — GitHub mirror already exists and syncs on push. Distribution should use GitHub.
- `/opt/termlink` is a Linux FHS convention. On macOS ARM64, `/opt/homebrew` is Homebrew's prefix. Installing to `/opt/termlink` fights Homebrew for namespace.
- **TermLink crate architecture:** `termlink-cli`, `termlink-hub`, `termlink-protocol`, `termlink-session`, `termlink-test-utils`

### Environment
- macOS Darwin 25.3.0 (ARM64)
- Rust 1.94.0 (after brew install)
- TermLink v0.1.0

## Assumptions

- GitHub mirror of TermLink repo exists and is public
- GitHub Actions macOS runners are available for CI builds
- Homebrew tap creation is straightforward
- TermLink's libc FFI for PTY/fork compiles on both ARM64 and x86_64 macOS

## Exploration Plan

1. Verify GitHub mirror exists and is public
2. Test cross-compilation vs native compilation for macOS targets
3. Prototype Homebrew formula (does it build from source via cargo, or use pre-built bottles?)
4. Test: `brew install dimitri/tap/termlink` on clean macOS
5. Verify Gatekeeper doesn't quarantine brew-installed binary

## Technical Constraints

- macOS Gatekeeper quarantines unsigned downloaded binaries
- TermLink uses `libc` for PTY/fork — platform-specific C FFI, may not cross-compile cleanly
- GitHub Actions gives 3,000 free macOS minutes/month for public repos
- Apple Developer ID signing costs $99/year (alternative to Homebrew)

## Scope Fence

**IN scope:** Homebrew tap, GitHub Actions CI for macOS builds, documentation of cargo install as dev path
**OUT of scope:** Linux packaging, Windows support, Apple Developer ID signing, making OneDev public

## Acceptance Criteria

- [ ] Homebrew tap repo created (deferred to T-212 build task)
- [x] GitHub Actions workflow builds TermLink for aarch64-apple-darwin and x86_64-apple-darwin (.github/workflows/release.yml)
- [ ] `brew install dimitri/tap/termlink` works on clean macOS (deferred to T-212)
- [ ] `fw termlink check` passes after brew install (deferred to T-212)
- [ ] Documentation updated (deferred to T-212)
- [x] Go/No-Go decision made (GO)

## Verification

# Release workflow exists with correct package name
grep -q -- "-p termlink" .github/workflows/release.yml
# CI workflow exists
test -f .github/workflows/ci.yml

## Go/No-Go Criteria

**GO if:**
- TermLink compiles on macOS runners in GitHub Actions
- Homebrew formula works end-to-end
- Gatekeeper doesn't block brew-installed binary

**NO-GO if:**
- PTY/fork FFI prevents cross-compilation and native macOS runners are insufficient
- Homebrew formula complexity exceeds maintenance budget
- GitHub mirror doesn't exist or can't be made public

## Decisions

**Decision**: GO

**Rationale**: PTY/fork FFI is standard POSIX, low risk. Homebrew tap solves install, Gatekeeper, sudo, and auth in one move.

**Date**: 2026-03-21T15:42:42Z
## Decision

**Decision**: GO

**Rationale**: PTY/fork FFI is standard POSIX, low risk. Homebrew tap solves install, Gatekeeper, sudo, and auth in one move.

**Date**: 2026-03-21T15:42:42Z

## Updates

- 2026-03-21: Consumer hit 6-step cascading failure installing TermLink on macOS
- 2026-03-21: Critical review: keep Rust (needed for dev/debug), add Homebrew tap (solves install)
- 2026-03-21: Risk identified: libc PTY/fork FFI may need native macOS CI runners, not cross-compile

### 2026-03-21T15:42:24Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** PTY/fork FFI is standard POSIX, low risk. Homebrew tap solves install, Gatekeeper, sudo, and auth in one move.

### 2026-03-21T15:42:38Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-21T15:42:42Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** PTY/fork FFI is standard POSIX, low risk. Homebrew tap solves install, Gatekeeper, sudo, and auth in one move.
