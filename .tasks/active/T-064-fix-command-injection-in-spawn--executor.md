---
id: T-064
name: "Fix command injection in spawn — executor.rs input validation"
description: >
  Security: executor.rs passes user-controlled strings to sh -c with no escaping. Fix with input validation/escaping. Ref: security reflection agent finding.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-10T08:44:07Z
last_update: 2026-03-10T12:48:02Z
date_finished: null
---

# T-064: Fix command injection in spawn — executor.rs input validation

## Context

Security vulnerability found by reflection fleet security agent. executor.rs:21-22 passes user-controlled strings to `sh -c` with no escaping. See [docs/reports/reflection-result-security.md]. Related: T-008.

## Acceptance Criteria

### Agent
- [x] `cmd_spawn` escapes user command args via `shell_escape()` before embedding in shell script
- [x] `executor::execute()` validates command: rejects empty, null bytes, oversized (>64KB)
- [x] `ExecError::Validation` variant added for input validation failures
- [x] Validation tests pass: empty, null bytes, oversized, normal commands
- [x] All existing executor tests still pass (no regression)
- [x] Full CLI builds successfully

## Verification

# Spawn args are escaped
grep -q "shell_escape(arg)" crates/termlink-cli/src/main.rs
# Executor validates commands
grep -q "validate_command" crates/termlink-session/src/executor.rs
# Validation rejects null bytes
grep -q "null bytes" crates/termlink-session/src/executor.rs
# ExecError::Validation exists
grep -q "Validation" crates/termlink-session/src/executor.rs
# Tests pass
/Users/dimidev32/.cargo/bin/cargo test -p termlink-session -- executor --quiet

## Decisions

### 2026-03-10 — Scope of executor.rs fix
- **Chose:** Input validation (empty, null bytes, length) + security doc comment, NOT command sanitization
- **Why:** `command.execute` is designed for shell commands (pipes, redirects, etc.). Sanitizing metacharacters would break the API. Real fix for untrusted callers is auth (T-008/G-002).
- **Rejected:** Command allowlist (too restrictive), full metacharacter escaping (breaks legitimate use), disabling `sh -c` (breaking change)

## Updates

### 2026-03-10T08:44:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-064-fix-command-injection-in-spawn--executor.md
- **Context:** Initial task creation

### 2026-03-10T12:48:02Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
