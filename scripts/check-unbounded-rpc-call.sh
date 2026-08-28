#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-unbounded-rpc-call.sh (T-2669, G-019 prevention for the T-2641 hang class)
#
# WHY: `termlink_session::client::rpc_call` / `rpc_call_addr` connect UNBOUNDED and
# then call the UNBOUNDED `Client::call`. Per the T-2641 doc-comment on `client.rs`:
# "if the hub accepts the connection but never writes a response line, the caller
# blocks forever." That is a Directive #2 violation of the worst shape — not an
# error, but an indefinite silent hang with nothing surfaced to the caller. T-2641
# added the bounded `rpc_call_addr_with_timeout` precisely to fix the class, and
# T-2659 / T-2650 / T-2651 each migrated ONE site — but the caller sweep was never
# done, so the convention stayed DISCIPLINE-ONLY and the remaining handler functions
# kept the unbounded form. T-2669 did the sweep; this check is what makes it stick.
#
# WHAT: a grep/AST-lite scanner over the handler crates. For each `fn <name>` it
# brace-matches the function body, then keeps ONLY bodies that call the unbounded
# form:
#     rpc_call(  |  rpc_call_addr(
# The regex is anchored so the BOUNDED variants never match: after `rpc_call` the
# next character must be `(`, or the suffix must be exactly `_addr(`. So
# `rpc_call_with_timeout(` and `rpc_call_addr_with_timeout(` are excluded by
# construction, not by an exception. A candidate body FIRES when it contains no
# `tokio::time::timeout` token anywhere — an explicit wrap is an equally valid bound,
# so a handler that wraps its own call clears without being forced onto the helper.
#
# SCOPE — stated on every output path including the clean one (T-2680): this detects
# ONE shape, an unbounded client RPC in a function with no timeout in scope. A clean
# result means no unacknowledged unbounded call site. It does NOT mean every RPC in
# the tree is correctly bounded: a timeout token anywhere in a large function clears
# every call in it, and a bound enforced one frame up the call stack is invisible
# here. This is a regression ratchet on a swept surface, not a proof of boundedness.
#
# ALLOWLIST: `.context/checks/unbounded-rpc-call-allowlist` (git-tracked per T-2681)
# acknowledges sites that are unbounded ON PURPOSE, one drift-stable
# `<relpath>::<enclosing-fn>::unbounded-rpc-call` signature per line with a cited
# reason. The reason must say WHY the call may block forever — "known safe" is not a
# reason. The population is the T-2669 Bucket-B set: INTENTIONAL long-polls
# (`event.subscribe` / `event.poll` / `kv.watch` / `termlink.wait`), where the bound
# genuinely belongs server-side and a naive client timeout would break the feature by
# tearing down a healthy wait. Signatures are fn-name-based so they survive line
# moves; a fn RENAME re-fires the site, which is the intended re-review on meaningful
# change (same trade-off as check-alloc-sink-clamps / check-drain-sink-caps /
# check-silent-exit / check-busy-spin).
#
# LOAD-BEARING: reverting any T-2669-migrated site to the raw `rpc_call` form re-fires
# the check on that function; restoring returns the tree to clean. Fixtures:
# `bash tests/unbounded-rpc-call-fixtures.sh`.
#
# Exit codes: 0 = clean, 1 = an unacknowledged unbounded call site, 2 = tooling error.
set -uo pipefail

WANT_JSON=0 QUIET=0 HEARTBEAT=1
ROOTS=()
# T-2681 — tracked-first allowlist resolution.
_default_allowlist() {
    if [ -f ".context/checks/unbounded-rpc-call-allowlist" ]; then
        printf '%s' ".context/checks/unbounded-rpc-call-allowlist"
    else
        printf '%s' ".context/working/.unbounded-rpc-call-allowlist"
    fi
}
ALLOWLIST="${UNBOUNDED_RPC_ALLOWLIST:-$(_default_allowlist)}"

while [ $# -gt 0 ]; do
    case "$1" in
        --json) WANT_JSON=1; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) HEARTBEAT=0; shift ;;
        --root) shift; [ $# -gt 0 ] || { echo "check-unbounded-rpc-call: --root needs a value" >&2; exit 2; }; ROOTS+=("$1"); shift ;;
        --allowlist) shift; [ $# -gt 0 ] || { echo "check-unbounded-rpc-call: --allowlist needs a value" >&2; exit 2; }; ALLOWLIST="$1"; shift ;;
        -h|--help) sed -n '2,50p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "check-unbounded-rpc-call: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

command -v grep >/dev/null 2>&1 || { echo "check-unbounded-rpc-call: grep not found (required)" >&2; exit 2; }
command -v awk  >/dev/null 2>&1 || { echo "check-unbounded-rpc-call: awk not found (required)" >&2; exit 2; }

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(crates/termlink-cli/src crates/termlink-mcp/src)
fi
for r in "${ROOTS[@]}"; do
    [ -e "$r" ] || { echo "check-unbounded-rpc-call: scan root not found: $r" >&2; exit 2; }
done

# T-1723 heartbeat: prove this check ran, even on clean/error cycles.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.unbounded-rpc-call-canary.heartbeat}"
if [ "$HEARTBEAT" -eq 1 ]; then
    touch "$HEARTBEAT_FILE" 2>/dev/null || true
fi

# The UNBOUNDED client entry points. Anchored so the bounded variants cannot match:
# after `rpc_call` the next char must be `(`, or the suffix must be exactly `_addr(`.
UNBOUNDED_RE='(^|[^A-Za-z0-9_])rpc_call(_addr)?\('
# An explicit wrap is an equally valid bound.
TIMEOUT_RE='tokio::time::timeout'
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

