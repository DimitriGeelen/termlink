#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-error-swallowing-predicate.sh (T-2792, G-019 prevention for the T-2791 class)
#
# WHY: `std::path::Path::exists()` / `is_dir()` / `is_file()` answer a THREE-state
# question with a BOOLEAN. From the std docs for `exists`:
#
#     "If you cannot access the metadata of the file, e.g. because of a permission
#      error or broken symbolic links, this will return `false`."
#
# So "it is not there" and "I was not allowed to look" produce the same value, and the
# second silently becomes the first. When such a predicate gates an EARLY RETURN OF AN
# EMPTY SUCCESS, a failed read is converted into a confident, plausible, wrong answer —
# which is worse than an error (Directive #2), because nothing downstream can tell.
#
# T-2791 was exactly this. `discovery.rs` filtered candidate session dirs on
# `.is_dir()`, so a non-root process pointed at a 0700 root-owned TERMLINK_RUNTIME_DIR
# discovered zero sessions, and `termlink topics` reported `No event topics found.`
# (exit 0) — byte-identical to the answer for a genuinely empty, fully-readable dir. The
# JSON surface was worse still: it asserted `sessions_skipped: 0`, a positive claim that
# the inventory was COMPLETE. It was found by a peer project (999-AEF, OBS-302) building
# an entire diagnosis on filesystem reads that were quietly returning nothing, not by
# anything in this repo.
#
# The guard layer was structurally blind to it. `check-silent-exit.sh` (T-2666) covers
# the neighbouring directive but detects a NON-ZERO exit that prints nothing. This is the
# inverse and strictly worse shape: a ZERO exit that prints a positive wrong claim. No
# member asked "can this success path be reached by a failed read?"
#
# WHAT: a grep/AST-lite scanner over the product crates that flags each
#
#     if !<expr>.exists() {          // or .is_dir() / .is_file()
#         return Ok(vec![]);         // or Ok(None) / None / Vec::new() / ...
#
# — a negated existence predicate whose immediate consequent is an EMPTY-SUCCESS return.
#
# SCOPE CUT, stated plainly (T-2747 census discipline). There are 384 bare
# exists/is_dir/is_file sites in these crates. Flagging all of them would produce a
# ledger nobody works down, not a review list — and most are benign (`if !p.exists() {
# create(p) }` fails loudly by itself; `if !p.exists() { bail!(...) }` is already loud).
# Only the empty-success consequent turns a failed read into a wrong ANSWER, so only that
# shape is in scope. 9 sites match. This check therefore says nothing about the other
# 375; see LIMITATION below.
#
# ALLOWLIST: `.context/checks/error-swallowing-allowlist` (git-tracked per T-2681 — NOT
# the gitignored `.context/working/`, which would make the guard's own green depend on
# unversioned local state). One drift-stable signature per line:
#
#     <relpath>::<enclosing-fn>::empty-success-gate   # <reason>
#
# fn-name-based, so it survives line moves; a fn RENAME re-fires the site, which is the
# intended re-review on meaningful change (same trade-off as every sibling check).
#
# The acknowledgement rule is the STRICTER one, borrowed from platform-lock (T-2693): the
# reason must state WHAT THE CODE DOES when the predicate is false for a reason other
# than absence. "It's fine" / "known safe" is not a reason. A grep cannot distinguish
# "returns empty and is right to" from "returns empty and is lying", so the
# acknowledgement IS the analysis, kept next to the signature so it stays in sync.
#
# LIMITATION — stated on EVERY output path including the clean one (T-2680): this detects
# ONE known error-swallowing SHAPE. A clean result means no unacknowledged empty-success
# gate, NOT that every filesystem predicate in the tree is used correctly. `.exists()` in
# any other position — a filter closure, a match guard, a bare `if` whose body computes
# rather than returns — is out of scope and unexamined.
#
# NOT a runtime cron canary. A source-level static check, sibling of
# check-alloc-sink-clamps.sh (T-2527), check-drain-sink-caps.sh (T-2531),
# check-silent-exit.sh (T-2666), check-busy-spin.sh (T-2672), check-platform-lock.sh
# (T-2693).
#
# EXIT CODES:
#   0  clean    -- no unacknowledged empty-success gate.
#   1  firing   -- >=1 site not cleared by the allowlist.
#   2  tooling  -- missing dep / bad scan root (fail-closed; never a silent clean).
#
# USAGE:
#   check-error-swallowing-predicate.sh [--json] [--quiet] [--no-heartbeat]
#                                       [--root <dir>]... [--allowlist <file>]
#     --json          emit {ok, firing:[{file,line,fn,predicate,returns}], checked, candidates, scope}
#     --quiet         print only on firing (cron mode); clean prints nothing
#     --no-heartbeat  skip the heartbeat touch (meta-canary invokes with this)
#     --root <dir>    override scan roots (repeatable; default = the four product crates)
#     --allowlist <f> override allowlist path (fixtures point this at a scratch file)
#
# Origin: T-2792, from the T-2791 RCA's own "structural follow-up" (filed separately —
# one bug, one task). Load-bearing proof: reverting T-2791's `classify_candidate` to a
# bare `.filter(|d| d.is_dir())` does NOT fire this check (wrong shape — that is the
# filter-closure position, out of scope), but reverting `manager.rs::list_sessions_in`'s
# guard re-fires it. See tests/error-swallowing-check-fixtures.sh.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
ROOTS=()
# T-2681 — tracked-first allowlist resolution.
_default_allowlist() {
    if [ -f ".context/checks/error-swallowing-allowlist" ]; then
        printf '%s' ".context/checks/error-swallowing-allowlist"
    else
        printf '%s' ".context/working/.error-swallowing-allowlist"
    fi
}
ALLOWLIST="${ERROR_SWALLOWING_ALLOWLIST:-$(_default_allowlist)}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --root) shift; [ $# -gt 0 ] || { echo "check-error-swallowing-predicate: --root needs a value" >&2; exit 2; }; ROOTS+=("$1"); shift ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-error-swallowing-predicate: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1"; shift ;;
        -h|--help) sed -n '2,88p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-error-swallowing-predicate: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-error-swallowing-predicate: grep not found (required)" >&2; exit 2; }

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(crates/termlink-cli/src crates/termlink-mcp/src crates/termlink-hub/src crates/termlink-session/src)
fi
for r in "${ROOTS[@]}"; do
    [ -e "$r" ] || { echo "check-error-swallowing-predicate: scan root not found: $r" >&2; exit 2; }
