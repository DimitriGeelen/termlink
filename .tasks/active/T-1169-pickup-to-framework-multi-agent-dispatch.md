---
id: T-1169
name: "Pickup to framework: multi-agent dispatch safety model (isolation + coordination for parallel agent work)"
description: >
  Formulate and deliver an inception-pickup proposal to the framework: a structural safety model for dispatching multiple agents to work concurrently on a single repo. Today's dispatch primitive (termlink_dispatch / fw dispatch) is solid for investigation + scope-writing but unsafe for concurrent code edits — no worktree isolation, no touches/conflicts metadata on tasks, no dispatch-gate that refuses unsafe parallel runs. Proposal composes on T-789 (worktree isolation) + T-914/T-916 (dispatch reliability) + T-1155 bus decision. Not asking framework to implement — asking them to open an inception that scopes the primitives.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [pickup, framework, dispatch, multi-agent, worktree, T-789, T-1155]
components: []
related_tasks: []
created: 2026-04-20T18:56:49Z
last_update: 2026-04-20T18:56:59Z
date_finished: null
---

# T-1169: Pickup to framework: multi-agent dispatch safety model (isolation + coordination for parallel agent work)

## Problem Statement

Today's framework dispatch primitive (`termlink_dispatch` MCP tool, `fw dispatch`, `termlink dispatch` CLI) is solid for **investigation** and **independent-file scope-writing** but **unsafe for concurrent code edits on a shared repo**. We hit this immediately when trying to parallelize the T-1155 bus build (T-1158 + T-1159 both touch workspace `Cargo.toml` → merge-conflict territory). The current framework has no structural primitive that makes "dispatch 3 workers to work on 3 tasks" safe when those tasks edit the same tree.

**This is a meta-task:** formulate and deliver a pickup envelope to the framework inception queue. Not asking framework to implement — asking them to OPEN an inception that scopes the primitives.

## Proposed scope for framework inception

The pickup envelope proposes exploring:

1. **Worktree isolation primitive** — `fw worktree spawn T-XXX` creates `.worktrees/T-XXX/` from HEAD, registers it with the task frontmatter, auto-cleans on completion. Composes on T-789 (captured, termlink-side).
2. **Task parallelism metadata** — frontmatter fields `touches: [paths]` and `parallelism_class: scope-only | file-isolated | worktree-required | serial`. Lets dispatch refuse unsafe parallel runs.
3. **Dispatch gate** — `fw dispatch T-XXX` checks class + active-worker table; refuses if `worktree-required` but no worktree exists, or if `touches` overlaps any running worker.
4. **Result reconciliation** — extend `fw bus` with a `parallel-merge` mode that flags overlapping edits when consolidating worker outputs.
5. **Bootstrap note** — framework should be aware that building this *before* T-1155 bus lands means using event.broadcast/inbox for worker coordination (fine, but circular when T-1155 delivers).

## Scope Fence

**IN scope for THIS termlink task:**
- Draft the pickup envelope YAML.
- Deliver to framework inbox via termlink (belt-and-suspenders: `termlink file send` + direct filesystem drop given PL-011).
- Verify framework pickup processor either picked it up (→ moved to `processed/`) or it's sitting in `inbox/`.

**OUT of scope here:**
- Implementing worktree/parallelism primitives locally — framework's work if they GO.
- Running the actual multi-agent dispatch experiment — blocked on the primitives.

## Acceptance Criteria

### Agent
- [ ] Pickup envelope YAML drafted at `/tmp/P-T-1169-framework-dispatch-safety.yaml`, valid YAML, includes summary + detail + proposed scope + acceptance criteria
- [ ] Envelope delivered to framework inbox via `termlink file send` to framework-agent session (with ok:true observed)
- [ ] Envelope present in `/opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/` OR already moved to `processed/` (belt-and-suspenders direct drop if file send didn't deliver)
- [ ] Framework-side auto-created task visible (check `/opt/999-Agentic-Engineering-Framework/.tasks/active/` for a T-13XX referencing T-1169 after processor cycle)

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

python3 -c "import yaml; yaml.safe_load(open('/tmp/P-T-1169-framework-dispatch-safety.yaml'))"
test -f /opt/999-Agentic-Engineering-Framework/.context/pickup/inbox/P-T-1169-framework-dispatch-safety.yaml || test -f /opt/999-Agentic-Engineering-Framework/.context/pickup/processed/P-T-1169-framework-dispatch-safety.yaml
grep -q "multi-agent dispatch safety" /tmp/P-T-1169-framework-dispatch-safety.yaml

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
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

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-20T18:56:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
