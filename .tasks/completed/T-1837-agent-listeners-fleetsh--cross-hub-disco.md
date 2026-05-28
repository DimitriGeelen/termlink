---
id: T-1837
name: "agent-listeners-fleet.sh — cross-hub discovery merge (T-1830 follow-up)"
description: >
  agent-listeners-fleet.sh — cross-hub discovery merge (T-1830 follow-up)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T13:44:12Z
last_update: 2026-05-28T13:44:12Z
date_finished: null
---

# T-1837: agent-listeners-fleet.sh — cross-hub discovery merge (T-1830 follow-up)

## Context

T-1833 ships `agent-listeners.sh` as a single-hub discovery reader. G-060 means channel topics (including `agent-presence`) are hub-local — there is no inter-hub federation primitive. For cross-host doorbell+mail to work, a sender on host A must see listeners heartbeating on host B's hub. Operator workaround today is "run `agent-listeners.sh --hub <B>` per hub" — tedious and easy to forget. This task ships `scripts/agent-listeners-fleet.sh`: walks `~/.termlink/hubs.toml` profiles, calls the single-hub verb per hub in parallel, and merges by `agent_id` with a deterministic preference rule (LIVE > STALE > OFFLINE; ties → most-recent `last_seen_ts`).

## Acceptance Criteria

### Agent
- [ ] `scripts/agent-listeners-fleet.sh` exists, executable, runs from any CWD
- [ ] Parses `~/.termlink/hubs.toml` (minimal `[hubs.NAME] address = "..."` parser — mirror T-1831 pattern)
- [ ] Calls `agent-listeners.sh --hub <addr> --json [...]` per profile in parallel
- [ ] Merges by `agent_id`: LIVE > STALE > OFFLINE; ties → most-recent `last_seen_ts`
- [ ] Each output row carries `hub` (which profile saw the heartbeat last)
- [ ] Supports `--topic`, `--include-offline`, `--filter-agent-id`, `--filter-role`, `--filter-listen-topic`
- [ ] Supports `--hubs-file <path>` override
- [ ] `--json` emits `{ok, hubs_scanned, hubs_failed, total_listeners, live, stale, offline, listeners}`
- [ ] Default text output is a fixed-width table similar to single-hub `agent-listeners.sh`
- [ ] Exit codes: 0 OK, 2 usage, 3 all-hubs-unreachable (partial = OK, surface failed list)
- [ ] `--help` documents the verb
- [ ] `scripts/test-agent-listeners-fleet.sh` covers: help, usage error, local-only happy path, simulated multi-hub merge with stub `AGENT_LISTENERS_BIN`, all-hubs-unreachable

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

bash scripts/agent-listeners-fleet.sh --help >/dev/null
bash scripts/test-agent-listeners-fleet.sh

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

### 2026-05-28T13:44:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1837-agent-listeners-fleetsh--cross-hub-disco.md
- **Context:** Initial task creation
