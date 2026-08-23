#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-error-code-emission.sh (T-2699, T-2698 G1)
#
# The sixth source-level static check (sibling of T-2527 alloc-sink, T-2531 drain-sink,
# T-2666 silent-exit, T-2672 busy-spin, T-2693 platform-lock).
#
# WHY THIS EXISTS
#
# TermLink publishes a refusal taxonomy: `control.rs::error_code` defines the errors it
# documents itself as able to return, and `check-error-code-docs.sh` (T-2213..T-2217)
# verifies that every doc-cited `SYMBOL(-320NN)` pairing matches the definition. That
# guards the docs against the CODE. Nothing guarded either against REALITY — whether
# the error can actually be emitted at all.
#
# T-2698 found three that cannot. Of 23 defined codes:
#   SESSION_BUSY (-32002)              documented as "Target cannot accept commands
#                                      (already executing)" — never emitted
#   MESSAGE_EXPIRED (-32004)           documented as "TTL exceeded before delivery"
#                                      — never emitted
#   PROTOCOL_VERSION_TOO_OLD (-32011)  has a BUILDER (`check_protocol_version`) that
#                                      constructs the structured error, and a passing
#                                      unit test — and ZERO callers
#
# The cost is not the dead constant. It is that the published contract overstates what
# is enforced: a client written from the protocol's error table would infer that a busy
# session refuses a second command, that an expired message is rejected rather than
# delivered late, and that the hub turns away peers speaking too old a protocol. None
# of those protections exist.
#
# PROTOCOL_VERSION_TOO_OLD is the shape that matters most, and the reason a naive
# "unused constant" lint would not have caught it: the mechanism is fully built and
# unit-tested, so every conventional coverage signal reads green. Coverage of a builder
# says nothing about whether the builder is called. That is the T-2683 pattern — a
# guard that exists and nothing executes — reproduced in compiled Rust rather than in
# shell scripts.
#
# WHAT COUNTS AS AN EMISSION
#   * `error_code::NAME` referenced anywhere in a product crate, OR
#   * the bare numeric literal (e.g. `-32004`) referenced there
# The DEFINING file is excluded — a constant must never count as its own use. A helper
# in the defining file that builds the error also does not count on its own; if nothing
# outside calls it, the code is unreachable and is reported. That is deliberate: it is
# precisely the PROTOCOL_VERSION_TOO_OLD case.
#
# WHY AN ALLOWLIST. A code genuinely reserved for a future protocol revision is
# legitimate — but it must be DECLARED, with a reason saying why it is not emitted and
# what would change that. Silence is what let three of them read as ordinary refusals.
# `.context/checks/error-code-emission-allowlist` is that ledger (git-tracked, T-2681).
#
# Exit codes: 0 clean · 1 unacknowledged never-emitted code · 2 tooling error
set -uo pipefail

DEF_FILE="${ERROR_CODE_DEF_FILE:-crates/termlink-protocol/src/control.rs}"
ROOTS=()
ALLOWLIST=""
FORMAT=human
QUIET=0

_default_allowlist() {
    if [ -f ".context/checks/error-code-emission-allowlist" ]; then
        printf '%s' ".context/checks/error-code-emission-allowlist"
    else
        printf '%s' ".context/working/.error-code-emission-allowlist"
    fi
}

usage() {
    cat <<'EOF'
check-error-code-emission.sh — every documented error code must be emittable.

Flags any `error_code::` constant with no emission site outside its defining file.
An emission is a reference to `error_code::NAME` or to the bare numeric literal.
A builder in the defining file does NOT count if nothing outside calls it — that is
the PROTOCOL_VERSION_TOO_OLD case this check exists for.

Usage: check-error-code-emission.sh [OPTIONS]
  --def-file PATH   Where the constants live (default crates/termlink-protocol/src/control.rs)
  --root PATH       Emission scan root (repeatable; defaults to the product crates)
  --allowlist PATH  Reserved-code ledger
  --json            Emit {ok, firing:[{code,value}], checked, allowlisted}
  --quiet           Print only on firing
  --no-heartbeat    Accepted for guard-layer parity; writes no heartbeat
  -h, --help        This help

Fixtures: bash tests/error-code-emission-fixtures.sh
Exit: 0 clean · 1 unacknowledged dead code · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --def-file) DEF_FILE="$2"; shift 2 ;;
        --root) ROOTS+=("$2"); shift 2 ;;
        --allowlist) ALLOWLIST="$2"; shift 2 ;;
        --json) FORMAT=json; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-error-code-emission: unknown arg: $1" >&2; exit 2 ;;
    esac
done

[ -f "$DEF_FILE" ] || { echo "check-error-code-emission: definition file not found: $DEF_FILE" >&2; exit 2; }

if [ "${#ROOTS[@]}" -eq 0 ]; then
    ROOTS=(
        crates/termlink-cli/src
        crates/termlink-mcp/src
        crates/termlink-session/src
        crates/termlink-hub/src
        crates/termlink-bus/src
        crates/termlink-protocol/src
    )
