# Framework Fix F1: Allow Episodic Generation for Human-Owned Completed Tasks

## Pickup Prompt for Framework Agent

### Problem

When a human-owned task reaches `work-completed`, the sovereignty gate (R-033) in `update-task.sh` blocks agent-initiated completion. This also blocks episodic generation, because `generate-episodic` is only called AFTER successful completion (lines 154-156). Result: human-owned inception tasks with GO decisions never get episodic summaries — the most important architectural decisions in the project become invisible to future sessions.

### Real-world impact (T-258 incident)

T-233 (specialist agent orchestration, GO decision with 23 research artifacts and 5 architectural decisions) was completed as `owner: human`. No episodic was ever generated. A new session starting days later had no queryable memory of the architectural vision, and incorrectly NO-GO'd 5 child build tasks.

### Files to modify

**`agents/task-create/update-task.sh`**
- Lines 154-156: `generate-episodic` call (only reached after successful completion)
- Lines 201-217: Sovereignty gate (R-033) — blocks human task completion by agent

### Proposed fix

Decouple episodic generation from the sovereignty gate. Episodic generation is a MECHANICAL operation (summarize the task file), not an AUTHORITY operation (approve completion). The human's approval is the gate for completion; episodic capture should happen regardless.

**Option A (recommended):** When `--force` is used to complete a human-owned task, generate episodic BEFORE moving to completed. Currently `--force` bypasses the sovereignty gate but episodic still isn't generated because the flow jumps over it.

**Option B:** Add a separate command `fw context generate-episodic T-XXX` that can be called independently of task completion. This already exists via `context.sh generate-episodic` but isn't exposed through the update-task flow for human tasks.

**Option C:** Generate episodic on ANY task completion (including human-owned), just don't change the status without `--force`. The episodic is metadata about the task, not an approval action.

### Acceptance criteria

- [ ] When `fw task update T-XXX --status work-completed --force` is used on a human-owned task, episodic is generated
- [ ] Episodic generation for human tasks does not bypass the sovereignty gate — it just decouples from it
- [ ] Test: create a human-owned inception task, complete with `--force`, verify episodic exists

### Test commands

```bash
# Create test task
fw task create --name "Test episodic for human task" --type inception --owner human
# Add some content to the task file (simulate work)
# Complete with --force
fw task update T-XXX --status work-completed --force
# Verify episodic was generated
test -f .context/episodic/T-XXX.yaml && echo "PASS" || echo "FAIL"
```
