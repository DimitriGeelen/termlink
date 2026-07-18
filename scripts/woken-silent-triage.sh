#!/usr/bin/env bash
# scripts/woken-silent-triage.sh (T-2416) — re-verify-and-clear the
# woken-but-silent canary log (G-085 prevention).
#
# WHY THIS EXISTS
#   G-083 built the DETECT side: agent-send.sh's escalate_woken_but_silent()
#   appends a framed entry to .woken-but-silent-canary.log when a rung recipient
#   never acks, and /canaries (T-2172) surfaces it. But that entry is written
#   ONCE, at send time, and nothing ever re-evaluates it. Two now-fixed bugs
#   produced false entries that then never cleared:
#     (a) pre-T-2413 the matcher was blind to msg_type=turn replies (the
#         canonical reply type) → a peer that DID reply was logged as silent;
#     (b) pre-T-2414 the confirmation window (90s) was tighter than measured
#         peer reply latency (aef 44s, sonnenstall 98s) → a reply arriving a
#         few seconds late left a permanent entry.
#   The RESIDUE keeps /canaries permanently RED — the cry-wolf failure that
#   trains an operator to ignore the signal, so a REAL woken-but-silent then
#   hides in the noise. G-085 is the mirror of G-083: G-083 = blind to the
#   silent send; G-085 = blind to the silent send being LATER resolved.
#
# WHAT IT DOES
#   Re-runs the live matcher (wake-confirm.sh) over each logged entry:
#     - CONSUMED (exit 0)      → RESOLVED: the reply arrived (false/late) →
#                                archived to .woken-but-silent-canary.resolved.log
#     - NOT-CONSUMED (exit 3)  → STILL-SILENT: genuinely unanswered → kept
#     - anything else          → INCONCLUSIVE (tooling/network) → kept (never
#                                clear an entry we could not actually re-verify)
#   Default is REPORT-ONLY (no mutation). --apply rewrites the live log with
#   only the still-silent (+ inconclusive) entries and archives the resolved
#   ones. A daily cron runs `--apply --quiet` so late replies self-clear within
#   a day (PL-168: a manual-only tool is dormant, not prevention).
#
# USAGE
#   scripts/woken-silent-triage.sh [--apply] [--json] [--quiet]
#                                  [--timeout N] [--log PATH] [--no-heartbeat]
#
# EXIT: 0 = no still-silent entries remain (healthy / all cleared)
#       1 = >=1 still-silent (or inconclusive) entry remains (real signal kept)
#       2 = usage / tooling error
#
# TEST SEAM (PL-213): WOKEN_TRIAGE_CONFIRM_CMD overrides the matcher invocation
#   so the triage is verified hub-independently. The override receives the same
#   args the real matcher would (--topic/--cid/--since-offset[/--hub]/--timeout)
#   and must exit 0 (consumed) / 3 (not-consumed) / other (inconclusive), and —
#   when it wants the resolved offset surfaced — print a wake-confirm-shaped JSON
#   line `{"consumed":true,"receipt_offset":N,...}` to stdout.
set -u

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── pure helpers (sourced in lib mode by the test harness) ────────────────────
# Extract a single field from one framed entry's text.
wst_field_cid()    { printf '%s' "$1" | sed -n 's/.*no receipt for cid=\([^ ]*\) on topic=.*/\1/p' | head -1; }
wst_field_topic()  { printf '%s' "$1" | sed -n 's/.*on topic=\(.*\)$/\1/p' | head -1; }
wst_field_offset() { printf '%s' "$1" | sed -n 's/.*turn posted at offset=\([0-9][0-9]*\).*/\1/p' | head -1; }
wst_field_hub()    { printf '%s' "$1" | grep -oE 'hub=[^ ]+' | head -1 | sed 's/^hub=//'; }
wst_field_ts()     { printf '%s' "$1" | sed -n 's/^=== \(.*\) ===$/\1/p' | head -1; }

# Classify an entry as parseable: needs a cid, a topic and a numeric offset.
wst_entry_valid() { # $1=block
    local b="$1"
    [ -n "$(wst_field_cid "$b")" ] && [ -n "$(wst_field_topic "$b")" ] \
        && [ -n "$(wst_field_offset "$b")" ]
}

# Lib mode: source helpers only, skip the driver (test harness sets this).
[ -n "${WOKEN_TRIAGE_LIB:-}" ] && return 0 2>/dev/null

