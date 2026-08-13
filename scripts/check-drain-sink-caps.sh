#!/usr/bin/env bash
# check-drain-sink-caps.sh (T-2531, G-019 prevention for the T-2518/2524/2525/2529 class)
#
# WHY: four separate fixes in one campaign each closed ONE instance of the SAME class —
# peer-influenced / externally-driven bytes accumulated UNBOUNDED into the long-lived
# daemon's OWN address space, risking OOM of the supervisor process (not a transient child):
#   T-2518  bus line reader        -> read_capped_line (16 MiB)
#   T-2524  artifact download      -> capped
#   T-2525  hub staging            -> capped
#   T-2529  executor::execute      -> cmd.output() drained child stdout+stderr unbounded
# Each was a one-line omission, invisible until a human read that exact line. T-2527 made
# the *pre-allocation* clamp convention load-bearing (with_capacity / Semaphore::new / vec!).
# The *accumulation* sinks (.output() full child drain, read_to_end/read_to_string stream
# drains, full collects) were still unguarded. G-019: fix the symptom (the four caps), then
# fix why the framework was blind — this check makes the "cap externally-driven accumulation"
# convention load-bearing for the drain-sink half of the class.
#
# WHAT: a grep/AST-lite scanner over the daemon crates that flags each
#   <cmd>.output()                       -- drains a child's ENTIRE stdout+stderr
#   <reader>.read_to_end(...)            -- drains a reader fully into a Vec
#   <reader>.read_to_string(...)         -- drains a reader fully into a String
#   <iter>.collect::<Vec<u8>>()          -- full byte collect
#   <iter>.bytes().collect               -- full byte collect
# A `<reader>.take(N).read_to_*(...)` on the SAME line is bounded and is NOT flagged.
# `//`-comment lines are skipped (exactly like check-alloc-sink-clamps.sh).
#
# ALLOWLIST: `.context/working/.drain-sink-allowlist` acknowledges confirmed-safe sites —
# trusted-arg subprocesses (self-binary `current_exe fleet ...` or a TERMLINK_SCRIPTS_DIR
# bash script, output bounded by the subcommand's own emission), or drains whose source is
# a trusted fixed-size local resource. One drift-stable signature per line:
#   <relpath>::<enclosing-fn>::<sink>
# The signature is fn-name-based (NOT line-number-based) so it survives line moves; a fn
# RENAME re-fires the site (a meaningful change worth re-review — same trade-off as T-2527).
# The check trends toward empty as new sinks are confirmed-and-acknowledged.
#
# Output is a REVIEW list, not a hard gate — the value is surfacing a NEW drain sink for a
# human/agent to confirm-and-cap (then either cap the code or allowlist the site).
#
# EXIT CODES:
#   0  clean    -- no unacknowledged drain-sink sites.
#   1  firing   -- >=1 drain sink not cleared by proximity .take( or allowlist.
#   2  tooling  -- missing dep / bad scan root.
#
# USAGE:
#   check-drain-sink-caps.sh [--json] [--quiet] [--no-heartbeat]
#                            [--root <dir>]... [--allowlist <file>]
#     --json          emit {ok, firing:[{file,line,fn,sink}], checked, candidates}
#     --quiet         print only on firing (cron mode); clean prints nothing
#     --no-heartbeat  skip the heartbeat touch (meta-canary invokes with this)
#     --root <dir>    override scan roots (repeatable; default = the 5 daemon crates)
#     --allowlist <f> override allowlist path (fixtures point this at a scratch file)
#
# Origin: T-2531 (this build). Sibling of scripts/check-alloc-sink-clamps.sh (T-2527) and
# the 13 *-canary checks in CLAUDE.md; /canaries can auto-discover the optional daily log.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
ROOTS=()
# T-2681 — tracked-first allowlist resolution; see the header of
# .context/checks/alloc-sink-allowlist for why. Legacy gitignored path kept as a
# fallback; explicit DRAIN_SINK_ALLOWLIST / --allowlist always wins over both.
_default_allowlist() {
    if [ -f ".context/checks/drain-sink-allowlist" ]; then
        printf '%s' ".context/checks/drain-sink-allowlist"
    else
        printf '%s' ".context/working/.drain-sink-allowlist"
    fi
}
ALLOWLIST="${DRAIN_SINK_ALLOWLIST:-$(_default_allowlist)}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --root) shift; [ $# -gt 0 ] || { echo "check-drain-sink: --root needs a value" >&2; exit 2; }; ROOTS+=("$1"); shift ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-drain-sink: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1"; shift ;;
        -h|--help) sed -n '2,64p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-drain-sink: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-drain-sink: grep not found (required)" >&2; exit 2; }

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(crates/termlink-mcp/src crates/termlink-hub/src crates/termlink-session/src \
           crates/termlink-bus/src crates/termlink-protocol/src)
fi
for r in "${ROOTS[@]}"; do
    [ -e "$r" ] || { echo "check-drain-sink: scan root not found: $r" >&2; exit 2; }
done

