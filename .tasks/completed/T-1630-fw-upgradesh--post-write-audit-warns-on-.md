---
id: T-1630
name: "fw upgrade.sh — post-write audit warns on lost CLAUDE.md customizations (G-055 prevention)"
description: >
  fw upgrade.sh — post-write audit warns on lost CLAUDE.md customizations (G-055 prevention)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-09T21:19:28Z
last_update: 2026-05-09T21:19:28Z
date_finished: null
---

# T-1630: fw upgrade.sh — post-write audit warns on lost CLAUDE.md customizations (G-055 prevention)

## Context

T-1629 detected that `.agentic-framework/lib/upgrade.sh:355-360` rewrites
CLAUDE.md governance sections wholesale from a template, silently nuking
inline customizations (extra Quick Reference rows, modified bullet text).
PL-124 captured the failure class on 2026-05-02; the very next day the
same regression bit again, sat undetected for 6 days, and was caught only
by an unrelated git diff inspection. G-055 registers this as systemic.
This task ships G-055's prevention path #2: a post-write audit that
diffs CLAUDE.md.bak against the new CLAUDE.md and surfaces any lines lost
to the merge, so the operator sees the regression at write-time, not 6
days later.

Channel-1 mirror required — fix lands in upstream
`/opt/999-Agentic-Engineering-Framework/lib/upgrade.sh` and consumer
`.agentic-framework/lib/upgrade.sh`.

## Acceptance Criteria

### Agent
- [x] Consumer copy of `lib/upgrade.sh` patched: post-write audit block
      surfaces lost lines after the CLAUDE.md template write. Bash syntax
      passes `bash -n`.
- [x] Audit fires only when lost lines exist (`grep -Fxv -f new bak` returns
      non-empty after blank-line trim) — silent in the no-customization case.
- [x] Output format: count + first-8-lines preview + diff hint + G-055/PL-124
      reference. Verified by simulating a regression scenario.
- [x] Upstream `/opt/999-AEF/lib/upgrade.sh` mirrored via channel-1 dispatch
      (workdir=/opt/999-Agentic-Engineering-Framework). Same patch text.
      Pushed to onedev (`b826a96ec..d7526505f master -> master`).
- [x] Both copies pass `bash -n` syntax check.

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

bash -n /opt/termlink/.agentic-framework/lib/upgrade.sh
grep -q "T-1629/G-055" /opt/termlink/.agentic-framework/lib/upgrade.sh
grep -q "lost_lines=" /opt/termlink/.agentic-framework/lib/upgrade.sh

# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
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

## Updates

### 2026-05-09T21:19:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1630-fw-upgradesh--post-write-audit-warns-on-.md
- **Context:** Initial task creation
