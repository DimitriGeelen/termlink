#!/usr/bin/env bash
# check-pickup-cron-lock.sh — every installed `fw pickup process` cron line for this
# project must be serialised under ONE shared flock lock (T-2870).
#
# WHY. `fw pickup process` is scheduled twice on this host (agentic-pickup-termlink
# at * * * * *, agentic-audit-termlink at */15) and unserialised copies race: a run
# creates the task for an envelope BEFORE the dedup marker lands, so an overlapping
# run creates it again. Measured 3× as byte-identical duplicate task pairs
# (T-2260/T-2261 same second; T-2862/T-2863 and T-2867/T-2868 both one cron period
# apart). The AEF project runs the identical job on the same host WITH flock — the
# pattern exists; termlink's copies lacked it.
#
# WHAT IT ASSERTS — the PROPERTY, not the current text (so a reinstall that drops
# the lock re-fires rather than passing on a stale match):
#   1. every non-comment cron line in the cron dir that invokes THIS project's
#      `pickup process` carries `flock`;
#   2. all such lines share ONE lock path — two flocks on different paths do not
#      exclude each other, so distinct paths fail the serialisation property too.
#
# MATCHING NOTE. Lines are matched on `pickup process` + the project root as
# substrings, NOT on `fw pickup process` — the installed lines quote the fw path
# (`.../bin/fw" pickup process`), so the naive anchor matches nothing and would
# report a vacuous clean. That miss is why this script exists as the check instead
# of an inline grep (T-2870 found its own task's verification line had the bug).
#
# TIER. Deploy-time / ad-hoc REVIEW check, NOT a cron canary and NOT guard-layer
# `source` (it reads host state under /etc/cron.d). Sibling of
# check-cron-install-drift.sh (T-2561) — run it after installing or editing a
# pickup crontab. A host with no cron dir or no pickup lines is informational,
# never firing (macOS/dev hosts legitimately run no pickup rail — PL-219 shape).
#
# Exit codes: 0 = all pickup lines serialised (or none installed) · 1 = an
# unserialised line or divergent lock paths · 2 = tooling error (unreadable dir).
# Flags: --json, --quiet. Test seams: PICKUP_LOCK_CRON_DIR (fixture dir),
# PICKUP_LOCK_PROJECT_ROOT (project path to match; default /opt/termlink).
# Fixtures: bash tests/pickup-cron-lock-fixtures.sh
set -u

CRON_DIR="${PICKUP_LOCK_CRON_DIR:-/etc/cron.d}"
PROJECT_MATCH="${PICKUP_LOCK_PROJECT_ROOT:-/opt/termlink}"
JSON=0
QUIET=0
for arg in "$@"; do
  case "$arg" in
    --json) JSON=1 ;;
    --quiet) QUIET=1 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

SCOPE="scope: asserts flock serialisation of ${PROJECT_MATCH} 'pickup process' cron lines in ${CRON_DIR} only — not that the job runs, nor anything about other jobs"

if [ ! -d "$CRON_DIR" ]; then
  # No cron dir (macOS / dev host) — informational, never firing.
  if [ "$JSON" -eq 1 ]; then
    printf '{"ok": true, "checked_lines": 0, "locked": 0, "firing": [], "note": "cron dir absent — nothing to check", "cron_dir": "%s"}\n' "$CRON_DIR"
  elif [ "$QUIET" -eq 0 ]; then
    echo "check-pickup-cron-lock: cron dir $CRON_DIR absent — nothing to check (informational)"
  fi
  exit 0
fi
if [ ! -r "$CRON_DIR" ]; then
  echo "check-pickup-cron-lock: TOOLING ERROR — $CRON_DIR exists but is not readable" >&2
  exit 2
fi

# Collect matching lines as "<file>|<line>" records. Comment lines are skipped:
# prose about the job is not an installation of it.
MATCHES=()
while IFS= read -r rec; do
  MATCHES+=("$rec")
