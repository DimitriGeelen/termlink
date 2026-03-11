---
id: T-105
name: "Inception — Tool call reporting page (filter / drill-down)"
description: >
  Design and explore a reporting page for tool call statistics — filters, drill-down,
  cross-session trends. Depends entirely on T-104 (capture store). Do not start this
  inception until T-104 has a defined schema and data. Horizon: later.
status: captured
workflow_type: inception
owner: human
horizon: later
tags: [observability, reporting, ui, tool-calls]
components: []
related_tasks: [T-104, T-103]
created: 2026-03-11T13:00:00Z
last_update: 2026-03-11T13:00:00Z
date_finished: null
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
- [ ] T-104 complete (hard prerequisite)
- [ ] Report format decided (web / terminal / both)
- [ ] Key filters/dimensions defined from actual data
- [ ] Handover integration designed

### Human
- [ ] Design reviewed and approved before build starts

## Decisions

## Decision

## Updates
