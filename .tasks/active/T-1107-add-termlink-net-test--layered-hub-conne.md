---
id: T-1107
name: "Wire net test into Watchtower /fleet page — per-hub diagnose button"
description: >
  Add /api/fleet/net-test endpoint and a Diagnose button on each hub card in the /fleet
  page. Fetches per-layer (TCP/TLS/AUTH/PING) breakdown from `termlink net test --json`
  and renders inline. Closes the operator loop from detection (fleet status) to diagnosis
  (net test) without terminal switching.

status: started-work
workflow_type: build
owner: claude
horizon: now
tags: []
components: []
related_tasks: [T-1103, T-1106]
created: 2026-04-17T15:50:44Z
last_update: 2026-04-17T16:10:00Z
date_finished: null
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
