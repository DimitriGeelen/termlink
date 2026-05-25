---
id: T-1806
name: "Injectable-listener spawn recipe + governance doc (T-1800 build #3)"
description: >
  T-1800 build #3. Document the canonical recipe to run a persistent injectable Claude listener: persistent termlink --shell session + claude (default permission mode, NOT claude -p), governed via FW_SAFE_MODE=1 (keeps Tier-0+budget) OR its own started-work task, with Bash(termlink:*) allowlisted for hands-free replies. Per T-1800 Evidence: do NOT use ungoverned /tmp (drops Tier-0+budget+boundary). Write to docs/operations/.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-25T17:35:12Z
last_update: 2026-05-25T20:03:36Z
date_finished: null
---

# T-1806: Injectable-listener spawn recipe + governance doc (T-1800 build #3)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `docs/operations/injectable-listener-spawn-recipe.md` exists and documents the canonical recipe to run a persistent injectable Claude listener: a long-lived `termlink` PTY session running `claude` in DEFAULT permission mode, explicitly NOT `claude -p` (with the re-pay-context-per-call rationale).
- [x] The doc specifies the two sanctioned governance modes — `FW_SAFE_MODE=1` (keeps Tier-0 + budget gates, drops only the task gate) OR running the listener under its own `started-work` task — and explicitly warns against an ungoverned `/tmp` session (drops Tier-0 + budget + project-boundary protections), per the T-1800 Evidence.
- [x] The doc ties the recipe to the full doorbell+mail loop: doorbell (`agent-send.sh` injecting `/check-arc`, T-1804) wakes the listener → `/check-arc` respond mode (T-1805) → `agent-respond.sh` posts receipt+reply. It cross-references both scripts by path, and both referenced scripts exist on disk.
- [x] The doc includes a copy-pasteable spawn command block, an injectability verification step (confirm the session shows up and accepts an inject), and a teardown step.
- [x] The doc notes the `Bash(termlink:*)` allowlist requirement so the woken listener can post receipts/replies hands-free.

### Human

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
test -f docs/operations/injectable-listener-spawn-recipe.md
grep -q "FW_SAFE_MODE" docs/operations/injectable-listener-spawn-recipe.md
grep -q "claude -p" docs/operations/injectable-listener-spawn-recipe.md
grep -q "agent-send.sh" docs/operations/injectable-listener-spawn-recipe.md
grep -q "agent-respond.sh" docs/operations/injectable-listener-spawn-recipe.md
test -f scripts/agent-send.sh
test -f scripts/agent-respond.sh

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

### 2026-05-25T17:35:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1806-injectable-listener-spawn-recipe--govern.md
- **Context:** Initial task creation

### 2026-05-25T20:03:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