fi
[ -n "$ALLOWLIST" ] || ALLOWLIST="${ERROR_CODE_EMISSION_ALLOWLIST:-$(_default_allowlist)}"

EXIST=()
for r in "${ROOTS[@]}"; do [ -d "$r" ] && EXIST+=("$r"); done
[ "${#EXIST[@]}" -gt 0 ] || { echo "check-error-code-emission: no scan root exists" >&2; exit 2; }

declare -A ALLOW=()
allow_n=0
if [ -f "$ALLOWLIST" ]; then
    while IFS= read -r line; do
        entry="$(printf '%s' "$line" | sed -E 's/[[:space:]]*#.*$//; s/^[[:space:]]+//; s/[[:space:]]+$//')"
        [ -n "$entry" ] || continue
        ALLOW["$entry"]=1
        allow_n=$((allow_n + 1))
    done < "$ALLOWLIST"
fi

# Every file in scope EXCEPT the defining file. A constant must not count as its own
# use, and a builder that lives beside the definition proves nothing about reachability.
SCAN_FILES="$(find "${EXIST[@]}" -type f -name '*.rs' 2>/dev/null | grep -vF "$DEF_FILE" | sort)"
[ -n "$SCAN_FILES" ] || { echo "check-error-code-emission: no .rs files to scan" >&2; exit 2; }
SCAN_LIST="$(mktemp)"; printf '%s\n' "$SCAN_FILES" > "$SCAN_LIST"
trap 'rm -f "$SCAN_LIST"' EXIT

checked=0
firing=()

while IFS= read -r decl; do
    [ -n "$decl" ] || continue
    # [A-Z0-9_] not [A-Z_]: a SCREAMING_CASE constant may contain a digit
    # (HTTP2_ERROR, V2_REQUIRED). The narrower class silently SKIPPED such a code —
    # i.e. the check would have reported clean on a taxonomy it never fully read.
    # Found by this check's own fixture 8, which happened to name codes A1/A2/A3.
    name="$(printf '%s' "$decl" | sed -E 's/.*pub const ([A-Z0-9_]+): i64.*/\1/')"
    value="$(printf '%s' "$decl" | sed -E 's/.*=[[:space:]]*(-?[0-9]+).*/\1/')"
    [ -n "$name" ] || continue
    checked=$((checked + 1))

    # Symbolic use anywhere outside the defining file...
    if xargs -a "$SCAN_LIST" grep -l "error_code::${name}\b" >/dev/null 2>&1; then
        continue
    fi
    # ...or the bare numeric literal (a code emitted as `-32004` is still emitted).
    #
    # COMMENTS ARE STRIPPED FIRST. The check's own first run got this wrong and
    # cleared PROTOCOL_VERSION_TOO_OLD because `tools.rs` mentions "(-32011)" in a doc
    # comment. Prose about a code is not an emission of it — the same distinction the
    # platform-lock and silent-exit checks make, and the same way a false NEGATIVE
    # hides in a guard that otherwise looks thorough.
    if [ -n "$value" ] && xargs -a "$SCAN_LIST" sed -E 's://.*$::' 2>/dev/null \
        | grep -q -- "$value"; then
        continue
    fi
    [ -n "${ALLOW[$name]:-}" ] && continue
    firing+=("${name}|${value}")
done < <(grep -E "pub const [A-Z0-9_]+: i64" "$DEF_FILE")

n=${#firing[@]}

if [ "$FORMAT" = json ]; then
    printf '{"ok":%s,"firing":[' "$([ "$n" -eq 0 ] && echo true || echo false)"
    i=0
    while [ "$i" -lt "$n" ]; do
        [ "$i" -eq 0 ] || printf ','
        e="${firing[$i]}"
        printf '{"code":%s,"value":%s}' \
            "$(printf '%s' "${e%%|*}" | jq -R .)" "$(printf '%s' "${e##*|}" | jq -R .)"
        i=$((i+1))
    done
    printf '],"checked":%s,"allowlisted":%s}\n' "$checked" "$allow_n"
    [ "$n" -eq 0 ] && exit 0 || exit 1
fi

if [ "$n" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || echo "check-error-code-emission: clean — every defined error code is emittable ($checked scanned, $allow_n declared-reserved)."
    exit 0
fi

echo "check-error-code-emission: FIRING — $n error code(s) defined but NEVER emitted:"
for e in "${firing[@]}"; do
    echo "  ↳ ${e%%|*} (${e##*|})"
done
cat <<'EOF'
  These are published refusals the system cannot actually issue. The cost is not the
  dead constant — it is that the contract overstates what is enforced: a client written
  from the docs will infer a protection that does not exist.
  Fix: EMIT it from the path it describes, OR — if it is genuinely reserved for a future
  protocol revision — declare it in .context/checks/error-code-emission-allowlist with a
  reason saying why it is not emitted and what would change that. Silence is what let
  three of these read as ordinary refusals for months (T-2698).
  NOTE: a builder that constructs the error but has no callers does NOT count as an
  emission — that is exactly the PROTOCOL_VERSION_TOO_OLD case.
EOF
exit 1
