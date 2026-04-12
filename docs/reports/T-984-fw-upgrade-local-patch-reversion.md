# T-984: fw upgrade silently reverts local patches

## Problem Statement

`fw upgrade` on vendored consumer projects (like termlink) silently overwrites locally-patched framework files. The T-1157 upstream refactor replaced per-file sync with a bulk `do_vendor` call, which has no local modification detection. T-978 had added checksum manifest + backup logic, but it was in the old per-file sync code that T-1157 replaced.

**Impact observed in session S-2026-0412-1825 (this session):**
- 6 local fixes were reverted: T-911 (harvest.sh), T-913 (fw doctor), T-949 (inception.sh), T-938a (verify-acs.sh), T-938b (block-task-tools.sh). T-981 (check-tier0.sh) survived because upstream also includes it.
- Fixes had to be re-applied every session via `git checkout HEAD --` or manual edits.
- The T-978 backup code itself was overwritten — the safeguard was its own first victim.

## Evidence

### Files reverted by fw upgrade

| File | Fix | Task | Status after upgrade |
|------|-----|------|---------------------|
| lib/harvest.sh | PROJECT_ROOT paths | T-911 | FRAMEWORK_ROOT (reverted) |
| bin/fw | Doctor symlink check | T-913 | Check 1b missing (reverted) |
| lib/inception.sh | Auto-transition captured→started-work | T-949 | Missing (reverted) |
| lib/verify-acs.sh | Dynamic fw_cmd resolution | T-938 | bin/fw hardcoded (reverted) |
| agents/context/block-task-tools.sh | Bare fw in hints | T-938 | bin/fw (reverted) |
| lib/upgrade.sh | Checksum manifest + backup | T-978 | Entire function removed by T-1157 |

### Root cause chain

1. Consumer project vendors framework into `.agentic-framework/`
2. Agent patches vendored files to fix consumer-specific issues
3. Patches are committed to consumer's git history
4. `fw upgrade` runs (manually or via cron)
5. `do_vendor` (T-1157) does bulk rsync from upstream → `.agentic-framework/`
6. No checksum comparison, no backup, no warning
7. Local patches silently overwritten in working tree
8. Next session starts with broken vendored files
9. Agent spends time diagnosing and re-applying fixes

### The T-978 irony

T-978 added checksum manifest and backup to the **old** per-file sync in upgrade.sh.
T-1157 upstream replaced the per-file sync with a single `do_vendor()` call.
The `fw upgrade` that brought T-1157 also overwrote the T-978 backup code.
The protection mechanism was destroyed by the exact threat it was designed to protect against.

## Options

### Option A: Add modification detection to do_vendor()

**Approach:** Before `do_vendor` copies each file, compare local checksum against `.upstream-checksums`. If different, back up to `.upgrade-backup/` and warn.

**Pros:** Solves the problem at the source (upstream). Framework-wide fix.
**Cons:** Requires upstream PR. May conflict with upstream's intent (T-1157 simplified sync deliberately).
**Effort:** Medium (adapt T-978 logic into do_vendor pattern).

### Option B: Git-based protection — never commit upstream fw upgrade without review

**Approach:** After `fw upgrade`, run `git diff .agentic-framework/` and require human review before committing. Add a pre-commit hook that detects `.agentic-framework/` changes and warns about potential local patch reversion.

**Pros:** No upstream dependency. Uses existing git infrastructure.
**Cons:** Doesn't prevent the reversion, just catches it before commit. Requires discipline.
**Effort:** Low.

### Option C: Patch tracking file — .agentic-framework/.local-patches

**Approach:** Maintain a manifest of files with local patches and the task IDs that introduced them. `fw upgrade` reads this file and skips listed files (or backs them up with a prominent warning). `fw upgrade --force` overrides.

**Pros:** Simple, declarative, consumer-controlled.
**Cons:** Manual maintenance (agent must remember to add entries).
**Effort:** Low-medium.

### Option D: Upstream the fixes

**Approach:** Instead of patching vendored files, push fixes upstream to the framework repo so `fw upgrade` brings the fix rather than reverting it.

**Pros:** Permanent solution. No divergence to manage.
**Cons:** Upstream may not accept consumer-specific fixes (e.g., PROJECT_ROOT resolution is consumer-specific by nature). Slow cycle time.
**Effort:** Per-fix, ongoing.

## Recommendation

**GO** with Option C (patch tracking file) as primary, supplemented by Option D (upstream appropriate fixes).

**Rationale:**
- Option C is low-effort, consumer-controlled, and immediately actionable
- Option D is the right long-term approach for fixes that are genuinely upstream bugs
- Option A requires upstream acceptance and is at odds with T-1157's simplification intent
- Option B is reactive (catches after reversion) rather than preventive

**Build scope:** 
1. `.agentic-framework/.local-patches` manifest format (YAML: file path, task ID, description)
2. `do_vendor` modification to check manifest before overwriting
3. `fw patch register <file> --task T-XXX` command to add entries
4. `fw upgrade` warning when a patched file would be overwritten
5. `fw upgrade --force` to override (with backup)
