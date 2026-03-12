---
id: T-105
name: "Inception — Tool call reporting page (filter / drill-down)"
description: >
  Design and explore a reporting page for tool call statistics — filters, drill-down,
  cross-session trends. Depends entirely on T-104 (capture store). Do not start this
  inception until T-104 has a defined schema and data. Horizon: later.
status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [observability, reporting, ui, tool-calls]
components: []
related_tasks: [T-104, T-103]
created: 2026-03-11T13:00:00Z
last_update: 2026-03-12T17:04:04Z
date_finished: 2026-03-12T07:59:21Z
---

# T-105: Inception — Tool Call Reporting Page

## Problem Statement

Once T-104 (capture store) exists, the data needs to be interpretable by humans.
A page with filter and drill-down capability makes the data accessible and actionable.

**Hard dependency:** This inception cannot start until T-104 has a defined schema
and at least one session's worth of data captured.

## What We Know So Far (from dialogue)

- Wants: filter capability, drill-down, cross-session view
- Format: page (web UI or terminal report — to be decided)
- Philosophy: report shape emerges from the data, not designed upfront
- Session handover integration: summary numbers (total calls, error count) should
  appear in the handover document automatically

## Questions to Explore (when T-104 is done)

1. **Delivery format:**
   - Web page (extends existing `fw serve` web UI)
   - Terminal report (`fw tool-stats` command with table output)
   - Both — terminal for quick checks, web for drill-down

2. **What filters/dimensions are most useful?**
   - By session, by task, by tool type, by error/success, by time range
   - "Show me all Bash errors in the last 7 days" — is that the right query?

3. **Handover integration:**
   - Add a `## Tool Call Summary` section to handover template
   - Numbers: total calls this session, error count, most-used tool, error rate
   - Trend: higher/lower than previous session average

4. **Drill-down:**
   - Click session → see all tool calls in that session
   - Click error → see full error content + task context

## Scope Fence

**IN:** Design the reporting interface once data exists
**OUT:** Any implementation before T-104 is complete

## Acceptance Criteria

### Agent
- [x] T-104 complete — GO decision, T-111 extractor built, T-112 hook integrated
- [x] Report format decided — terminal (analyze-errors.py for errors, tool-stats.py for full stats)
- [x] Key filters/dimensions defined — session, tool, error/success (from T-113 work)
- [x] Handover integration designed — add tool call summary to handover output
- [x] GO/NO-GO framed

### Human
- [x] Design reviewed and approved before build starts

## Decisions

**Decision**: GO — T-104 data layer exists, T-113 error report delivered. Build tool-stats.py for full reporting and integrate summary into handover agent.

**Date**: 2026-03-12

## Updates

### 2026-03-12T07:18:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** owner: human → agent
- **Change:** horizon: later → now

### 2026-03-12T07:59:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
