# T-196: Architectural Mitigation for macOS BSD/GNU Shell Incompatibilities

> Inception research — 5 parallel agents investigated the full scope of macOS compatibility
> issues in the Agentic Engineering Framework and evaluated architectural solutions.

## Problem Statement

The framework's bash scripts use GNU-specific features that break on macOS:
- macOS ships **bash 3.2** (2007, no associative arrays, no readarray)
- macOS ships **BSD date** (no `-d` flag for date parsing)
- macOS ships **BSD head** (no negative line counts)
- macOS ships **BSD stat** (different format flags)

This causes: episodic summaries fail, audit reports break, metrics skip stale detection,
and task updates throw `declare -A: invalid option` on every invocation.

## Full GNU-ism Audit (Agent 1)

**26+ instances across 15 framework scripts. 12 critical.**

### Critical (blocks execution)

| Category | Instances | Files |
|----------|-----------|-------|
| `date -d` (GNU date) | 6 | episodic.sh, metrics.sh, audit.sh |
| `declare -A` (bash 4+) | 3 | audit.sh, diagnose.sh, update-task.sh |
| `find -printf` (GNU find) | 1 | status.sh |
| `stat -c` (GNU stat, no fallback) | 2 | checkpoint.sh, audit.sh |

### Warning (wrong output)

| Category | Instances | Files |
|----------|-----------|-------|
| `head -n -N` (GNU head) | 5 | episodic.sh (×2), audit.sh, metrics.sh, update-task.sh |
| `stat -c` (incomplete fallback) | 2 | hooks.sh, status.sh |

### Minor (non-portable but often works)

| Category | Instances | Files |
|----------|-----------|-------|
| `echo -e` (not POSIX) | 20+ | watchtower.sh, preflight.sh, test scripts |
| `realpath --relative-to` | 1 | check-active-task.sh |

## Existing Framework Architecture (Agent 5)

The framework **already started** solving this:

- `lib/compat.sh` exists (27 lines) with `_sed_i()` — portable sed in-place
- `episodic.sh` has `_date_to_epoch()` with GNU → BSD → fallback chain
- **But:** `_date_to_epoch()` isn't used consistently (lines 176/182/183 bypass it)
- **But:** No other script sources compat.sh for date operations

The architecture is right. The execution is incomplete.

## Performance Analysis (Agent 4)

### Runtime Baselines (macOS)

| Runtime | Startup time |
|---------|-------------|
| `/bin/sh` | 7ms |
| `bash` | 18ms |
| `python3` | 47ms |

### Path Tiers and GNU-ism Distribution

| Tier | Scripts | Runs when | GNU-isms | Impact |
|------|---------|-----------|----------|--------|
| **Hot** (PreToolUse hooks) | check-tier0, check-active-task, budget-gate | Every tool use | 1 minor (`realpath`) | Blocking logic works fine on macOS |
| **Warm** (PostToolUse + task ops) | checkpoint, error-watchdog, update-task | Every tool use + task changes | `stat -c`, `declare -A` | Warning only; tasks still move |
| **Cold** (reports) | episodic, audit, metrics | Occasional | `date -d`, `head -n -1`, `declare -A` | **All breakage is here** |

**Key finding:** Hot-path hooks don't have critical GNU-isms. The problem is entirely
in warm/cold-path scripts. Performance constraints are relaxed for the fix.

## Portable Alternatives Evaluated (Agent 2)

### Date Operations

| Approach | macOS | Linux | Performance | Dependencies |
|----------|-------|-------|-------------|--------------|
| BSD `date -j -f` | native | no | 1.8ms | none |
| GNU `date -d` | no | native | 1.8ms | none |
| `python3 datetime` | yes | yes | 3.2ms | python3 |
| `perl POSIX::mktime` | yes | yes | 4.5ms | perl |
| `gdate` (Homebrew) | yes | native | 2.1ms | coreutils |
| Pure bash parsing | yes | yes | 0.3ms | none (complex) |

**Recommendation:** Cascading detection — GNU → BSD → python3 fallback. Cache result
for session. This is what `_date_to_epoch()` already does but needs to be shared.

