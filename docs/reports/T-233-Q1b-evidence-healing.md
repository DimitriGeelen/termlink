# T-233 Evidence: Healing Loop Effectiveness

## Verdict: PARTIALLY WORKING — patterns recorded, feedback loop broken

The Healing Loop exists as infrastructure but is not functioning as a closed feedback loop. Patterns are captured; they are never reused.

## Evidence

### 1. Patterns (`.context/project/patterns.yaml`)

- **13 universal patterns** (seeded from framework): FP-003 through FP-007, SP-001 through SP-004, AF-001, WP-001, WP-002
- **0 project-specific patterns** — no `scope: project` entries exist despite 210+ completed tasks
- All patterns were `inherited_from: framework`, none added via `fw healing resolve` or `fw context add-pattern`

**Assessment:** The seeded patterns are real and well-written (each has origin task, mitigation, context). But no organic pattern has ever been recorded in this project.

### 2. Learnings (`.context/project/learnings.yaml`)

- **16 learnings** (L-001 through L-016), spanning T-043 to T-170
- Topics: fabric init, shell escaping, CLI field naming, ENV_LOCK isolation, PTY handling, Rust unsafe, context window thresholds
- All added via `fw context add-learning` — the learning capture path works

**Assessment:** Learnings are actively recorded and project-specific. This is the healthy part of the system.

### 3. Git History

- **Zero commits** reference `healing`, `fw healing diagnose`, or `fw healing resolve`
- **Zero commits** reference the `issues` task status
- No task file was found that transitioned through `status: issues` → healing diagnosis → resolution

**Assessment:** The healing agent (`agents/healing/`) has never been invoked in this project's 210-task history.

### 4. Episodic Memory (`.context/episodic/`, 210 files)

- **3 episodic files** mention healing: T-097, T-103, T-113
- T-097: Updated FP-004 pattern manually (not via healing agent)
- T-103: Inception exploring auto-population of the escalation ladder — concluded with DEFER
- T-113: Built `analyze-errors.py` classifier — tooling to feed the ladder, but no evidence it was used afterward
- **1 episodic file** (T-097) references a pattern ID (FP-004) — the only evidence of pattern awareness

**Assessment:** The escalation ladder concept is discussed but never operationalized. Tooling was built (T-113) but never exercised.

### 5. Pattern Reuse (the feedback loop)

- **Zero evidence** of a pattern from an early task being consulted or applied in a later task
- No task file contains language like "per FP-003" or "known pattern" or "similar to previous"
- The `fw healing diagnose` command (which searches patterns.yaml) was never invoked

**Assessment:** The feedback loop — the core value proposition of the healing system — is completely absent.

## Summary Table

| Aspect | Status | Evidence |
|--------|--------|----------|
| Pattern recording infrastructure | Working | 13 seeded patterns, YAML well-formed |
| Learning capture | **Active** | 16 project-specific learnings |
| Healing agent invocation | **Never used** | 0 git commits, 0 task references |
| `status: issues` workflow | **Never used** | 0 tasks transitioned through issues |
| Pattern reuse (feedback loop) | **Broken** | 0 references to patterns in later tasks |
| Escalation ladder (A→D) | Designed, not practiced | T-103 DEFER, T-113 built tooling, never run |

## Implication for T-233

The healing loop as a supervision mechanism for specialist agents would require significant activation work. The infrastructure exists but has never been battle-tested. Relying on it for automated specialist supervision would be premature — it hasn't proven it works even for human-driven workflows.
