---
id: T-1829
name: "agent-conversation-selftest.sh — loopback doorbell+mail validation (drives adoption)"
description: >
  agent-conversation-selftest.sh — loopback doorbell+mail validation (drives adoption)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-28T11:33:24Z
last_update: 2026-05-28T11:35:58Z
date_finished: 2026-05-28T11:35:58Z
---

# T-1829: agent-conversation-selftest.sh — loopback doorbell+mail validation (drives adoption)

## Context

**The gap:** Live audit 2026-05-28 found zero active doorbell+mail conversations on
.107 across 91 topics. The runtime works (T-1807 validated end-to-end). The
diagnostics work (T-1826/T-1827). But adoption is zero.

**Hypothesis for the adoption gap:** `agent-send.sh` requires a known live peer
(`--to-session <name>` + topic/peer-fp) BEFORE you can validate "doorbell+mail
works on this host." That's a chicken-and-egg pre-flight problem — operators
won't turn loose autonomous traffic without first confirming the loop runs
healthy locally, but they have no path to confirm it without already having a
live peer running `/check-arc respond`.

**This task closes the pre-flight gap.** A single-host loopback selftest that
posts a turn AND its receipt on an ephemeral topic — both impersonating
distinct sides via metadata — then verifies via `agent-conversation-status.sh`
that the conversation registers as DELIVERED (turn count matches receipt
watermark coverage; pending_count = 0). No live peer required; no PTY
injection; purely a runtime health gate.

Useful for: pre-deployment validation, CI fleet-wide health check, "is this
host's doorbell+mail loop healthy?" diagnostic before turning loose autonomous
agents on it.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-conversation-selftest.sh` exists, executable, `bash -n` + `shellcheck` clean.
- [x] Accepts `--hub <addr>` (optional; default local) + `--verbose` flag + `--json` flag for machine-readable output.
- [x] Creates an ephemeral topic (unique per-run, name encodes `$$` + epoch), posts ≥1 synthetic turn with a synthetic conversation_id, posts a synthetic receipt acking that turn, then runs `agent-conversation-status.sh --json` against the created cid.
- [x] **Pass criterion:** the status verb reports `turn_count >= 1`, `receipt_count >= 1`, `pending_count == 0`. Verified inside the selftest with explicit jq assertions; exit 0 only when all hold.
- [x] **Fail modes** distinguished by exit code: 0 = pass; 1 = assertion-fail; 2 = usage; 3 = setup-fail.
- [x] `--verbose` mode prints each step.
- [x] `--json` mode emits `{ok, hub, ephemeral_topic, conversation_id, turns_posted, receipts_posted, status_result, elapsed_ms, verdict}`.
- [x] `scripts/test-agent-conversation-selftest.sh` covers all 4 paths (T1..T4). **4 PASS / 0 FAIL / 0 SKIP** at HEAD.
- [x] All static checks clean; test suite ALL PASS.

## Live Findings (2026-05-28 fleet validation)

Cross-hub selftest results — three hubs reachable on the local fleet:

| Hub | Verdict | Elapsed |
|---|---|---|
| local (.107) | pass | 51 ms |
| 192.168.10.121:9100 (ring20-dashboard) | pass | 391 ms |
| 192.168.10.122:9100 (ring20-management) | pass | 453 ms |

**Diagnostic finding:** the doorbell+mail runtime is healthy on every
reachable hub. The "no active conversations" gap surfaced by today's
audit is therefore NOT an infrastructure failure — it's a coordination
gap. Agents (cohort-agent, penelope, framework-agent, termlink-agent)
aren't using the loop because there's no convention/wiring driving them
to, not because it doesn't work.

## Recommendation

**Ship.** Selftest closes the pre-flight gate operators want before
adopting doorbell+mail. Three concrete follow-ups (file as needed):

1. **Cron-based fleet health gate (T-1830 candidate):** run
   `agent-conversation-selftest.sh --hub <addr> --json` against every
   profile in `hubs.toml` daily; alert on any non-pass verdict.
   Cron-replacement primitive — same shape as
   `release-mirror-canary.sh`.
2. **MCP parity (T-1831 candidate):** `termlink_agent_conversation_selftest`
   MCP tool — agents on shared hosts can pre-flight before initiating real
   doorbell+mail traffic.
3. **Adoption push:** write a short docs/reports artifact analyzing why
   no agents currently use the loop (coordination gap, not runtime gap).
   This unblocks the next push to wire cohort-agent / penelope into
   doorbell+mail. Filed as separate task — selftest is the foundation,
   adoption is the next sprint.

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

bash -n scripts/agent-conversation-selftest.sh
shellcheck scripts/agent-conversation-selftest.sh
bash -n scripts/test-agent-conversation-selftest.sh
shellcheck scripts/test-agent-conversation-selftest.sh
bash scripts/test-agent-conversation-selftest.sh

# Hint: stale toolchain reminder elided (L-291).
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

### 2026-05-28T11:33:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1829-agent-conversation-selftestsh--loopback-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-f5116f6d
- **Timestamp:** 2026-05-28T11:35:58Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T11:35:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
