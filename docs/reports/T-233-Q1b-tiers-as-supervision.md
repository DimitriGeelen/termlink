# T-233 Q1b: Enforcement Tiers as Supervision Model

## Question
Can the existing Enforcement Tier model (Tier 0–3) be extended to compute supervision levels for specialist scripts?

## Existing Tier Model (from CLAUDE.md)

| Tier | Description | Bypass | Current Use |
|------|------------|--------|-------------|
| 0 | Consequential actions (force push, hard reset, rm -rf, DROP TABLE) | Human approval via `fw tier0 approve` | PreToolUse hook on Bash |
| 1 | All standard operations (default) | Create task or escalate to Tier 2 | PreToolUse hook on Write/Edit |
| 2 | Human situational authorization | Single-use, mandatory logging | Partial (bypass log) |
| 3 | Pre-approved categories (health checks, status queries) | Configured | Spec only |

## Mapping Tiers to Supervision Levels

The tier model maps naturally to supervision intensity:

| Tier | Supervision Level | Behavior |
|------|------------------|----------|
| 3 (read-only) | **None** — fire-and-forget | No output review, no approval, parallel-safe. Examples: `fw doctor`, `git status`, `fw metrics` |
| 1 (standard write) | **Post-hoc review** — run, then verify | Script executes autonomously; orchestrator checks exit code + output summary. Rollback on failure. Examples: `fw git commit`, file edits within task scope |
| 2 (situational) | **Pre-approved with logging** — run with audit trail | Orchestrator logs intent before execution, captures full output. Human can review asynchronously. Examples: one-off config changes, deploying to staging |
| 0 (destructive) | **Full supervision + human gate** — blocked until approved | Script cannot execute without explicit human approval. Orchestrator presents what will happen, waits for `fw tier0 approve`. Examples: force push, database migration, production deploy |

## Classifying Scripts into Tiers

A script's tier is **computed from its action manifest**, not declared by the author:

1. **Static analysis**: Parse the script for action verbs — `rm`, `git push --force`, `DROP`, `DELETE` → Tier 0. `write`, `edit`, `commit` → Tier 1. `read`, `list`, `status` → Tier 3.
2. **Capability declaration**: Scripts declare their maximum action level in a frontmatter block (e.g., `tier: 1`). The framework validates this against actual tool calls at runtime.
3. **Runtime escalation**: If a Tier 1 script attempts a Tier 0 action, the framework intercepts and blocks (existing PreToolUse hook pattern).

## Cross-Tier Scripts

Scripts that read AND write cross tier boundaries. Resolution:

1. **Highest-action-wins**: A script that reads config (Tier 3) then writes a file (Tier 1) is classified as Tier 1. The supervision level matches the most consequential action.
2. **Phase separation**: For scripts with distinct read-then-write phases, the orchestrator can apply Tier 3 supervision during the read phase and escalate to Tier 1 for the write phase. This requires the script to declare phase boundaries.
3. **Composite decomposition**: Prefer splitting cross-tier scripts into a Tier 3 investigator + Tier 1 actor. The orchestrator reviews the investigator's output before dispatching the actor. This aligns with the existing "parallel investigation → sequential action" dispatch pattern.

## Verdict

**Yes — the tier model extends cleanly.** The mapping is mechanical: tier number determines supervision intensity, not agent judgment. Key advantages:

- **No new abstraction**: Reuses existing enforcement infrastructure (PreToolUse hooks, tier0 approve flow)
- **Computed, not guessed**: Tier is derived from action manifest + runtime validation
- **Graduated response**: Matches the Error Escalation Ladder philosophy — proportional response to risk

**Gap**: Tier 2 (situational authorization) is currently "spec only" in the framework. Extending it to supervision would require implementing the single-use authorization + mandatory logging flow that's currently just documented.
