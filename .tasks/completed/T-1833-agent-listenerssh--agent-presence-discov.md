---
id: T-1833
name: "agent-listeners.sh — agent-presence discovery reader (T-1830 sub-build b)"
description: >
  agent-listeners.sh — agent-presence discovery reader (T-1830 sub-build b)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-28T12:56:10Z
last_update: 2026-05-28T12:59:35Z
date_finished: 2026-05-28T12:59:35Z
---

# T-1833: agent-listeners.sh — agent-presence discovery reader (T-1830 sub-build b)

## Context

T-1830 GO sub-build (b) — the discovery side that consumes the heartbeat convention established by T-1832. Reads recent envelopes on `agent-presence`, dedupes to newest per `agent_id`, applies the TTL convention (LIVE/STALE/OFFLINE based on age vs declared `interval_secs`), and returns active listeners. This is what an agent or operator runs to answer "who's listening right now, and on what?" — the missing piece between healthy runtime (T-1829) and active conversations (the adoption gap T-1830 named).

TTL convention (from T-1832): age = now - last_seen_ts. status = LIVE if age <= 2*interval_secs, STALE if 2x < age <= 5x, OFFLINE otherwise. Default output filters to LIVE+STALE; --include-offline adds OFFLINE.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-listeners.sh` exists, executable, with flags: `--topic agent-presence` (default), `--hub addr`, `--limit N` (default 200, max 1000), `--include-offline` (default off — show LIVE+STALE), `--filter-role R`, `--filter-listen-topic T`, `--filter-agent-id ID`, `--json`, `--help`
- [x] Reads up to `--limit` most-recent envelopes from `--topic`; groups by `metadata.agent_id`; keeps newest per id by `ts`
- [x] Applies TTL convention using each envelope's own `metadata.interval_secs` — status is LIVE / STALE / OFFLINE per the 2x/5x rule
- [x] Text mode prints fixed-width table: `agent_id | role | status | age | listen_topics | host`
- [x] JSON envelope: `{ok, topic, hub, total_listeners, live, stale, offline, listeners: [{agent_id, role, status, age_secs, last_seen_ts, listen_topics, host, interval_secs}]}`
- [x] Empty topic / no matching listeners → exit 0 with `total_listeners=0` (NOT an error)
- [x] Unknown topic at hub → exit 3 (mirrors T-1826/T-1827 convention)
- [x] `scripts/test-agent-listeners.sh` covers: T1 --help exit=0 + usage; T2 unknown arg exit=2; T3 empty topic → ok=true total=0; T4 populated topic with one LIVE listener → live=1 status=LIVE; T5 stale (interval=5, last post >10s ago) → status=STALE; T6 --filter-agent-id narrows correctly; T7 --include-offline includes OFFLINE entries (interval=5, last post >25s ago)
- [x] All tests pass
- [x] Live verification: with the heartbeat from T-1832 running (`bash scripts/listener-heartbeat.sh --agent-id live-T-1833 --interval 5 --listen-topic agent-chat-arc &`), `bash scripts/agent-listeners.sh --json` shows the live agent with status=LIVE

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
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
test -x scripts/agent-listeners.sh
test -x scripts/test-agent-listeners.sh
bash scripts/test-agent-listeners.sh
bash scripts/agent-listeners.sh --help >/dev/null

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
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

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-28T12:56:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1833-agent-listenerssh--agent-presence-discov.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-17897717
- **Timestamp:** 2026-05-28T13:00:13Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Per-AC findings:**

- **AC#10 (Agent)** — Live verification: with the heartbeat from T-1832 running (`bash scripts/listener-heartbeat.sh --agent-id live-T-1833 --interval 5 --listen-topic agent-chat-arc &`), `bash scripts/agent-listeners.sh -
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/listener-heartbeat.sh in: Live verification: with the heartbeat from T-1832 running (`bash scripts/listener-heartbeat.sh --agent-id live-T-1833 --interval 5 --listen-topic agen`

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 12
     - evidence: `bash scripts/agent-listeners.sh --help >/dev/null`

### 2026-05-28T12:59:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
