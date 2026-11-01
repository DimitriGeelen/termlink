#!/usr/bin/env bash
# guard-layer: source
# check-pipefail-grep-q.sh (T-3631, L-387 at script scale)
#
# WHY: under `set -o pipefail`, `producer | grep -q PAT` returns 141 when grep matches and exits while the producer is
# still writing (SIGPIPE). The pipeline then reads as "no match", at random for small outputs and every time for large
# ones. Measured here: runme fixture flakes (T-3627), install-sh-fixtures picking the wrong artifact 2 times in 300
# (2026-11-01), and AEF's Tier 0 / boundary hooks failing OPEN on large commands (AEF T-4920/T-4921).
# check-verification-pipefail.sh guards task Verification blocks only; nothing guarded scripts and tests.
#
# WHAT: a RATCHET. For every scripts/*.sh and tests/*.sh (plus runme.sh, install.sh) that turns on pipefail, count the
# non-comment lines with `| grep -q…`. The git-tracked baseline (.context/checks/pipefail-grep-q-baseline) records
# today's count per file. FIRES when a file not in the baseline has a site, or a file's count went UP. A count that
# went down is reported (lower the baseline with --update-baseline); it never fires.
#
# FIX a site: read all input instead — `grep_all` (grep without -q, output discarded; see runme.sh), a here-string
# (`grep -q PAT <<< "$out"`), or `grep PAT >/dev/null`.
#
# Usage: scripts/check-pipefail-grep-q.sh [--quiet] [--no-heartbeat] [--update-baseline]
#   --update-baseline  rewrite the baseline to today's counts; REFUSES if any count went up (exit 1)
# Exit: 0 no new or grown site · 1 firing · 2 tooling (fail-closed).
# Seams (PL-213): PIPEFAIL_GREPQ_ROOT (scan root), PIPEFAIL_GREPQ_BASELINE (baseline path).
set -uo pipefail

ROOT="${PIPEFAIL_GREPQ_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
BASELINE="${PIPEFAIL_GREPQ_BASELINE:-$ROOT/.context/checks/pipefail-grep-q-baseline}"
QUIET=0; UPDATE=0
for a in "$@"; do
    case "$a" in
        --quiet) QUIET=1 ;;
        --no-heartbeat) : ;;
        --update-baseline) UPDATE=1 ;;
        -h|--help) sed -n '2,24p' "$0"; exit 0 ;;
        *) echo "check-pipefail-grep-q: unknown argument: $a" >&2; exit 2 ;;
    esac
done
[ -d "$ROOT/scripts" ] || { echo "check-pipefail-grep-q: no scripts/ under $ROOT (fail-closed)" >&2; exit 2; }

# A pipe, not the second `|` of `||` (`a || grep -q x f` is no pipeline; T-3631 worker finding).
SITE_RE='^([^#]*[^|#])?\|[[:space:]]*grep[[:space:]]+-q'
declare -A now=() base=()
scanned=0
shopt -s nullglob
for f in "$ROOT"/scripts/*.sh "$ROOT"/tests/*.sh "$ROOT"/runme.sh "$ROOT"/install.sh; do
    [ -f "$f" ] || continue
    grep -qE 'pipefail' "$f" || continue
    scanned=$((scanned+1))
    n="$(grep -cE "$SITE_RE" "$f")"
    [ "$n" -gt 0 ] && now["${f#"$ROOT"/}"]="$n"
done
[ "$scanned" -gt 0 ] || { echo "check-pipefail-grep-q: no pipefail script found under $ROOT (fail-closed)" >&2; exit 2; }

if [ -f "$BASELINE" ]; then
    while read -r n p; do
        case "$n" in ''|\#*) continue ;; *[!0-9]*) echo "check-pipefail-grep-q: bad baseline line: $n $p" >&2; exit 2 ;; esac
        base["$p"]="$n"
    done < "$BASELINE"
elif [ "$UPDATE" -eq 0 ]; then
    echo "check-pipefail-grep-q: baseline $BASELINE missing (fail-closed; create it with --update-baseline)" >&2; exit 2
fi

grew=(); fell=(); total=0
for p in "${!now[@]}"; do
    total=$((total + now[$p]))
    b="${base[$p]:-0}"
    if [ "${now[$p]}" -gt "$b" ]; then grew+=("$p: ${now[$p]} (baseline $b)")
    elif [ "${now[$p]}" -lt "$b" ]; then fell+=("$p: ${now[$p]} (baseline $b)"); fi
done
for p in "${!base[@]}"; do [ -z "${now[$p]:-}" ] && [ "${base[$p]}" -gt 0 ] && fell+=("$p: 0 (baseline ${base[$p]})"); done

if [ "$UPDATE" -eq 1 ]; then
    if [ "${#grew[@]}" -gt 0 ] && [ -f "$BASELINE" ]; then
        echo "check-pipefail-grep-q: refusing to raise the baseline; fix these instead:"; printf '  %s\n' "${grew[@]}"; exit 1
    fi
    { echo "# T-3631 ratchet: '<count> <path>' of '| grep -q' sites in pipefail scripts. Counts may only go down."
      for p in "${!now[@]}"; do echo "${now[$p]} $p"; done | sort -k2; } > "$BASELINE"
    echo "check-pipefail-grep-q: baseline written ($total site(s) in ${#now[@]} file(s))"; exit 0
fi

if [ "${#grew[@]}" -gt 0 ]; then
    echo "check-pipefail-grep-q: FIRING — ${#grew[@]} file(s) gained a '| grep -q' under pipefail (L-387: reads as no-match at random)"
    printf '  %s\n' "${grew[@]}" | sort
    echo "  fix: grep_all (grep without -q, output discarded), a here-string, or 'grep PAT >/dev/null'"
    exit 1
fi
if [ "$QUIET" -eq 0 ]; then
    echo "check-pipefail-grep-q: clean — no new or grown site; $total remaining in ${#now[@]} file(s) under the ratchet"
    [ "${#fell[@]}" -gt 0 ] && { echo "  went down (lower the baseline: --update-baseline):"; printf '    %s\n' "${fell[@]}" | sort; }
fi
exit 0
