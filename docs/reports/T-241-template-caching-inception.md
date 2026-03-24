# T-241: Template Caching Inception

## Problem Statement

Is a 3-layer template caching system (agent-local → shared → specialist canonical) needed for agent-specialist collaboration, or are current dispatch patterns sufficient?

## Research Questions

### Q1: Evidence of Repeated Specialist Interactions
How many times has the same agent type interacted with the same specialist type? Is there repetition that caching would optimize?

### Q2: Current Template/Schema Handling
Does schema-in-prompt (T-257/T-128) already solve the problem template caching aims to address?

### Q3: Prerequisites — Do Specialists Exist?
What specialist infrastructure exists that template caching would cache FROM?

## Findings

### Q1 Findings — Zero Repeated Specialist Interactions

- Across 233+ completed tasks and episodic memory: **zero instances** of the same agent calling the same specialist across separate tasks
- T-233 inception dispatched 22 agents in 3 rounds, but these were one-shot research agents, not repeated specialist calls
- Role-specific specialists exist (T-063: reviewer, tester, documenter, git-committer, infrastructure) but are tested, not used in production workflows
- **No agent-specialist interaction pattern exists to cache**

### Q2 Findings — Schema-in-Prompt Already Solves It

- T-128 delivered `wrap_prompt()` in `agents/mesh/prompt-template.sh` — all mesh workers receive format instructions via prompt injection
- T-257 convention defines structured payloads: `task.completed` with `{task_id, summary, status, blob_path}`
- Protocol-level types in `events.rs` provide strict schema: `TaskProgress`, `TaskCompleted`, `TaskFailed`
- Schema-in-prompt is zero-latency, zero-infrastructure, and already operational
- T-240 inception confirmed: schema-in-prompt catches 85-90% of format issues at ~5% complexity

### Q3 Findings — No Specialist Infrastructure Exists

- `.context/specialists/` directory **does not exist**
- No persistent specialist agents are deployed — all agents are ephemeral (spawn → work → die)
- T-233 designed the full architecture (3-layer cache, negotiation protocol, specialist manifests) but **nothing was built**
- The entire template caching design assumes a specialist ecosystem that doesn't exist yet
- Building the cache before specialists exist violates YAGNI

## Assumption Validation

| Assumption | Status | Evidence |
|------------|--------|----------|
| A1: Agents interact with specialists repeatedly | **DISPROVED** | Zero repeated interactions in 233+ tasks |
| A2: Template formats change frequently enough for versioning | **UNTESTABLE** | No templates exist to change |
| A3: Per-agent variants differ meaningfully | **UNTESTABLE** | No per-agent variants exist |
| A4: Pull-on-miss is the right strategy | **VALID** but premature | Design is sound, nothing to pull from |
| A5: 5-use/0-correction threshold appropriate | **UNTESTABLE** | No usage data exists |

## Synthesis

### Decision: NO-GO on template caching

**Evidence:**
- A1 (repeated interactions exist): **DISPROVED** — the pattern template caching optimizes doesn't occur yet
- Schema-in-prompt (T-128 + T-257) already handles format expectations at zero infrastructure cost
- The specialist ecosystem this depends on (persistent agents, specialist manifests, negotiation protocol) doesn't exist
- T-240 NO-GO on negotiation protocol removes the primary use case for Layer 3 caching

**The T-233 Q2b design remains valid** — when persistent specialists emerge and repeated interactions become measurable, the 3-layer cache design can be implemented. The signal to revisit: >10 repeated agent-specialist interactions per session, or schema-in-prompt failure rate >10%.

**Relationship to other decisions:**
- T-240 (negotiation protocol): NO-GO — premature
- T-241 (template caching): NO-GO — premature, depends on specialists that don't exist
- Both share the same root cause: the specialist orchestration model from T-233 is well-designed but the project hasn't reached the scale where it's needed
