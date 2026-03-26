# T-205: Drop PyYAML Phantom Dependency from Installer

**Decision:** GO (2026-03-21)
**Rationale:** Zero PyYAML imports confirmed. Phantom dependency blocks macOS Homebrew installs.

## Problem

The framework installer's prerequisite check requires PyYAML (`pip install pyyaml`), which fails on macOS with Homebrew-managed Python 3.14+ due to PEP 668 (`externally-managed-environment`). This blocks installation entirely on modern macOS.

## Key Finding

**Zero PyYAML imports exist in the framework codebase.** All YAML parsing uses grep/awk on flat frontmatter:
```bash
TASK=$(grep -m1 '^current_task:' "$FOCUS_FILE" | awk '{print $2}' || true)
```
No anchors, no merge keys, no complex nesting. PyYAML is a phantom dependency — listed as a prerequisite but never imported.

## Resolution

Upstream fix: T-508/T-513 removed PyYAML from install.sh prerequisites.

## Environment

- macOS Darwin 25.3.0 (ARM64), Python 3.14.3 (Homebrew)
- PEP 668 enforced — `pip install` system-wide blocked by design
