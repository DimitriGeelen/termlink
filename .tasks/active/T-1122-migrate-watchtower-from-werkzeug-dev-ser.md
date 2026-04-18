---
id: T-1122
name: "Migrate Watchtower from Werkzeug dev server to production-grade WSGI"
description: >
  Inception: Migrate Watchtower from Werkzeug dev server to production-grade WSGI

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T09:39:11Z
last_update: 2026-04-18T09:40:32Z
date_finished: null
---

# T-1122: Migrate Watchtower from Werkzeug dev server to production-grade WSGI

## Problem Statement

Watchtower currently runs under the Werkzeug development server (`python3 -m web.app`), which prints on every startup:

> WARNING: This is a development server. Do not use it in a production deployment. Use a production WSGI server instead.

For whom: operators who run Watchtower long-lived on this and other hosts (ring20-management, ring20-dashboard, etc.) — effectively a production deployment.
Why now: the dev server lacks proper concurrency, graceful reload, signal handling, HTTPS, request-size limits, and is single-threaded by default. As more agents/operators consume Watchtower (fleet status polling, LLM-routing pages, htmx-driven widgets), the limits matter. We've also seen restart races during this session that a real WSGI server with a process manager (gunicorn/uwsgi/waitress + systemd) would handle cleanly.

## Assumptions

- A1: Watchtower is treated as a long-lived service in practice (not just dev)
- A2: A drop-in WSGI server (gunicorn or waitress) can serve `web.app:app` with no code changes
- A3: HTTPS is desirable for cross-host fleet UIs but not blocking for v1
- A4: Process management belongs in systemd, not the WSGI server
- A5: The framework's start command (e.g. `fw watchtower start`) can be updated to invoke the new server transparently

## Exploration Plan

1. **Spike — gunicorn vs waitress vs hypercorn (timebox 30m):** Compare on Linux + macOS support, htmx/SSE compatibility, startup time, dependency footprint
2. **Compatibility check (timebox 15m):** Does the existing app cleanly expose a WSGI callable? Any Flask globals or blueprint init that breaks under multi-worker?
3. **Hooks/signals (timebox 15m):** Will the existing PreToolUse/PostToolUse framework hooks that spawn Python conflict with multi-worker WSGI?
4. **Startup ergonomics (timebox 15m):** What does `fw watchtower start/stop/status` look like with the new server?

## Technical Constraints

- Must continue to serve `web.app:app` (Flask blueprint app) without code changes beyond the entrypoint
- Must work on Linux (primary) and macOS (consumer installs)
- Must keep htmx + SSE (`htmx-ext-sse.js`) working — long-lived connections require a server that supports streaming
- Must integrate with existing framework hooks (PreToolUse, PostToolUse) without subprocess collisions
- Single-host first; multi-host TLS/reverse-proxy is out of scope for v1

## Scope Fence

**IN scope:**
- Switching the WSGI server (gunicorn / waitress / hypercorn / etc.) for local single-node deploys
- Updating `fw watchtower` startup/shutdown commands
- Documenting the new run model in CLAUDE.md / FRAMEWORK.md

**OUT of scope:**
- Reverse proxy (nginx/caddy) configuration
- HTTPS / TLS termination
- Multi-host load balancing
- Migrating off Flask (e.g. to FastAPI) — large rewrite, separate inception

## Acceptance Criteria

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- A WSGI server is identified that supports htmx + SSE without code rewrites
- Change is bounded to entrypoint (`fw watchtower start`) and one new dependency
- Migration is reversible (can fall back to Werkzeug for dev)

**NO-GO if:**
- All viable WSGI servers require non-trivial Flask app refactors
- Multi-worker mode breaks the framework's hook/state model
- Operator value (no warning, better concurrency) doesn't justify dependency churn for a single-node tool

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

### 2026-04-18T09:40:32Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
