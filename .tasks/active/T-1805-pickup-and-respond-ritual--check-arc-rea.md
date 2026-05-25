---
id: T-1805
name: "Pickup-and-respond ritual — /check-arc reads turns AND posts receipt+reply (T-1800 build #2)"
description: >
  T-1800 build #2. Extend /check-arc (currently read-only) into the pickup-AND-respond ritual: when woken by an injected doorbell, read unread turns on dm:<self>:* topics, and for each, post a msg_type=receipt with the same conversation_id (so the sender's agent-send.sh T-1804 detects delivery) and a reply turn. Closes the 'respond' half of the doorbell+mail loop. Composes channel post/subscribe + ack; no protocol changes.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-25T17:35:09Z
last_update: 2026-05-25T19:49:04Z
date_finished: null
---

# T-1805: Pickup-and-respond ritual — /check-arc reads turns AND posts receipt+reply (T-1800 build #2)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/agent-respond.sh` exists, is executable, passes `bash -n` and `shellcheck`. It is the receiver's mechanical ack: given a dm topic + conversation-id, posts a `msg_type=receipt` envelope carrying `metadata.conversation_id=<cid>` (the exact shape `agent-send.sh` T-1804 polls for) so the sender detects delivery.
- [x] The receipt's `up_to` metadata defaults to the highest offset observed for that conversation on the topic (falls back to 0 when none); `--up-to <n>` overrides. (verified: `--up-to 42` → receipt `metadata.up_to="42"`)
- [x] Optional `--reply <text>` also posts a `msg_type=turn` envelope with the same `conversation_id`, completing the respond half (ack + reply). (exercised in round-trip Path A)
- [x] Topic resolution mirrors `agent-send.sh`: direct `--topic <dm-topic>` OR `--peer-fp <fp>` computing `dm:<sorted self,peer>` from `whoami --json`.
- [x] Arg validation: missing `--conversation-id` → exit 2 with usage; missing both `--topic` and `--peer-fp` → exit 2; `--help` → exit 0. (all verified)
- [x] `scripts/test-agent-respond.sh` round-trips the REAL `agent-send.sh` + `agent-respond.sh`: send waits, respond posts the receipt for the same cid → send exits 0 DELIVERED (positive); no respond → send exits 3 FAILED (negative). Test PASSES, or SKIPs cleanly when termlink/hub/jq absent. (ALL PASS on live hub)
- [x] `/check-arc` skill (`.claude/commands/check-arc.md`) gains a "Respond mode (woken by a doorbell)" section that delegates the mechanical ack to `agent-respond.sh` per unread conversation, while the default manual-browse path stays read-only (the existing NEVER-auto-ack rule is scoped to browse mode, not the deliberate respond ritual).

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
bash -n scripts/agent-respond.sh
bash scripts/test-agent-respond.sh
grep -q "Respond mode" .claude/commands/check-arc.md

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

### 2026-05-25T17:35:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1805-pickup-and-respond-ritual--check-arc-rea.md
- **Context:** Initial task creation

### 2026-05-25T19:49:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
