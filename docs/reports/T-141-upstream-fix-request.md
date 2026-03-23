# Upstream Fix Request: Pre-push hook isolation bug

**From:** TermLink project (010-termlink)
**To:** Agentic Engineering Framework
**Task:** T-141
**Priority:** Medium (blocks audit D2 for consumer projects)

## Problem

The pre-push hook's audit script runs against the **framework repo** instead of the **consumer project** when invoked via `hooks.sh`. The `PROJECT_ROOT` env var is not passed through to the audit subprocess.

## Root Cause

In `agents/git/lib/hooks.sh` line ~328, the audit script is invoked as:
```bash
"$AUDIT_SCRIPT"
```

But `PROJECT_ROOT` is set earlier in the hook and not exported or passed. The audit script defaults to its own repo location.

## Fix (one line)

Change line ~328 in `agents/git/lib/hooks.sh` from:
```bash
"$AUDIT_SCRIPT"
```
To:
```bash
PROJECT_ROOT="$PROJECT_ROOT" "$AUDIT_SCRIPT"
```

## Verification

After fix:
1. `fw git install-hooks` (reinstall hooks in consumer project)
2. `git push` from consumer project
3. Audit header should show `Project: /path/to/consumer-project` (not framework path)

## Additional Issues Found

### `declare -A` on macOS bash 3.2 (T-160)
- `audit.sh` uses `declare -A` (associative arrays) which requires bash 4+
- macOS ships bash 3.2; `/usr/local/bin/bash` is 5.x but shebangs use `#!/bin/bash`
- Audit crashes silently on macOS, causing pre-push hook to exit with syntax error
- Fix: Use `#!/usr/bin/env bash` or `#!/usr/local/bin/bash`, or replace `declare -A` with compat patterns

### D2 false-positive on blocked tasks
- D2 counts tasks waiting >72h as FAIL (exit code 2 = push blocked)
- Tasks blocked on upstream fixes (like T-141 itself) can't be resolved by the consumer
- Suggestion: D2 should be WARN (exit code 1) not FAIL, or exclude tasks with `horizon: later`