### head -n -1 Replacement

| Approach | Portable | Notes |
|----------|----------|-------|
| `sed '$d'` | yes | Deletes last line. Clean, universal. |
| `awk 'NR>1{print prev}{prev=$0}'` | yes | Buffer approach. Works everywhere. |

### declare -A Replacement

| Approach | Min bash | Notes |
|----------|----------|-------|
| Indexed array with `key=value` strings | 3.x | Scan with grep/case. O(n) lookup. |
| `eval` dynamic vars | any | `eval "${prefix}_${key}=$value"`. Fragile. |
| Temp file per map | any | Write key=value lines. `grep ^key=`. |

**Recommendation:** Indexed `key=value` array with helper functions. The framework's
associative arrays are small (<20 entries) — O(n) scan is fine.

## Architectural Strategies Compared (Agent 3)

### Real-World Evidence

| Project | Approach | Lesson |
|---------|----------|--------|
| **nvm** | Pure POSIX sh | Maximum portability; no bash features needed |
| **rbenv** | Bash 3.x subset | Careful feature selection |
| **bats-core** | Explicit bash 3.2 support | Tests + bash compat is possible |
| **Git** | Shell scripts + C core | Heavy lifting in compiled code |
| **Homebrew** | Ruby core + shell wrappers | Can afford hard dependency |

### Strategy Comparison

| Strategy | Portability | Complexity | Performance | Maintenance |
|----------|-------------|-----------|-------------|-------------|
| Require bash 4+ (Homebrew) | poor | low | zero overhead | requires user setup |
| **Compat shim layer** | **excellent** | **low-medium** | **~5ms source** | **stable** |
| Feature detection runtime | moderate | medium | +2-5ms | self-healing |
| Python rewrite (hot paths) | good | high | 0ms (already running) | language overhead |
| Compile-time templates | good | very high | zero runtime | build complexity |

## Recommended Architecture: Complete lib/compat.sh

### Why This Approach

1. **Framework already has the pattern** — compat.sh exists, just needs expansion
2. **Minimal change** — ~130 lines added to one file, 6 files updated to use it
3. **Zero new dependencies** — uses macOS-native BSD date, falls back to python3
4. **Cached detection** — OS/tool detection runs once per script invocation
5. **Incremental** — can be deployed one function at a time

### Proposed compat.sh Functions (~130 lines)

#### 1. Date to Epoch (replaces `date -d`)

```bash
_COMPAT_DATE_CMD=""

_detect_date_strategy() {
    if date -d "2026-01-01" +%s >/dev/null 2>&1; then
        _COMPAT_DATE_CMD="gnu"
    elif date -j -f "%Y-%m-%d" "2026-01-01" +%s >/dev/null 2>&1; then
        _COMPAT_DATE_CMD="bsd"
    elif command -v python3 >/dev/null 2>&1; then
        _COMPAT_DATE_CMD="python"
    else
        _COMPAT_DATE_CMD="none"
    fi
}

portable_date_to_epoch() {
    [ -z "$_COMPAT_DATE_CMD" ] && _detect_date_strategy
    local input="$1"
    case "$_COMPAT_DATE_CMD" in
        gnu) date -d "$input" +%s 2>/dev/null || echo 0 ;;
        bsd)
            # Try full ISO first, then date-only
            date -j -f "%Y-%m-%dT%H:%M:%SZ" "$input" +%s 2>/dev/null ||
            date -j -f "%Y-%m-%dT%H:%M:%S%z" "$input" +%s 2>/dev/null ||
            date -j -f "%Y-%m-%d" "$input" +%s 2>/dev/null ||
            echo 0 ;;
        python)
            python3 -c "
from datetime import datetime
d = '$input'.replace('Z','').split('T')[0]
print(int(datetime.strptime(d, '%Y-%m-%d').timestamp()))" 2>/dev/null || echo 0 ;;
        *) echo 0 ;;
    esac
}

portable_date_diff_days() {
    local d1 d2
    d1=$(portable_date_to_epoch "$1")
    d2=$(portable_date_to_epoch "$2")
    echo $(( (d2 - d1) / 86400 ))
}
```

