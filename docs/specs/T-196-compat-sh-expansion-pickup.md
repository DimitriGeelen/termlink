# T-196 Pickup Prompt: Expand lib/compat.sh for macOS Compatibility

> Paste this entire document into a Claude Code session in the **framework** project.

---

## Task

Expand `lib/compat.sh` with portable function wrappers, then update all scripts that use
GNU-specific features to call the portable versions instead. This fixes macOS compatibility
while keeping Linux as the primary code path.

**Priority:** Linux compatibility is a MUST (don't break it). macOS fix is the goal.
Windows/WSL gets Linux compat for free.

## Background

macOS ships BSD userland (bash 3.2, BSD date, BSD head, BSD stat). The framework has
26+ GNU-specific features that break on macOS. The 3 most impactful categories are:

1. `date -d` — GNU date flag for parsing date strings (6 locations)
2. `declare -A` — bash 4+ associative arrays (3 locations)
3. `head -n -1` — GNU head negative line count (5 locations)
4. `stat -c %Y` — GNU stat format (7 locations, 2 already have BSD fallback)
5. `find -printf` — GNU find (1 location)

`lib/compat.sh` already exists with `_sed_i()`. This task expands it.

## Step 1: Expand lib/compat.sh

Add the following functions **after** the existing `_sed_i()` function in `lib/compat.sh`:

### 1a. Portable date-to-epoch

```bash
# --- Portable date-to-epoch ---
# Converts ISO 8601 date string to Unix epoch seconds.
# Tries GNU date first (Linux), then BSD date (macOS), then python3 fallback.
# Detection result is cached for the session.
#
# Usage: epoch=$(portable_date_to_epoch "2026-03-20T13:42:10Z")
#        epoch=$(portable_date_to_epoch "2026-03-20")

_COMPAT_DATE_CMD=""

_detect_date_cmd() {
    if date -d "2000-01-01" +%s >/dev/null 2>&1; then
        _COMPAT_DATE_CMD="gnu"
    elif date -j -f "%Y-%m-%d" "2000-01-01" +%s >/dev/null 2>&1; then
        _COMPAT_DATE_CMD="bsd"
    elif command -v python3 >/dev/null 2>&1; then
        _COMPAT_DATE_CMD="python"
    else
        _COMPAT_DATE_CMD="none"
    fi
}

portable_date_to_epoch() {
    [ -z "$_COMPAT_DATE_CMD" ] && _detect_date_cmd
    local input="$1"
    case "$_COMPAT_DATE_CMD" in
        gnu)
            date -d "$input" +%s 2>/dev/null || echo 0
            ;;
        bsd)
            # Try ISO 8601 with time, then date-only
            date -j -f "%Y-%m-%dT%H:%M:%SZ" "$input" +%s 2>/dev/null ||
            date -j -f "%Y-%m-%dT%H:%M:%S%z" "$input" +%s 2>/dev/null ||
            date -j -f "%Y-%m-%d" "${input%%T*}" +%s 2>/dev/null ||
            echo 0
            ;;
        python)
            python3 -c "
from datetime import datetime, timezone
import sys
s = sys.argv[1].rstrip('Z')
for fmt in ('%Y-%m-%dT%H:%M:%S', '%Y-%m-%d'):
    try:
        print(int(datetime.strptime(s.split('+')[0].split('-05')[0][:19] if 'T' in s else s[:10], fmt).replace(tzinfo=timezone.utc).timestamp()))
        sys.exit(0)
    except ValueError:
        continue
print(0)
" "$input" 2>/dev/null || echo 0
            ;;
        *) echo 0 ;;
    esac
}

# Days between two ISO date strings. Returns integer.
# Usage: days=$(portable_date_diff_days "2026-03-15" "2026-03-20")  # → 5
portable_date_diff_days() {
    local d1 d2
    d1=$(portable_date_to_epoch "$1")
    d2=$(portable_date_to_epoch "$2")
    if [ "$d1" -eq 0 ] || [ "$d2" -eq 0 ]; then
        echo 0
    else
        echo $(( (d2 - d1) / 86400 ))
    fi
}

# Relative date offset to epoch. Handles "7 days ago" (GNU) and "-7d" (BSD).
# Usage: epoch=$(portable_date_relative "-7 days")
portable_date_relative() {
    [ -z "$_COMPAT_DATE_CMD" ] && _detect_date_cmd
    local offset="$1"
    case "$_COMPAT_DATE_CMD" in
        gnu)
            date -d "$offset" +%s 2>/dev/null || echo 0
            ;;
        bsd)
            # Convert common patterns: "7 days ago" → "-v-7d"
            local bsd_flag
            if echo "$offset" | grep -qE '^[0-9]+ days? ago$'; then
                local n=$(echo "$offset" | grep -oE '^[0-9]+')
                bsd_flag="-v-${n}d"
            elif echo "$offset" | grep -qE '^-[0-9]+d$'; then
                bsd_flag="-v${offset}"
            else
                echo 0; return
            fi
            date "$bsd_flag" +%s 2>/dev/null || echo 0
            ;;
        python)
            local n=$(echo "$offset" | grep -oE '[0-9]+')
            python3 -c "
from datetime import datetime, timedelta, timezone
print(int((datetime.now(timezone.utc) - timedelta(days=${n:-0})).timestamp()))
" 2>/dev/null || echo 0
            ;;
        *) echo 0 ;;
    esac
}
```

### 1b. Portable head-but-last

```bash
# --- Portable head-but-last ---
# Outputs all lines except the last N from stdin.
# Replaces GNU-only `head -n -N`.
#
# Usage: some_command | portable_head_but_last
#        some_command | portable_head_but_last 2   # skip last 2 lines
portable_head_but_last() {
    local n="${1:-1}"
    if [ "$n" -eq 1 ]; then
        sed '$d'
    else
        # Buffer last N lines, print with delay
        awk -v n="$n" '{buf[NR%n]=$0} NR>n{print buf[(NR)%n]}'
    fi
}
```

### 1c. Portable stat mtime

```bash
# --- Portable stat mtime ---
# Returns file modification time as Unix epoch seconds.
# Tries GNU stat (Linux) then BSD stat (macOS).
#
# Usage: mtime=$(portable_stat_mtime /path/to/file)
portable_stat_mtime() {
    local file="$1"
    stat -c %Y "$file" 2>/dev/null ||
    stat -f %m "$file" 2>/dev/null ||
    echo 0
}
```

### 1d. Portable find-recent

```bash
# --- Portable find-recent ---
# Lists files sorted by modification time (newest first).
# Replaces GNU-only `find -printf '%T@ %f\n' | sort -rn`.
#
# Usage: portable_find_recent /path "*.yaml" 5
portable_find_recent() {
    local dir="$1" pattern="$2" limit="${3:-10}"
    if find "$dir" -maxdepth 1 -name "$pattern" -printf '%T@ %f\n' 2>/dev/null | sort -rn | head -n "$limit"; then
        return 0
    fi
    # BSD fallback: use stat to get mtime
    find "$dir" -maxdepth 1 -name "$pattern" -type f -exec stat -f '%m %N' {} \; 2>/dev/null |
        sort -rn | head -n "$limit" | while read -r _ path; do basename "$path"; done
}
```

## Step 2: Fix episodic.sh (3 date calls + 2 head calls)

File: `agents/context/lib/episodic.sh`

**Source compat.sh near the top** (after the shebang and comments, before the first function):
```bash
source "${FRAMEWORK_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}/lib/compat.sh"
```

**Line 93** — Replace:
```bash
local updates_section=$(sed -n '/^## Updates/,/^## /p' "$task_file" | head -n -1)
```
With:
```bash
local updates_section=$(sed -n '/^## Updates/,/^## /p' "$task_file" | portable_head_but_last)
```

**Line 125** — Replace:
```bash
local decisions_section=$(sed -n '/^## Decisions/,/^## /p' "$task_file" 2>/dev/null | head -n -1)
```
With:
```bash
local decisions_section=$(sed -n '/^## Decisions/,/^## /p' "$task_file" 2>/dev/null | portable_head_but_last)
```

**Line 176** — Replace:
```bash
duration_days=$(( ($(date -d "$completed_date" +%s) - $(date -d "$created_date" +%s)) / 86400 )) 2>/dev/null || duration_days=0
```
With:
```bash
duration_days=$(portable_date_diff_days "$created_date" "$completed_date") || duration_days=0
```

**Lines 182-183** — Replace:
```bash
start_epoch=$(date -d "$created" +%s 2>/dev/null) || start_epoch=0
end_epoch=$(date -d "$last_update" +%s 2>/dev/null) || end_epoch=0
```
With:
```bash
start_epoch=$(portable_date_to_epoch "$created") || start_epoch=0
end_epoch=$(portable_date_to_epoch "$last_update") || end_epoch=0
```

## Step 3: Fix metrics.sh (2 date calls + 1 head call)

File: `metrics.sh`

**Source compat.sh near the top:**
```bash
source "$FRAMEWORK_ROOT/lib/compat.sh"
```

**Line 63** — Replace `head -n -1` with `portable_head_but_last`:
```bash
desc=$(sed -n '/^description:/,/^[a-z_]*:/p' "$f" | portable_head_but_last | sed 's/^description: //' | sed 's/^> *//' | sed 's/^  //' | tr '\n' ' ')
```

**Line 80** — Already has dual fallback, but can be cleaner:
```bash
seven_days_ago=$(portable_date_relative "7 days ago")
```

**Line 94** — Replace:
```bash
last_ts=$(date -d "$last_update" +%s 2>/dev/null || echo 0)
```
With:
```bash
last_ts=$(portable_date_to_epoch "$last_update")
```

## Step 4: Fix audit.sh (1 date call + 1 head call + 1 declare -A + 2 stat calls)

File: `agents/audit/audit.sh`

**Source compat.sh near the top:**
```bash
source "$FRAMEWORK_ROOT/lib/compat.sh"
```

**Line 551** — Replace:
```bash
created_ts=$(date -d "$created_date" +%s 2>/dev/null || date -j -f "%Y-%m-%d" "$created_date" +%s 2>/dev/null || true)
```
With:
```bash
created_ts=$(portable_date_to_epoch "$created_date")
```

**Line 1096** — Replace `head -n -1`:
```bash
trigger_output=$(echo "$triggered_gaps" | portable_head_but_last)
```

**Line 1402** — Replace `stat -c`:
```bash
budget_mtime=$(portable_stat_mtime "$BUDGET_FILE")
```

**Line 2775** — Replace `declare -A issue_counts` with indexed array pattern:
This one requires more context. The `declare -A issue_counts` is likely used as a counter map.
Replace the pattern:
```bash
# OLD:
declare -A issue_counts
# ... issue_counts[$key]=$(( ${issue_counts[$key]:-0} + 1 )) ...
# ... for key in "${!issue_counts[@]}" ...

# NEW:
issue_counts=()
# To increment: find existing or append
_ic_increment() {
    local key="$1"
    local i
    for i in "${!issue_counts[@]}"; do
        case "${issue_counts[$i]}" in "${key}="*)
            local val="${issue_counts[$i]#${key}=}"
            issue_counts[$i]="${key}=$(( val + 1 ))"
            return
        esac
    done
    issue_counts+=("${key}=1")
}
# To read: grep from array
_ic_get() {
    local key="$1"
    for item in "${issue_counts[@]}"; do
        case "$item" in "${key}="*) echo "${item#${key}=}"; return ;; esac
    done
    echo 0
}
# To iterate keys:
_ic_keys() {
    for item in "${issue_counts[@]}"; do echo "${item%%=*}"; done
}
```
**Note:** Read the actual usage of `issue_counts` around line 2775 to adapt this pattern
to the specific iteration/access patterns used there.

## Step 5: Fix update-task.sh (1 declare -A + 1 head call)

File: `agents/task-create/update-task.sh`

**Source compat.sh near the top:**
```bash
source "$FRAMEWORK_ROOT/lib/compat.sh"
```

**Line 562** — Replace `head -n -1`:
```bash
HUMAN_AC_SECTION=$(sed -n '/^### Human/,/^## \|^### [^H]/p' "$TASK_FILE" 2>/dev/null | portable_head_but_last)
```

**Line 599** — Replace `declare -A LOC_TO_ID` with indexed array.
Read the surrounding code to understand how `LOC_TO_ID` is used, then apply the
same `key=value` indexed array pattern as audit.sh above.

## Step 6: Fix diagnose.sh (1 declare -A)

File: `agents/healing/lib/diagnose.sh`

**Line 9** — Replace `declare -A FAILURE_TYPES`.
This is likely a static lookup table. Convert to a function:
```bash
# OLD:
declare -A FAILURE_TYPES=(
    [code]="Code error"
    [dependency]="Dependency issue"
    ...
)

# NEW:
_failure_type_label() {
    case "$1" in
        code) echo "Code error" ;;
        dependency) echo "Dependency issue" ;;
        environment) echo "Environment issue" ;;
        design) echo "Design flaw" ;;
        external) echo "External failure" ;;
        *) echo "Unknown" ;;
    esac
}
```
**Note:** Read the actual FAILURE_TYPES definition and usage to build the complete case statement.

## Step 7: Fix remaining stat -c calls

These files use `stat -c %Y` without BSD fallback:

| File | Line | Fix |
|------|------|-----|
| `bin/claude-fw` | 48 | Replace `stat -c %Y "$signal_file"` with `portable_stat_mtime "$signal_file"` |
| `bin/claude-fw` | 112 | Same replacement |
| `agents/context/checkpoint.sh` | 262 | Same replacement |
| `agents/audit/audit.sh` | 1402 | Already covered in Step 4 |

Source compat.sh in `bin/claude-fw` and `checkpoint.sh` if not already sourced.

**Already have fallback (no change needed):**
- `agents/git/lib/hooks.sh:265` — has `|| stat -f %m` fallback
- `agents/git/lib/status.sh:19` — has `|| stat -f %m` fallback

## Step 8: Fix find -printf (1 location)

File: `agents/context/lib/status.sh`

**Line 70** — Replace:
```bash
find "$CONTEXT_DIR/episodic" -name "T-*.yaml" -type f -printf '%T@ %f\n' 2>/dev/null | \
```
With:
```bash
portable_find_recent "$CONTEXT_DIR/episodic" "T-*.yaml" 9999 2>/dev/null | \
```
Or inline the BSD-compatible version:
```bash
(find "$CONTEXT_DIR/episodic" -name "T-*.yaml" -type f -printf '%T@ %f\n' 2>/dev/null ||
 find "$CONTEXT_DIR/episodic" -name "T-*.yaml" -type f -exec stat -f '%m %N' {} \; 2>/dev/null |
 while read -r ts path; do echo "$ts $(basename "$path")"; done) | \
```

## Step 9: Verify

After all changes, run on macOS:
```bash
# Framework tests
fw doctor

# Episodic generation (the original failure)
fw context generate-episodic T-028   # or any multi-day task

# Audit
fw audit

# Metrics
fw metrics

# Task update (triggers declare -A path)
fw task update T-XXX --status started-work  # any test task
```

## Summary

| What | Files changed | Lines added |
|------|--------------|-------------|
| lib/compat.sh expansion | 1 | ~130 |
| Source compat.sh + use functions | 7 | ~15 per file |
| Total | 8 files | ~235 lines |

All changes maintain Linux compatibility (GNU path is tried first, cached for session).
macOS gets BSD fallback. Python3 is last resort (available on both platforms).
