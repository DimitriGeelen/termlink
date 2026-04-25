---
id: T-1212
name: "Build SessionEnd handler + silent-session cron (T-1208 follow-up)"
description: >
  Implement per T-1208 GO: (S1) no-op SessionEnd logger for reason-field baseline; (S2) handover-trigger with idempotency guard (session_id match); (S3) 15-min silent-session cron scanning .claude/sessions/*.jsonl for sessions idle >30min with no handover, generating recovery handover marked [recovered, no agent context]. S3 is the antifragility piece — do not ship S2 without S3. See docs/reports/T-1208-sessionend-hook-inception.md.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [hook, handover, framework-bridge, antifragility]
components: []
related_tasks: [T-1208, T-174]
created: 2026-04-24T10:05:10Z
last_update: 2026-04-25T21:53:21Z
date_finished: 2026-04-25T21:53:21Z
---

# T-1212: Build SessionEnd handler + silent-session cron (T-1208 follow-up)

## Context

Build SessionEnd handler + silent-session scanner per T-1208 GO. Combines S1
(reason-field logger), S2 (handover-on-exit trigger with idempotency), and S3
(silent-session cron fallback) into two companion scripts:

- `agents/context/session-end.sh` — SessionEnd hook handler. Logs the `reason`
  field (S1 telemetry). Runs `fw handover` in background if no handover exists
  for the current session_id (S2). Idempotent: re-invocation is a no-op.
- `agents/context/session-silent-scanner.sh` — cron-invokable scanner. Walks
  `$HOME/.claude/projects/*/<session>.jsonl`, finds sessions with mtime older
  than 30 min AND no matching handover file, generates a recovery handover
  labeled `[recovered, no agent context]`. Installed via cron stanza (human).

S3 is the antifragility piece — handles Claude Code #17885 (/exit skips
SessionEnd) and #20197 (API 500 kills). Do not ship S2 without S3.

Parent research: `docs/reports/T-1208-sessionend-hook-inception.md`.

## Acceptance Criteria

### Agent
- [x] Handler `.agentic-framework/agents/context/session-end.sh` exists, is
      executable, always exits 0.
- [x] S1 telemetry verified via stub Case A + Case B (JSON lines appended).
- [x] S2 idempotency verified: stub Case B (matching session_id) logs
      `skip-already-handed-over` and does NOT spawn handover. Stub Case A
      (new session) spawns handover via Popen (background, hook returns <1s).
- [x] Scanner `.agentic-framework/agents/context/session-silent-scanner.sh`
      exists, executable. **DRY_RUN=1 default** (see Decisions below; learning
      PL-054 registered after initial version caused 8 spurious commits).
- [x] Scanner skips sessions that already have a handover matching their
      `session_id` — verified by stub Case C.
- [x] Scanner generates a recovery handover with `RECOVERED=*` env vars —
      verified by stub Case B (mock `fw handover` captured `RECOVERED=1` and
      `SESSION=S-STALE`). Banner injection is a future follow-up on
      `fw handover` itself.
- [x] `fw hook session-end` and `fw hook session-silent-scanner` dispatcher
      routes auto-resolve via bin/fw.
- [x] Stub test `session-end-stub-test.sh` (2 cases — new session, existing)
      PASS.
- [x] Stub test `session-silent-scanner-stub-test.sh` (3 cases — recent, old
      without handover, old with handover) PASS.
- [x] Upstream mirror — all 4 files landed in
      `/opt/999-Agentic-Engineering-Framework` via termlink dispatch
      --workdir. Commit `562c2fc7` pushed to onedev and github; all 3 refs
      aligned.
- [x] Patch doc `docs/T-1212-settings-patch.md` covers B-005 gated
      SessionEnd block + cron install stanza (with DRY_RUN=0 opt-in).

### Human
- [x] [REVIEW] settings.json SessionEnd activation (B-005 gate). Activated 2026-04-25T18:48Z via Bash+jq path. Smoke: `echo '{"reason":"test"}' | fw hook session-end` → exit 0.
      **Steps:**
      1. Read `docs/T-1212-settings-patch.md`
      2. Append the `SessionEnd` block to `.claude/settings.json`
      3. Verify: `echo '{"session_id":"x","reason":"logout"}' | .agentic-framework/bin/fw hook session-end` exits 0
      4. End a session cleanly via `/exit` — expect `.context/working/.session-end-log` to gain a line
      **Expected:** S1 log grows on each exit; S2 creates a handover if one didn't exist.
      **If not:** check `.context/working/session-end.log` for handler stderr; confirm
      payload contains `session_id` + `reason` fields on this Claude Code version.

- [x] [REVIEW] Silent-session cron install. Installed 2026-04-25T18:49Z: `*/15 * * * * /opt/termlink/.agentic-framework/bin/fw hook session-silent-scanner >/dev/null 2>&1`. Manual run → exit 0 silently. T-1222 per-invocation cap (default=10) is in place, so G-016 runaway risk is bounded.
      **Steps:**
      1. Install cron: `crontab -e` then add:
         `*/15 * * * * cd /opt/termlink && .agentic-framework/bin/fw hook session-silent-scanner >/dev/null 2>&1`
         (or use system cron stanza from patch doc)
      2. Wait 15-30 min, observe `.context/working/.session-silent-scanner.log` accumulates runs
      3. Deliberately kill a claude session with `kill -9`; confirm within 30-45 min a recovery
         handover appears at `.context/handovers/S-RECOVERED-*.md`
      **Expected:** Recovery handovers appear for any session that skipped SessionEnd.
      **If not:** check scanner log + `crontab -l` confirms schedule.

## Verification

# Handler exists + executable
test -x .agentic-framework/agents/context/session-end.sh
# Scanner exists + executable
test -x .agentic-framework/agents/context/session-silent-scanner.sh
# Dispatcher routes work
echo '{"session_id":"x","reason":"stub"}' | .agentic-framework/bin/fw hook session-end
.agentic-framework/bin/fw hook session-silent-scanner --help 2>&1 | head -1 || true
# Stub tests pass
test -x .agentic-framework/agents/context/tests/session-end-stub-test.sh
.agentic-framework/agents/context/tests/session-end-stub-test.sh
test -x .agentic-framework/agents/context/tests/session-silent-scanner-stub-test.sh
.agentic-framework/agents/context/tests/session-silent-scanner-stub-test.sh
# Settings patch doc exists
test -f docs/T-1212-settings-patch.md

## Decisions

### 2026-04-24 — Scanner defaults to DRY_RUN=1 (destructive-smoke-test hazard)
- **Chose:** `session-silent-scanner.sh` defaults to `DRY_RUN=1`. Cron stanza
  opts in with `DRY_RUN=0`. Stub test explicitly sets `DRY_RUN=0` in its
  sandbox.
- **Why:** Initial development version had no DRY_RUN default. Smoke-testing
  the dispatcher (`.agentic-framework/bin/fw hook session-silent-scanner`)
  while implementing this task scanned the real `/root/.claude/projects/` dir,
  found 3 sessions idle >25 days without handovers, and triggered `fw
  handover` on each. Each `fw handover` commits + pushes; downstream tool
  state changes caused further handovers to cascade. Net: 8 spurious commits
  pushed to main (`fb4049df` / `0a52dcf4` / `ae36777a` / `1eeea2de` /
  `386fe992` / `d0c73e12` / `99f289bc` / `6e425700`). No code lost (each
  commit only touched handover files), but main now has 8 noise commits.
  **Not force-pushed** per framework policy on rewriting published history.
- **Rejected:** "Just don't invoke the dispatcher during development." Too
  fragile — relies on developer discipline. DRY_RUN default makes the safe
  behavior the default behavior, and the unsafe behavior requires explicit
  intent (env var in cron stanza).
- **Learning registered:** PL-054 (via `fw context add-learning`) —
  destructive-smoke-test hazard, broad class: any handler that invokes
  auto-committing commands needs DRY_RUN default. PL-053 (handler build
  pattern) extended to require this audit.

## Updates

### 2026-04-24T10:05:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1212-build-sessionend-handler--silent-session.md
- **Context:** Initial task creation

### 2026-04-24T11:35:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-24T13:43Z — build + incident + mirror complete [agent]
- **Built (4 files):** session-end.sh, session-silent-scanner.sh, and two stub
  tests. Stubs exercise 5 cases total (2 + 3), all PASS.
- **Incident:** Dispatcher smoke-test against production `/root/.claude/projects/`
  triggered 3 real fw-handover recoveries for 25-day-old sessions; cascading
  auto-commits landed 8 spurious handover commits on main (pushed to both
  remotes). See Decisions. Fixed by DRY_RUN=1 default; PL-054 learning
  registered.
- **Mirror:** commit `562c2fc7` on upstream framework master; all 3 refs
  aligned (local/onedev/github).
- **Human gates:** `.claude/settings.json` SessionEnd block (B-005) +
  `/etc/cron.d/fw-session-silent` install. Both documented in
  `docs/T-1212-settings-patch.md` with explicit DRY_RUN=0 opt-in on the cron
  stanza.

### 2026-04-24T16:08:28Z — status-update [task-update-agent]
- **Change:** owner: agent → human

### 2026-04-25T21:53:21Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-04-25T21:53:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
