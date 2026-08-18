---
id: T-1107
name: "Wire net test into Watchtower /fleet page — per-hub diagnose button"
description: >
  Add /api/fleet/net-test endpoint and a Diagnose button on each hub card in the /fleet
  page. Fetches per-layer (TCP/TLS/AUTH/PING) breakdown from `termlink net test --json`
  and renders inline. Closes the operator loop from detection (fleet status) to diagnosis
  (net test) without terminal switching.

status: work-completed
workflow_type: build
owner: claude
horizon:
tags: []
components: []
related_tasks: [T-1103, T-1106]
created: 2026-04-17T15:50:44Z
last_update: '2026-08-18T18:58:44Z'
date_finished: 2026-04-17T16:29:37Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:45Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=0 
      (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:44Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-1107: Wire net test into Watchtower /fleet page — per-hub diagnose button

## Context

T-1106 shipped `termlink net test` as a CLI tool. T-1103 shipped the `/fleet` page.
This task bridges them: when a hub shows as down/degraded on /fleet, the operator
can click "Diagnose" and see per-layer results inline, without opening a terminal.

## Acceptance Criteria

### Agent
- [x] `/api/fleet/net-test?profile=<name>` endpoint runs `termlink net test --json --profile <name>` and returns parsed JSON
- [x] `/api/fleet/net-test` (no profile) returns results for all hubs
- [x] Hub cards on /fleet have a "net-test" button that fetches and expands layer results
- [x] Per-layer rows show TCP/TLS/AUTH/PING with pass/fail + latency
- [x] Watchtower Python process runs cleanly after changes (no import errors)
- [x] Profile name sanitized against shell-metacharacter injection (alnum/dash/underscore only)
- [x] Curl test: endpoint returns valid JSON with expected fields

## Verification

curl -sf 'http://localhost:3000/api/fleet/net-test?profile=local-test' | python3 -c "import sys,json; d=json.load(sys.stdin); assert 'hubs' in d and len(d['hubs']) == 1, d"
curl -sf 'http://localhost:3000/fleet' | grep -q "net-test"

## Decisions

## Updates

### 2026-04-17T15:50:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1107-add-termlink-net-test--layered-hub-conne.md
- **Context:** Initial task creation (renamed from auto-generated title)

### 2026-04-17T16:29:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
