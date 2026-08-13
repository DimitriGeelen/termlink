#!/usr/bin/env bash
# check-alloc-sink-clamps.sh (T-2527, G-019 prevention for the T-2523/T-2526 class)
#
# WHY: twice in one window an adversarial hunter found a caller-supplied param
# reaching an eager allocation / resource sink with no upper bound —
#   max_parallel -> tokio::sync::Semaphore::new(max_parallel)   (T-2523: panic + 0-hang)
#   count        -> Vec::with_capacity(count as usize)          (T-2526: ~100 GB pre-alloc OOM)
# Each was a one-line omission, invisible until someone read that exact line. The
# repo has a strong "clamp every numeric caller param" convention (clamp_max_parallel,
# max_depth.clamp(1,1024), since_days.clamp(1,365), ...) but the convention is BY
# DISCIPLINE, NOT ENFORCED. Two instances in one window means the mechanism recurs.
# G-019: fix the symptom (the two clamps), then fix why the framework was blind —
# this check makes the convention load-bearing.
#
# WHAT: a grep/AST-lite scanner over the handler crates that flags each
#   Vec/String/HashMap/HashSet/VecDeque/BTreeMap::with_capacity(<x>)
#   Semaphore::new(<x>)
#   vec![_; <x>]
#   <expr>.repeat(<x>)
# whose size argument <x> is a BARE IDENTIFIER or a `p.<field>` expression
# (optionally `<ident> as usize`) — i.e. plausibly a caller-supplied count — and is
# NOT a literal / not `.len()` of a materialized collection / not inline
# `.clamp(`/`.min(`/`.take(`/`.saturating_`-bounded. Compound expressions
# (`m + 1`, `a * b`) are OUT OF SCOPE (conservative — fewer false positives).
#
# BINDING-AWARE CLEAR: for a bare-ident site `foo(x)`, if the same file contains
#   let [mut] x = <...clamp...|....min(...|...take(...|validate_...>
# the arg is treated as clamped-at-its-binding and the site is cleared. This is why
# the T-2523 sites (`let max_parallel = clamp_max_parallel(p.max_parallel);`) read
# clean, and why reverting that clamp makes the site fire again (load-bearing).
#
# ALLOWLIST: `.context/working/.alloc-sink-allowlist` acknowledges confirmed-safe
# sites the binding-grep can't clear (internally-derived counts, early-return
# validate_* guards, library constructor params). One drift-stable signature per
# line: `<relpath>::<sink>(<normalized-arg>)`. Lines starting with # are comments.
# The check trends toward empty as new sinks are confirmed-and-acknowledged.
#
# Output is a REVIEW list, not a hard gate — false positives are acceptable and
# expected; the value is surfacing NEW sinks for a human/agent to confirm-and-clamp
# (then either clamp the code or allowlist the site).
#
# EXIT CODES:
#   0  clean    -- no unacknowledged candidate sites.
#   1  firing   -- >=1 candidate site not cleared by binding-grep or allowlist.
#   2  tooling  -- missing dep / bad scan root.
#
# USAGE:
#   check-alloc-sink-clamps.sh [--json] [--quiet] [--no-heartbeat]
#                              [--root <dir>]... [--allowlist <file>]
#     --json          emit {ok, firing:[{file,line,sink,arg}], checked, candidates}
#     --quiet         print only on firing (cron mode); clean prints nothing
#     --no-heartbeat  skip the heartbeat touch (meta-canary invokes with this)
#     --root <dir>    override scan roots (repeatable; default = the 3 handler crates)
#     --allowlist <f> override allowlist path (fixtures point this at a scratch file)
#
# Origin: T-2527 (this build). Sibling of the 13 *-canary checks in CLAUDE.md;
# /canaries can auto-discover the optional daily log.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
ROOTS=()
# T-2681 — tracked-first allowlist resolution. The canonical home is the
# git-tracked `.context/checks/`; `.context/working/` is gitignored, so an
# allowlist there is invisible to a fresh clone / CI runner / git worktree and the
# check fires on every acknowledged site there while reporting CLEAN on the one
# machine that holds the untracked copy. The legacy path is still honoured as a
# fallback so an un-migrated checkout keeps working. An explicit
# ALLOC_SINK_ALLOWLIST / --allowlist always wins over both.
_default_allowlist() {
    if [ -f ".context/checks/alloc-sink-allowlist" ]; then
        printf '%s' ".context/checks/alloc-sink-allowlist"
    else
        printf '%s' ".context/working/.alloc-sink-allowlist"
    fi
}
ALLOWLIST="${ALLOC_SINK_ALLOWLIST:-$(_default_allowlist)}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --root) shift; [ $# -gt 0 ] || { echo "check-alloc-sink: --root needs a value" >&2; exit 2; }; ROOTS+=("$1"); shift ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-alloc-sink: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1"; shift ;;
        -h|--help) sed -n '2,60p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-alloc-sink: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-alloc-sink: grep not found (required)" >&2; exit 2; }

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(crates/termlink-mcp/src crates/termlink-hub/src crates/termlink-session/src)
fi
for r in "${ROOTS[@]}"; do
    [ -e "$r" ] || { echo "check-alloc-sink: scan root not found: $r" >&2; exit 2; }