done

# T-1723 heartbeat: prove this check ran, even on clean/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.error-swallowing-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# The gate: a NEGATED existence predicate opening a block.
GATE_RE='if[[:space:]]+!.*\.(exists|is_dir|is_file)\(\)[[:space:]]*\{'
# The consequent that turns a failed read into a wrong answer: an EMPTY SUCCESS.
# Deliberately `return`-anchored — a tail expression is harder to attribute and the
# `return` form is what every real instance in this tree uses.
EMPTY_RE='return[[:space:]]+(Ok\([[:space:]]*(vec!\[\]|Vec::new\(\)|None|Default::default\(\)|HashMap::new\(\)|BTreeMap::new\(\)|String::new\(\))[[:space:]]*\)|None|vec!\[\]|Vec::new\(\)|Default::default\(\))[[:space:]]*;'
FN_RE='(^|[^A-Za-z0-9_])fn[[:space:]]+[A-Za-z0-9_]+'

fn_name_of() {
    printf ' %s' "$1" | sed -E 's/.*[^A-Za-z0-9_]fn[[:space:]]+([A-Za-z0-9_]+).*/\1/'
}

# --- load allowlist signatures ------------------------------------------------
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
[ -n "$FILES" ] || { echo "check-error-swallowing-predicate: no .rs files under scan roots" >&2; exit 2; }

checked=0
firing_lines=""

