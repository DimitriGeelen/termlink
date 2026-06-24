---
id: T-1808
name: "agent-send.sh receipt detection not offset-aware — false DELIVERED on multi-turn"
description: >
  agent-send.sh accepts any receipt for the cid rather than one acking the posted turn; on turn 2+ of a same-cid conversation the prior turn's receipt yields false DELIVERED. Fix: require receipt up_to >= post_offset. Found during T-1807 e2e validation.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-25T20:12:40Z
last_update: 2026-05-25T20:14:15Z
date_finished: 2026-05-25T20:14:15Z
---

# T-1808: agent-send.sh receipt detection not offset-aware — false DELIVERED on multi-turn

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/agent-send.sh` receipt detection is offset-aware: it accepts a receipt only when `metadata.up_to >= post_offset` (the offset of the turn it just posted), not merely any receipt carrying the conversation_id.
- [x] `scripts/test-agent-send.sh` gains a multi-turn regression assertion: turn-1 acked, then turn-2 posted on the SAME cid but NOT acked → `agent-send.sh` does NOT report DELIVERED (exits 3), proving the stale turn-1 receipt no longer triggers a false positive. (PASS C)
- [x] Existing test paths still pass (`agent-send.sh` single-turn DELIVERED on receipt; FAILED with no receipt) and the T-1805 round-trip `scripts/test-agent-respond.sh` still ALL PASS.
- [x] `bash -n` and `shellcheck` clean on `scripts/agent-send.sh` and `scripts/test-agent-send.sh`.

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
bash -n scripts/agent-send.sh
shellcheck scripts/agent-send.sh scripts/test-agent-send.sh
bash scripts/test-agent-send.sh
bash scripts/test-agent-respond.sh

## RCA

**Symptom:** During T-1807 multi-turn end-to-end validation, `agent-send.sh`
reports DELIVERED for turn 2+ of a conversation immediately — before the
receiver acks that turn — because a receipt from an earlier turn on the same
conversation_id already exists.

**Root cause:** The receipt poll selects `[.[] | select(.msg_type=="receipt")] | .[0].offset` —
i.e. ANY receipt carrying the cid satisfies it. It ignores the `up_to` field
that exists specifically to express "acked up to offset N". A single-turn send
is correct (only one receipt can exist), but a multi-turn conversation reuses
one cid, so a stale receipt from turn N-1 falsely satisfies turn N's wait.

**Why structurally allowed:** T-1804's smoke test only exercised single-turn
exchanges (one cid, one receipt), so the stale-receipt case never appeared. The
`up_to` ack-watermark was emitted by `agent-respond.sh` (T-1805) but never
consumed by the sender — an unused field is invisible to a single-turn test.

**Prevention:** Multi-turn regression assertion added to `test-agent-send.sh`
(turn-1 acked, turn-2 unacked → must NOT report DELIVERED). Any future change to
the receipt-detection logic that drops the `up_to >= post_offset` guard will
fail this test.

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

### 2026-05-25T20:12:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1808-agent-sendsh-receipt-detection-not-offse.md
- **Context:** Initial task creation

### 2026-05-25T20:12:51Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-3326e2a8
- **Timestamp:** 2026-05-25T20:14:27Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-25T20:14:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