done

# T-1723 heartbeat: prove this check ran, even on clean/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.alloc-sink-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# --- classify a single sink argument -----------------------------------------
# stdin: the raw argument text (already extracted). Prints one of:
#   CANDIDATE <normalized-ident>   -- bare ident / p.field / `<x> as usize`
#   SAFE                            -- literal / .len() / inline-bounded
#   SKIP                            -- compound expr, out of scope
classify_arg() {
    local raw="$1" a
    # normalize: trim, drop a trailing `as usize`/`as u64` cast, collapse spaces
    a="$(printf '%s' "$raw" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
    a="$(printf '%s' "$a" | sed -E 's/[[:space:]]+as[[:space:]]+u(size|8|16|32|64|128)[[:space:]]*$//')"
    a="$(printf '%s' "$a" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
    [ -n "$a" ] || { echo "SKIP"; return; }
    # inline-bounded or materialized -> SAFE
    case "$a" in
        *".len()"*|*".len ("*|*".clamp("*|*".min("*|*".take("*|*".saturating_"*) echo "SAFE"; return ;;
    esac
    # pure numeric / arithmetic-of-literals / SCREAMING_CONST (optionally * / + with literals) -> SAFE
    if printf '%s' "$a" | grep -qE '^[0-9][0-9_]*([[:space:]]*[*+][[:space:]]*[0-9][0-9_]*)*$'; then echo "SAFE"; return; fi
    if printf '%s' "$a" | grep -qE '^[A-Z][A-Z0-9_]+([[:space:]]*[*+][[:space:]]*([0-9][0-9_]*|[A-Z][A-Z0-9_]+))*$'; then echo "SAFE"; return; fi
    # bare lowercase ident -> CANDIDATE
    if printf '%s' "$a" | grep -qE '^[a-z_][a-z0-9_]*$'; then echo "CANDIDATE $a"; return; fi
    # p.field / params.field / self.field style -> CANDIDATE (report the leaf ident)
    if printf '%s' "$a" | grep -qE '^[a-z_][a-z0-9_]*\.[a-z_][a-z0-9_]*$'; then
        echo "CANDIDATE ${a##*.}"; return
    fi
    # anything else (compound expr with operators, method chains we don't recognize)
    echo "SKIP"
}

# --- binding-aware clear ------------------------------------------------------
# Given a file and a bare ident, return 0 if the file binds it with an
# upper-bounding expression (clamp / .min( / .take( / validate_).
binding_is_clamped() {
    local file="$1" ident="$2"
    grep -qE "let[[:space:]]+(mut[[:space:]]+)?${ident}[[:space:]]*=.*(\.clamp\(|clamp_|\.min\(|\.take\(|validate_)" "$file" 2>/dev/null
}

# --- load allowlist signatures into a lookup ---------------------------------
declare -A ALLOW=()
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line; do
        line="${line%%#*}"
        line="$(printf '%s' "$line" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
        [ -n "$line" ] && ALLOW["$line"]=1
    done < "$ALLOWLIST"
fi

# --- scan ---------------------------------------------------------------------
# Sink regexes (extended). Each capture group 1 is the size-arg text.
FILES="$(find "${ROOTS[@]}" -type f -name '*.rs' 2>/dev/null | sort)"
[ -n "$FILES" ] || { echo "check-alloc-sink: no .rs files under scan roots" >&2; exit 2; }

checked=0
firing_lines=""

