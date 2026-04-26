---
id: T-1122
name: "Migrate Watchtower from Werkzeug dev server to production-grade WSGI"
description: >
  Inception: Migrate Watchtower from Werkzeug dev server to production-grade WSGI

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-18T09:39:11Z
last_update: 2026-04-26T10:56:45Z
date_finished: 2026-04-26T10:56:45Z
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
- [x] Problem statement validated — Watchtower runs `socketio.run(app, ..., allow_unsafe_werkzeug=True)` (`web/app.py:420`), under Werkzeug. Confirmed long-lived service in practice.
- [x] Assumptions tested — A1 confirmed; A2 NOT confirmed (Flask-SocketIO with `async_mode=threading` constrains WSGI choice to single-worker servers); A3/A4 confirmed; A5 confirmed (already wrapped via lib/watchtower.sh).
- [x] Recommendation written with rationale — see ## Recommendation

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings


**Agent evidence (auto-batch 2026-04-22, G-008 remediation, inception-recommendation, t-1122):** Research artifact: `docs/reports/T-1122-watchtower-wsgi-migration-recommendation.md`. **Recommendation: DEFER on WSGI swap; GO on systemd wrapping.** Root cause of restart races is process management, not WSGI. Werkzeug warning is aesthetic on single-host LAN tool. Spike matrix evaluated waitress/gunicorn/hypercorn — gunicorn would race .tool-counter under multi-worker. Companion artifact `T-1122-watchtower-production-server-inception.md` holds full spike evidence. Review-ready.
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

**Recommendation:** DEFER on WSGI swap; GO on systemd wrapping (the actual root cause).

**Rationale:** Re-reading the problem statement, the failure mode that matters ("restart races during this session") is a process-management problem, not a WSGI-server problem. Swapping Werkzeug for gunicorn does not fix restart races; systemd does. The Werkzeug warning is aesthetic on a single-host LAN tool. Adding gunicorn + gevent + flask-socketio_websocket adds dependency surface without proportional benefit.

## Findings

**Spike 1 — gunicorn vs waitress vs hypercorn:**
- **waitress**: pure-Python, cross-platform, threaded. **Cannot serve websockets** — would force Flask-SocketIO into long-poll fallback. Acceptable for HTTP-only Watchtower, breaks the web terminal (T-964) UX.
- **gunicorn**: Linux-only. Standard production for Flask-SocketIO requires `--worker-class gevent` + gevent + gevent-websocket. Adds ~3 dependencies. For multi-worker, **the framework's `_client_sessions` dict and `.context/working/.tool-counter` would race across workers**. So in practice: `--workers 1 --threads N` — which gives no concurrency benefit over Werkzeug's threaded mode.
- **hypercorn**: ASGI; Flask supports ASGI via `asgiref` but Flask-SocketIO behavior changes; not a drop-in.
- **Status quo (Werkzeug + `socketio.run(allow_unsafe_werkzeug=True)`)**: Already serves websockets via socketio's threading mode. Single warning at startup. Otherwise functional for the actual workload.

**Spike 2 — Compatibility:**
- App exposes WSGI callable (`web.app:app`) — confirmed at `web/app.py:376`.
- `SocketIO(app, async_mode="threading")` (`web/app.py:159`) is single-process by design. Multi-worker would break (a) websocket session affinity, (b) framework hook counters, (c) in-process `_client_sessions` dict.

**Spike 3 — Hooks/signals:**
- Multi-worker WSGI WOULD break the framework's PreToolUse/PostToolUse counters and the `.tool-counter` file (race-prone). Single-worker WSGI is the only safe option, which negates the main concurrency argument for migration.

**Spike 4 — Startup ergonomics:**
- The actual restart-race problem is solved by systemd: `Restart=on-failure`, `Type=notify` for clean handoff, `KillMode=mixed` for graceful shutdown. None of that requires a different WSGI server.

## Proposed follow-up tasks (post-DEFER on WSGI, GO on systemd)

1. **[framework, S]** Ship a `watchtower.service` systemd unit template under `agents/monitor/` (or similar) that operators can install with one command. Type=notify if practical, otherwise Type=simple with PIDFile. `ExecStart=python3 -m web.app`. Restart=on-failure, RestartSec=2.
2. **[framework, XS]** `fw watchtower start` learns to detect a systemd unit and prefer `systemctl --user start watchtower` over direct spawn when installed.
3. **[framework, XS]** Suppress the Werkzeug warning in production-mode startup OR document explicitly in CLAUDE.md that on a single-host LAN tool the warning is acceptable. (The warning is `flask.cli`-emitted; can be silenced via env or filter.)
4. **[deferred — revisit if]** Watchtower is exposed across LAN with auth, OR the web terminal sees real production load, OR multi-host federation lands. Then re-open this inception with the changed constraints.

**The mistake to avoid:** swapping the WSGI server is a tempting "production-ize" move that doesn't address the actual symptom (restart races) and adds dependency surface. Fix the cause, not the smell.

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

**Decision**: GO

**Rationale**: Re-reading the problem statement, the failure mode that matters ("restart races during this session") is a process-management problem, not a WSGI-server problem. Swapping Werkzeug for gunicorn does not fix restart races; systemd does. The Werkzeug warning is aesthetic on a single-host LAN tool. Adding gunicorn + gevent + flask-socketio_websocket adds dependency surface without proportional benefit.

**Date**: 2026-04-26T10:56:45Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

**Research artifacts:**
- `docs/reports/T-1122-watchtower-production-server-inception.md` — problem-statement validation and assumption tests
- `docs/reports/T-1122-watchtower-wsgi-migration-recommendation.md` — recommendation + DEFER rationale

### 2026-04-18T09:40:32Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-18T20:48:03Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** Recommendation: DEFER on WSGI swap; GO on systemd wrapping (the actual root cause).

Rationale: Re-reading the problem statement, the failure mode that matters ("restart races during this session") is a process-management problem, not a WSGI-server problem. Swapping Werkzeug for gunicorn does not fix restart races; systemd does. The Werkzeug warning is aesthetic on a single-host LAN tool. Adding gunicorn + gevent + flask-socketio_websocket adds dependency surface without proportional benefit.

### 2026-04-23T12:10:46Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** DEFER — Flask-SocketIO + async_mode=threading constrains WSGI choice; spikes needed first (captured as T-1124).

### 2026-04-23T12:15:19Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** DEFER — Flask-SocketIO threading constrains WSGI; spikes captured as T-1124.

### 2026-04-23T12:18:56Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** DEFER
- **Rationale:** DEFER per recommendation: Flask-SocketIO threading constrains WSGI choice; spikes captured as T-1124.

### 2026-04-26T10:56:45Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Re-reading the problem statement, the failure mode that matters ("restart races during this session") is a process-management problem, not a WSGI-server problem. Swapping Werkzeug for gunicorn does not fix restart races; systemd does. The Werkzeug warning is aesthetic on a single-host LAN tool. Adding gunicorn + gevent + flask-socketio_websocket adds dependency surface without proportional benefit.

### 2026-04-26T10:56:45Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