done < <(
  for f in "$CRON_DIR"/*; do
    [ -f "$f" ] || continue
    # grep -H is not used: filenames may contain '|'; keep the pairing explicit.
    while IFS= read -r line; do
      case "$line" in
        \#*) continue ;;
      esac
      if [[ "$line" == *"pickup process"* && "$line" == *"$PROJECT_MATCH"* ]]; then
        printf '%s|%s\n' "$(basename "$f")" "$line"
      fi
    done < "$f"
  done
)

CHECKED=${#MATCHES[@]}
FIRING=()      # "<file>: <reason>"
LOCK_PATHS=()  # distinct lock paths seen
LOCKED=0

for rec in ${MATCHES[@]+"${MATCHES[@]}"}; do
  file="${rec%%|*}"
  line="${rec#*|}"
  if [[ "$line" != *flock* ]]; then
    FIRING+=("$file: pickup line has NO flock — unserialised (the T-2870 race)")
    continue
  fi
  LOCKED=$((LOCKED + 1))
  # Extract the lock path: first argument after flock that looks like a path.
  lockpath=$(printf '%s\n' "$line" | sed -n 's/.*flock[[:space:]]\+\(-[a-zA-Z]\+[[:space:]]\+\)*\(\/[^[:space:]"'"'"']*\).*/\2/p')
  if [ -z "$lockpath" ]; then
    FIRING+=("$file: flock present but no lock path could be parsed — cannot prove mutual exclusion")
    continue
  fi
  seen=0
  for lp in ${LOCK_PATHS[@]+"${LOCK_PATHS[@]}"}; do
    [ "$lp" = "$lockpath" ] && seen=1
  done
  [ "$seen" -eq 0 ] && LOCK_PATHS+=("$lockpath")
done

if [ ${#LOCK_PATHS[@]} -gt 1 ]; then
  FIRING+=("(cross-file) $LOCKED locked line(s) use ${#LOCK_PATHS[@]} DISTINCT lock paths — flocks on different paths do not exclude each other: ${LOCK_PATHS[*]}")
fi

if [ ${#FIRING[@]} -gt 0 ]; then
  if [ "$JSON" -eq 1 ]; then
    printf '{"ok": false, "checked_lines": %d, "locked": %d, "distinct_lock_paths": %d, "firing": [' "$CHECKED" "$LOCKED" "${#LOCK_PATHS[@]}"
    first=1
    for f in "${FIRING[@]}"; do
      [ "$first" -eq 0 ] && printf ', '
      printf '"%s"' "$(printf '%s' "$f" | sed 's/\\/\\\\/g; s/"/\\"/g')"
      first=0
    done
    printf '], "cron_dir": "%s"}\n' "$CRON_DIR"
  else
    echo "check-pickup-cron-lock: FIRING — ${#FIRING[@]} finding(s) across $CHECKED pickup cron line(s):"
    for f in "${FIRING[@]}"; do echo "  $f"; done
    echo ""
    echo "Remediation (T-2870): wrap each line so both copies share ONE lock, e.g."
    echo "  flock -n /var/lock/agentic-pickup-termlink.lock -c '<original command>'"
    echo "$SCOPE"
  fi
  exit 1
fi

if [ "$JSON" -eq 1 ]; then
  printf '{"ok": true, "checked_lines": %d, "locked": %d, "distinct_lock_paths": %d, "firing": [], "cron_dir": "%s"}\n' "$CHECKED" "$LOCKED" "${#LOCK_PATHS[@]}" "$CRON_DIR"
elif [ "$QUIET" -eq 0 ]; then
  if [ "$CHECKED" -eq 0 ]; then
    echo "check-pickup-cron-lock: no $PROJECT_MATCH pickup cron lines in $CRON_DIR — nothing to serialise (informational)"
  else
    echo "check-pickup-cron-lock: clean — $CHECKED pickup cron line(s), all flock-serialised on one lock path"
  fi
  echo "$SCOPE"
fi
exit 0
