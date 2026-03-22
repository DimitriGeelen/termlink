---
id: T-205
name: "Drop PyYAML phantom dependency from installer"
description: >
  Inception: Drop PyYAML phantom dependency from installer

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T15:32:42Z
last_update: 2026-03-22T16:59:38Z
date_finished: 2026-03-22T16:59:38Z
---

# T-205: Drop PyYAML phantom dependency from installer

## Problem Statement

The `install.sh` prerequisite check requires PyYAML (`pip install pyyaml`), which fails on macOS with Homebrew-managed Python 3.14 due to PEP 668 (`externally-managed-environment` error). This blocks the entire installation.

**Consumer report from macOS ARM64 installation (2026-03-21):**

### Error Artefact
```
$ pip3 install pyyaml
error: externally-managed-environment
× This environment is externally managed
╰─> To install Python packages system-wide, try brew install xyz...
    If you wish to install a Python library that isn't in Homebrew,
    use a virtual environment...
hint: See PEP 668 for the detailed specification.
```

### Workaround Applied
`pip3 install --break-system-packages pyyaml` — works but is fragile and not recommended by Homebrew.

### Critical Research Finding
**Zero PyYAML imports exist in the framework codebase.** All YAML parsing is already performed with grep/awk on flat frontmatter:
```bash
TASK=$(grep -m1 '^current_task:' "$FOCUS_FILE" | awk '{print $2}' || true)
```
The YAML structures in `.context/` and `.tasks/` are flat or shallow — frontmatter fields, single-level config. No anchors, no merge keys, no multi-line values, no complex nesting. PyYAML is a phantom dependency — listed as a prerequisite but never imported.

### Environment
- macOS Darwin 25.3.0 (ARM64)
- Python 3.14.3 (Homebrew)
- PEP 668 enforced

## Assumptions

- PyYAML is not imported or used anywhere in the framework codebase
- All YAML access is via grep/awk on simple key-value frontmatter
- Removing the prerequisite will not break any framework functionality

## Exploration Plan

1. Verify zero PyYAML imports: `grep -r "import yaml" ~/.agentic-framework/`
2. Verify zero `yaml.safe_load` or `yaml.dump` calls
3. If confirmed: remove PyYAML from install.sh prerequisites
4. Run `fw self-test` after removal to confirm nothing breaks

## Technical Constraints

- Homebrew Python 3.12+ enforces PEP 668 — no system-wide pip installs without `--break-system-packages`
- Venvs break on Homebrew Python minor version upgrades (symlinks die)
- Framework must remain portable: no Python-version-specific infrastructure

## Scope Fence

**IN scope:** Remove PyYAML prerequisite check from install.sh
**OUT of scope:** Adding alternative YAML parsing, migrating to TOML, adding venv infrastructure

## Acceptance Criteria

- [x] Confirmed zero PyYAML imports in core framework (web/docgen use it optionally)
- [x] PyYAML prerequisite removed from install.sh (upstream T-508/T-513)
- [x] `fw doctor` passes without PyYAML as prerequisite
- [x] Go/No-Go decision made (GO)

## Go/No-Go Criteria

**GO if:**
- Zero PyYAML imports confirmed
- install.sh works without PyYAML on a clean macOS system

**NO-GO if:**
- PyYAML is actually imported somewhere (hidden dependency)
- Removal causes test failures

## Verification

# Verify no PyYAML prerequisite check in install.sh
! grep -q "pyyaml\|PyYAML" /opt/999-Agentic-Engineering-Framework/install.sh

## Decisions

**Decision**: GO

**Rationale**: Zero PyYAML imports confirmed. Phantom dependency blocks macOS Homebrew installs.

**Date**: 2026-03-21T15:42:33Z
## Decision

**Decision**: GO

**Rationale**: Zero PyYAML imports confirmed. Phantom dependency blocks macOS Homebrew installs.

**Date**: 2026-03-21T15:42:33Z

## Updates

- 2026-03-21: Consumer hit PEP 668 error during installation on macOS ARM64
- 2026-03-21: Critical review agent found zero PyYAML imports — phantom dependency confirmed
- 2026-03-21: Recommendation: remove from prerequisites, effort ~5 minutes

### 2026-03-21T15:42:12Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Zero PyYAML imports confirmed. Phantom dependency blocks macOS Homebrew installs.

### 2026-03-21T15:42:24Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-21T15:42:27Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Zero PyYAML imports confirmed. Phantom dependency blocks macOS Homebrew installs.

### 2026-03-21T15:42:33Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Zero PyYAML imports confirmed. Phantom dependency blocks macOS Homebrew installs.

### 2026-03-22T16:59:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