# T-1723 heartbeat: prove this check ran, even on clean/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.drain-sink-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# The sink-detection regex (extended). Kept in one place so scan + comment-strip agree.
SINK_RE='\.output\(\)|\.read_to_end\(|\.read_to_string\(|\.collect::<[[:space:]]*Vec[[:space:]]*<[[:space:]]*u8|\.bytes\(\)[[:space:]]*\.collect'
# A Rust fn declaration line (pub / pub(crate) / async / const combos all end in `fn NAME`).
FN_RE='(^|[^A-Za-z0-9_])fn[[:space:]]+[A-Za-z0-9_]+'

# classify which sink a (comment-stripped) code line contains; echoes the sink token or "".
sink_token() {
    local code="$1"
    if   printf '%s' "$code" | grep -qE '\.output\(\)';                                    then echo "output";
    elif printf '%s' "$code" | grep -qE '\.read_to_end\(';                                 then echo "read_to_end";
    elif printf '%s' "$code" | grep -qE '\.read_to_string\(';                              then echo "read_to_string";
    elif printf '%s' "$code" | grep -qE '\.collect::<[[:space:]]*Vec[[:space:]]*<[[:space:]]*u8'; then echo "collect_vec_u8";
    elif printf '%s' "$code" | grep -qE '\.bytes\(\)[[:space:]]*\.collect';                then echo "bytes_collect";
    else echo ""; fi
}

# extract the fn name from a matched fn-declaration line (space-prefixed for the boundary).
fn_name_of() {
    printf ' %s' "$1" | sed -E 's/.*[^A-Za-z0-9_]fn[[:space:]]+([A-Za-z0-9_]+).*/\1/'
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
FILES="$(find "${ROOTS[@]}" -type f -name '*.rs' 2>/dev/null | sort)"
[ -n "$FILES" ] || { echo "check-drain-sink: no .rs files under scan roots" >&2; exit 2; }

checked=0
firing_lines=""

while IFS= read -r file; do
    [ -n "$file" ] || continue
    # Precompute the file's fn-declaration lines: "lineno:fnname", ascending.
    fnmap="$(grep -nE "$FN_RE" "$file" 2>/dev/null | while IFS= read -r fl; do
        fln="${fl%%:*}"; fcode="${fl#*:}"
        printf '%s:%s\n' "$fln" "$(fn_name_of "$fcode")"
    done)"

    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        lineno="${hit%%:*}"
        code="${hit#*:}"
        # strip line comments, then confirm a sink survives (was not inside a comment)
        codestripped="$(printf '%s' "$code" | sed -E 's://.*$::')"
        sink="$(sink_token "$codestripped")"
        [ -n "$sink" ] || continue

        # proximity SAFE: a `.take(N)` on the same line bounds a read drain
        case "$sink" in
            read_to_end|read_to_string)
                printf '%s' "$codestripped" | grep -qE '\.take\(' && continue ;;
        esac

        checked=$((checked + 1))

        # enclosing fn = the fn-decl line with the largest lineno <= this sink's lineno
        encfn="-"
        while IFS= read -r fm; do
            [ -n "$fm" ] || continue
            fln="${fm%%:*}"; fnm="${fm#*:}"
            [ "$fln" -le "$lineno" ] && encfn="$fnm"
            [ "$fln" -gt "$lineno" ] && break
        done <<< "$fnmap"

        sig="${file}::${encfn}::${sink}"
        [ -n "${ALLOW[$sig]:-}" ] && continue
        firing_lines="${firing_lines}${file}:${lineno}:${encfn}:${sink}"$'\n'
    done < <(grep -nE "$SINK_RE" "$file" 2>/dev/null)
done <<< "$FILES"

fire_count="$(printf '%s' "$firing_lines" | grep -c . || true)"

if command -v jq >/dev/null 2>&1; then
    firing_json="$(printf '%s' "$firing_lines" | grep . | \
        sed -E 's/^([^:]+):([0-9]+):([^:]*):(.*)$/{"file":"\1","line":\2,"fn":"\3","sink":"\4"}/' | \
        jq -sc '.' 2>/dev/null || echo '[]')"
else
    firing_json="[]"
fi

if [ "${fire_count:-0}" -eq 0 ]; then
    if [ "$WANT_JSON" -eq 1 ]; then
        printf '{"ok":true,"firing":[],"checked":%d,"candidates":0}\n' "$checked"
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-drain-sink: clean — 0 unacknowledged drain-sink sites ($checked sink call(s) scanned)."
    fi
    exit 0
fi

if [ "$WANT_JSON" -eq 1 ]; then
    printf '{"ok":false,"firing":%s,"checked":%d,"candidates":%d}\n' "$firing_json" "$checked" "$fire_count"
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-drain-sink: FIRING — $fire_count unbounded drain-sink site(s) with no cap / allowlist ack:"
    printf '%s' "$firing_lines" | grep . | sed -E 's/^([^:]+):([0-9]+):([^:]*):(.*)$/  ↳ \1:\2  \4  (in fn \3)/'
    echo "  Each drains externally-driven bytes into the long-lived daemon heap with no bound."
    echo "  Fix: cap it (chunked read + size check like executor::execute_capped, or reader.take(N)),"
    echo "  OR — if confirmed trusted-arg subprocess / trusted fixed source — add its signature to"
    echo "  $ALLOWLIST. This is the T-2518/2524/2525/2529 class (peer-driven accumulation -> OOM)."
    echo "---"
fi
exit 1
