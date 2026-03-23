# T-233 Q2b: Bypass Mechanism Design

## Question
When should the orchestrator say "just do it locally" instead of orchestrating? What criteria make something bypass-eligible, and how does bypass relate to the trust/supervision model from Q1b?

## Core Insight: Bypass = Tier 3 Operationalized

Q1b mapped enforcement tiers to supervision levels. Tier 3 (pre-approved, read-only) was labeled "fire-and-forget" — no output review, no approval, parallel-safe. **Bypass IS Tier 3 made real.** The framework already has the concept; it just lacks the registry and the teaching mechanism.

## Bypass Eligibility Criteria

A command is bypass-eligible when ALL of:

1. **Low risk** — Tier 3 actions only (read-only, no mutations, no network writes). Maps directly to the Q1b supervision model: high trust + low risk = no supervision needed.
2. **Deterministic** — Same input produces same output. No side effects, no environmental dependencies that could silently change behavior. `fw doctor`, `git status`, `fw metrics` qualify. `curl` does not.
3. **Local script exists** — A concrete executable the agent can invoke without the orchestrator composing a command. The script IS the pre-approval — its existence in a known location with a known interface means someone already vetted it.
4. **Bounded output** — Result fits in agent context without summarization. Unbounded output (e.g., `git log` without `-n`) requires orchestrator truncation/summarization, defeating bypass.

## Mechanism: Pre-Approval Registry

```yaml
# .framework/bypass-registry.yaml (or in fabric cards)
bypass:
  - pattern: "fw doctor"
    tier: 3
    reason: "Read-only health check, bounded output"
  - pattern: "fw metrics"
    tier: 3
    reason: "Read-only status query"
  - pattern: "fw context status"
    tier: 3
    reason: "Read-only context query"
  - pattern: "fw fabric deps *"
    tier: 3
    reason: "Read-only dependency lookup"
  - pattern: "git status"
    tier: 3
    reason: "Read-only VCS query"
```

**Implementation path:** A PreToolUse hook (or dispatch.sh pre-check) matches the command against the registry. If matched → execute locally, skip orchestration queue. If not matched → normal dispatch.

## Teaching Agents: The Promotion Pattern

The orchestrator doesn't "teach" bypass — it **promotes** commands through evidence:

1. **Observation:** Command X has been orchestrated 5+ times with zero failures, zero escalations, Tier 3 classification.
2. **Proposal:** Orchestrator adds X to bypass-registry.yaml with evidence trail.
3. **Approval:** Human reviews (or auto-approves if trust threshold met from Q1b fabric cards).
4. **Learning:** Agent's next dispatch of X hits the registry and executes locally.

This mirrors the existing promotion pipeline (`fw promote`) — learnings that prove stable become structural rules. Bypass eligibility is a learnable property, not a static declaration.

## Relationship to Q1b Trust Model

| Q1b Concept | Bypass Implication |
|-------------|-------------------|
| Tier 3 (fire-and-forget) | = bypass-eligible by definition |
| Tier 1 (post-hoc review) | NOT bypass-eligible — needs output verification |
| Fabric trust cards | Could store per-script bypass flag alongside trust score |
| Healing loop | Failed bypass commands get de-promoted back to orchestrated |

**Key constraint from Q1b evidence:** Tier 3 is currently "spec only" — no implementation exists. Bypass mechanism IS the Tier 3 implementation. Building bypass and building Tier 3 are the same work.

## Anti-Patterns to Avoid

- **Bypass by default:** New/unknown commands must be orchestrated until proven safe. Default is supervised, not bypassed.
- **Bypass without logging:** Even fire-and-forget should log invocation counts for the promotion pipeline. Silent bypass creates blind spots.
- **Agent self-promotion:** Agents cannot add commands to the bypass registry. Only the orchestrator (or human) can promote, preserving the Authority Model.

## Verdict

Bypass is not a new mechanism — it's the missing Tier 3 implementation plus a promotion pipeline for commands to earn bypass status through track record. The infrastructure pieces exist (hooks, fabric cards, promotion system). The gap is the registry and the PreToolUse check that consults it.
