# Framework Pickup Prompt — Fix `declare -A` macOS Bash 3.2 Incompatibility

> Paste everything below the line into a Claude Code session in the framework project.

---

## Bug Report from TermLink project (010-termlink)

macOS ships with **bash 3.2** which does not support `declare -A` (associative arrays, bash 4+ only). Three framework files use `declare -A`, causing errors on every macOS run.

### Observed errors

```
# On task completion (fw task update T-XXX --status work-completed):
/usr/local/opt/agentic-fw/libexec/agents/task-create/update-task.sh: line 599: declare: -A: invalid option
declare: usage: declare [-afFirtx] [-p] [name[=value] ...]

# On audit (fw audit) and pre-push hook:
/usr/local/opt/agentic-fw/libexec/agents/audit/audit.sh: line 2858: declare: -A: invalid option
/usr/local/opt/agentic-fw/libexec/agents/audit/audit.sh: line 2864: Uncommitted changes present: syntax error in expression
```

### Impact

- **update-task.sh**: Auto-populate components feature (T-224) silently fails on macOS. Non-blocking but feature is dead.
- **audit.sh**: Trend analysis section crashes. Audit completes but trend data is lost.
- **diagnose.sh**: Healing classifier fails entirely on macOS. Blocks `fw healing diagnose`.

### Affected files and lines

#### 1. `agents/task-create/update-task.sh` — line 599

```bash
# Current (broken on macOS):
declare -A LOC_TO_ID
for card in "$FABRIC_DIR"/*.yaml; do
    [ -f "$card" ] || continue
    c_loc=$(grep "^location:" "$card" 2>/dev/null | sed 's/^location:[[:space:]]*//' | head -1)
    c_id=$(grep "^id:" "$card" 2>/dev/null | sed 's/^id:[[:space:]]*//' | head -1)
    if [ -n "$c_loc" ] && [ -n "$c_id" ]; then
        LOC_TO_ID["$c_loc"]="$c_id"
    fi
done
# Later used as: ${LOC_TO_ID[$path]:-}
```

#### 2. `agents/audit/audit.sh` — line 2775

```bash
# Current (broken on macOS):
declare -A issue_counts
for audit_file in "${past_audits[@]}"; do
    while IFS= read -r line; do
        if [[ "$line" =~ ^[[:space:]]+check:[[:space:]]* ]]; then
            check_name=$(echo "$line" | sed 's/.*check: "//' | sed 's/"$//')
            issue_counts["$check_name"]=$((${issue_counts["$check_name"]:-0} + 1))
        fi
    done < <(grep -A1 "level: WARN\|level: FAIL" "$audit_file" 2>/dev/null)
done
# Later iterated: for check in "${!issue_counts[@]}"
```

#### 3. `agents/healing/lib/diagnose.sh` — line 9

```bash
# Current (broken on macOS):
declare -A FAILURE_TYPES
FAILURE_TYPES[dependency]="dependency|package|module|..."
FAILURE_TYPES[external]="api|service|network|..."
FAILURE_TYPES[environment]="environment|config|..."
FAILURE_TYPES[design]="design|architecture|..."
FAILURE_TYPES[code]="error|exception|bug|..."
# Later used as: ${FAILURE_TYPES[$type]}
```

### Suggested fix pattern

Replace `declare -A` with POSIX-compatible key-value lookup using parallel arrays or a lookup function:

```bash
# Pattern A: Parallel arrays (simplest, good for small maps)
LOC_TO_ID_KEYS=()
LOC_TO_ID_VALS=()
# Store:
LOC_TO_ID_KEYS+=("$c_loc")
LOC_TO_ID_VALS+=("$c_id")
# Lookup:
_assoc_get() {
    local -n keys=$1 vals=$2
    local needle="$3"
    for i in "${!keys[@]}"; do
        if [ "${keys[$i]}" = "$needle" ]; then
            echo "${vals[$i]}"
            return 0
        fi
    done
    return 1
}
result=$(_assoc_get LOC_TO_ID_KEYS LOC_TO_ID_VALS "$path")

# Pattern B: Temp file lookup (better for large maps)
LOOKUP_FILE=$(mktemp)
trap "rm -f $LOOKUP_FILE" EXIT
# Store:
echo "$c_loc=$c_id" >> "$LOOKUP_FILE"
# Lookup:
result=$(grep "^${path}=" "$LOOKUP_FILE" | head -1 | cut -d= -f2-)
```

**Note:** The framework's existing episodic memory already flagged this — see `T-028.yaml`:
> alternatives_rejected: ["Reorder declare -A (fragile, bash-version-dependent)"]

So the framework team was already aware associative arrays are fragile. The fix now is to eliminate them entirely.

### Also investigate

- Search for any other `declare -A` usage: `grep -rn "declare -A" agents/ bin/`
- The `date -d` Linux-only syntax is a separate macOS compatibility issue (affects `episodic.sh` line 176) — same class of bug, different fix
- Consider adding a macOS CI check or a `fw doctor` bash-version warning

### How to verify the fix

```bash
# On macOS (bash 3.2):
fw task update T-XXX --status work-completed  # No declare -A error
fw audit                                       # Trend analysis works
fw healing diagnose T-XXX                      # Classifier works
```
