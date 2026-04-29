---
id: T-1410
name: "Group api-usage peer_addr breakdown by IP not IP:port (rollup)"
description: >
  Group api-usage peer_addr breakdown by IP not IP:port (rollup)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T21:53:57Z
last_update: 2026-04-29T21:55:42Z
date_finished: 2026-04-29T21:55:42Z
---

# T-1410: Group api-usage peer_addr breakdown by IP not IP:port (rollup)

## Context

T-1409 added `peer_addr` ("ip:port") to rpc-audit and api-usage agent. But each TCP connection draws a new ephemeral source port, so the "Legacy callers by addr" breakdown fragments into N rows of count=1 — one per connection. After 24h of an 11x/min poller from one host that's ~16K rows, all displaying the same IP. The operator's question is "which host?", not "which connection?". Strip port, group by IP-only. Preserve raw `peer_addr` in the audit log (port can still be useful for forensics) — only the agent-side rollup changes.

## Acceptance Criteria

### Agent
- [x] `.agentic-framework/agents/metrics/api-usage.sh` — strip port and group `legacy_addrs` keys by IP only (`addr.rsplit(':', 1)[0]` to handle IPv6 `[::1]:9100`-style)
- [x] Heading renamed to "Legacy callers by IP (last Nd):" in trend + single-window text output
- [x] JSON `legacy_callers_by_addr` field renamed to `legacy_callers_by_ip` and entries use key `peer_ip` (additive — old field would only break consumers; the JSON shape is documented as still stabilizing post-T-1409)
- [x] Live verify: 192.168.10.143 collapses to one row with cumulative count from all ephemeral ports
- [x] Mirrored upstream to /opt/999-Agentic-Engineering-Framework

## Verification

out=$(.agentic-framework/bin/fw metrics api-usage --last-Nd 1 2>&1 || true); echo "$out" | grep -q "Legacy callers by IP"
out=$(.agentic-framework/bin/fw metrics api-usage --last-Nd 1 --json 2>&1 || true); echo "$out" | python3 -c "import sys,json; d=json.loads(sys.stdin.read()); assert 'legacy_callers_by_ip' in d, 'missing legacy_callers_by_ip'"

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-04-29T21:53:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1410-group-api-usage-peeraddr-breakdown-by-ip.md
- **Context:** Initial task creation

### 2026-04-29T21:55:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
