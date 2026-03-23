# T-233 Q1b: Healing Loop as Supervision Mechanism

## Question

Could the existing Healing Loop (`agents/healing/`) serve as the supervision mechanism for specialist agents? Can `patterns.yaml` function as the trust evidence ledger?

## Analysis

### Current Healing Loop Architecture

The healing loop operates in four phases: **Classify** (keyword-scored failure typing across 5 categories), **Lookup** (semantic pattern matching via `fw ask`), **Suggest** (Error Escalation Ladder: A→D), and **Log** (resolution recorded as FP-XXX pattern + L-XXX learning). It is task-scoped — triggered when a task enters `issues` status, resolved when the fix is applied.

### How Healing Maps to Supervision

The mapping is natural:

| Supervision Need | Healing Equivalent |
|---|---|
| Detect script failure | Task enters `issues` status (already triggers `diagnose`) |
| Classify severity | `classify_failure()` — 5 failure types with keyword scoring |
| Suggest remediation | Escalation Ladder (A: don't repeat → D: change ways of working) |
| Record outcome | `resolve` writes FP-XXX to `patterns.yaml` + L-XXX to `learnings.yaml` |
| Build trust over time | Pattern accumulation = trust evidence |

### Patterns.yaml as Trust Ledger

`patterns.yaml` already has typed sections: `failure_patterns`, `success_patterns`, `antifragile_patterns`, `workflow_patterns`. Each pattern carries `id`, `origin_task`, `mitigation`, and `scope`. This structure naturally supports trust scoring:

- **Trust score** = `(resolved_failures + success_patterns) / total_failures` for a given script/agent
- **Maturity signal** = scripts with many resolved FP-XXX entries have known failure modes with documented mitigations
- **Risk signal** = scripts with unresolved or recurring patterns need tighter supervision

### Wiring Proposal

1. **Tag patterns by agent/script**: Add `agent_id` or `script_id` field to pattern entries. Currently patterns are tagged only by `origin_task`.
2. **Add a `trust-score` subcommand**: `healing.sh trust-score <agent-id>` — computes resolved/total ratio from tagged patterns.
3. **Supervision ramp-down**: Map trust scores to supervision levels:
   - Score 0.0–0.3 (new/failing): Full supervision — human approves every action
   - Score 0.3–0.7 (learning): Partial — human approves destructive actions only
   - Score 0.7–1.0 (mature): Autonomous — post-hoc audit only
4. **Context-dependent reset**: When `scope: project` patterns don't transfer to new projects, trust resets. Only `scope: universal` patterns carry over.

### Gaps

1. **No agent-level identity in patterns**: Patterns are task-scoped (`origin_task`), not agent-scoped. Adding `agent_id` is simple but breaks the current assumption that patterns are project-level, not agent-level.
2. **No frequency tracking**: A pattern resolved once vs. resolved 50 times looks the same. Trust needs occurrence counts.
3. **Passive, not active**: The healing loop is reactive (triggered on failure). Supervision needs proactive checkpoints — the loop doesn't currently intercept actions *before* they happen.
4. **No decay**: Trust should decay without recent evidence. A script that hasn't failed in 6 months might have untested code paths, not proven reliability.

### Verdict

The healing loop is a **strong foundation** but not sufficient alone. It covers the *reactive* half of supervision (failure → diagnosis → remediation → learning). The *proactive* half (pre-execution gates, checkpoint frequency, approval routing) needs a separate mechanism that *reads* the trust data the healing loop produces.

**Recommended architecture**: Healing loop as trust evidence *producer*. A separate supervision controller as trust evidence *consumer* that sets gate thresholds based on the pattern database.
