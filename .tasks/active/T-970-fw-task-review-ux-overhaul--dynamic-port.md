---
id: T-970
name: "fw task review UX overhaul — dynamic port, browser open, push-based delivery"
description: >
  Inception: fw task review UX overhaul — dynamic port, browser open, push-based delivery

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T09:50:48Z
last_update: 2026-04-12T09:53:55Z
date_finished: null
---

# T-970: fw task review UX overhaul — dynamic port, browser open, push-based delivery

## Problem Statement

`fw task review` is the single entry point for inception decisions, but it has multiple UX failures that compound into a broken experience:

1. **Hardcoded port 3000** — Watchtower runs on :3002 (or any configured port), but `fw task review` generates URLs with :3000. Result: 404 for the human.
2. **No browser open** — The command prints a URL but doesn't open the browser. On a system with `xdg-open`/`open`, it should auto-open.
3. **Agent amnesia** — Agents keep outputting bare `fw inception decide` commands instead of calling `fw task review`. PL-007 rule added to CLAUDE.md + memory, but this feedback has been given 3+ times across sessions.
4. **Auto-invoke partial** — T-969 made `fw inception decide` auto-invoke `fw task review`, but the underlying port/browser issues make the auto-invocation produce a broken URL.
5. **No termlink inject integration** — When a persistent agent session exists, `fw task review` should inject a notification into the user's terminal.

**For whom:** Human reviewers who need to make inception GO/NO-GO decisions.
**Why now:** The compounding issues mean the human sees broken links, has to manually find the right port, and gets frustrated. This is friction that actively discourages the review process.

## Assumptions

1. Watchtower port is discoverable at runtime (PID file, config, or probing)
2. `xdg-open` / `open` is available on Linux/macOS for browser opening
3. The human's terminal is reachable via termlink inject (when a persistent session exists)
4. Consolidating these fixes into one task prevents scope drift vs. individual tasks

## Exploration Plan

1. **Spike 1: Port discovery** (15min) — How does Watchtower store its port? PID file? Config? Can we probe :3000 and :3002?
2. **Spike 2: Browser open** (10min) — Check `xdg-open` availability on .107, test auto-opening from CLI
3. **Spike 3: termlink inject integration** (15min) — When `fw task review` runs, inject notification to human's session if discoverable
4. **Spike 4: End-to-end test** (10min) — Full flow: agent calls `fw task review` → correct port → browser opens → QR renders → human decides

## Technical Constraints

- Server may run on different ports across environments (.107 uses 3002, others may use 3000)
- Headless servers (CI, remote SSH) have no browser — `xdg-open` would fail silently
- termlink inject requires a named/discoverable session
- QR code rendering requires terminal width >= 31 columns

## Scope Fence

**IN scope:**
- Dynamic port detection for Watchtower URLs in `fw task review`
- Auto-open browser when available (non-blocking, fail-silent)
- termlink inject notification to human session
- CLAUDE.md behavioral rule (PL-007, already done)
- `fw inception decide` auto-invokes `fw task review` (T-969, already done)

**OUT of scope:**
- Watchtower redesign (separate task)
- MCP-based approval flow (separate task)
- Multi-user notification (future)

## Acceptance Criteria

### Agent
- [x] Port detection probes for project-specific Watchtower (review.sh)
- [x] Browser auto-opens via xdg-open/open (review.sh)
- [x] fw inception decide auto-invokes fw task review when marker missing (inception.sh)
- [x] PL-007 codified in CLAUDE.md
- [x] Pickup P-019 sent to framework agent

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- Port discovery is feasible without adding new config files
- Browser open works on at least Linux (xdg-open) and macOS (open)
- Fixes are small enough to implement and send as pickup to framework agent

**NO-GO if:**
- Port discovery requires invasive changes to Watchtower startup
- Browser open creates security/UX issues in headless environments
- Scope is too large for a single implementation pass

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T09:52:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
