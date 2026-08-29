#!/usr/bin/env bash
#
# check-hook-counter-integrity.sh (T-2795)
#
# TIER: on-demand diagnostic. NOT a guard-layer source check, and NOT a cron canary.
#
# It carries no `# guard-layer:` marker on purpose, and the reason is the finding itself.
# The file it inspects — `.context/working/.hook-counter` — is corrupted continuously by
# ordinary concurrent hook fires (see MECHANISM below). Nothing inside this repository can
# stop that; the write path lives in the vendored framework. A guard-layer member would
# therefore report FAIL on every run forever, which is precisely the failure mode this
# repo has spent T-2709, T-2787 and T-2680 documenting: a guard that fires regardless of
# system state teaches its operator to stop reading the roll-up. Enrolling this would
# damage the layer it joined. Run it when you need the number; do not schedule it.
#
# --- MECHANISM ---
# `lib/hook-telemetry.sh::_fw_telemetry_increment` maintains a flat `key=count` file with
# an unlocked read-modify-write:
#
#     mapfile -t lines < "$file"      # read
#     ...                             # modify in memory
#     printf '%s\n' "${lines[@]}" > "$file"   # truncate + write
#
# There is no lock and no temp+rename. Claude Code fires PreToolUse and PostToolUse hooks
# concurrently — and runs independent tool calls in parallel — so two fires interleave:
# both read the same snapshot, both write, and the later write erases the earlier one's
# increment. The truncate is not atomic either, so a reader (or another writer's mapfile)
# can observe a partially written file. The two artefacts are:
#
#   * DUPLICATE keys   — a stale in-memory copy appended after the file was rewritten
#   * MALFORMED lines  — a bare key with no `=count`, caught mid-write
#
# This is L-023 verbatim ("truncate+write races silently lose data under parallel calls",
# recorded 2026-03-26 from T-248's bypass registry) recurring in a file written by every
# single hook fire. The learning existed before this code did.
#
# --- WHY IT MATTERS: THE TWO READERS DISAGREE ---
# The corruption would be tolerable if every consumer read it the same wrong way. They do
# not:
#
#   fw_hook_counter_get   awk '$1==k {print $2; exit}'   -> FIRST match wins
#   hook-threshold.py     counts[key] += val             -> SUMS duplicates
#
# So a hook with `error-watchdog=29` and `error-watchdog=28` in the file reads as 29 to
# `fw doctor` and 58 to the T-1626 decay alarm. The alarm's denominator is inflated by
# exactly the concurrency a busy session produces, which SUPPRESSES the failure ratio —
# every duplicate key roughly halves it. The alarm therefore goes blinder the busier the
# session, which is when hooks matter most.
#
# Note the sign, because it is the sharp part. T-2713 shows the NUMERATOR is inflated
# (intentional exit-2 blocks counted as failures, biasing toward false alarms). This
# defect deflates the DENOMINATOR-derived ratio, biasing toward false silence. The two
# point in opposite directions, so the net error is not a consistent bias you could
# calibrate around — its sign depends on how parallel the session happened to be.
#
# A malformed line is worse still for `fw_hook_counter_get`: `$1==k` matches the bare key
# and `$2` is empty, so it prints an EMPTY STRING rather than a number. Observed live for
# `check-active-task` — the hook enforcing the framework's Core Principle — while a valid
# `check-active-task=12` sat further down the same file.
#
# SCOPE: this reports corruption of the counter FILE and disagreement between its readers.
# It says nothing about whether any hook is actually healthy, and a clean result does not
# mean the telemetry is trustworthy — T-2713's exit-code conflation is a separate defect
# that leaves the file perfectly well-formed.
#
# Exit codes: 0 = no corruption found, 1 = corruption present, 2 = tooling error.
set -u

COUNTER=""
FAILCOUNTER=""
FORMAT=human
QUIET=0

die() {
    if [ "$FORMAT" = json ]; then printf '{"ok":false,"error":"%s"}\n' "$1"
    else echo "check-hook-counter-integrity: $1" >&2; fi
    exit 2
}

while [ $# -gt 0 ]; do
    case "$1" in
        --counter)         COUNTER="${2:-}"; shift 2 ;;
        --failure-counter) FAILCOUNTER="${2:-}"; shift 2 ;;
        --json)            FORMAT=json; shift ;;
        --quiet)           QUIET=1; shift ;;
        -h|--help)         sed -n '3,70p' "$0"; exit 0 ;;
        *) echo "check-hook-counter-integrity: unknown arg: $1" >&2; exit 2 ;;
    esac
done

[ -n "$COUNTER" ]     || COUNTER=".context/working/.hook-counter"
[ -n "$FAILCOUNTER" ] || FAILCOUNTER=".context/working/.hook-failure-counter"

