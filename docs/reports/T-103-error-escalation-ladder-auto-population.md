# T-103: Inception — Error Escalation Ladder Auto-Population via JSONL

> Created: 2026-03-12 | Status: Research complete — GO/NO-GO pending

## Problem Statement

The Error Escalation Ladder (A→B→C→D) is manually populated via `fw healing resolve`.
Can we extract tool errors from the JSONL transcript automatically and feed them into
the ladder — making pattern detection proactive?

## Findings

### JSONL error format is not what we expected

The JSONL transcript does NOT contain `tool_result` events with `is_error: true`.
Event types present: `assistant`, `user`, `progress`, `file-history-snapshot`,
`system`, `queue-operation`, `last-prompt`.

**Actual error structure (from session):**
```json
{
  "type": "assistant",
  "error": "authentication_failed",
  "isApiErrorMessage": true,
  "timestamp": "2026-03-11T16:23:41.059Z",
  "message": {
    "content": [{ "type": "text", "text": "API Error: 401 {...}" }]
  }
}
```

**Available fields:** error type code, timestamp, error text (as prose in content)
**Missing:** tool name, exit code, operation context, retry count, structured categorization

### Signal quality for auto-classification

| Escalation level | Required signal | Available? |
|-----------------|----------------|-----------|
| A — don't repeat same failure | Error type + context | Partial (type only) |
| B — improve technique | Cluster of similar errors | No — needs cross-session store |
| C — improve tooling | Systematic failure in one tool | No — no tool identity on errors |
| D — change ways of working | Session-wide systemic pattern | No — no aggregate view |

**Count:** 1 error event across 1,304 total events in current session. Error rate is low.

### Feasibility assessment

Auto-population **is feasible** but requires infrastructure not yet in place:

1. **Hook enrichment:** `budget-gate.sh`, `check-tier0.sh`, `checkpoint.sh` would need
   to write structured error records (tool name, exit code, task context) that JSONL
   extraction can consume. Currently errors appear as prose.

2. **Cross-session store:** A/B/C/D classification requires seeing the same error
   across multiple sessions. That's T-104's job — T-103 depends on T-104 existing.

3. **Pattern deduplication:** Same error in 5 sessions → 1 pattern, not 5 entries.
   Needs a fingerprinting scheme (hash of tool + error type + context).

### Dependency graph

```
T-103 (auto-population) depends on:
  → T-104 (cross-session tool call store) — not built
  → Hook enrichment (structured error output) — not built
```

Neither dependency exists. T-103 is premature.

### Alternative: simpler near-term value

A lighter version is feasible now:
- `fw errors harvest` command reads current session JSONL
- Surfaces `isApiErrorMessage: true` events + progress hook failures
- Outputs a summary for human review (no auto-patching of patterns.yaml)
- Cost: ~1 day. Value: makes errors visible without requiring T-104.

## GO/NO-GO Recommendation

**DEFER** — premature. T-104 must exist first.

The full auto-population vision is sound but requires the cross-session tool call store
(T-104) as prerequisite. Building T-103 before T-104 means building the analysis
layer before the data layer.

**If the lightweight harvest command is appealing, open a separate scoped build task.**

## Options Explored

| Option | Description | Verdict |
|--------|-------------|---------|
| Full auto-population | Extract errors → classify → patch patterns.yaml | DEFER — needs T-104 |
| Harvest command | Read-only session error summary for human review | Feasible now, limited value |
| Hook enrichment first | Improve hooks to write structured errors, then extract | Correct order, 2-step |

## Open Questions

- Should T-104 explicitly scope error events as a first-class record type?
- Is the `fw errors harvest` lightweight alternative worth a separate build task?
