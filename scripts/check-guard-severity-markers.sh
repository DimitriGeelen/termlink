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
# GUARD CAP (T-3644, adopted purpose DP-3, T-3612 draft v0.9 section 6.7). The same run
# also enforces the cap: 99 scripts/check-*.sh and 34 canary cron jobs, one in, one out.
# It lives here, not in a new check script, because a new check would itself break the
# cap. Against the tracked baseline .context/checks/guard-cap-baseline it fires:
#
#   NEWCOMER    a scripts/check-*.sh or .context/cron/*.crontab that the baseline does
#               not list (named). Retire a guard and update the baseline in one commit.
#   OVER-CAP    the baseline itself lists more checks or canaries than its cap line.
#
# A baseline entry whose file is gone is a retirement: reported, never a failure (the
# count went down). Cannot count (no baseline, no cap line, no scripts/ or cron dir,
# zero files found) is exit 2.
#
# Exit: 0 clean · 1 a malformed demotion marker, a NEWCOMER or OVER-CAP · 2 tooling
# (fail-closed).
# Seams: GUARD_SEVERITY_DIRS="dirA dirB" (default "scripts tests");
#        GUARD_CAP_ROOT (default: the repo this script sits in) holds scripts/ and
#        .context/cron/; GUARD_CAP_BASELINE (default $GUARD_CAP_ROOT/.context/checks/guard-cap-baseline).
set -uo pipefail

DIRS="${GUARD_SEVERITY_DIRS:-scripts tests}"
CAP_ROOT="${GUARD_CAP_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
CAP_BASELINE="${GUARD_CAP_BASELINE:-$CAP_ROOT/.context/checks/guard-cap-baseline}"
QUIET=0
for a in "$@"; do
    case "$a" in
        --quiet) QUIET=1 ;;
        --no-heartbeat) : ;;   # accepted for guard-layer symmetry; nothing heartbeats
        -h|--help) sed -n '3,46p' "$0"; exit 0 ;;
        *) echo "check-guard-severity-markers: unknown argument: $a" >&2; exit 2 ;;
    esac
done