#### 2. Head Without Last N Lines (replaces `head -n -1`)

```bash
portable_head_but_last() {
    sed '$d'
}
```

#### 3. Portable Associative Arrays (replaces `declare -A`)

```bash
portable_assoc_set() {
    local varname="$1" key="$2" value="$3"
    eval "${varname}+=(\"${key}=${value}\")"
}

portable_assoc_get() {
    local varname="$1" key="$2"
    eval "local items=(\"\${${varname}[@]}\")"
    for item in "${items[@]}"; do
        case "$item" in "${key}="*)
            echo "${item#${key}=}"; return 0 ;;
        esac
    done
    echo ""
}

portable_assoc_increment() {
    # For counter maps: increment value for key, init to 1 if absent
    local varname="$1" key="$2"
    local current
    current=$(portable_assoc_get "$varname" "$key")
    if [ -z "$current" ]; then
        portable_assoc_set "$varname" "$key" "1"
    else
        # Remove old entry, add updated
        eval "local new=()"
        eval "local items=(\"\${${varname}[@]}\")"
        for item in "${items[@]}"; do
            case "$item" in "${key}="*) ;; *) eval "new+=(\"$item\")" ;; esac
        done
        eval "new+=(\"${key}=$((current + 1))\")"
        eval "${varname}=(\"\${new[@]}\")"
    fi
}
```

#### 4. Portable stat mtime (replaces `stat -c %Y`)

```bash
portable_stat_mtime() {
    local file="$1"
    stat -c %Y "$file" 2>/dev/null ||   # GNU
    stat -f %m "$file" 2>/dev/null ||    # BSD
    echo 0
}
```

### Files Requiring Updates

| File | Changes | Priority |
|------|---------|----------|
| `lib/compat.sh` | Add ~130 lines (4 function families) | P0 |
| `agents/context/lib/episodic.sh` | Replace 3x `date -d` + 2x `head -n -1` with compat functions | P0 |
| `agents/audit/audit.sh` | Replace `date -d` + `declare -A` + `head -n -1` + `stat -c` | P1 |
| `metrics.sh` | Replace `date -d` + `head -n -1` | P1 |
| `agents/task-create/update-task.sh` | Replace `declare -A` + `head -n -1` | P1 |
| `agents/healing/lib/diagnose.sh` | Replace `declare -A` | P2 |

### Implementation Plan

| Phase | Work | Impact |
|-------|------|--------|
| **Phase 1** | Expand `lib/compat.sh` with all 4 function families | Foundation |
| **Phase 2** | Fix episodic.sh (unblocks episodic generation on macOS) | **Immediate pain relief** |
| **Phase 3** | Fix audit.sh, metrics.sh, update-task.sh | Full macOS compatibility |
| **Phase 4** | Fix diagnose.sh, status.sh, checkpoint.sh | Complete coverage |
| **Phase 5** | Add `fw doctor` bash version check + CI matrix | Prevention |

## Go/No-Go Criteria

**GO if:**
- Solution fits in one shared file (~160 lines total)
- No new external dependencies required
- Hot-path hooks remain unaffected
- Can be deployed incrementally (one script at a time)

**NO-GO if:**
- Requires bash 4+ as hard dependency (breaks macOS OOTB)
- Requires Python as hard dependency (some minimal systems lack it)
- Adds >10ms latency to hot-path hooks

**Assessment: All GO criteria met. No NO-GO criteria triggered.**

## Dialogue Log

### Session S-2026-0320 (this session)
- **User:** "do an ultra deep research, send out 5 agents to investigate how we can architecturally mitigate the macOS limitation"
- **Agent:** Spawned 5 parallel research agents:
  1. Full audit of GNU-isms (found 26+ across 15 files)
  2. POSIX date alternatives (evaluated 6 approaches)
  3. Bash version strategies (5 strategies + real-world evidence from nvm/rbenv/git)
  4. Hook performance constraints (measured runtimes, mapped GNU-isms to path tiers)
  5. compat.sh design (concrete function implementations)
- **Finding:** Framework already has the right architecture (compat.sh exists, _date_to_epoch pattern exists). Just needs completion.
