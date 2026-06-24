---
id: T-1811
name: "agent-send.sh --await-reply: synchronous conversational round-trip (T-1800 follow-up)"
description: >
  agent-send.sh --await-reply: synchronous conversational round-trip (T-1800 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-25T20:35:30Z
last_update: 2026-05-25T20:39:42Z
date_finished: 2026-05-25T20:39:42Z
---

# T-1811: agent-send.sh --await-reply: synchronous conversational round-trip (T-1800 follow-up)

## Context

The T-1800 doorbell+mail loop (T-1804 `agent-send.sh`) confirms *delivery* of a
turn (receipt seen) but stops there — it never surfaces the receiver's *reply*.
For an interactive conversation that means a caller must separately subscribe and
hunt for the reply turn. This task adds `--await-reply <secs>`: after DELIVERED,
poll the same dm topic for the peer's reply turn (the first `msg_type=turn` with
`offset > post_offset` on this `conversation_id`) and print it — making one full
synchronous request→confirm→response round-trip available in a single command.
Composes existing `channel.*` primitives only; no protocol change. The receipt
path (T-1808 offset-aware fix) is untouched.

Predecessors: T-1804 (send verb), T-1808 (offset-aware receipt), T-1807 (loop
validation showed listener replies at offsets 2/5/8 that the sender never read).

## Acceptance Criteria

### Agent
- [x] `agent-send.sh --help` documents `--await-reply <secs>` and the new exit code 4 (delivered-but-no-reply)
- [x] `--await-reply` validates its argument as a positive integer; non-numeric or `<1` exits 2 (usage) — verified: `foo`→2, `0`→2
- [x] With `--await-reply <secs>`, after a receipt is seen the script polls the dm topic for the first `msg_type=turn` with `offset > post_offset` (the peer's reply), prints its payload, and exits 0 — verified Path D (PONG_D_REPLY printed, rc=0)
- [x] When delivered but no reply arrives within the `--await-reply` window, the script exits 4 with a clear "delivered, no reply" message (delivery still succeeded — distinct from exit 3 not-acked) — verified Path E (rc=4)
- [x] Without `--await-reply`, behavior is unchanged (exit 0 delivered / 3 not-acked / 2 usage) — Paths A/B/C still pass
- [x] `test-agent-send.sh` gains Path D (delivered + reply → exit 0, reply payload printed) and Path E (delivered, no reply → exit 4); full suite passes — ALL PASS (A–E)
- [x] `bash -n` clean on both scripts — plus shellcheck CLEAN

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
bash -n scripts/test-agent-send.sh
scripts/agent-send.sh --help 2>&1 | grep -q -- '--await-reply'
scripts/agent-send.sh --help 2>&1 | grep -qi 'no reply'
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

### 2026-05-25T20:35:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1811-agent-sendsh---await-reply-synchronous-c.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-75937cdb
- **Timestamp:** 2026-05-25T20:39:59Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-25T20:39:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