# ── args ──────────────────────────────────────────────────────────────────────
apply=0 json=0 quiet=0 timeout=5 no_heartbeat=0
LOG="${TERMLINK_WOKEN_SILENT_LOG:-$SELF_DIR/../.context/working/.woken-but-silent-canary.log}"

die() { echo "woken-silent-triage: $*" >&2; exit 2; }
while [ $# -gt 0 ]; do
    case "$1" in
        --apply)         apply=1; shift ;;
        --json)          json=1; shift ;;
        --quiet)         quiet=1; shift ;;
        --no-heartbeat)  no_heartbeat=1; shift ;;
        --timeout)       timeout="${2:-}"; shift 2 ;;
        --log)           LOG="${2:-}"; shift 2 ;;
        -h|--help)
            sed -n '2,40p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) die "unknown arg: $1 (see --help)" ;;
    esac
done
[[ "$timeout" =~ ^[0-9]+$ ]] || die "--timeout must be a non-negative integer (got '$timeout')"

CONFIRM_CMD="${WOKEN_TRIAGE_CONFIRM_CMD:-bash $SELF_DIR/wake-confirm.sh}"
RESOLVED_LOG="${LOG%.log}.resolved.log"
HEARTBEAT="${LOG%.log}.heartbeat"

# Heartbeat is the liveness proof that the self-healer (this cron) ran. Only the
# mutating --apply path advances it; a report-only run is a non-mutating preview
# and must touch nothing. CRITICAL (canary-status FIRING = log_mtime >
# heartbeat_mtime): when still-silent entries REMAIN, the rewritten log must end
# up strictly NEWER than the heartbeat, or a genuine silent send would be masked
# as HEALTHY — see backdate_heartbeat below.
write_heartbeat()    { [ "$no_heartbeat" -eq 1 ] && return 0; date -u +%Y-%m-%dT%H:%M:%SZ > "$HEARTBEAT" 2>/dev/null || true; }
backdate_heartbeat() { [ "$no_heartbeat" -eq 1 ] && return 0; touch -d '-3 seconds' "$HEARTBEAT" 2>/dev/null || true; }

# Empty / missing log = healthy — nothing to triage.
if [ ! -s "$LOG" ]; then
    [ "$apply" -eq 1 ] && write_heartbeat
    if [ "$json" -eq 1 ]; then
        echo '{"ok":true,"resolved":[],"still_silent":[],"summary":{"total":0,"resolved":0,"still_silent":0,"inconclusive":0,"applied":'"$([ "$apply" -eq 1 ] && echo true || echo false)"'}}'
    elif [ "$quiet" -eq 0 ]; then
        echo "woken-silent-triage: log empty (healthy) — nothing to re-verify."
    fi
    exit 0
fi

# ── split the log into framed entries ─────────────────────────────────────────
workdir="$(mktemp -d)"; trap 'rm -rf "$workdir"' EXIT
awk -v dir="$workdir" '
    /^=== / { inblk=1; n++; blk="" }
    inblk   { blk = blk $0 "\n" }
    /^---$/ { if (inblk) { printf "%s", blk > (dir "/e" sprintf("%06d", n)); close(dir "/e" sprintf("%06d", n)); inblk=0 } }
' "$LOG"

keep_file="$workdir/keep"; : > "$keep_file"
resolved_file="$workdir/resolved"; : > "$resolved_file"
res_json="$workdir/res.json"; : > "$res_json"
sil_json="$workdir/sil.json"; : > "$sil_json"

total=0 n_res=0 n_sil=0 n_inc=0
now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

