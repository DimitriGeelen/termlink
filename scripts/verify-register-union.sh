#!/usr/bin/env bash
# verify-register-union.sh (T-2830)
#
# WHY: the t2687 -> main integration collides on append-only registers
# (.context/project/{decisions,learnings,metrics-history}.yaml and .context/episodic/*).
# These are logs, not state: an entry recorded on either side is a fact that happened,
# so a merge may only ever ADD. The failure mode is silent — resolving such a conflict
# by picking one side compiles, tests green, audit passes, and quietly deletes history.
# Nothing else in the tree would notice.
#
# WHAT: for each register, take the union of entry ids across BOTH merge parents and
# assert every one survives in the working-tree file. Reports which ids were lost and
# from which side, because "the merge dropped something" is useless without the list.
#
# It deliberately does NOT check that no ids were ADDED — a merge commit may legitimately
# introduce new entries, and forbidding that would fire on every honest resolution.
#
# USAGE:
#   verify-register-union.sh [--parents <ref1> <ref2>]
# Defaults to the current commit's first two parents (i.e. run it on the merge commit),
# falling back to HEAD + origin/main when HEAD is not a merge (i.e. mid-resolution).
#
# EXIT: 0 = every id from both parents survives; 1 = at least one lost; 2 = tooling.

set -uo pipefail

REGISTERS=(
    ".context/project/decisions.yaml"
    ".context/project/learnings.yaml"
    ".context/project/concerns.yaml"
    ".context/project/patterns.yaml"
)

P1="" P2=""
while [ $# -gt 0 ]; do
    case "$1" in
        --parents) P1="${2:-}"; P2="${3:-}"; shift 3 ;;
        -h|--help) sed -n '2,26p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "verify-register-union: unknown arg '$1'" >&2; exit 2 ;;
    esac
done

# Resolve the two sides being merged. On a merge commit use its real parents; while the
# merge is still in progress (or already committed onto the branch) fall back to the
# recorded MERGE_HEAD, then to origin/main. Guessing silently would let this pass
# vacuously by comparing a ref against itself, so an unresolvable second side is fatal.
if [ -z "$P1" ]; then
    P1="$(git rev-parse --verify HEAD^1 2>/dev/null || git rev-parse --verify HEAD 2>/dev/null || true)"
fi
if [ -z "$P2" ]; then
    P2="$(git rev-parse --verify HEAD^2 2>/dev/null || true)"
    [ -n "$P2" ] || P2="$(git rev-parse --verify MERGE_HEAD 2>/dev/null || true)"
    [ -n "$P2" ] || P2="$(git rev-parse --verify origin/main 2>/dev/null || true)"
fi
if [ -z "$P1" ] || [ -z "$P2" ]; then
    echo "verify-register-union: could not resolve both merge parents" >&2
    exit 2
fi
if [ "$P1" = "$P2" ]; then
    echo "verify-register-union: both parents resolved to the same commit ($P1) — refusing a vacuous pass" >&2
    exit 2
fi

ids_of() {
    # $1 = ref or "-" for working tree, $2 = path. Missing file on one side is not an
    # error: the register may have been created on only one branch.
    if [ "$1" = "-" ]; then
        [ -r "$2" ] || return 0
        grep -oE '\bid: [A-Za-z]+-[0-9]+' "$2" 2>/dev/null | awk '{print $2}' | sort -u
    else
        git show "$1:$2" 2>/dev/null | grep -oE '\bid: [A-Za-z]+-[0-9]+' | awk '{print $2}' | sort -u
    fi
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

total_lost=0
checked=0

for reg in "${REGISTERS[@]}"; do
    ids_of "$P1" "$reg" > "$TMP/a"
    ids_of "$P2" "$reg" > "$TMP/b"
    sort -u "$TMP/a" "$TMP/b" > "$TMP/want"
    [ -s "$TMP/want" ] || continue          # register exists on neither side
    checked=$((checked + 1))

    ids_of "-" "$reg" > "$TMP/have"
    comm -23 "$TMP/want" "$TMP/have" > "$TMP/lost"

    n_lost="$(grep -c . < "$TMP/lost" || true)"
    n_want="$(grep -c . < "$TMP/want" || true)"
    if [ "${n_lost:-0}" -eq 0 ]; then
        echo "  ok   $reg — all ${n_want} id(s) from both parents survive"
    else
        total_lost=$((total_lost + n_lost))
        echo "  LOST $reg — ${n_lost} of ${n_want} id(s) dropped by the merge:"
        while read -r lost_id; do
            [ -n "$lost_id" ] || continue
            side=""
            grep -qx "$lost_id" "$TMP/a" && side="ours"
            grep -qx "$lost_id" "$TMP/b" && side="${side:+$side+}theirs"
            echo "    ↳ $lost_id (was on: ${side:-unknown})"
        done < "$TMP/lost"
    fi
done

if [ "$checked" -eq 0 ]; then
    echo "verify-register-union: no registers found on either parent — nothing verified" >&2
    exit 2
fi

if [ "$total_lost" -gt 0 ]; then
    echo
    echo "verify-register-union: FAIL — ${total_lost} register entr(ies) lost. These are"
    echo "append-only logs; a merge may add but never drop. Recover the missing ids from"
    echo "the parent named above and re-run."
    exit 1
fi

echo "verify-register-union: OK — ${checked} register(s), no entries lost across the merge."
exit 0
