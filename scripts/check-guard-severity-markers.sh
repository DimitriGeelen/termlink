#!/usr/bin/env bash
# guard-layer: source
#
# check-guard-severity-markers.sh (T-3258, T-3260) — every demoting guard-layer
# marker (`warn`, `info`, or the T-3258 alias `advisory`) must carry a reason, must
# sit where the runner reads it, and must not be on a member that is always FAIL.
#
# run-guard-layer.sh lets a member demote itself from the default FAIL tier:
#
#     # guard-layer: source warn [extra args...]  # <reason>
#     # guard-layer: source info [extra args...]  # <reason>
#
# A WARN or INFO member never blocks push CI or a release. That is a real weakening
# of both gates, so it must never be silent or unexplained. Three shapes fire:
#
#   NO-REASON   a demotion word with no `# <reason>` after it. The runner already
#               refuses to honour it (runs the member as FAIL), so nothing is weakened —
#               but the author's intent is not what runs, and nothing else says so.
#   MISPLACED   a demotion word appears among the invocation args instead of as the
#               first word after `source`. The runner would pass it to the script as an
#               argument and treat the member as FAIL; the author believes otherwise.
#   ALWAYS-FAIL a demotion word on a tests/*fixtures*.sh suite. Fixture suites prove
#               the guards can fire; the runner ignores the word (T-3260).
#
# SCOPE (T-2680): checks the marker's FORM only. It does not judge whether a member
# SHOULD be warn/info — that classification is the operator's
# (docs/reports/T-3258-guard-classification-draft.md § Operator rulings).
#
# Exit: 0 clean · 1 a malformed demotion marker · 2 tooling (fail-closed).
# Seam: GUARD_SEVERITY_DIRS="dirA dirB" (default "scripts tests").
set -uo pipefail

DIRS="${GUARD_SEVERITY_DIRS:-scripts tests}"
QUIET=0
for a in "$@"; do
    case "$a" in
        --quiet) QUIET=1 ;;
        --no-heartbeat) : ;;   # accepted for guard-layer symmetry; nothing heartbeats
        -h|--help) sed -n '3,30p' "$0"; exit 0 ;;
        *) echo "check-guard-severity-markers: unknown argument: $a" >&2; exit 2 ;;
    esac
done

scanned=0; demoted=0; n_warn=0; n_info=0; firing=()
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
        case "$first" in
            warn|info|advisory)
                demoted=$((demoted+1))
                if [ "$first" = info ]; then n_info=$((n_info+1)); else n_warn=$((n_warn+1)); fi
                case "$(basename "$f")" in
                    *fixtures*) firing+=("ALWAYS-FAIL $f — fixture suites are always FAIL; '$first' is ignored by the runner") ;;
                esac
                [ -n "$reason" ] || firing+=("NO-REASON  $f — '$first' needs '# <reason>' (runs as FAIL until fixed)") ;;
            *)
                rest_words="$(printf '%s' "$pre" | awk '{$1=""; print}')"
                if [ "$first" != fail ] && printf '%s' " $first $rest_words " | grep -qwE 'warn|info|advisory'; then
                    firing+=("MISPLACED  $f — a severity word must be the first word after 'source' (currently passed to the script as an argument)")
                elif [ "$first" = fail ] && printf '%s' "$rest_words" | grep -qwE 'warn|info|advisory'; then
                    firing+=("MISPLACED  $f — a severity word must be the first word after 'source' (currently passed to the script as an argument)")
                fi ;;
        esac
    done
done

if [ "$scanned" -eq 0 ]; then
    echo "check-guard-severity-markers: no guard-layer markers found under: $DIRS (fail-closed)" >&2
    exit 2
fi

if [ "${#firing[@]}" -gt 0 ]; then
    echo "guard severity markers: ${#firing[@]} malformed demotion marker(s) ($scanned marked member(s), $n_warn warn, $n_info info)"
    for x in "${firing[@]}"; do echo "  $x"; done
    echo "Scope: checks marker FORM only; whether a member should be warn/info is the operator's call."
    exit 1
fi
[ "$QUIET" -eq 1 ] || {
    echo "guard severity markers: clean — $scanned marked member(s), $n_warn warn, $n_info info (rest FAIL)"
    echo "Scope: checks marker FORM only; whether a member should be warn/info is the operator's call."
}
exit 0
