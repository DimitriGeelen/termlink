---
id: T-1842
name: "agent-listeners.sh + fleet variant: treat -32013 unknown topic as 0 listeners, not subscribe failure (G-060 graceful degradation)"
description: >
  agent-listeners.sh + fleet variant: treat -32013 unknown topic as 0 listeners, not subscribe failure (G-060 graceful degradation)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-28T15:03:27Z
last_update: 2026-05-28T15:06:48Z
date_finished: 2026-05-28T15:06:48Z
---

# T-1842: agent-listeners.sh + fleet variant: treat -32013 unknown topic as 0 listeners, not subscribe failure (G-060 graceful degradation)

## Context

Per G-060, `agent-presence` is hub-local — a hub that's never had a heartbeat
posted won't have the topic. `agent-listeners.sh` currently treats this as
"channel subscribe failed (exit=3)" and `agent-listeners-fleet.sh` buckets the
hub into `hubs_failed`. Result: a fresh fleet (post-T-1841 adoption push but
pre-real-uptake, which is today) reports 3-of-5 hubs as "broken" when really
they're just empty.

Reproducer (confirmed):
```
$ termlink channel subscribe agent-presence --hub 192.168.10.122:9100
Error: Hub returned error for channel.subscribe: JSON-RPC error -32013: unknown topic: agent-presence
EXIT=1
```

Fix: detect `-32013` / `unknown topic` in subscribe stderr and exit 0 with an
empty rollup. Other subscribe failures (auth, network) still exit 3 with the
real error surfaced. The fleet variant then naturally sees scanned-ok-with-0
instead of failed.

This unblocks `agent-listeners-fleet.sh --json` as a useful adoption-state
dashboard — it correctly counts non-adopting hubs as healthy-but-empty.

## Acceptance Criteria

### Agent
- [x] `agent-listeners.sh` detects `-32013` / `unknown topic` in subscribe stderr and exits 0 with empty listener rollup
- [x] Other subscribe failures (auth, network) still exit 3 with the error surfaced (T10 verifies via unreachable-hub)
- [x] `agent-listeners.sh --json` on a topic-less hub returns valid envelope: `total_listeners=0, live=0, stale=0, offline=0, listeners=[]` (T9)
- [x] `agent-listeners-fleet.sh` counts topic-less hubs as scanned-ok (NOT as `hubs_failed`) — verified live below
- [x] Unit tests in `test-agent-listeners.sh` cover the -32013 path (T9 + T10 added)
- [x] Live verification: `agent-listeners-fleet.sh --json` BEFORE: `hubs_scanned=2, hubs_failed=3`. AFTER: `hubs_scanned=5, hubs_failed=0`. fleet snapshot now: `total_listeners=1, live=1` (just me — accurate fleet adoption count, no false-failure noise).

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

bash scripts/test-agent-listeners.sh
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

### 2026-05-28T15:03:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1842-agent-listenerssh--fleet-variant-treat--.md
- **Context:** Initial task creation

### 2026-05-28 — fix shipped

**Shipped:**
- `scripts/agent-listeners.sh` — capture subscribe stderr to tmpfile, detect
  `-32013` / `unknown topic`, treat as empty rollup + exit 0. Other subscribe
  failures preserve exit 3 + error surfaced.
- `scripts/test-agent-listeners.sh` — added T9 (unknown-topic→empty-envelope) and
  T10 (unreachable-hub→exit-3-preserved).

**Live verification:**
- Pre-fix fleet scan: `hubs_scanned=2, hubs_failed=[laptop-141, ring20-management, ring20-dashboard with "channel subscribe failed (exit=1)"]`.
- Post-fix fleet scan: `hubs_scanned=5, hubs_failed=0, total_listeners=1, live=1` — accurate dashboard.

**Why structurally allowed:** G-060 (channel topics are hub-local) means a fresh hub
won't have agent-presence created until someone first emits a heartbeat. The original
agent-listeners.sh was written assuming subscribe failure = bug. After T-1830/T-1841
opened the doorbell+mail arc to wider adoption, the common case became "most hubs
have NEVER had a presence heartbeat" — turning the assumption inside-out.

**Prevention:** The T9 test now pins the contract: on a healthy hub with a missing
topic, the script MUST exit 0 with an empty envelope. Any future refactor that
breaks this will fail T9 immediately.

## Recommendation

**Recommendation:** GO

**Rationale:** All 6 Agent ACs ticked + 2 Verification commands pass (10/10
test-agent-listeners + 6/6 test-agent-listeners-fleet). Live fleet scan went from
2/5 healthy with 3 false-failures to 5/5 healthy with 0 failures. Test pins the
contract.

**Evidence:**
- `scripts/agent-listeners.sh` lines 99-117 — stderr-capture + -32013-detect path
- `scripts/test-agent-listeners.sh` T9 + T10 — new test cases
- Live fleet scan output: `{hubs_scanned:5, hubs_failed:0, total_listeners:1, live:1, offline:0}`

## Reviewer Verdict (v1.4)

- **Scan ID:** R-99541e24
- **Timestamp:** 2026-05-28T15:07:37Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T15:06:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
