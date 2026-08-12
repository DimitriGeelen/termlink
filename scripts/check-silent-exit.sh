#!/usr/bin/env bash
# check-silent-exit.sh (T-2666, G-019 prevention for the T-2663 silent-text-exit class)
#
# WHY: a CLI subcommand that reaches `std::process::exit(<non-zero>)` on a user-facing
# path while printing NOTHING leaves the operator staring at a non-zero exit with zero
# explanation — indistinguishable from a crash (Directive #2, "no silent failures").
# T-2663 was exactly this: `termlink discover --first` on a no-match did a bare
# `std::process::exit(1)` in TEXT mode (its JSON sibling branch called `json_error_exit`,
# which prints; the text branch emitted nothing). It was found only by a human reading
# that one line. The recurring shape is a copy-paste divergence:
#
#     } else {
#         if display.json {
#             super::json_error_exit(json!({"ok": false, "error": "..."}));   // JSON: loud
#         }
#         std::process::exit(1);                                             // TEXT: SILENT
#     }
#
# The blessed remediation (T-2663) mirrors the JSON branch with an actionable stderr line
# just before the bare exit:
#         eprintln!("No matching sessions.");
#         std::process::exit(1);
#
# T-2663 fixed the `discover` instance but the SAME pattern sat un-migrated in two more
# commands (`session.rs` session-list no-match, `remote.rs` discover no-match) — a Class-C
# "hardened in one place, sibling not migrated" divergence the framework was blind to. This
# check makes the "name what happened before a bare text-mode exit" convention load-bearing.
#
# WHAT: a grep/AST-lite scanner over the user-facing CLI crate that flags each
#   std::process::exit(<non-zero int literal>)
# whose IMMEDIATELY-preceding non-blank line is a lone `}` (a closed block, NOT an output
# statement or a `flush()`), where that closed block carries a `json_error_exit` call within
# a short window above, and NO output macro (eprintln!/println!/eprint!/print!) sits between
# the closing brace and the exit. That is precisely the json-gated-output / bare-text-exit
# shape — high precision (the ~40 LOUD exit sites in this tree print or flush immediately
# above the exit, so their preceding line is not a lone `}`, and they do not match).
#
# Exit-code-FORWARDING sites are OUT OF SCOPE by construction: `exit(code)`,
# `exit(exit)`, `exit(exit_code as i32)`, `exit(exec_result.exit_code)` forward a wrapped
# subcommand's own exit status (which may be 0) and the wrapped op already produced the
# output — the regex only matches a non-zero INTEGER LITERAL, so these never fire.
#
# ALLOWLIST: `.context/working/.silent-exit-allowlist` acknowledges confirmed-loud sites
# (output emitted through a path the window cannot see). One drift-stable signature per line:
#   <relpath>::<enclosing-fn>::silent-exit
# fn-name-based (survives line moves; a fn rename re-fires — same trade-off as the sibling
# source checks). After the T-2667 sibling migration the current tree scans CLEAN (0
# unacknowledged), so the allowlist is empty.
#
# Output is a REVIEW list, not a hard gate. NOT a runtime cron canary — a source-level
# static check (sibling of scripts/check-alloc-sink-clamps.sh (T-2527) and
# scripts/check-drain-sink-caps.sh (T-2531)); run ad-hoc / in the meta-check tier.
#
# EXIT CODES:
#   0  clean    -- no unacknowledged silent-exit sites.
#   1  firing   -- >=1 silent non-zero exit not cleared by allowlist.
#   2  tooling  -- missing dep / bad scan root.
#
# USAGE:
#   check-silent-exit.sh [--json] [--quiet] [--no-heartbeat]
#                        [--root <dir>]... [--allowlist <file>]
#     --json          emit {ok, firing:[{file,line,fn}], checked, candidates}
#     --quiet         print only on firing (cron mode); clean prints nothing
#     --no-heartbeat  skip the heartbeat touch (meta-canary invokes with this)
#     --root <dir>    override scan roots (repeatable; default = the CLI crate src)
#     --allowlist <f> override allowlist path (fixtures point this at a scratch file)
#
# Origin: T-2666 (this build). Load-bearing proof: tests/silent-exit-check-fixtures.sh, and
# reverting T-2663/T-2667's `eprintln!` on any of the three fixed sites re-fires the check.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
ROOTS=()
ALLOWLIST="${SILENT_EXIT_ALLOWLIST:-.context/working/.silent-exit-allowlist}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --root) shift; [ $# -gt 0 ] || { echo "check-silent-exit: --root needs a value" >&2; exit 2; }; ROOTS+=("$1"); shift ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-silent-exit: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1"; shift ;;
        -h|--help) sed -n '2,88p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-silent-exit: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-silent-exit: grep not found (required)" >&2; exit 2; }

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(crates/termlink-cli/src)
fi
for r in "${ROOTS[@]}"; do
    [ -e "$r" ] || { echo "check-silent-exit: scan root not found: $r" >&2; exit 2; }