command -v awk >/dev/null 2>&1 || die "awk not in PATH"

[ -f "$COUNTER" ] || die "counter file not found: $COUNTER"

# Fail closed on an empty corpus. "0 corrupt keys out of 0" is vacuously true and would
# report green over a path that stopped matching (T-2747 zero-census lesson).
if [ ! -s "$COUNTER" ]; then
    die "counter file is empty: $COUNTER — refusing to report clean"
fi

export HCI_COUNTER="$COUNTER"
export HCI_FAILCOUNTER="$FAILCOUNTER"
export HCI_FORMAT="$FORMAT"
export HCI_QUIET="$QUIET"

awk '
BEGIN {
    counter    = ENVIRON["HCI_COUNTER"]
    fmt        = ENVIRON["HCI_FORMAT"]
    quiet      = (ENVIRON["HCI_QUIET"] == "1")
    total = 0; malformed = 0; dupkeys = 0

    while ((getline line < counter) > 0) {
        total++
        if (line == "") { blanks++; continue }
        if (index(line, "=") == 0) {
            malformed++
            mal[malformed] = line
            # first-match-wins readers return "" for this key
            emptyread[line] = 1
            continue
        }
        k = line; sub(/=.*$/, "", k)
        v = line; sub(/^[^=]*=/, "", v)
        seen[k]++
        if (seen[k] == 2) { dupkeys++; duplist[dupkeys] = k }
        if (seen[k] == 1) { firstval[k] = v }
        sumval[k] += v + 0
    }
    close(counter)

    # Which keys do the two readers disagree on?
    ndis = 0
    for (k in seen) {
        if (seen[k] > 1) {
            ndis++
            dis_key[ndis]   = k
            dis_first[ndis] = firstval[k]
            dis_sum[ndis]   = sumval[k]
        }
    }
    # A malformed bare key that ALSO exists in valid form: first-match reader gets "".
    for (k in emptyread) {
        if (k in seen) {
            ndis++
            dis_key[ndis]   = k
            dis_first[ndis] = "(empty)"
            dis_sum[ndis]   = sumval[k]
        }
    }

    corrupt = (malformed > 0 || dupkeys > 0)
    scope = "counter-file corruption and reader disagreement only — a clean result does NOT mean hook telemetry is trustworthy (T-2713 exit-code conflation leaves the file well-formed)"

    if (fmt == "json") {
        printf "{\"ok\":%s,\"lines\":%d,\"malformed\":%d,\"duplicate_keys\":%d,\"disagreements\":[",
               (corrupt ? "false" : "true"), total, malformed, dupkeys
        for (i = 1; i <= ndis; i++) {
            printf "%s{\"key\":\"%s\",\"first_match_reader\":\"%s\",\"summing_reader\":%d}",
                   (i > 1 ? "," : ""), dis_key[i], dis_first[i], dis_sum[i]
        }
        printf "],\"scope\":\"%s\"}\n", scope
        exit (corrupt ? 1 : 0)
    }

    if (corrupt) {
        printf "check-hook-counter-integrity: FIRING — counter file is corrupt (%d line(s) scanned)\n", total
        if (malformed > 0) {
            printf "\n  MALFORMED (no `=`), caught mid-write; first-match readers return empty:\n"
            for (i = 1; i <= malformed; i++) printf "    line: %s\n", mal[i]
        }
        if (dupkeys > 0) {
            printf "\n  DUPLICATE keys, stale copy re-appended after a concurrent rewrite:\n"
            for (i = 1; i <= dupkeys; i++) printf "    %s\n", duplist[i]
        }
        if (ndis > 0) {
            printf "\n  READER DISAGREEMENT on the same file:\n"
            printf "    %-26s %-18s %s\n", "key", "fw_hook_counter_get", "hook-threshold.py"
            for (i = 1; i <= ndis; i++)
                printf "    %-26s %-18s %d\n", dis_key[i], dis_first[i], dis_sum[i]
            printf "\n  The summing reader feeds the T-1626 decay alarm denominator, so each\n"
            printf "  duplicate roughly halves the apparent failure ratio (false silence).\n"
        }
        printf "\n  MECHANISM: unlocked truncate+write in lib/hook-telemetry.sh\n"
        printf "             _fw_telemetry_increment. This is L-023 recurring.\n"
        printf "  SCOPE: %s\n", scope
    } else if (!quiet) {
        printf "check-hook-counter-integrity: clean — %d line(s), no duplicate or malformed keys\n", total
        printf "  SCOPE: %s\n", scope
    }
    exit (corrupt ? 1 : 0)
}
' </dev/null
