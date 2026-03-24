# T-242: Supervision Integration Inception

## Problem Statement

Should supervision be extended beyond binary enforcement tiers (block/allow) to include trust scoring and graduated autonomy for specialist agents?

## Research Questions

### Q1: What Does Current Enforcement Already Provide?
### Q2: Is Trust Scoring Data Available?
### Q3: What Would Supervision Change in Practice?

## Findings

### Q1 Findings — Enforcement Tiers Are Bypassed for Mesh Workers

**Critical discovery:** Mesh workers run with `--dangerously-skip-permissions`, which **disables all framework hooks** including Tier 0 (destructive gate), Tier 1 (task gate), and budget gate. This is a deliberate decision from T-119, not an oversight.

| Mechanism | Main session | Mesh worker |
|-----------|-------------|-------------|
| Tier 0 (destructive gate) | Active | **Bypassed** |
| Tier 1 (task gate) | Active | **Bypassed** |
| Budget gate | Active | **Bypassed** |
| Error watchdog | Active | **Bypassed** |

**Current mitigations (behavioral, not structural):**
- Ephemeral sessions (`--no-session-persistence`)
- Timeout enforcement (default 120s via TermLink)
- Optional worktree isolation (`--isolate` flag)
- Prompt discipline (orchestrator controls what workers are told to do)

**The gap:** No mechanism prevents a mesh worker from running destructive commands during its lifetime. The framework's structural enforcement philosophy ("hooks enforce, not agent discipline") does not extend to mesh workers.

### Q2 Findings — No Trust Scoring Data Exists

| Data source | Status | Evidence |
|-------------|--------|----------|
| Script run/fail history | **Doesn't exist** | No runtime tracking for any script |
| Healing patterns (project-specific) | **Zero entries** | 13 seeded framework patterns, 0 project-specific |
| Healing loop usage | **Never invoked** | Zero git commits reference `fw healing diagnose` |
| Fabric card trust metadata | **Doesn't exist** | Cards have structural topology only |
| `status: issues` workflow | **Never used** | Zero tasks transitioned through issues status |

**The trust formula `f(script_maturity, context_familiarity, blast_radius)` has no inputs:**
- `script_maturity`: No run counts, no failure diversity data
- `context_familiarity`: No per-project context history
- `blast_radius`: Available from fabric (the only functional input)

### Q3 Findings — Scenarios Where Trust Scoring Would Change Behavior

**Scenario A: New script, first run**
- Current: Runs with full permissions (bypass)
- With trust: Would run under Tier 1 supervision (post-hoc review)
- **Benefit: Marginal.** The orchestrator already reviews results via collect.

**Scenario B: Script that previously failed**
- Current: Runs identically to any other script
- With trust: Lower maturity score → higher supervision
- **Benefit: Real, but requires healing loop to be operational first.**

**Scenario C: Script touching high-blast-radius components**
- Current: No awareness of blast radius
- With trust: Fabric-informed supervision escalation
- **Benefit: Real and implementable today** (fabric blast-radius is functional)

**Scenario D: Graduated autonomy for proven scripts**
- Current: All scripts get same treatment (full bypass)
- With trust: Mature scripts could run with less oversight
- **Benefit: Negative — current bypass already gives maximum autonomy.** Trust scoring would ADD restrictions to the default, not remove them.

## Assumption Validation

| Assumption | Status | Evidence |
|------------|--------|----------|
| A1: Tiers insufficient for multi-agent supervision | **PARTIALLY VALID** | Tiers are bypassed entirely, not insufficient — they don't apply at all |
| A2: Script maturity can be measured | **UNVALIDATABLE** | No run/fail history exists; healing loop never used |
| A3: Healing loop provides usable data | **DISPROVED** | Zero project-specific patterns, zero invocations in 233+ tasks |
| A4: Fabric cards right for trust metadata | **VALID but premature** | Blast radius is functional; trust overlay needs runtime data that doesn't exist |
| A5: Graduated autonomy > binary block/allow | **INVERTED** | Current default is "allow everything"; graduation would ADD restrictions |

## Synthesis

### Decision: NO-GO on trust-based supervision system

**The fundamental inversion:** T-242 was designed assuming mesh workers operate under enforcement tiers and need graduation to MORE autonomy. Reality is the opposite — mesh workers already have MAXIMUM autonomy (full bypass). A trust system would be imposing NEW restrictions, not relaxing existing ones.

**Three prerequisites don't exist:**
1. **Runtime data** — No script execution history to compute maturity scores
2. **Healing loop** — Never used in 233+ tasks; can't feed trust assessments
3. **Specialist agents** — No persistent specialists exist to supervise over time

**What IS a real gap:** Mesh workers having zero governance is a documented, deliberate choice (T-119) but represents a structural enforcement gap. This is a simpler problem than trust scoring:

**Recommended alternative:** Instead of the full trust-scoring system, consider a lightweight **capability baseline** for mesh workers:
- Workers declare what tools they need (read-only, write-to-worktree, full)
- Orchestrator enforces capability scope via prompt + optional hook re-enablement
- No maturity scoring, no healing integration, no fabric trust overlay
- Signal to revisit trust scoring: healing loop activation + 50+ mesh worker dispatches with run/fail data
