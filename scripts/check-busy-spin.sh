#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-busy-spin.sh (T-2672, G-019 prevention for the T-2658/T-2636/T-2640/T-2670/T-2671
# long-poll busy-spin class)
#
# WHY: a `loop {}` that re-dispatches a long-poll RPC (event.subscribe / event.collect /
# event.poll) whose error arm re-iterates with NO `tokio::time::sleep` busy-spins a CPU
# core the instant the hub goes dead/half-open (Directive #1 antifragility + Directive #2
# "no silent failures"). On a LIVE hub the long-poll paces the loop; on a DEAD/half-open
# hub the RPC errors near-instantly, so a bare `continue` re-dispatches with zero delay —
# and silently (the `tracing::warn!` above the continue is gated out at the default log
# level). T-2670 fixed two such sites in agent.rs (cmd_agent_negotiate / cmd_agent_listen
# subscribe-retry loops) and T-2671 fixed one in tools.rs (termlink_dispatch's
# event.collect loop). The blessed remediation is a 500ms sleep-on-error before the next
# iteration — the established convention across events.rs:805/900/1349 and dispatch.rs
# (COLLECT_ERR_BACKOFF). But the convention was DISCIPLINE-ONLY: nothing detected the
# next long-poll loop shipped without the backoff. This check makes it load-bearing.
#
# WHAT: a grep/AST-lite scanner over the long-poll-adjacent crates. For each `loop {`
# it brace-matches the loop body, then keeps ONLY bodies that dispatch a long-poll RPC
# method string:
#     "event.subscribe"  |  "event.collect"  |  "event.poll"
# (a bounded `recv`/`accept`/iterator loop, or an analytics parse loop, is NOT a
# candidate — it does not re-dispatch a long-poll RPC, so the class cannot apply). A
# candidate body FIRES when it contains NO `tokio::time::sleep` call anywhere in the
# body. This is deliberately narrow and low-FP: the ~74 analytics loops in tools.rs and
# the 5 known-safe loops (bounded recv / accept-blocks / Err-arm-breaks / bounded-SQLite
# drain) do not dispatch the long-poll strings, so they are excluded by construction; and
# every convention-correct long-poll loop carries the 500ms backoff, so it clears on the
# `sleep` presence.
#
# The "sleep present -> clear" rule is intentionally the SAFE side of the trade-off: a
# long-poll loop that paces its error path some other way (e.g. an outer per-iteration
# `tokio::time::timeout` that ALSO breaks the loop on the timeout branch) is a confirmed
# exception, acknowledged in the allowlist. A per-call `timeout` alone does NOT clear a
# site — the timeout bounds the SLOW-success case, not the INSTANT-error case, so an
# error-arm `continue` still busy-spins; such a loop must carry a sleep OR break out on
# error. When in doubt the check fires and a human confirms-and-clamps or allowlists.
#
# ALLOWLIST: `.context/working/.busy-spin-allowlist` acknowledges confirmed-safe long-poll
# loops (error path exits the loop, or the loop is paced by a mechanism the grep cannot
# see). One drift-stable signature per line:
#     <relpath>::<enclosing-fn>::busy-spin
# fn-name-based (survives line moves; a fn RENAME re-fires — the same trade-off as the
# sibling source checks). After T-2670/T-2671 the current tree scans CLEAN, so the
# allowlist is empty.
#
# Output is a REVIEW list, not a hard gate. NOT a runtime cron canary — a source-level
# static check (sibling of scripts/check-silent-exit.sh (T-2666),
# scripts/check-alloc-sink-clamps.sh (T-2527), scripts/check-drain-sink-caps.sh (T-2531)).
#
# EXIT CODES:
#   0  clean    -- every long-poll loop carries a backoff (or is allowlisted).
#   1  firing   -- >=1 long-poll loop with no sleep-on-error, not cleared by allowlist.
#   2  tooling  -- missing dep / bad scan root.
#
# USAGE:
#   check-busy-spin.sh [--json] [--quiet] [--no-heartbeat]
#                      [--root <dir>]... [--allowlist <file>]
#     --json          emit {ok, firing:[{file,line,fn}], checked, candidates}
#     --quiet         print only on firing (cron mode); clean prints nothing
#     --no-heartbeat  skip the heartbeat touch (meta-canary invokes with this)
#     --root <dir>    override scan roots (repeatable; default = the long-poll crates)
#     --allowlist <f> override allowlist path (fixtures point this at a scratch file)
#
# Origin: T-2672. Load-bearing proof: tests/busy-spin-check-fixtures.sh, and reverting
# T-2670/T-2671's sleep-on-error on any fixed site re-fires the check on that loop.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
ROOTS=()
# T-2681 — tracked-first allowlist resolution; see the header of
# .context/checks/alloc-sink-allowlist for why. Legacy gitignored path kept as a
# fallback; explicit BUSY_SPIN_ALLOWLIST / --allowlist always wins over both.
_default_allowlist() {
    if [ -f ".context/checks/busy-spin-allowlist" ]; then
        printf '%s' ".context/checks/busy-spin-allowlist"
    else
        printf '%s' ".context/working/.busy-spin-allowlist"
    fi
}
ALLOWLIST="${BUSY_SPIN_ALLOWLIST:-$(_default_allowlist)}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --root) shift; [ $# -gt 0 ] || { echo "check-busy-spin: --root needs a value" >&2; exit 2; }; ROOTS+=("$1"); shift ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-busy-spin: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1"; shift ;;
        -h|--help) sed -n '2,96p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-busy-spin: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-busy-spin: grep not found (required)" >&2; exit 2; }