# --- fn-body brace matcher ----------------------------------------------------
# Given a file and the 1-indexed line of a `fn` declaration, print the END line (the
# line holding the matching close brace) by counting braces from the first `{` at or
# after the declaration. Comment awareness is coarse (strips // line comments);
# adequate for this tree's style, and matched to the sibling checks. Emits nothing if
# unbalanced (defensive — treated as no-body).
fn_body_end() {
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
[ -n "$FILES" ] || { echo "check-unbounded-rpc-call: no .rs files under scan roots" >&2; exit 2; }

checked=0
acked=0
firing_lines=""
acked_lines=""

while IFS= read -r file; do
    [ -n "$file" ] || continue

    # Every fn declaration in the file, outermost-first by line number.
    while IFS= read -r hit; do
        [ -n "$hit" ] || continue
        startln="${hit%%:*}"
        fcode="${hit#*:}"
        fname="$(fn_name_of "$fcode")"

        endln="$(fn_body_end "$file" "$startln")"
        [ -n "$endln" ] || continue
        [ "$endln" -gt "$startln" ] || continue

        # Strip `//` line comments BEFORE testing. Prose about a call is not a call:
        # a commented-out `rpc_call` must not fire, and — the direction that actually
        # bites — a commented-out `tokio::time::timeout` must not CLEAR a live one.
        # (The T-2688 lesson, applied to both sides of the predicate rather than only
        # the one that produces a visible false positive.)
        body="$(sed -n "${startln},${endln}p" "$file" | sed -E 's://.*$::')"

        # candidate iff the body calls an UNBOUNDED client entry point
        printf '%s' "$body" | grep -qE "$UNBOUNDED_RE" || continue

        checked=$((checked + 1))

        # clear if the body carries an explicit timeout wrap anywhere
        printf '%s' "$body" | grep -q "$TIMEOUT_RE" && continue

        sig="${file}::${fname}::unbounded-rpc-call"
        if [ -n "${ALLOW[$sig]:-}" ]; then
            acked=$((acked + 1))
            acked_lines="${acked_lines}${file}:${startln}:${fname}"$'\n'
            continue
        fi
        firing_lines="${firing_lines}${file}:${startln}:${fname}"$'\n'
    done < <(grep -nE "$FN_RE" "$file" 2>/dev/null)
done <<< "$FILES"

fire_count="$(printf '%s' "$firing_lines" | grep -c . || true)"

SCOPE_NOTE="scope: detects an unbounded client rpc_call in a fn with no tokio::time::timeout in scope; a clean result is not a proof that every RPC in the tree is bounded"

# Render a `<file>:<line>:<fn>` block as a JSON array. The empty-input case is
# handled explicitly: under `pipefail` a `grep .` on empty input fails the whole
# pipeline, so a trailing `|| echo '[]'` would fire AFTER jq had already printed its
# own `[]` — emitting `[]\n[]` and producing invalid JSON. Acknowledged sites can
# legitimately be empty while firing sites are not, so this path is reachable.
_json_arr() {
    local lines="$1" body
    body="$(printf '%s' "$lines" | grep . || true)"
    if [ -z "$body" ] || ! command -v jq >/dev/null 2>&1; then
        printf '[]'
        return
    fi
    printf '%s' "$body" \
        | sed -E 's/^([^:]+):([0-9]+):(.*)$/{"file":"\1","line":\2,"fn":"\3"}/' \
        | jq -sc '.' 2>/dev/null || printf '[]'
}

firing_json="$(_json_arr "$firing_lines")"
acked_json="$(_json_arr "$acked_lines")"

if [ "${fire_count:-0}" -eq 0 ]; then
    if [ "$WANT_JSON" -eq 1 ]; then
        printf '{"ok":true,"firing":[],"checked":%d,"candidates":0,"acknowledged_count":%d,"acknowledged":%s,"scope":"%s"}\n' \
            "$checked" "$acked" "$acked_json" "$SCOPE_NOTE"
    elif [ "$QUIET" -eq 0 ]; then
        echo "check-unbounded-rpc-call: clean — 0 unacknowledged unbounded rpc_call site(s) ($checked call site(s) scanned, $acked acknowledged in the ledger)."
        echo "  NOTE: an acknowledgement is NOT a claim the site is bounded. The ledger holds two"
        echo "  classes — intentional long-polls (bound belongs server-side) and sites NOT YET"
        echo "  MIGRATED (still the T-2641 hang class). Read $ALLOWLIST for the split."
        echo "  ($SCOPE_NOTE)"
    fi
    exit 0
fi

if [ "$WANT_JSON" -eq 1 ]; then
    printf '{"ok":false,"firing":%s,"checked":%d,"candidates":%d,"acknowledged_count":%d,"acknowledged":%s,"scope":"%s"}\n' \
        "$firing_json" "$checked" "$fire_count" "$acked" "$acked_json" "$SCOPE_NOTE"
else
    echo "=== $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
    echo "check-unbounded-rpc-call: FIRING — $fire_count fn(s) calling an unbounded rpc_call with no timeout in scope:"
    printf '%s' "$firing_lines" | grep . | sed -E 's/^([^:]+):([0-9]+):(.*)$/  ↳ \1:\2  (in fn \3)/'
    echo "  An unbounded client::rpc_call / rpc_call_addr blocks FOREVER if the hub accepts the"
    echo "  connection but never writes a response line (T-2641). Fix: migrate to the bounded"
    echo "  rpc_call_with_timeout / rpc_call_addr_with_timeout (the T-2669 remediation), or wrap"
    echo "  the call in an explicit tokio::time::timeout, OR — if the call is an INTENTIONAL"
    echo "  long-poll whose bound belongs server-side — add its signature to $ALLOWLIST"
    echo "  with a reason stating why it may block forever."
    echo "  ($SCOPE_NOTE)"
    echo "---"
fi
exit 1
