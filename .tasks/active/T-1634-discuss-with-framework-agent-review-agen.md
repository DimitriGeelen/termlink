---
id: T-1634
name: "Discuss with framework-agent: review-agent path for autonomous closure of RUBBER-STAMP-only tasks"
description: >
  Discuss with framework-agent: review-agent path for autonomous closure of RUBBER-STAMP-only tasks

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-13T06:28:31Z
last_update: 2026-05-13T06:28:31Z
date_finished: null
---

# T-1634: Discuss with framework-agent: review-agent path for autonomous closure of RUBBER-STAMP-only tasks

## Context

Dimitri pushed back on the framework rule that owner=human tasks must be closed by the human, even when all Human ACs are [RUBBER-STAMP] and agent-evidenced. Live trigger: T-1294 — 4/4 ACs ticked Apr 26-28, re-verified today via live `hub.bus_state` probe; still gated on human typing `fw task update --status work-completed`.

He recalls prior discussion with framework-agent (upstream /opt/999-AEF) about standing up a "review-agent" or `fw task rubber-stamp` verb that lets the agent close when all ACs are [RUBBER-STAMP]. Asking me to find out where that work stands.

Concrete proposal he restated: split closure authority on AC-prefix.
- all [RUBBER-STAMP] + agent-evidenced → agent may close
- any [REVIEW] → human must close

Prior shipped on this side (termlink project):
- PL-154 + T-1628: triage flag pattern (`--compact --by-age --rubber-stamp-only`)
- G-XXX concern (concerns.yaml) — status=watching, refilled 0→5 today
- `fw task verify --rubber-stamp-only --compact --by-age` is live

## Acceptance Criteria

### Agent
- [x] Status-check posted to framework-agent — posted to `framework:pickup` topic offset 9 at 2026-05-13T13:29Z (direct DM rejected: framework-agent session pre-dates T-1436 identity_fingerprint registration, hint: restart on current binary)
- [ ] Response received from framework-agent (on framework:pickup or via restarted-session DM)
- [ ] Findings reported back to Dimitri with concrete next-step recommendation (proceed / wait / unblock differently)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
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

### 2026-05-13T06:28:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1634-discuss-with-framework-agent-review-agen.md
- **Context:** Initial task creation
