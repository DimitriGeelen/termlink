---
id: T-1899
name: "G-020 task-create-time gate failed — RCA + pickup to framework-agent"
description: >
  Inception: the G-020 / T-469 Pickup Message Handling rule (Build Readiness Gate) is supposed to prevent unscoped build tasks from being created with placeholder ACs. Empirically: on 2026-05-31T17:45Z I created T-1898 via `fw work-on "<name>" --type build`, the task file landed with `[First criterion]` / `[Second criterion]` placeholder ACs from the default template, status was set to `started-work`, and no hook blocked the creation. The G-020 gate DID fire on the next Bash tool call (preventing read of inception.md template), proving the gate exists and runs — but only at use-time, not at create-time. This inception scopes the RCA: was the gate designed to be use-time-only by intent (spec gap → needs feature), or was it designed for create-time AND we have a regression (bug → needs fix)? Deliverable post-GO: pickup envelope to framework-agent describing the structural gap with proposed prevention path.

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [governance, hook-gates, framework-agent, pickup, G-020]
components: []
related_tasks: [T-1898, T-469]
created: 2026-05-31T17:52:28Z
last_update: 2026-05-31T17:55:18Z
date_finished: null
---

# T-1899: G-020 task-create-time gate failed — RCA + pickup to framework-agent

## Problem Statement

CLAUDE.md §"Pickup Message Handling (G-020, T-469)" mandates:

> *"Before acting on a pickup message: **Assess scope** — if it describes >3 new files, a new subsystem, a new CLI route, or a new Watchtower page, create an **inception** task (not build). **Write real ACs before editing any source file** — the build readiness gate (G-020) will block tasks with placeholder ACs."*

Observable fact, 2026-05-31T17:45Z: I ran `fw work-on "Vendored agent runner Phase 1 — minimum viable headless claude-code service that reads dm topics and replies" --type build`. The command:

1. **Succeeded** with `Ready to work on T-1898: ...`.
2. **Created** `.tasks/active/T-1898-vendored-agent-runner-phase-1--minimum-v.md` with the default template body — including the literal placeholder strings `- [ ] [First criterion]` and `- [ ] [Second criterion]` under `### Agent`.
3. **Set status** to `started-work` (the work-on shortcut path).
4. **Set focus** to T-1898 via `.context/working/focus.yaml`.

No hook blocked any of steps 1-4. The Bash command itself was not rejected, the task file was not refused, the started-work status was not flagged.

The G-020 hook **did** fire on my very next Bash command (`cat .agentic-framework/.tasks/templates/inception.md`), printing the exact intended block message:

> *"BLOCKED: Task T-1898 is a build task with placeholder/missing ACs. Build tasks require real acceptance criteria before editing source files. This prevents unscoped building. (G-020: Scope-Aware Task Gate)"*

So the gate **exists, is wired into PreToolUse, and is correct** — but its **enforcement surface is post-creation, not at-creation**.

**Why this matters:**
- The operator's mental model of G-020 includes "the gate stops the violation before it starts." Today it stops the violation only at the *next* action, after the task is already in `active/`, status=started-work, focus pointing at it.
- A momentum-biased agent (i.e. me, per the T-1898 reflection) treats "task created successfully" as social proof that the framework is OK with the action. The actual rule violation isn't surfaced until the next tool call, by which time the agent has internalized the wrong frame.
- If the agent has *only* read/grep tool calls between creation and operator scrutiny (e.g. agent reads documentation for the build), the gate may not fire at all in the window where the operator could catch it — the violation becomes invisible until the agent eventually tries to write/edit.

**Who's affected:**
- Every future agent operating under the same pattern — the gate's deterrent value is reduced compared to its design intent.
- The operator's framework-governance trust — "I thought we hook-gated this" is a real and reasonable expectation that was violated by experience this session.
- Pickup-message-driven build tasks in particular (the literal G-020 / T-469 origin case), which often start with a detailed spec and a tempting "start now" path.