command -v awk  >/dev/null 2>&1 || { echo "check-busy-spin: awk not found (required)" >&2; exit 2; }

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(crates/termlink-cli/src crates/termlink-mcp/src)
fi
for r in "${ROOTS[@]}"; do
    [ -e "$r" ] || { echo "check-busy-spin: scan root not found: $r" >&2; exit 2; }
done

# T-1723 heartbeat: prove this check ran, even on clean/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.busy-spin-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# The long-poll RPC method strings whose re-dispatch defines a candidate loop.
LONGPOLL_RE='"event\.subscribe"|"event\.collect"|"event\.poll"'
# The backoff whose presence clears a candidate.
SLEEP_RE='tokio::time::sleep'
# A Rust fn declaration line.
FN_RE='(^|[^A-Za-z0-9_])fn[[:space:]]+[A-Za-z0-9_]+'

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

# --- loop-body brace matcher --------------------------------------------------
# Given a file and the 1-indexed line of a `loop {`, print the END line (the line
# holding the matching close brace) by counting braces from the opening `{`.
# Comment/string awareness is coarse (strips // line comments); adequate for this
# tree's style. Emits nothing if unbalanced (defensive — treated as no-body).
loop_body_end() {
    local file="$1" startln="$2"
    awk -v start="$startln" '
        NR < start { next }
        {
            line = $0
            sub(/\/\/.*$/, "", line)   # drop line comments (coarse)
            n = length(line)
            for (i = 1; i <= n; i++) {
                c = substr(line, i, 1)
                if (c == "{") { depth++; seen = 1 }
                else if (c == "}") {
                    depth--
                    if (seen && depth == 0) { print NR; exit }
                }
            }
        }
    ' "$file"
}

# --- scan ---------------------------------------------------------------------
FILES="$(find "${ROOTS[@]}" -type f -name '*.rs' 2>/dev/null | sort)"
[ -n "$FILES" ] || { echo "check-busy-spin: no .rs files under scan roots" >&2; exit 2; }

checked=0
firing_lines=""

while IFS= read -r file; do
    [ -n "$file" ] || continue

    fnmap="$(grep -nE "$FN_RE" "$file" 2>/dev/null | while IFS= read -r fl; do
        fln="${fl%%:*}"; fcode="${fl#*:}"
        printf '%s:%s\n' "$fln" "$(fn_name_of "$fcode")"
    done)"

    # every `loop {` opener (the token `loop` immediately followed by `{` on the line)
    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        startln="${hit%%:*}"

        endln="$(loop_body_end "$file" "$startln")"
        [ -n "$endln" ] || continue
        [ "$endln" -gt "$startln" ] || continue

        body="$(sed -n "${startln},${endln}p" "$file")"

        # candidate iff the body re-dispatches a long-poll RPC method string
        printf '%s' "$body" | grep -qE "$LONGPOLL_RE" || continue

        checked=$((checked + 1))

        # clear if the body carries a backoff sleep anywhere
        printf '%s' "$body" | grep -q "$SLEEP_RE" && continue

        # enclosing fn = the fn-decl line with the largest lineno <= this loop's opener
        encfn="-"
        while IFS= read -r fm; do
            [ -n "$fm" ] || continue
            fln="${fm%%:*}"; fnm="${fm#*:}"
            [ "$fln" -le "$startln" ] && encfn="$fnm"
            [ "$fln" -gt "$startln" ] && break
        done <<< "$fnmap"

        sig="${file}::${encfn}::busy-spin"
        [ -n "${ALLOW[$sig]:-}" ] && continue
        firing_lines="${firing_lines}${file}:${startln}:${encfn}"$'\n'
    done < <(grep -nE '(^|[^A-Za-z0-9_])loop[[:space:]]*\{' "$file" 2>/dev/null)
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
        echo "check-busy-spin: clean — 0 unacknowledged long-poll busy-spin loops ($checked long-poll loop(s) scanned)."
    fi
    exit 0
fi

if [ "$WANT_JSON" -eq 1 ]; then
    printf '{"ok":false,"firing":%s,"checked":%d,"candidates":%d}\n' "$firing_json" "$checked" "$fire_count"
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-busy-spin: FIRING — $fire_count long-poll loop(s) with no sleep-on-error backoff:"
    printf '%s' "$firing_lines" | grep . | sed -E 's/^([^:]+):([0-9]+):(.*)$/  ↳ \1:\2  (in fn \3)/'
    echo "  A long-poll loop (event.subscribe/collect/poll) whose error arm re-iterates with no"
    echo "  tokio::time::sleep busy-spins a CPU core the instant the hub goes dead/half-open, and"
    echo "  silently (the warn! is gated out at the default log level). Fix: add the 500ms"
    echo "  sleep-on-error before the next iteration (the T-2670/T-2671 remediation; convention at"
    echo "  events.rs:805/900/1349, dispatch.rs COLLECT_ERR_BACKOFF), OR — if the error path"
    echo "  provably exits the loop — add the loop's signature to $ALLOWLIST with a cited reason."
    echo "---"
fi
exit 1
