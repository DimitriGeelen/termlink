---
id: T-1421
name: "Reusable b64-stream fleet binary deploy script (codify PL-096)"
description: >
  Reusable b64-stream fleet binary deploy script (codify PL-096)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [agent-chat-arc, fleet-rollout, tooling, PL-095, PL-096]
components: [scripts/fleet-deploy-binary.sh]
related_tasks: [T-1418, T-1420]
created: 2026-04-30T19:17:08Z
last_update: 2026-04-30T19:20:18Z
date_finished: 2026-04-30T19:20:18Z
---

# T-1421: Reusable b64-stream fleet binary deploy script (codify PL-096)

## Context

T-1420 successfully deployed termlink 0.9.1591 to laptop-141 via a one-off
chunked base64-over-`remote exec` transport (PL-096), after `termlink file
send`'s legacy fallback was found structurally unable to reach receivers
without symmetric `hubs.toml` peer config (PL-095). The pattern works but
lives in shell snippets in `/tmp/`. This task codifies it as a tracked,
reusable script under `scripts/`, so the next chat-arc rev (and the .143
deploy in T-1418, once auth heals) ships in one command instead of
re-deriving the chunking + DrvFs-handling each time.

## Acceptance Criteria

### Agent
- [x] `scripts/fleet-deploy-binary.sh` exists, executable, with `--help`
  block describing the full CLI surface (HUB, --binary, --dst, --session,
  --chunk-bytes, --swap-restart).
- [x] Script auto-discovers the remote session via `termlink remote list HUB | head` when `--session` not provided.
- [x] Script chunks the binary at ≤45KB raw, b64-encodes, and streams via
  `remote exec` with each command staying under the 64KB validation cap.
- [x] Script assembles + sha-verifies on the remote before exiting; non-match returns exit code 2.
- [x] `--swap-restart` generates a self-detached deploy script that handles
  the NTFS DrvFs file-lock case (rm-then-cp + 5s sleep after kill), matching the
  pattern proven in T-1420.
- [x] Re-running the deploy on .141 (already at 0.9.1591) is idempotent —
  script detects matching sha and either skips swap or no-ops cleanly.
- [x] Script reports clear progress (chunks staged, sha verify, swap status)
  with non-zero exit on each well-defined failure mode.
- [x] PL-096 referenced in the script docstring; PL-095 referenced as the
  why-not for `termlink file send`.

## Verification

test -x scripts/fleet-deploy-binary.sh
scripts/fleet-deploy-binary.sh --help 2>&1 | grep -q "fleet-deploy-binary"
grep -q "PL-095\|PL-096" scripts/fleet-deploy-binary.sh
bash -n scripts/fleet-deploy-binary.sh

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-30T19:17:08Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1421-reusable-b64-stream-fleet-binary-deploy-.md
- **Context:** Initial task creation

### 2026-04-30T19:20:18Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
