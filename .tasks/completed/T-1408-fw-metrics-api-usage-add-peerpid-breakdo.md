---
id: T-1408
name: "fw metrics api-usage: add peer_pid breakdown for legacy callers (T-1407 follow-up)"
description: >
  fw metrics api-usage: add peer_pid breakdown for legacy callers (T-1407 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T20:45:30Z
last_update: 2026-04-29T20:48:15Z
date_finished: 2026-04-29T20:48:15Z
---

# T-1408: fw metrics api-usage: add peer_pid breakdown for legacy callers (T-1407 follow-up)

## Context

T-1407 added `peer_pid` to the rpc-audit JSONL schema. The
`fw metrics api-usage` agent (in `.agentic-framework/agents/metrics/api-usage.sh`)
already aggregates legacy callers by `(method, from)`; this task adds a
parallel `(method, peer_pid)` breakdown so anonymous callers (with no `from`
field) become identifiable by their PID. Closes the diagnostic loop opened
by the T-1166 mystery-poller incident: instead of inferring the source
from cadence patterns, the operator gets a printable PID list and
`ps -p <pid>` finishes the job.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/metrics/api-usage.sh` parses the
      optional `peer_pid` field from each JSONL line.
- [x] Trend mode and single-window mode both print a
      `Legacy callers by PID (last Nd)` section after the existing
      `Legacy callers` table — only printed when at least one entry has
      a peer_pid.
- [x] JSON mode emits `legacy_callers_by_pid` (mirroring
      `legacy_callers` shape).
- [x] Live verification on the local hub: after the T-1407 audit-log
      contains the `event.broadcast` entry with `peer_pid:723266`,
      running `fw metrics api-usage` prints
      `Legacy callers by PID (last 60d):  1  event.broadcast  pid=723266`
      and `--json` returns
      `legacy_callers_by_pid: [{'method': 'event.broadcast', 'peer_pid': 723266, 'count': 1}]`.

## Verification

grep -q "peer_pid" .agentic-framework/agents/metrics/api-usage.sh
out=$(.agentic-framework/bin/fw metrics api-usage 2>&1 || true); echo "$out" | grep -q "Legacy callers by PID"
js=$(.agentic-framework/bin/fw metrics api-usage --json 2>&1 || true); echo "$js" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); assert 'legacy_callers_by_pid' in d, 'legacy_callers_by_pid missing'"

## Updates

### 2026-04-29T20:45:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1408-fw-metrics-api-usage-add-peerpid-breakdo.md
- **Context:** Initial task creation

### 2026-04-29T20:48:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
