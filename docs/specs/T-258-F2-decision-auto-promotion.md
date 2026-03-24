# Framework Fix F2: Auto-Promote Decisions from Task Files to decisions.yaml

## Pickup Prompt for Framework Agent

### Problem

When a task completes and episodic generation runs, the `## Decisions` section from the task file is parsed and stored in the episodic YAML file (`agents/context/lib/episodic.sh` lines 121-133, 307-331). But decisions are NEVER promoted to `.context/project/decisions.yaml`. This means:

- `decisions.yaml` has 173 lines of framework-seeded universals, ZERO project-specific decisions
- Despite 233+ completed tasks with many containing decisions
- Decisions are captured but unreachable — they exist only in individual episodic files
- No session startup process reads individual episodics to discover decisions

### Real-world impact (T-258 incident)

T-233 had 5 architectural decisions (GO on specialist orchestration, deterministic-first execution, framework/TermLink separation, layered capability discovery, qualitative trust supervision). None were promoted to `decisions.yaml`. A new session couldn't discover them without reading the T-233 task file directly.

### Files to modify

**`agents/context/lib/episodic.sh`**
- Lines 307-331: After writing decisions to the episodic YAML, add a promotion step
- Reference: `agents/context/lib/decision.sh` lines 55-112 — the `add_decision()` function

### Proposed fix

After episodic generation writes the `decisions:` section to the episodic YAML, iterate over each decision and call the decision promotion path:

```bash
# Pseudocode — after episodic YAML is written
if [ "$has_decisions" = "true" ]; then
    # Extract each decision from the task file's ## Decisions section
    # For each decision found:
    #   - Generate next D-XXX id
    #   - Add to .context/project/decisions.yaml with:
    #     - decision: (the decision text)
    #     - scope: project
    #     - date: (task completion date)
    #     - task: T-XXX
    #     - rationale: (from the Why/Rationale field)
    # Skip if a decision with the same task ID already exists (idempotent)
fi
```

**Key design decisions:**
- **Idempotent:** If `decisions.yaml` already has an entry for this task ID, skip (don't duplicate)
- **Scope: project** — auto-promoted decisions are always project-scoped (not universal)
- **Parse format:** Task `## Decisions` uses markdown format with `### date — topic` headers. Extract the `Chose:` line as the decision text, `Why:` as rationale.

### Acceptance criteria

- [ ] When a task with `## Decisions` content completes, decisions are added to `decisions.yaml`
- [ ] Promotion is idempotent — running episodic generation twice doesn't duplicate
- [ ] Auto-promoted decisions have `scope: project` and reference the task ID
- [ ] Existing `decisions.yaml` format is preserved (framework-seeded entries untouched)

### Test commands

```bash
# Complete a task that has decisions
fw task update T-XXX --status work-completed
# Verify decisions were promoted
grep "T-XXX" .context/project/decisions.yaml && echo "PASS" || echo "FAIL"
```
