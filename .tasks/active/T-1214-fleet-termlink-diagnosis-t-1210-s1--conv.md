---
id: T-1214
name: "Fleet termlink diagnosis (T-1210 S1) + converge-vs-federate decision"
description: >
  Execute T-1210 S1: probe every reachable peer for termlink binary lineage (version, subcommand list, mtime, source path if discoverable). Classify as same-lineage-older / same-lineage-newer / forked / stranger. Preliminary finding from T-1210 probe: .122 has 0.9.844 install with no channel subcommand and no /opt/termlink source → stranger lineage. After S1 complete, produce converge-vs-federate recommendation. Pilot S2 (unified install) or S3 (capability probe) depending on direction. See .tasks/completed/T-1210-fleet-termlink-version-divergence--unifi.md.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [fleet, install, capability-probe]
components: []
related_tasks: [T-1210, T-1165, T-1168]
created: 2026-04-24T10:05:17Z
last_update: 2026-04-24T10:25:40Z
date_finished: null
---

# T-1214: Fleet termlink diagnosis (T-1210 S1) + converge-vs-federate decision

## Context

Execute T-1210 Spike 1 (fleet lineage probe). Classify each reachable termlink peer
as **same-lineage / forked / stranger** based on version, subcommand surface, binary
mtime, and source path. Preliminary T-1210 evidence: `.122` reports termlink 0.9.844
with no `channel` subcommand and no `/opt/termlink` source → stranger-lineage.

Output is a capability matrix and a converge-vs-federate recommendation written to
`docs/reports/T-1214-fleet-diagnosis.md`.

## Acceptance Criteria

### Agent
- [x] Probe local (.109) termlink: version, `channel -h` exit, binary path + mtime,
      source commit if in repo — **0.9.206 install, no channel, source 0.9.398 (192 commits ahead)**
- [x] Probe ring20-management (.122) via hub: version, `channel -h` exit, binary path + mtime
      — **remote_exec unsupported (Method not found); T-1210 prior evidence recorded: 0.9.844, no channel, no source**
- [x] Probe ring20-dashboard (.121) via hub: version, `channel -h` exit, binary path + mtime
      — **hub reachable (42 ms), 0 sessions registered → no agent to probe; hub-only federation applies**
- [x] Classify each peer (same-lineage / forked / stranger) with evidence
- [x] Write `docs/reports/T-1214-fleet-diagnosis.md` containing the capability matrix
      and converge-vs-federate recommendation (GO one path, defer the other)
- [ ] Commit probe findings under T-1214

## Verification

# Report artifact exists
test -f docs/reports/T-1214-fleet-diagnosis.md
# Report names all three peers
grep -q "ring20-management" docs/reports/T-1214-fleet-diagnosis.md
grep -q "ring20-dashboard" docs/reports/T-1214-fleet-diagnosis.md
# Recommendation section present
grep -q "Recommendation" docs/reports/T-1214-fleet-diagnosis.md

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

### 2026-04-24T10:05:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1214-fleet-termlink-diagnosis-t-1210-s1--conv.md
- **Context:** Initial task creation

### 2026-04-24T10:25:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-24T10:30Z — S1 probe complete [agent]
- **Findings summary:**
  - local (.109): 0.9.206 install, no `channel`, source at 0.9.398 (dc111330) — same-lineage but **install 192 commits stale**
  - .122 (ring20-management): 0.9.844, no `channel`, no `/opt/termlink`, hub lacks `command.exec` → **stranger lineage**
  - .121 (ring20-dashboard): hub healthy (G-013 appears healed), 0 sessions → **unprobed agent surface**
- **Decision:** GO Option B (federate at JSON-RPC layer), DEFER Option A (unified install).
- **Rationale:** Strangers are a given; we cannot safely converge .122; .121 has no agent to push to. Building T-1165 against the JSON-RPC methods + capability probe is drift-tolerant and aligns with Directive 4 (portability).
- **Artifact:** docs/reports/T-1214-fleet-diagnosis.md
- **Immediate follow-up:** rebuild local termlink from source so `channel` subcommand is present where T-1155 is being developed.
- **Deferred follow-ups:** hub.capabilities method + client cache (capture new task); update T-1165 description to reference Option B.
