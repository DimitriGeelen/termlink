#!/usr/bin/env bash
# guard-layer: source
#
# check-guard-severity-markers.sh (T-3258) — every `advisory` guard-layer marker
# must carry a reason, and must sit where the runner reads it.
#
# run-guard-layer.sh lets a member demote itself from BLOCKING to ADVISORY:
#
#     # guard-layer: source advisory [extra args...]  # <reason>
#
# Under `--gate release` an advisory member's failure no longer blocks a release.
# That is a real weakening of the release gate, so it must never be silent or
# unexplained. Two shapes fire:
#
#   NO-REASON   `advisory` with no `# <reason>` after it. The runner already refuses
#               to honour it (runs the member as BLOCKING), so nothing is weakened —
#               but the author's intent is not what runs, and nothing else says so.
#   MISPLACED   `advisory` appears among the invocation args instead of as the first
#               word after `source`. The runner would pass it to the script as an
#               argument and treat the member as BLOCKING; the author believes it is
#               advisory. Same silent divergence, other direction.
#
# SCOPE (T-2680): checks the marker's FORM only. It does not judge whether a member
# SHOULD be advisory — that classification is the operator's
# (docs/reports/T-3258-guard-classification-draft.md).
#
# Exit: 0 clean · 1 a malformed advisory marker · 2 tooling (fail-closed).
# Seam: GUARD_SEVERITY_DIRS="dirA dirB" (default "scripts tests").
set -uo pipefail

DIRS="${GUARD_SEVERITY_DIRS:-scripts tests}"
QUIET=0
for a in "$@"; do
    case "$a" in
        --quiet) QUIET=1 ;;
        --no-heartbeat) : ;;   # accepted for guard-layer symmetry; nothing heartbeats
        -h|--help) sed -n '3,27p' "$0"; exit 0 ;;
        *) echo "check-guard-severity-markers: unknown argument: $a" >&2; exit 2 ;;
    esac
done

scanned=0; advisory=0; firing=()
for d in $DIRS; do
    [ -d "$d" ] || { echo "check-guard-severity-markers: dir not found: $d (fail-closed)" >&2; exit 2; }
    for f in "$d"/*.sh; do
        [ -e "$f" ] || continue
        marker="$(grep -m1 -E '^#[[:space:]]*guard-layer:[[:space:]]*source' "$f" 2>/dev/null || true)"
        [ -n "$marker" ] || continue
        scanned=$((scanned+1))
        rest="$(printf '%s' "$marker" | sed -E 's/^#[[:space:]]*guard-layer:[[:space:]]*source[[:space:]]*//')"
        pre="${rest%%#*}"
        reason=""
        [ "$pre" != "$rest" ] && reason="$(printf '%s' "${rest#*#}" | tr -d '[:space:]')"
        first="$(printf '%s' "$pre" | awk '{print $1}')"
        if [ "$first" = advisory ]; then
            advisory=$((advisory+1))
            [ -n "$reason" ] || firing+=("NO-REASON  $f — 'advisory' needs '# <reason>' (runs as BLOCKING until fixed)")
        elif printf '%s' "$pre" | grep -qwE 'advisory'; then
            firing+=("MISPLACED  $f — 'advisory' must be the first word after 'source' (currently passed to the script as an argument)")
        fi
    done
done

if [ "$scanned" -eq 0 ]; then
    echo "check-guard-severity-markers: no guard-layer markers found under: $DIRS (fail-closed)" >&2
    exit 2
fi

if [ "${#firing[@]}" -gt 0 ]; then
    echo "guard severity markers: ${#firing[@]} malformed advisory marker(s) ($scanned marked member(s), $advisory advisory)"
    for x in "${firing[@]}"; do echo "  $x"; done
    echo "Scope: checks marker FORM only; whether a member should be advisory is the operator's call."
    exit 1
fi
[ "$QUIET" -eq 1 ] || {
    echo "guard severity markers: clean — $scanned marked member(s), $advisory advisory"
    echo "Scope: checks marker FORM only; whether a member should be advisory is the operator's call."
}
exit 0
