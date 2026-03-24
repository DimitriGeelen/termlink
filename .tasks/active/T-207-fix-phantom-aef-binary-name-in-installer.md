---
id: T-207
name: "Fix phantom aef binary name in installer"
description: >
  Inception: Fix phantom aef binary name in installer

status: started-work
workflow_type: inception
owner: human
horizon: later
tags: []
components: []
related_tasks: []
created: 2026-03-21T15:32:44Z
last_update: 2026-03-24T08:03:27Z
date_finished: null
---

# T-207: Fix phantom aef binary name in installer

## Problem Statement

The installer references a binary called `aef` but the actual binaries are `fw` and `claude-fw`. When the sudo symlink step failed (T-206), guidance suggested manually creating a symlink for `aef` — which doesn't exist. This caused confusion and wasted debugging time.

**Consumer report from macOS ARM64 installation (2026-03-21):**

### Error Artefact
```
# Claude Code suggested (based on installer):
$ sudo ln -sf /Users/dimitri/.agentic-framework/bin/aef /usr/local/bin/aef

# Actual binaries found:
$ ls /Users/dimitri/.agentic-framework/bin/
claude-fw    fw    watchtower.sh

# Installer symlink code (install.sh:189-193):
ln -sf "$fw_path" "$SYMLINK_DIR/fw"        # references fw
sudo ln -sf "$fw_path" "$SYMLINK_DIR/fw"   # references fw
# But some other part of installer or docs references "aef"
```

### Critical Research Finding
- **"aef" does not exist anywhere in either codebase** — zero references found
- It appears to be a phantom name from an earlier naming decision ("Agentic Engineering Framework" → "aef") that was never implemented
- `fw` is used 100+ times in documentation (CLAUDE.md, README, help text)
- `fw` follows Unix convention: short, lowercase, high-frequency commands earn short names (`ls`, `git`, `gh`)
- `claude-fw` follows `tool-context` convention (`git-lfs`, `docker-compose`)
- No widely-used `fw` binary exists in common package managers (Homebrew, apt)

### Environment
- macOS Darwin 25.3.0 (ARM64)
- Framework v1.2.6

## Assumptions

- "aef" was a planned name that was never adopted
- No external documentation or scripts reference "aef"
- `fw` has no real-world conflicts with other tools

## Exploration Plan

1. Search entire codebase for "aef" references: install.sh, docs, scripts
2. Verify `fw` doesn't conflict: `brew search fw`, check common Linux packages
3. Fix references and add post-install verification
4. Investigate: was "aef" a planned rename? Is there an open issue?

## Technical Constraints

- Must not break any existing documentation or scripts that reference `fw`
- Homebrew formula name (if created) should be descriptive: `agentic-framework`

## Scope Fence

**IN scope:** Fix installer references from `aef` to `fw`/`claude-fw`, add verification
**OUT of scope:** Renaming the binary, adding aliases, Homebrew formula creation

## Acceptance Criteria

- [x] Zero "aef" references in installer and documentation (upstream fix)
- [x] Post-install verification checks: `command -v fw`, `fw version`, `fw doctor` (upstream T-515)
- [x] Each verification failure prints the specific manual fix command (upstream install.sh lines 224-267)
- [x] Go/No-Go decision made (GO)

## Verification

# No "aef" references in installer
! grep -qi "aef" /opt/999-Agentic-Engineering-Framework/install.sh
# Post-install verification exists
grep -q "Post-install verification" /opt/999-Agentic-Engineering-Framework/install.sh

## Go/No-Go Criteria

**GO if:**
- Confirmed "aef" is phantom — find-replace is safe
- Post-install verification is feasible (~10 lines of bash)

**NO-GO if:**
- "aef" is referenced by external tools or user scripts that would break

## Decisions

**Decision**: GO

**Rationale**: aef is phantom name with zero references. fw is correct and established.

**Date**: 2026-03-21T15:42:30Z
## Decision

**Decision**: GO

**Rationale**: aef is phantom name with zero references. fw is correct and established.

**Date**: 2026-03-21T15:42:30Z

## Updates

- 2026-03-21: Consumer confused by "aef" reference during manual symlink attempt
- 2026-03-21: Critical review confirmed zero "aef" references in codebase — phantom name
- 2026-03-21: Recommendation: find-replace in installer, add 3-step verification, effort ~5 minutes

### 2026-03-21T15:42:20Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** aef is phantom name with zero references. fw is correct and established.

### 2026-03-21T15:42:30Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** aef is phantom name with zero references. fw is correct and established.

### 2026-03-24T08:03:27Z — status-update [task-update-agent]
- **Change:** horizon: now → later
