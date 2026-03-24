# Framework Fix F5: Handover Validation — Warn if Inception GO Has No Episodic

## Pickup Prompt for Framework Agent

### Problem

When a handover is generated, it checks for episodic completeness (lines 215-271 in `handover.sh`) but does NOT validate that inception tasks with GO decisions have their architectural context preserved. An inception can complete with GO, all research artifacts committed, but zero framework memory (no episodic, no decisions in decisions.yaml).

### Real-world impact (T-258 incident)

T-233 completed with GO, 23 research artifacts, and 5 architectural decisions. The handover listed it as "awaiting human close" but never warned that the most important architectural decision in the project had no episodic summary and no entries in decisions.yaml.

### Files to modify

**`agents/handover/handover.sh`**
- Lines 215-271: Episodic completeness gate (Step 1.5)
- Lines 375-392: Inception section in template

### Proposed fix

Add an inception-specific validation after the existing episodic completeness check:

```bash
# After existing episodic completeness gate (line 271)
# Check inception tasks with GO decisions

INCEPTION_WARNINGS=""
for task_file in "$PROJECT_ROOT/.tasks/completed/"*.md "$PROJECT_ROOT/.tasks/active/"*.md; do
    [ -f "$task_file" ] || continue

    # Check if inception workflow
    workflow=$(grep '^workflow_type:' "$task_file" | head -1 | awk '{print $2}')
    [ "$workflow" = "inception" ] || continue

    # Check if GO decision exists in file
    if grep -q "Decision.*GO\|decision.*go" "$task_file"; then
        task_id=$(grep '^id:' "$task_file" | head -1 | awk '{print $2}')

        # Check episodic exists
        if [ ! -f "$PROJECT_ROOT/.context/episodic/$task_id.yaml" ]; then
            INCEPTION_WARNINGS="${INCEPTION_WARNINGS}\n- **$task_id**: Inception decided GO but NO episodic summary exists"
        fi

        # Check decisions.yaml has entries for this task
        if ! grep -q "$task_id" "$PROJECT_ROOT/.context/project/decisions.yaml" 2>/dev/null; then
            INCEPTION_WARNINGS="${INCEPTION_WARNINGS}\n- **$task_id**: Inception decided GO but NO decisions in decisions.yaml"
        fi
    fi
done

if [ -n "$INCEPTION_WARNINGS" ]; then
    echo ""
    echo "## ⚠ Inception Memory Gaps"
    echo ""
    echo "The following inception tasks have GO decisions but missing framework memory:"
    echo -e "$INCEPTION_WARNINGS"
    echo ""
    echo "Fix: Run \`fw context generate-episodic T-XXX\` and \`fw context add-decision\` for each."
fi
```

### Acceptance criteria

- [ ] Handover generation warns when inception GO tasks lack episodic summaries
- [ ] Handover generation warns when inception GO tasks lack entries in decisions.yaml
- [ ] Warnings include the task ID and what's missing
- [ ] Warning suggests fix commands
- [ ] No false positives for inception tasks with NO-GO decisions

### Test commands

```bash
# Create inception task, record GO, but don't generate episodic
fw inception start "Test validation"
fw inception decide T-XXX go --rationale "test"
# Generate handover
fw handover
# Verify warning appears
grep -q "Inception Memory Gaps\|missing framework memory" .context/handovers/LATEST.md && echo "PASS" || echo "FAIL"
```
