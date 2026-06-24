---
id: T-1809
name: "Doorbell respond-mode signal — woken /check-arc must enter respond mode not browse"
description: >
  G-b from T-1807: agent-send.sh injects /check-arc as the doorbell, but /check-arc defaults to read-only browse mode. A woken listener has no way to know it was rung by a peer (ack+reply) vs invoked manually (read-only), so a live claude reads the turn but never posts a receipt and the sender never sees DELIVERED. Add an explicit respond-mode signal (e.g. /check-arc --respond, or a distinct doorbell text the skill recognizes) so a woken listener auto-acks via agent-respond.sh. Blocks T-1810.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-25T20:16:19Z
last_update: 2026-05-25T20:19:37Z
date_finished: 2026-05-25T20:19:37Z
---

# T-1809: Doorbell respond-mode signal — woken /check-arc must enter respond mode not browse

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/agent-send.sh` default doorbell text changes from `/check-arc` to a respond-signalling form (`/check-arc respond`) so a woken listener enters respond mode, not read-only browse. `--doorbell-text` override still works. (verified: ring output shows `inject '/check-arc respond'`)
- [x] `.claude/commands/check-arc.md` recognizes the `respond` argument: when invoked as `/check-arc respond` it enters Respond mode (Step 6) directly; bare `/check-arc` still defaults to read-only browse mode. The arg contract is documented in the Invocation section.
- [x] The doorbell-text change does not regress the loop: `scripts/test-agent-send.sh` and `scripts/test-agent-respond.sh` still ALL PASS (they target a non-existent doorbell session, so the inject text is non-fatal and irrelevant to receipt detection).
- [x] `bash -n` + `shellcheck` clean on `scripts/agent-send.sh`; the skill markdown documents both invocation forms (browse vs respond).

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
shellcheck scripts/agent-send.sh
grep -q "check-arc respond" scripts/agent-send.sh
grep -q "respond" .claude/commands/check-arc.md
bash scripts/test-agent-send.sh
bash scripts/test-agent-respond.sh

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

### 2026-05-25T20:16:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1809-doorbell-respond-mode-signal--woken-chec.md
- **Context:** Initial task creation

### 2026-05-25T20:18:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.4)

- **Scan ID:** R-cc631525
- **Timestamp:** 2026-05-25T20:19:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-25T20:19:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
