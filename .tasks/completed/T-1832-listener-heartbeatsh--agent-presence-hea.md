---
id: T-1832
name: "listener-heartbeat.sh — agent-presence heartbeat emitter (T-1830 sub-build a)"
description: >
  listener-heartbeat.sh — agent-presence heartbeat emitter (T-1830 sub-build a)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T12:53:04Z
last_update: 2026-05-28T12:53:04Z
date_finished: null
---

# T-1832: listener-heartbeat.sh — agent-presence heartbeat emitter (T-1830 sub-build a)

## Context

T-1830 GO decided 2026-05-28T12:37:55Z — inception confirmed the doorbell+mail runtime is healthy but adoption is zero, and recommended three sub-builds: (a) heartbeat/listener-presence topic, (b) discovery verb, (c) `agent-send.sh --auto-discover`. This task ships (a) — the heartbeat emitter side. Without presence signals there's nothing for the discovery verb to read. Convention established here: topic `agent-presence`, msg_type=heartbeat, metadata{agent_id, role, listen_topics, started_at}. Per-hub topic (G-060 — channels are hub-local); discovery reader (T-1833) will merge across hubs.

## Acceptance Criteria

### Agent
- [x] `scripts/listener-heartbeat.sh` exists, executable, with flags: `--agent-id NAME` (required), `--role R` (default "listener"), `--listen-topic T` (repeatable), `--topic agent-presence` (default), `--interval 30` (default seconds), `--hub addr`, `--once` (single post then exit), `--json` (emit posted envelope), `--help`
- [x] Default mode loops, posting one heartbeat per `--interval` seconds, exiting cleanly on SIGINT/SIGTERM
- [x] `--once` posts exactly one heartbeat and exits 0
- [x] Posted envelope shape: `msg_type=heartbeat`, payload=role string, metadata = `{agent_id, role, listen_topics: comma-joined, started_at: RFC3339, interval_secs, host}`
- [x] On first run, auto-creates the `agent-presence` topic (best-effort; ignored if already exists) with retention `messages:200`
- [x] Exit codes: 0 normal exit/once-success, 2 usage (missing required flag, unknown arg), 3 hub-side error (post failure)
- [x] `scripts/test-listener-heartbeat.sh` exists: T1 --help exit=0 + usage, T2 unknown arg exit=2, T3 missing --agent-id exit=2, T4 --once --json against local hub → posts one envelope, JSON parseable, msg_type=heartbeat, metadata.agent_id matches, T5 --once with --listen-topic foo --listen-topic bar → metadata.listen_topics contains both
- [x] All tests pass
- [x] Live verification: `bash scripts/listener-heartbeat.sh --agent-id selftest-T-1832 --once --json` against local hub returns a parseable envelope with the expected metadata; envelope visible via `termlink channel subscribe agent-presence --limit 1 --json`

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
test -x scripts/listener-heartbeat.sh
test -x scripts/test-listener-heartbeat.sh
bash scripts/test-listener-heartbeat.sh
bash scripts/listener-heartbeat.sh --help >/dev/null

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

### 2026-05-28T12:53:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1832-listener-heartbeatsh--agent-presence-hea.md
- **Context:** Initial task creation
