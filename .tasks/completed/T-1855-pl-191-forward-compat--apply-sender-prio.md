---
id: T-1855
name: "PL-191 forward-compat — apply sender priority chain to conversation-status/list"
description: >
  PL-191 forward-compat — apply sender priority chain to conversation-status/list

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T21:33:24Z
last_update: 2026-05-28T21:34:40Z
date_finished: 2026-05-28T21:34:40Z
---

# T-1855: PL-191 forward-compat — apply sender priority chain to conversation-status/list

## Context

PL-191 codified that sender identity on chat-arc envelopes is multi-source
(`metadata.agent_id // metadata._from // sender_id`). The same logic
applies to DM-topic envelopes once T-1693 (per-agent keys) lands and
producers start writing `metadata.agent_id` on turn/receipt posts.
Today agent-send.sh / agent-respond.sh DO NOT write that field (they
explicitly defer to T-1693 — see agent-send.sh:202-204). Read-side
forward-compat costs ~4 lines per script and Just Works the moment
producers gain identity. Until then the priority chain falls through
to sender_id (current behavior — no regression).

Targets: `scripts/agent-conversation-status.sh` (line 88 turn-summary,
line 93 receipt-summary) and `scripts/agent-conversation-list.sh`
(line 119 senders, line 120 sender_count).

## Acceptance Criteria

### Agent
- [x] `agent-conversation-status.sh` resolves sender via priority chain `(.metadata.agent_id // .metadata._from // .sender_id // "")` for both turn and receipt projections
- [x] `agent-conversation-list.sh` resolves sender via same chain in both `senders` (line 119) and `sender_count` (line 120) projections
- [x] Both scripts retain the comment block citing PL-191 above the chain (consistency with fleet-adoption-snapshot.sh + agent-chat-arc-recent.sh)
- [x] `bash scripts/test-agent-conversation-status.sh` exits 0
- [x] `bash scripts/test-agent-conversation-list.sh` exits 0

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
grep -q "metadata.agent_id // .metadata._from // .sender_id" scripts/agent-conversation-status.sh
grep -q "metadata.agent_id // .metadata._from // .sender_id" scripts/agent-conversation-list.sh
bash scripts/test-agent-conversation-status.sh
bash scripts/test-agent-conversation-list.sh

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

### 2026-05-28T21:33:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1855-pl-191-forward-compat--apply-sender-prio.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-63f2fde8
- **Timestamp:** 2026-05-28T21:34:40Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T21:34:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
