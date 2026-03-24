# T-258: Context Amnesia Root Cause Analysis

## Incident

Session S-2026-0324 NO-GO'd 5 child tasks of T-233 (the project's most important architectural decision) because the incoming agent had no access to the architectural vision. The agent evaluated each task as an isolated feature ("is this needed today?") rather than as a building block of an approved layered architecture.

## Impact

- T-239 (route cache), T-240 (negotiation), T-241 (template cache), T-242 (supervision), T-256 (push messaging) — all incorrectly closed as NO-GO
- Research artifacts were created but conclusions were wrong
- User had to intervene and question decisions

## Root Cause: 5 Structural Gaps

### Gap 1: No episodic for T-233 (CRITICAL)

T-233 was `owner: human`. The sovereignty gate (R-033) prevented agent completion. The handover said "close with `--force`" but nobody did. Without formal completion, `update-task.sh` never triggered `generate-episodic`.

**Result:** The project's most important inception has no episodic summary. New sessions cannot discover it through the normal context loading path.

### Gap 2: No auto-promotion of decisions to `decisions.yaml` (CRITICAL)

T-233's task file has 5 architectural decisions in `## Decisions`. The episodic generator parses these but only stores them in the episodic file — never promotes to `decisions.yaml`. No auto-trigger exists for `fw context add-decision`.

**Result:** `decisions.yaml` has 173 lines of framework-seeded universals, zero project-specific decisions despite 233+ completed tasks. Decisions are write-only — captured but never queryable.

### Gap 3: Handovers are narrative, not structured (HIGH)

The handover said "T-239, T-240, T-241, T-242 — captured, needs inception" with no context about how they fit together. A new session sees "template caching" as an isolated feature, not as Layer 2 of a 3-layer architecture.

**Result:** Architectural relationships between tasks are invisible in handover prose.

### Gap 4: `/resume` doesn't load decisions (HIGH)

The resume skill reads "Where We Are" and "Suggested First Action". It does NOT load decisions from `decisions.yaml`, episodic summaries, or architectural patterns.

**Result:** Fresh sessions start blind to project strategy.

### Gap 5: No validation on handover transfer (MEDIUM)

When T-233 completed with 23 research artifacts and 5 architectural decisions, no validation asked: "Is this an inception with GO? Have decisions been recorded in framework memory?"

**Result:** The framework is blind to whether its memory systems actually preserved critical information.

## Immediate Fixes Applied (This Project)

| Fix | What | File |
|-----|------|------|
| A | Generated T-233 episodic manually | `.context/episodic/T-233.yaml` |
| B | Captured D-004 through D-008 in decisions.yaml | `.context/project/decisions.yaml` |
| C | Saved vision to Claude memory | `memory/project_t233_orchestration_vision.md` |
| D | Added feedback memory | `memory/feedback_no_unilateral_nogo.md` |

## Structural Fixes Needed (Framework Repo)

| Fix | Gap | Description |
|-----|-----|-------------|
| F1 | Gap 1 | Allow episodic generation for human-owned completed tasks (sovereignty gate shouldn't block mechanical capture) |
| F2 | Gap 2 | Auto-promote decisions from task `## Decisions` to `decisions.yaml` on episodic generation |
| F3 | Gap 3 | Add structured `architectural_context` section to handover template with task relationship metadata |
| F4 | Gap 4 | `/resume` loads top decisions from `decisions.yaml` into session context |
| F5 | Gap 5 | Handover validation: warn if inception GO tasks have no episodic or decisions captured |

## Behavioral Fix

Agent must never make unilateral GO/NO-GO decisions. "Proceed" delegates initiative, not authority. For child tasks of approved GO inceptions, the default assumption is GO unless evidence shows the parent design was wrong.