**Why now:** the operator surfaced this directly in this session ("YOU ARE VIOLATING FRAMEWORK GOVERNANCE!!!!" + "i thought we hook gated this") which is the exact trigger condition the framework's reflection rules say should open an inception (G-019-style "why did the framework allow this for >7d undetected" — here it's not 7d but it IS a structural blindness that's been live since G-020 landed).

## Assumptions

A1. **The G-020 hook is PreToolUse-only, not task-create-time.** The current implementation fires on Bash/Edit/Write but never inserts itself into the `task-create` / `work-on` command path. Validation: read `.claude/settings.json` (or framework hook config) for hook event types; read `agents/task-create/create-task.sh` for any G-020 invocation.

A2. **Task creation with placeholder ACs is technically detectable at create-time.** The placeholder strings `[First criterion]` / `[Second criterion]` are literal text in the default template. A check after writing the task file (or in the create-task script's post-write step) could grep for them and refuse / warn / convert.

A3. **The create-time check is not a duplicate of the use-time check.** Use-time catches the agent *acting* on a bad task; create-time would catch the agent *making* a bad task. The two are different prevention surfaces with different visibility properties.

A4. **The operator-facing surface (Watchtower) does not visually flag placeholder-AC tasks as "not ready."** A task with literal `[First criterion]` placeholders looks indistinguishable from a properly-scoped task on a glance, contributing to the "I thought we gated this" surprise. Validation: check Watchtower's task list rendering against a task with placeholders.

A5. **The G-020 design intent included create-time enforcement, OR did not.** This is the central RCA question. If "yes intended": we have a bug. If "no intended": we have a spec gap and the inception's output is a recommended spec extension. Validation: read T-469 task file in `.tasks/completed/` (if archived) plus G-020 entry in `concerns.yaml`.

A6. **A pickup-to-framework-agent is the right delivery mechanism for the recommendation.** Memory `[Channel 1 upstream-mirror pattern]` documents the AEF-side workflow; the framework-agent pickup channel is the equivalent. Direct fix here (in /opt/termlink) is wrong because the hook lives upstream in `.agentic-framework/`. Validation: confirm `.agentic-framework/` is read-only from consumer perspective (per T-559).

A7. **The framework-agent has reception capacity.** Per the parallel observation in T-1898, the framework-agent host may have the same vendored-host-without-attached-claude problem. If so, the pickup will sit until an operator attends. That doesn't change correctness of the delivery, but it does affect time-to-resolution. Validation: confirm where `framework:pickup` channel lives and whether anyone reads it.

## Exploration Plan

S1. **Locate the hook config and confirm event coverage.** Read `.claude/settings.json` + `.claude/settings.local.json` (or wherever the PreToolUse hook list lives) and any framework hook config. Goal: enumerate the events the G-020 hook subscribes to. Time-box: 20 min.

S2. **Read the G-020 hook script source.** Find `.agentic-framework/hooks/check-active-task.sh` or equivalent. Identify its decision logic: how does it detect placeholder ACs; what task-state does it gate on. Time-box: 20 min.

S3. **Read the task-create / work-on agent script source.** Find `.agentic-framework/agents/task-create/create-task.sh` + the `fw work-on` wrapper. Identify all hook invocations (or absence thereof) in the create path. Time-box: 30 min.

S4. **Clean replication.** From a clean state: `fw work-on "test scope task" --type build`. Observe: does the same placeholder-AC task land? Does any gate fire? Capture exact output. Time-box: 10 min.

S5. **Read T-469 (the originating task) + G-020 entry in concerns.yaml.** Identify whether the design intent was use-time-only or create-time-also. If T-469 is in completed/ check its acceptance criteria for "create-time" language. Time-box: 30 min.

S6. **Sketch the fix path (per outcome of S5).**
- If "design intent included create-time": file a bug task pointing at the gap in `create-task.sh` (the create script never asks G-020).
- If "design intent was use-time only": propose a NEW gate name (e.g. `G-020.1: Task-Creation Placeholder-AC Refusal`) with a spec proposal: post-template-write grep refuses + offers `--inception` or `--ac-from "<file>"` conversion.
- Either way: package the finding as a framework-agent pickup envelope.

S7. **Draft the framework-agent pickup envelope.** Structured payload: `{type: "framework-suggestion", finding: "G-020 task-create-time gap", evidence: [...], proposed_fix: "...", priority: "P1"}`. Send via `termlink channel post framework:pickup` (read-only proposal; framework-agent decides if/when to action). Time-box: 30 min.

S8. **Update memory.** Add a `feedback_*.md` entry capturing: "If you create a build task and the next action doesn't immediately block, the gate may not have fired yet — check task file ACs before continuing." This protects against the same agent failure mode pending framework-side fix. Time-box: 10 min.

**Total time-box:** ~2.5 hours of agent-side investigation. One session.

## Technical Constraints

- **`.agentic-framework/` is read-only from consumer.** Per T-559 project-boundary enforcement, I cannot edit the hook script directly from /opt/termlink. The fix must land upstream (framework-agent territory).
- **Hook config schema.** Claude Code's `.claude/settings.json` hook config has specific event types (PreToolUse, PostToolUse, SessionStart, PreCompact, etc.); a "task-create" custom event may or may not be supported natively. Constraint shapes which fix paths are viable.
- **Watchtower coupling.** If the fix needs operator-visible signaling in Watchtower (A4), Watchtower-side code change is required too — also framework-side.
- **No retroactive sweep.** Existing build tasks in `active/` with placeholder ACs (if any) are out of scope here — we're scoping prevention, not cleanup.

## Scope Fence

**IN scope for this inception:**
- Determine whether G-020 was *designed* to enforce at create-time and if so where the design intent was lost.
- Locate the structural gap (hook config, create-task script, design doc, OR all of these).
- Package the finding into a framework-agent pickup envelope with concrete proposed fix.
- Update local memory to protect against the same agent-side failure pending framework-side fix.

**OUT of scope:**
- Fixing the framework hook directly (T-559 + cross-project boundary; framework-agent does that work).
- Retroactive sweep of existing tasks with placeholder ACs.
- Broader meta-RCA on "why I matched emotional cues over rules" — that's a memory/reflection artifact already delivered in conversation + captured in T-1898 Updates entry.
- The vendored-agent runner itself (T-1898 — independent).

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated by inspecting actual create-task script output (S4)
- [ ] All 7 assumptions tested or marked deferred with rationale
- [ ] Root-cause-or-spec-gap determined (A5 resolved)
- [ ] Recommendation written with rationale + evidence + concrete proposed fix
- [ ] Framework-agent pickup envelope drafted (the deliverable, even pre-GO)
- [ ] Local memory entry drafted (the agent-side protection)

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go on the framework-agent pickup
  **Steps:**
  1. Run: `fw task review T-1899` (opens Watchtower)
  2. Review the Recommendation + proposed fix + pickup envelope draft
  3. Decide: GO (send pickup), NO-GO (don't send; reasons), DEFER (need more spike evidence)
  4. Record decision via `fw inception decide T-1899 go|no-go|defer --rationale "..."`
  **Expected:** Decision recorded with rationale
  **If not:** Ask agent to deepen the spike or rework the proposed fix

## Go/No-Go Criteria

**GO if:**
- Root cause is identified to a specific file/line (either the hook script that needs a new entry-point, OR the create-task script that needs to invoke the hook, OR the design doc that needs a clarification).
- Proposed fix is concrete (one PR-shape, not "consider redesigning hook architecture").
- Framework-agent pickup envelope is drafted and ready to send on GO click.

**NO-GO if:**
- Root cause turns out to be operator-side or environmental (e.g. local hook disabled by `.claude/settings.local.json`) and no framework-side change is warranted.
- Spike S5 reveals the use-time-only design was deliberate, well-documented, and operator's expectation was simply mistaken (in which case the fix is operator memory, not framework code).

**DEFER if:**
- Hook script source lives in a location I cannot read from /opt/termlink (consumer-side blind spot) and the operator needs to either expose it or do the investigation upstream.
- The framework-agent is unreachable AND the pickup channel is unhealthy, making delivery impossible until rail health is restored (revisit_at: YYYY-MM-DD, revisit_evidence_needed: "framework-agent presence on framework:pickup").

## Verification

# Inception verification: artifacts exist, can be reviewed.
test -f docs/reports/T-1899-g-020-task-create-time-gate-rca.md
grep -q "## Problem Statement" .tasks/active/T-1899-*.md
grep -q "## Go/No-Go Criteria" .tasks/active/T-1899-*.md
grep -q "framework-agent pickup" .tasks/active/T-1899-*.md

## Recommendation

**Recommendation:** DEFER

**Rationale:**

Need empirical evidence before recommending GO/NO-GO. Observable fact: I created T-1898 as workflow_type=build with placeholder ACs via `fw work-on --type build` and it succeeded; the G-020 gate only fired on the NEXT Bash tool call. The inception's job is to determine whether the gate's design intent included task-create-time enforcement (and we have a bug) OR whether task-create was deliberately outside the gate's surface (and we have a spec gap). Spike S1-S4 will read the hook config, task-create script, and run a clean replication. Until then DEFER is honest.

**Evidence:**

- 2026-05-31T17:45Z: `fw work-on "..." --type build` created `.tasks/active/T-1898-*.md` with literal `[First criterion]` placeholder ACs and `status: started-work`; no hook blocked the creation.
- 2026-05-31T17:48Z: subsequent `cat .agentic-framework/.tasks/templates/inception.md` Bash call WAS blocked by G-020 hook with correct message text.
- 2026-05-31T17:50Z: `fw task update T-1898 --type inception` ALSO blocked by G-020 (same gate, same message) — proving the gate is consistent at use-time.
- Operator: "i thought we hook gated this" — explicit confirmation that the operator's mental model includes create-time enforcement.

## Decisions

<!-- Architecture choice records here when made:
     ### YYYY-MM-DD — fix path
     - **Chose:** [bug-fix in create-task / spec-extension new gate / operator-memory-only]
     - **Why:** [rationale from spike evidence]
     - **Rejected:** [other options + why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-1899 go|no-go|defer --rationale "..." -->

## Updates

### 2026-05-31T17:52Z — task-created [agent autonomous via fw inception start]
- **Action:** Created via `fw inception start` after `fw work-on --type inception` failed to produce a separate task ID (it returned a recommendation message text instead).
- **Initial recommendation:** DEFER (honest — no spike evidence yet).
- **Why this exists:** operator explicitly instructed "file another inception for RCA and pickup to framework agent how this can still happen i thought we hook gated this" in response to T-1898 governance violation.
- **Sibling inception:** T-1898 (Vendored Agent Runner) — independent scope, same session.

### 2026-05-31T18:05Z — task file filled per inception discipline [agent autonomous]
- **Filled:** Problem Statement, Assumptions A1-A7, Exploration Plan S1-S8, Technical Constraints, Scope Fence, Go-NoGo Criteria, Recommendation Evidence section.
- **Research artifact:** `docs/reports/T-1899-g-020-task-create-time-gate-rca.md` (C-001).
- **Why filled now:** memory `[Inception task-file fills]` says Watchtower reads Problem/Recommendation/Go-NoGo from the TASK FILE not docs/reports/; filling both keeps the operator-facing review surface honest.
- **No spikes run yet** — awaiting operator review of the inception scope before doing S1-S8.

### 2026-05-31T17:55:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