while IFS= read -r file; do
    [ -n "$file" ] || continue
    # grep matching lines with numbers; classify each
    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        lineno="${hit%%:*}"
        code="${hit#*:}"
        # strip line comments (heuristic; sink lines never contain URLs)
        codestripped="$(printf '%s' "$code" | sed -E 's://.*$::')"
        # skip if the sink keyword is gone after comment-strip (was inside a comment)
        printf '%s' "$codestripped" | grep -qE '(with_capacity|Semaphore::new|vec!\[|\.repeat)\(' || \
          printf '%s' "$codestripped" | grep -qE 'vec!\[[^]]*;' || continue

        # try each sink shape; extract arg
        sink="" arg=""
        if printf '%s' "$codestripped" | grep -qE '(Vec|String|HashMap|HashSet|VecDeque|BTreeMap)::with_capacity\('; then
            sink="with_capacity"
            arg="$(printf '%s' "$codestripped" | sed -E 's/.*::with_capacity\(//; s/\).*//')"
        elif printf '%s' "$codestripped" | grep -qE 'Semaphore::new\('; then
            sink="Semaphore::new"
            arg="$(printf '%s' "$codestripped" | sed -E 's/.*Semaphore::new\(//; s/\).*//')"
        elif printf '%s' "$codestripped" | grep -qE 'vec!\[[^]]*;'; then
            sink="vec!"
            arg="$(printf '%s' "$codestripped" | sed -E 's/.*vec!\[[^;]*;[[:space:]]*//; s/\].*//')"
        elif printf '%s' "$codestripped" | grep -qE '\.repeat\('; then
            sink="repeat"
            arg="$(printf '%s' "$codestripped" | sed -E 's/.*\.repeat\(//; s/\).*//')"
        else
            continue
        fi

        checked=$((checked + 1))
        verdict="$(classify_arg "$arg")"
        case "$verdict" in
            CANDIDATE*)
                ident="${verdict#CANDIDATE }"
                # binding-aware clear
                if binding_is_clamped "$file" "$ident"; then continue; fi
                # allowlist signature (drift-stable: relpath::sink(normalized-arg))
                narg="$(printf '%s' "$arg" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//; s/[[:space:]]+as[[:space:]]+u(size|8|16|32|64|128)[[:space:]]*$//; s/[[:space:]]+$//')"
                sig="${file}::${sink}(${narg})"
                [ -n "${ALLOW[$sig]:-}" ] && continue
                firing_lines="${firing_lines}${file}:${lineno}: ${sink}(${arg})"$'\n'
                ;;
        esac
    done < <(grep -nE '(with_capacity|Semaphore::new|\.repeat)\(|vec!\[[^]]*;' "$file" 2>/dev/null)
done <<< "$FILES"

# Rebuild firing_json cleanly (the inline concat above is fragile); recompute from firing_lines count
fire_count="$(printf '%s' "$firing_lines" | grep -c . || true)"

if command -v jq >/dev/null 2>&1; then
    firing_json="$(printf '%s' "$firing_lines" | grep . | sed -E 's/^([^:]+):([0-9]+): (.*)$/{"file":"\1","line":\2,"site":"\3"}/' | jq -sc '.' 2>/dev/null || echo '[]')"
else
    firing_json="[]"
fi

if [ "${fire_count:-0}" -eq 0 ]; then
    if [ "$WANT_JSON" -eq 1 ]; then
        printf '{"ok":true,"firing":[],"checked":%d,"candidates":0}\n' "$checked"
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-alloc-sink: clean — 0 unacknowledged candidate sites ($checked sink call(s) scanned)."
    fi
    exit 0
fi

if [ "$WANT_JSON" -eq 1 ]; then
    printf '{"ok":false,"firing":%s,"checked":%d,"candidates":%d}\n' "$firing_json" "$checked" "$fire_count"
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-alloc-sink: FIRING — $fire_count caller-param sink site(s) with no clamp / allowlist ack:"
    printf '%s' "$firing_lines" | grep . | sed 's/^/  ↳ /'
    echo "  Each is a size arg that is a bare ident / p.field reaching an eager allocation."
    echo "  Fix: clamp it (\`.clamp(1, N)\` inline, or a \`let x = clamp_*(...)\` binding), OR — if"
    echo "  confirmed bounded-by-construction — add its signature to $ALLOWLIST."
    echo "  This is the T-2523/T-2526 class (unbounded caller-param -> OOM/panic). See CLAUDE.md."
    echo "---"
fi
exit 1
