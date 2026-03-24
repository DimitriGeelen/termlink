# Framework Fix F4: /resume Loads Decisions from decisions.yaml

## Pickup Prompt for Framework Agent

### Problem

The resume agent (`agents/resume/resume.sh`, `cmd_status` function lines 90-242) synthesizes session state from handovers, git history, and task files. It does NOT load architectural decisions from `.context/project/decisions.yaml`. Fresh sessions start blind to project strategy.

### Real-world impact (T-258 incident)

After compaction + resume, the agent had zero awareness of the T-233 GO decision and its 5 architectural principles. It evaluated child tasks as isolated features rather than building blocks of an approved architecture.

### Files to modify

**`agents/resume/resume.sh`**
- `cmd_status` function, around line 227 (before "Recommendations" section)
- Add a block that loads and displays recent/active project decisions

### Proposed fix

Add a "Project Decisions" section to the resume output:

```bash
# After existing state synthesis, before recommendations
echo ""
echo "### Active Project Decisions"

DECISIONS_FILE="$PROJECT_ROOT/.context/project/decisions.yaml"
if [ -f "$DECISIONS_FILE" ]; then
    # Extract project-scoped decisions (not framework-seeded)
    # Use python3 for YAML parsing (already used elsewhere in the framework)
    python3 -c "
import yaml, sys
with open('$DECISIONS_FILE') as f:
    data = yaml.safe_load(f)
decisions = [d for d in data.get('decisions', []) if d.get('scope') == 'project']
if not decisions:
    print('No project-specific decisions recorded.')
else:
    for d in decisions:
        print(f\"- **{d['id']}** ({d.get('task','?')}): {d['decision']}\")
" 2>/dev/null || echo "No project decisions found."
else
    echo "No decisions.yaml found."
fi
```

**Also update the `/resume` skill** (if it exists as a Claude Code skill in CLAUDE.md or `.claude/`):
- The skill's Step 2 template should include a "### Project Decisions" section
- Pull from the same `decisions.yaml` source

### Acceptance criteria

- [ ] `fw resume status` includes "Active Project Decisions" section
- [ ] Project-scoped decisions are listed with ID, task ref, and decision text
- [ ] Framework-seeded decisions (scope: universal) are excluded
- [ ] Empty project decisions show "No project-specific decisions recorded"
- [ ] `/resume` skill output includes project decisions if any exist

### Test commands

```bash
# Run resume
fw resume status 2>&1 | grep -q "Project Decisions" && echo "PASS" || echo "FAIL"
# Verify decisions appear
fw resume status 2>&1 | grep -q "D-004\|specialist" && echo "PASS (has decisions)" || echo "INFO (no decisions)"
```
