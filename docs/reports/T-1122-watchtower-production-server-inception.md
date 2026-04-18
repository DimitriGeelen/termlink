# T-1122: Migrate Watchtower from Werkzeug dev server to production-grade WSGI

**Status:** Inception (started 2026-04-18)
**Owner:** human (decision); agent (research)
**Trigger:** Operator (Dimitri) flagged the recurring `WARNING: This is a development server` in the Watchtower startup log and asked us to evaluate migration to a production-grade server.

## Context

Watchtower is the Flask web app at `.agentic-framework/web/app.py`. It is currently invoked as:

```
python3 -m web.app
```

…which runs Flask's built-in Werkzeug development server. Werkzeug's own warning explains the concern:

> WARNING: This is a development server. Do not use it in a production deployment. Use a production WSGI server instead.

Watchtower is treated as a long-lived service: it is started at the beginning of every session, polled by agents (fleet status, recent activity), and used by operators to inspect tasks, gaps, and audit results. In practice it *is* a production deployment for a single-host operator console.

## Why this matters now

1. **Recurring restart races.** During T-1117..T-1121 in this same session, restarting Watchtower repeatedly produced "running" → silent crash patterns. A real WSGI server with proper signal handling and a process manager (systemd) would handle reload cleanly.
2. **Concurrency limits.** Werkzeug serves one request at a time by default. With async htmx widgets (fleet status, ambient strip refreshes, scan refreshes) and CLI subprocess calls (`termlink fleet status`, ~5s), requests pile up.
3. **Production deployments coming.** The fleet now spans multiple hosts (ring20-management, ring20-dashboard). If any of those hosts also runs Watchtower for cross-host operator UX, the warning becomes a real risk, not a cosmetic one.
4. **Cosmetic noise.** Every operator who reads the startup log sees "do not use in production." This trains the team to ignore warnings — bad antifragile hygiene.

## Candidate servers (to research in spike phase)

| Server     | Pros | Cons | Linux | macOS | htmx/SSE |
|------------|------|------|-------|-------|----------|
| gunicorn   | Industry standard, well-documented, multi-worker | Unix-only (no native Windows; macOS works) | yes | yes | yes (with `--worker-class gthread` or `gevent`) |
| waitress   | Pure Python, cross-platform, single-process | Lower throughput than gunicorn | yes | yes | yes (single-threaded handles SSE) |
| hypercorn  | ASGI/WSGI, HTTP/2, async-friendly | Heavier dep; we're WSGI not ASGI | yes | yes | yes |
| uwsgi      | Battle-tested, many features | Heavy config, deprecation talk | yes | partial | yes |

Initial gut: **waitress** for v1 (zero-config, pure Python, no external deps, fewer surprises) with **gunicorn** as a fallback if we ever need multi-worker.

## Open questions

- **Q1:** Do any framework hooks (PreToolUse/PostToolUse) assume single-process state? (e.g., `.budget-status` writes, `.tool-counter`)
- **Q2:** Does the `fw watchtower start` wrapper need to add a graceful-shutdown signal beyond SIGTERM?
- **Q3:** Does anyone consume Watchtower over WAN where TLS would be required? If yes, escalate to a separate inception (TLS termination is out of scope here).

## Dialogue Log

### 2026-04-18 — Initial trigger
**User:** Asked to acknowledge the dev-server warning and create an inception to evaluate migration.
**Agent:** Created T-1122 with problem statement, assumptions (A1-A5), exploration plan (4 timeboxed spikes), scope fence, and go/no-go criteria. Created this research artifact per C-001.
**Decision:** None yet — awaiting human GO/NO-GO before any spike work or implementation.

## Status

Awaiting human review. Next action: human runs `fw task review T-1122` to see this artifact and the inception template, then decides GO (proceed with spikes) / NO-GO (close as accepted-risk) / DEFER.