scanned=0; demoted=0; n_warn=0; n_info=0; firing=()
for d in $DIRS; do
    [ -d "$d" ] || { echo "check-guard-severity-markers: dir not found: $d (fail-closed)" >&2; exit 2; }
    for f in "$d"/*.sh; do
        [ -e "$f" ] || continue
        marker="$(grep -m1 -E '^#[[:space:]]*guard-layer:[[:space:]]*source' "$f" 2>/dev/null || true)"
        # T-3632: a dated expected-red line must be valid and current; a bad one escalates anyway
        # (fail-safe), but it is named here so nobody believes it holds anything.
        if grep -qE '^#[[:space:]]*guard-layer-expected-red:' "$f" 2>/dev/null; then
            xr="$(GUARD_SEVERITY_DIRS="$DIRS" bash "$(dirname "${BASH_SOURCE[0]}")/guard-expected-red.sh" "$f" 2>/dev/null)"
            case "$xr" in
                active*) : ;;
                expired*) firing+=("EXPIRED    $f — guard-layer-expected-red date $(printf '%s' "$xr" | cut -f2) has passed: the member escalates again; remove the line or ask the operator") ;;
                *) firing+=("BAD-EXPECTED-RED $f — $(printf '%s' "$xr" | cut -f2-) (ignored: the member escalates as plain WARN)") ;;
            esac
        fi
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
                if [ "$first" != fail ] && printf '%s' " $first $rest_words " | grep -wE 'warn|info|advisory' >/dev/null; then
                    firing+=("MISPLACED  $f — a severity word must be the first word after 'source' (currently passed to the script as an argument)")
                elif [ "$first" = fail ] && grep -qwE 'warn|info|advisory' <<< "$rest_words"; then
                    firing+=("MISPLACED  $f — a severity word must be the first word after 'source' (currently passed to the script as an argument)")
                fi ;;
        esac
    done
done

if [ "$scanned" -eq 0 ]; then
    echo "check-guard-severity-markers: no guard-layer markers found under: $DIRS (fail-closed)" >&2
    exit 2
fi

# --- guard cap (T-3644) ------------------------------------------------------
cap_tooling=""
cap_firing=()
cap_retired=()
n_check=0; n_canary=0
cap_check=""; cap_canary=""
declare -A listed=()
listed_check=0; listed_canary=0
if [ ! -r "$CAP_BASELINE" ]; then
    cap_tooling="baseline not readable: $CAP_BASELINE"
elif [ ! -d "$CAP_ROOT/scripts" ] || [ ! -d "$CAP_ROOT/.context/cron" ]; then
    cap_tooling="cannot count: $CAP_ROOT/scripts or $CAP_ROOT/.context/cron is missing"
else
    while read -r kind val extra; do
        case "$kind" in
            ''|\#*) continue ;;
            cap)
                case "$val:$extra" in
                    check:[0-9]*) cap_check="$extra" ;;
                    canary:[0-9]*) cap_canary="$extra" ;;
                    *) cap_tooling="bad cap line in $CAP_BASELINE: cap $val $extra" ;;
                esac ;;
            check)       listed["$val"]=check;       listed_check=$((listed_check+1)) ;;
            canary)      listed["$val"]=canary;      listed_canary=$((listed_canary+1)) ;;
            operational) listed["$val"]=operational ;;
            *) cap_tooling="unknown line kind '$kind' in $CAP_BASELINE" ;;
        esac
    done < "$CAP_BASELINE"
    [ -n "$cap_check" ] && [ -n "$cap_canary" ] || cap_tooling="${cap_tooling:-$CAP_BASELINE has no 'cap check N' or 'cap canary N' line}"
fi
if [ -z "$cap_tooling" ]; then
    declare -A present=()
    for f in "$CAP_ROOT"/scripts/check-*.sh; do
        [ -e "$f" ] || continue
        rel="scripts/${f##*/}"; present["$rel"]=1
        case "${listed[$rel]:-}" in
            check) n_check=$((n_check+1)) ;;
            *) n_check=$((n_check+1)); cap_firing+=("NEWCOMER   $rel — not in the guard-cap baseline (cap $cap_check checks): retire a check and list this one in the same commit") ;;
        esac
    done
    for f in "$CAP_ROOT"/.context/cron/*.crontab; do
        [ -e "$f" ] || continue
        rel=".context/cron/${f##*/}"; present["$rel"]=1
        case "${listed[$rel]:-}" in
            canary) n_canary=$((n_canary+1)) ;;
            operational) : ;;
            *) cap_firing+=("NEWCOMER   $rel — cron job not in the guard-cap baseline: a canary must retire one (cap $cap_canary); an operational job is listed as 'operational'") ;;
        esac
    done
    if [ "$n_check" -eq 0 ] || [ "${#present[@]}" -eq "$n_check" ]; then
        cap_tooling="counted $n_check check script(s) and $(( ${#present[@]} - n_check )) cron job(s) under $CAP_ROOT — refusing to report a cap over nothing"
    fi
    for rel in "${!listed[@]}"; do
        [ -n "${present[$rel]:-}" ] || cap_retired+=("$rel")
    done
    [ "$listed_check" -le "$cap_check" ] || cap_firing+=("OVER-CAP   the baseline lists $listed_check check scripts; the cap is $cap_check")
    [ "$listed_canary" -le "$cap_canary" ] || cap_firing+=("OVER-CAP   the baseline lists $listed_canary canary cron jobs; the cap is $cap_canary")
fi

rc=0
if [ "${#firing[@]}" -gt 0 ]; then
    echo "guard severity markers: ${#firing[@]} malformed demotion marker(s) ($scanned marked member(s), $n_warn warn, $n_info info)"
    for x in "${firing[@]}"; do echo "  $x"; done
    echo "Scope: checks marker FORM only; whether a member should be warn/info is the operator's call."
    rc=1
else
    [ "$QUIET" -eq 1 ] || {
        echo "guard severity markers: clean — $scanned marked member(s), $n_warn warn, $n_info info (rest FAIL)"
        echo "Scope: checks marker FORM only; whether a member should be warn/info is the operator's call."
    }
fi

if [ -n "$cap_tooling" ]; then
    echo "check-guard-severity-markers: guard cap: $cap_tooling (fail-closed)" >&2
    exit 2
fi
if [ "${#cap_firing[@]}" -gt 0 ]; then
    echo "guard cap (DP-3): ${#cap_firing[@]} finding(s) — $n_check/$cap_check check scripts, $n_canary/$cap_canary canary cron jobs"
    for x in "${cap_firing[@]}"; do echo "  $x"; done
    echo "  baseline: ${CAP_BASELINE#$CAP_ROOT/}"
    rc=1
else
    [ "$QUIET" -eq 1 ] || echo "guard cap (DP-3): within baseline — $n_check/$cap_check check scripts, $n_canary/$cap_canary canary cron jobs"
fi
for r in "${cap_retired[@]:-}"; do
    [ -n "$r" ] || continue
    [ "$QUIET" -eq 1 ] || echo "  retired since baseline: $r (count went down; drop its line from the baseline)"
done
exit "$rc"