while IFS= read -r file; do
    [ -n "$file" ] || continue
    fnmap="$(grep -nE "$FN_RE" "$file" 2>/dev/null | while IFS= read -r fl; do
        fln="${fl%%:*}"; fcode="${fl#*:}"
        printf '%s:%s\n' "$fln" "$(fn_name_of "$fcode")"
    done)"

    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        lineno="${hit%%:*}"
        code="${hit#*:}"
        # A comment mentioning the shape is not the shape. Strip, then re-confirm.
        codestripped="$(printf '%s' "$code" | sed -E 's://.*$::')"
        printf '%s' "$codestripped" | grep -qE "$GATE_RE" || continue

        checked=$((checked + 1))

        # Which predicate — reported so the operator sees what was swallowed.
        pred="$(printf '%s' "$codestripped" | sed -E 's/.*\.(exists|is_dir|is_file)\(\).*/\1/')"

        # Nearest non-blank, non-comment line BELOW the gate = the consequent.
        # Comments are skipped for the same reason as T-2688: a comment can neither
        # return nor emit, so it must not become the consequent and hide the site.
        nextno=$lineno; nexttrim=""
        total_lines="$(wc -l < "$file")"
        while [ "$nextno" -lt "$total_lines" ]; do
            nextno=$((nextno + 1))
            cand="$(sed -n "${nextno}p" "$file")"
            candtrim="$(printf '%s' "$cand" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
            case "$candtrim" in ''|'//'*) continue ;; esac
            nexttrim="$candtrim"; break
        done

        printf '%s' "$nexttrim" | grep -qE "$EMPTY_RE" || continue

        # Normalise the returned value for the report (jq-safe: no quotes/backslashes).
        rettrim="$(printf '%s' "$nexttrim" | sed -E 's/[[:space:]]+/ /g; s/"/\x27/g; s/\\//g')"

        encfn="-"
        while IFS= read -r fm; do
            [ -n "$fm" ] || continue
            fln="${fm%%:*}"; fnm="${fm#*:}"
            [ "$fln" -le "$lineno" ] && encfn="$fnm"
            [ "$fln" -gt "$lineno" ] && break
        done <<< "$fnmap"

        sig="${file}::${encfn}::empty-success-gate"
        [ -n "${ALLOW[$sig]:-}" ] && continue
        firing_lines="${firing_lines}${file}|${lineno}|${encfn}|${pred}|${rettrim}"$'\n'
    done < <(grep -nE "$GATE_RE" "$file" 2>/dev/null)
done <<< "$FILES"

fire_count="$(printf '%s' "$firing_lines" | grep -c . || true)"

SCOPE="detects ONE shape (negated existence predicate gating an empty-success return); does not audit every filesystem predicate"

if command -v jq >/dev/null 2>&1; then
    firing_json="$(printf '%s' "$firing_lines" | grep . | \
        sed -E 's/^([^|]*)\|([0-9]+)\|([^|]*)\|([^|]*)\|(.*)$/{"file":"\1","line":\2,"fn":"\3","predicate":"\4","returns":"\5"}/' | \
        jq -sc '.' 2>/dev/null || echo '[]')"
else
    firing_json="[]"
fi

if [ "${fire_count:-0}" -eq 0 ]; then
    if [ "$WANT_JSON" -eq 1 ]; then
        printf '{"ok":true,"firing":[],"checked":%d,"candidates":0,"scope":"%s"}\n' "$checked" "$SCOPE"
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-error-swallowing-predicate: clean — 0 unacknowledged empty-success gates ($checked negated-existence gate(s) scanned)."
        echo "  SCOPE: $SCOPE."
    fi
    exit 0
fi

if [ "$WANT_JSON" -eq 1 ]; then
    printf '{"ok":false,"firing":%s,"checked":%d,"candidates":%d,"scope":"%s"}\n' "$firing_json" "$checked" "$fire_count" "$SCOPE"
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-error-swallowing-predicate: FIRING — $fire_count empty-success gate(s) ($checked scanned):"
    printf '%s' "$firing_lines" | grep . | sed -E 's/^([^|]*)\|([0-9]+)\|([^|]*)\|([^|]*)\|(.*)$/  ↳ \1:\2  (in fn \3)  !.\4() -> \5/'
    echo "  Path::exists()/is_dir()/is_file() return FALSE on a permission error, so each site"
    echo "  above converts \"I could not read this\" into \"there is nothing here\" and returns it"
    echo "  as a success (Directive #2 — a plausible wrong answer, not an error)."
    echo "  Fix: distinguish absent from unreadable (see T-2791's discovery::classify_candidate,"
    echo "  which matches on the io::ErrorKind instead), OR — if the empty answer is genuinely"
    echo "  correct under EACCES — add the signature to $ALLOWLIST with a reason stating what"
    echo "  the code does when the predicate is false for a reason other than absence."
    echo "  SCOPE: $SCOPE."
    echo "---"
fi
exit 1
