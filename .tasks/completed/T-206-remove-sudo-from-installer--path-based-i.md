---
id: T-206
name: "Remove sudo from installer — PATH-based install"
description: >
  Inception: Remove sudo from installer — PATH-based install

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T15:32:43Z
last_update: 2026-03-21T15:42:46Z
date_finished: 2026-03-21T15:42:46Z
---

# T-206: Remove sudo from installer — PATH-based install

## Problem Statement

`install.sh` calls `sudo ln -sf` to create a symlink in `/usr/local/bin`. This fails in non-interactive shells (Claude Code, piped installs, CI) where sudo cannot prompt for a password.

**Consumer report from macOS ARM64 installation (2026-03-21):**

### Error Artefact
```
$ curl -fsSL .../install.sh | bash
[+] Creating symlink in /usr/local/bin (requires sudo)...
sudo: a terminal is required to read the password; either use the -S option to read from standard input or configure an askpass helper
sudo: a password is required
```

The installer ran via `curl | bash` (piped mode) and via Claude Code's Bash tool — both non-interactive contexts where sudo cannot prompt. The framework cloned successfully but the installation exited with error code 1, making it appear as a total failure.

### Workaround Applied
Manually added `export PATH="$HOME/.agentic-framework/bin:$PATH"` to `~/.zshrc`. This works because the binaries (`fw`, `claude-fw`) already exist at `~/.agentic-framework/bin/`.

### Critical Research Finding
- `~/.local/bin` (initially proposed as alternative) is an XDG/Linux convention — NOT native to macOS. macOS doesn't add it to `$PATH` by default. Adopting it means importing a Linux convention and patching over it.
- `~/.agentic-framework/bin/` already contains the binaries — creating symlinks elsewhere is pure indirection.
- Auto-modifying `.zshrc`/`.bashrc` is invasive: causes duplicates on reinstall, conflicts with shell frameworks (oh-my-zsh, starship), users manage dotfiles in repos.
- "Try sudo, fall back" creates confusion: some installs are global, some local, user doesn't know which happened.
- Pattern precedent: rustup, cargo, and Homebrew's Linux installer all print a PATH line for the user to paste — they don't auto-modify RC files.

### Environment
- macOS Darwin 25.3.0 (ARM64)
- Contexts affected: `curl | bash`, Claude Code Bash tool, CI runners, Docker containers

## Assumptions

- The framework's own bin/ directory is sufficient — no need for a second location
- Users are capable of adding a PATH line (or CI scripts can pass a flag)
- No framework functionality depends on binaries being in /usr/local/bin specifically

## Exploration Plan

1. Audit install.sh for all sudo calls
2. Verify no post-install scripts assume /usr/local/bin/fw exists
3. Prototype `--modify-path` flag with idempotency (grep before append)
4. Test: `curl | bash` completes with exit 0, prints PATH instruction

## Technical Constraints

- Must work in: piped shells, Claude Code, CI, Docker, interactive terminals
- Must not modify user's shell RC files without explicit opt-in
- Must support both zsh and bash

## Scope Fence

**IN scope:** Remove sudo from install.sh, add `--modify-path` flag, print PATH instruction
**OUT of scope:** Homebrew formula for the framework itself, XDG compliance, fish/nushell support

## Acceptance Criteria

- [ ] Zero sudo calls in install.sh (upstream build AC — not inception scope)
- [ ] `curl -fsSL .../install.sh | bash` exits 0 (upstream build AC — not inception scope)
- [ ] PATH instruction printed at end of install (upstream build AC — not inception scope)
- [ ] `--modify-path` flag appends PATH to .zshrc/.bashrc idempotently (upstream build AC — not inception scope)
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- install.sh works in all non-interactive contexts without sudo
- Users who want /usr/local/bin can still do it manually

**NO-GO if:**
- Some framework functionality requires /usr/local/bin placement
- Removing sudo breaks Linux installs (need to check)

## Decisions

**Decision**: GO

**Rationale**: No downstream dependency on /usr/local/bin. PATH-based install validated by consumer.

**Date**: 2026-03-21T15:42:26Z
## Decision

**Decision**: GO

**Rationale**: No downstream dependency on /usr/local/bin. PATH-based install validated by consumer.

**Date**: 2026-03-21T15:42:26Z

## Updates

- 2026-03-21: Consumer hit sudo failure during piped and Claude Code installation
- 2026-03-21: Critical review found ~/.local/bin is wrong for macOS, framework's own bin/ is sufficient
- 2026-03-21: Recommendation: remove all sudo, print PATH line, add --modify-path flag

### 2026-03-21T15:42:10Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** No downstream dependency on /usr/local/bin. PATH-based install validated by consumer.

### 2026-03-21T15:42:16Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** No downstream dependency on /usr/local/bin. PATH-based install validated by consumer.

### 2026-03-21T15:42:20Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-21T15:42:23Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** No downstream dependency on /usr/local/bin. PATH-based install validated by consumer.

### 2026-03-21T15:42:26Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** No downstream dependency on /usr/local/bin. PATH-based install validated by consumer.

### 2026-03-21T15:42:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