for ef in "$workdir"/e*; do
    [ -f "$ef" ] || continue
    block="$(cat "$ef")"
    total=$((total+1))
    if ! wst_entry_valid "$block"; then
        # unparseable → keep verbatim, count as inconclusive (never silently drop)
        n_inc=$((n_inc+1)); cat "$ef" >> "$keep_file"
        [ "$quiet" -eq 0 ] && [ "$json" -eq 0 ] && echo "  SKIP (unparseable entry, kept)"
        continue
    fi
    cid="$(wst_field_cid "$block")"; topic="$(wst_field_topic "$block")"
    off="$(wst_field_offset "$block")"; hub="$(wst_field_hub "$block")"
    hub_args=(); [ -n "$hub" ] && hub_args=(--hub "$hub")

    out="$($CONFIRM_CMD --topic "$topic" --cid "$cid" --since-offset "$off" \
            "${hub_args[@]+"${hub_args[@]}"}" --timeout "$timeout" --json 2>/dev/null)"; rc=$?

    if [ "$rc" -eq 0 ]; then
        roff="$(printf '%s' "$out" | jq -r '.receipt_offset // empty' 2>/dev/null)"
        n_res=$((n_res+1))
        {
            printf '=== %s (resolved-at %s) ===\n' "$(wst_field_ts "$block")" "$now"
            printf 'RESOLVED: cid=%s on topic=%s%s — reply/receipt found at offset=%s\n' \
                   "$cid" "$topic" "${hub:+ hub=$hub}" "${roff:-?}"
            printf '  original turn was at offset=%s; re-verified CONSUMED by woken-silent-triage (T-2416)\n' "$off"
            printf -- '---\n'
        } >> "$resolved_file"
        printf '%s\t%s\t%s\t%s\t%s\n' "$cid" "$topic" "$off" "${roff:-}" "${hub:-}" >> "$res_json"
        [ "$quiet" -eq 0 ] && [ "$json" -eq 0 ] && \
            echo "  RESOLVED   cid=$cid off=$off -> reply at offset=${roff:-?}${hub:+ (hub=$hub)}"
    elif [ "$rc" -eq 3 ]; then
        n_sil=$((n_sil+1)); cat "$ef" >> "$keep_file"
        printf '%s\t%s\t%s\t%s\n' "$cid" "$topic" "$off" "${hub:-}" >> "$sil_json"
        [ "$quiet" -eq 0 ] && [ "$json" -eq 0 ] && \
            echo "  STILL-SILENT cid=$cid off=$off${hub:+ (hub=$hub)} — genuinely unanswered, KEPT"
    else
        # inconclusive (network/tooling) → keep; never clear what we could not re-verify
        n_inc=$((n_inc+1)); cat "$ef" >> "$keep_file"
        printf '%s\t%s\t%s\t%s\n' "$cid" "$topic" "$off" "${hub:-}" >> "$sil_json"
        [ "$quiet" -eq 0 ] && [ "$json" -eq 0 ] && \
            echo "  INCONCLUSIVE cid=$cid off=$off (matcher rc=$rc) — KEPT (not re-verifiable now)"
    fi
done

# still-silent OR inconclusive means the canary should stay red.
kept=$((n_sil + n_inc))

# ── apply (mutate) or report ──────────────────────────────────────────────────
if [ "$apply" -eq 1 ]; then
    [ -s "$resolved_file" ] && { cat "$resolved_file" >> "$RESOLVED_LOG" 2>/dev/null || true; }
    if [ "$kept" -gt 0 ]; then
        # Entries remain → canary must stay FIRING. Write heartbeat first, then
        # back-date it and re-stamp the (rewritten) log so log_mtime > heartbeat_mtime.
        cp "$keep_file" "$LOG"
        write_heartbeat
        backdate_heartbeat
        touch "$LOG" 2>/dev/null || true
    else
        # All cleared → empty log is HEALTHY regardless of mtime (log_size guard).
        : > "$LOG"
        write_heartbeat
    fi
fi

if [ "$json" -eq 1 ]; then
    resolved_arr="$(jq -R -s -c 'split("\n")|map(select(length>0)|split("\t")|{cid:.[0],topic:.[1],offset:.[2],reply_offset:.[3],hub:.[4]})' "$res_json" 2>/dev/null || echo '[]')"
    silent_arr="$(jq -R -s -c 'split("\n")|map(select(length>0)|split("\t")|{cid:.[0],topic:.[1],offset:.[2],hub:.[3]})' "$sil_json" 2>/dev/null || echo '[]')"
    jq -cn --argjson res "$resolved_arr" --argjson sil "$silent_arr" \
        --argjson total "$total" --argjson nres "$n_res" --argjson nsil "$n_sil" \
        --argjson ninc "$n_inc" --argjson applied "$([ "$apply" -eq 1 ] && echo true || echo false)" \
        --argjson kept "$kept" \
        '{ok:($kept==0), resolved:$res, still_silent:$sil,
          summary:{total:$total, resolved:$nres, still_silent:$nsil, inconclusive:$ninc, kept:$kept, applied:$applied}}'
elif [ "$quiet" -eq 0 ] || [ "$kept" -gt 0 ]; then
    echo "woken-silent-triage: total=$total resolved=$n_res still_silent=$n_sil inconclusive=$n_inc$([ "$apply" -eq 1 ] && echo ' (applied)' || echo ' (report-only; --apply to clear)')"
fi

[ "$kept" -eq 0 ] && exit 0 || exit 1