done

# T-1723 heartbeat: prove this check ran, even on clean/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.silent-exit-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# In-scope exit = a NON-ZERO INTEGER LITERAL arg (forwarding sites with a variable arg are
# out of scope — they may be 0 and the wrapped op already emitted output).
EXIT_RE='std::process::exit\(([1-9][0-9]*)\)'
# The json-gated-output marker whose presence-above + absence-of-print-between defines the class.
GATED_RE='json_error_exit'
# Any real output macro.
OUT_RE='eprintln!|println!|eprint!|print!'
# A Rust fn declaration line (pub / pub(crate) / async / const combos all end in `fn NAME`).
FN_RE='(^|[^A-Za-z0-9_])fn[[:space:]]+[A-Za-z0-9_]+'

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
[ -n "$FILES" ] || { echo "check-silent-exit: no .rs files under scan roots" >&2; exit 2; }

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
        # strip line comments, then confirm the exit survives (was not inside a comment)
        codestripped="$(printf '%s' "$code" | sed -E 's://.*$::')"
        printf '%s' "$codestripped" | grep -qE "$EXIT_RE" || continue

        checked=$((checked + 1))

        # nearest non-blank line ABOVE this exit; capture its lineno (prevno) + trimmed text
        prevno=$lineno; prevtrim=""
        while [ "$prevno" -gt 1 ]; do
            prevno=$((prevno - 1))
            cand="$(sed -n "${prevno}p" "$file")"
            candtrim="$(printf '%s' "$cand" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
            if [ -n "$candtrim" ]; then prevtrim="$candtrim"; break; fi
        done
        # the class signature: the exit's immediately-preceding statement is a closed block `}`,
        # NOT an output/flush line. (Loud sites print or flush() right above -> prevtrim != "}".)
        [ "$prevtrim" = "}" ] || continue

        # the closed block must carry a json_error_exit within a short window above the exit
        winstart=$((lineno - 6)); [ "$winstart" -lt 1 ] && winstart=1
        win="$(sed -n "${winstart},$((lineno - 1))p" "$file")"
        printf '%s' "$win" | grep -q "$GATED_RE" || continue

        # and NO output macro may sit between the closing brace and the exit (normally empty).
        between="$(sed -n "$((prevno + 1)),$((lineno - 1))p" "$file")"
        printf '%s' "$between" | grep -qE "$OUT_RE" && continue

        # enclosing fn = the fn-decl line with the largest lineno <= this exit's lineno
        encfn="-"
        while IFS= read -r fm; do
            [ -n "$fm" ] || continue
            fln="${fm%%:*}"; fnm="${fm#*:}"
            [ "$fln" -le "$lineno" ] && encfn="$fnm"
            [ "$fln" -gt "$lineno" ] && break
        done <<< "$fnmap"

        sig="${file}::${encfn}::silent-exit"
        [ -n "${ALLOW[$sig]:-}" ] && continue
        firing_lines="${firing_lines}${file}:${lineno}:${encfn}"$'\n'
    done < <(grep -nE "$EXIT_RE" "$file" 2>/dev/null)
done <<< "$FILES"

fire_count="$(printf '%s' "$firing_lines" | grep -c . || true)"

if command -v jq >/dev/null 2>&1; then
    firing_json="$(printf '%s' "$firing_lines" | grep . | \
        sed -E 's/^([^:]+):([0-9]+):(.*)$/{"file":"\1","line":\2,"fn":"\3"}/' | \
        jq -sc '.' 2>/dev/null || echo '[]')"
else
    firing_json="[]"
fi

if [ "${fire_count:-0}" -eq 0 ]; then
    if [ "$WANT_JSON" -eq 1 ]; then
        printf '{"ok":true,"firing":[],"checked":%d,"candidates":0}\n' "$checked"
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-silent-exit: clean — 0 unacknowledged silent-exit sites ($checked non-zero-literal exit(s) scanned)."
    fi
    exit 0
fi

if [ "$WANT_JSON" -eq 1 ]; then
    printf '{"ok":false,"firing":%s,"checked":%d,"candidates":%d}\n' "$firing_json" "$checked" "$fire_count"
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-silent-exit: FIRING — $fire_count silent non-zero exit(s) (json-gated output, bare text-mode exit):"
    printf '%s' "$firing_lines" | grep . | sed -E 's/^([^:]+):([0-9]+):(.*)$/  ↳ \1:\2  (in fn \3)/'
    echo "  The JSON branch emits via json_error_exit but the text path exits bare — a crash-indistinguishable"
    echo "  silent failure (Directive #2). Fix: mirror the JSON branch with an actionable stderr line before"
    echo "  the exit (see the T-2663 remediation at metadata.rs: eprintln!(\"No matching sessions.\");),"
    echo "  OR — if the site genuinely emits output the window cannot see — add its signature to $ALLOWLIST."
    echo "---"
fi
exit 1
