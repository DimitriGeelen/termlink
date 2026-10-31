#!/usr/bin/env bash
# guard-expected-red.sh — T-3632 (operator ruling C on T-3580, 2026-10-31): is a WARN guard member
# red BY DESIGN until a stated date?
#
# A WARN member may carry, anywhere in its header:
#     # guard-layer-expected-red: <YYYY-MM-DD>  # <reason — required>
# The member stays WARN (shown in the warning section, annotated in CI); only the 14-day ESCALATION
# (check-release-publication-freshness.sh + warn-escalation-file.sh) is held until the end of that day.
# Fail-safe: anything malformed is `invalid` and escalates exactly as before.
#
# Usage:  scripts/guard-expected-red.sh <member-name-or-path>
# Output: one tab-separated line
#   active   <date>  <reason>   declared, valid, not yet past     -> do not escalate
#   expired  <date>  <reason>   the date has passed               -> escalate
#   invalid  <why>              bad date / no reason / > cap / not a WARN member -> escalate
#   none                        no line, or the member file is not found         -> escalate
# Exit 0 with a verdict; 2 on usage error.
# Seams: GUARD_EXPECTED_RED_NOW (epoch), GUARD_SEVERITY_DIRS (member dirs, default "scripts tests"),
#        GUARD_EXPECTED_RED_CAP_DAYS (default 90).
set -uo pipefail
[ $# -eq 1 ] || { echo "usage: guard-expected-red.sh <member-name-or-path>" >&2; exit 2; }
NOW="${GUARD_EXPECTED_RED_NOW:-$(date -u +%s)}"
CAP_DAYS="${GUARD_EXPECTED_RED_CAP_DAYS:-90}"

m="$1"; f=""
if [ -f "$m" ]; then f="$(cd "$(dirname "$m")" && pwd)/$(basename "$m")"
else
    # A bare member name: look in the member dirs, relative to the caller first, then the repo root.
    root="$(git rev-parse --show-toplevel 2>/dev/null || (cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd))"
    for d in ${GUARD_SEVERITY_DIRS:-scripts tests}; do
        for base in "" "$root/"; do
            case "$d" in /*) cand="$d/$m" ;; *) cand="$base$d/$m" ;; esac
            [ -f "$cand" ] && { f="$cand"; break 2; }
        done
    done
fi
[ -n "$f" ] || { echo none; exit 0; }

line="$(grep -m1 -E '^#[[:space:]]*guard-layer-expected-red:' "$f" 2>/dev/null || true)"
[ -n "$line" ] || { echo none; exit 0; }

rest="${line#*guard-layer-expected-red:}"
pre="${rest%%#*}"
reason=""
[ "$pre" != "$rest" ] && reason="$(printf '%s' "${rest#*#}" | sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//')"
date="$(printf '%s' "$pre" | tr -d '[:space:]')"

marker="$(grep -m1 -E '^#[[:space:]]*guard-layer:[[:space:]]*source' "$f" 2>/dev/null || true)"
tier="$(printf '%s' "$marker" | sed -E 's/^#[[:space:]]*guard-layer:[[:space:]]*source[[:space:]]*//' | awk '{print $1}')"
case "$tier" in warn|advisory) : ;; *) printf 'invalid\tnot a WARN member (tier: %s)\n' "${tier:-fail}"; exit 0 ;; esac

[[ "$date" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]] || { printf 'invalid\tdate %s is not YYYY-MM-DD\n' "${date:-(empty)}"; exit 0; }
t="$(date -u -d "$date" +%s 2>/dev/null)" || { printf 'invalid\tdate %s does not parse\n' "$date"; exit 0; }
[ "$(date -u -d "@$t" +%F)" = "$date" ] || { printf 'invalid\tdate %s does not exist\n' "$date"; exit 0; }
[ -n "$reason" ] || { printf 'invalid\tno reason (needs "# <reason>")\n'; exit 0; }

end=$((t + 86400))   # the stated day is included
if [ $((end - NOW)) -gt $(( (CAP_DAYS + 1) * 86400 )) ]; then
    printf 'invalid\tdate %s is more than %s days ahead (cap)\n' "$date" "$CAP_DAYS"; exit 0
fi
if [ "$NOW" -ge "$end" ]; then printf 'expired\t%s\t%s\n' "$date" "$reason"
else printf 'active\t%s\t%s\n' "$date" "$reason"; fi
