---
id: T-1804
name: "Deterministic doorbell+mail send verb — post turn + ring doorbell + receipt ack + bounded re-ring (T-1800 build #1)"
description: >
  Deterministic doorbell+mail send verb — post turn + ring doorbell + receipt ack + bounded re-ring (T-1800 build #1)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/aggregator.rs, crates/termlink-hub/src/channel.rs, crates/termlink-protocol/src/events.rs]
related_tasks: []
created: 2026-05-25T17:28:59Z
last_update: 2026-05-25T17:35:31Z
date_finished: 2026-05-25T17:35:31Z
---

# T-1804: Deterministic doorbell+mail send verb — post turn + ring doorbell + receipt ack + bounded re-ring (T-1800 build #1)

## Context

First build deliverable of the T-1800 GO (interactive agent conversation
runtime). T-1800 proved the doorbell+mail mechanism end-to-end; this ships the
**deterministic send verb** from its scope fence: one atomic
post-turn → ring-doorbell → wait-for-receipt → bounded-re-ring loop so the
sender always learns *delivered-or-failed* (no silent loss — closes the
PL-011 "ok:true ≠ delivered" class for conversational turns).

**Composes existing primitives only — no protocol changes** (per T-1800 scope):
- **mail (post turn):** `termlink channel post <dm-topic> --msg-type turn --payload <text> --metadata conversation_id=<id>` (channel.rs:337). dm topic = `dm:<sorted_fp_a>:<sorted_fp_b>`.
- **doorbell (ring):** `termlink inject <peer-session> "/check-arc" --enter` (execution.rs) — wakes the turn-based listener.
- **receipt (ack):** receiver posts `msg_type=receipt` (T-1315 `channel ack`); sender detects it by polling the dm topic filtered on `conversation_id` (`channel subscribe --conversation-id <id>`, channel.rs:7744).
- **bounded re-ring:** if no receipt within per-attempt timeout, re-inject; cap attempts; then report FAILED.

Implemented as `scripts/agent-send.sh` (same composition pattern as the other
operator scripts in `scripts/`). A later follow-up may promote it to a native
`termlink agent send` CLI verb; this script proves + ships the mechanism now.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-send.sh` exists, is executable, and accepts: target session (for the doorbell), dm topic OR peer fingerprint (for mail), message text, a conversation id (auto-generated if omitted), per-attempt timeout, and max re-ring attempts — with `--help` and arg validation (verified: `--help`, missing-message → rc 2, no-topic/peer → rc 2)
- [x] Happy path: posts the turn, rings the doorbell, then polls the dm topic filtered by `conversation_id` for a `msg_type=receipt`; on receipt it prints the delivered offset and exits 0 (smoke PASS A)
- [x] Failure path: when no receipt arrives, it re-rings up to the cap (one ring per attempt), then exits non-zero with a clear "not acked after N attempts" message — never claims success on hub-accept alone (PL-011) (smoke PASS B: rc 3 after 3 rings)
- [x] Doorbell (`inject`) failure is non-fatal to the post (turn is still posted + still waits for receipt) and is surfaced as a warning, so a missing/renamed listener session doesn't lose the mail (smoke targets a non-existent session; mail still posts + receipt still detected)
- [x] Smoke test `scripts/test-agent-send.sh` against a local hub proves BOTH paths: (a) self-posted receipt → script exits 0 with the offset; (b) no receipt → script re-rings the capped number of times then exits non-zero. Test uses the local hub and cleans up via mktemp+trap (ALL PASS)
- [x] `bash -n scripts/agent-send.sh` and `bash -n scripts/test-agent-send.sh` parse clean; `shellcheck` has no errors (clean, no output)

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
bash -n scripts/agent-send.sh
bash -n scripts/test-agent-send.sh
test -x scripts/agent-send.sh
bash scripts/test-agent-send.sh

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

### 2026-05-25T17:28:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1804-deterministic-doorbellmail-send-verb--po.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-3bc0e4bb
- **Timestamp:** 2026-05-25T17:35:38Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-25T17:35:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
