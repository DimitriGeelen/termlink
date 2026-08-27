#!/usr/bin/env bash
# T-2840 — fixtures for canary-status.sh's NOT_SCHEDULED classification.
#
# WHY THIS EXISTS. discover_canaries() synthesizes a .log path from any
# .heartbeat so a cron canary that has never written a log still surfaces
# (T-2178). Correct — but it cannot tell that canary apart from a SOURCE-LEVEL
# STATIC CHECK which is not cron-scheduled at all and only has a heartbeat
# because somebody ran it by hand. Four of those exist here (alloc-sink,
# busy-spin, drain-sink, silent-exit; CLAUDE.md calls each "NOT a runtime cron
# canary"), and each read STALE forever. /canaries said "6 need attention" when
# 2 were real: a dashboard two-thirds noise is one nobody reads.
#
# THE LOAD-BEARING CASE IS THE LAST ONE. A crontab that EXISTS in git but is not
# installed to /etc/cron.d is the T-2561 shipped-but-dark class, and it must
# still classify STALE. If that leg ever fails, this fix has started hiding the
# exact failure the cron layer exists to catch, and the noise reduction was
# bought by going blind.
#
# Run: bash tests/canary-not-scheduled-fixtures.sh

set -uo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$REPO_ROOT/scripts/canary-status.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS + 1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; }
[ -r "$SCRIPT" ] || { echo "cannot read $SCRIPT" >&2; exit 2; }

echo "== canary-status NOT_SCHEDULED fixtures =="
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
W="$TMP/.context/working"; C="$TMP/.context/cron"
mkdir -p "$W" "$C"
OLD=$(( $(date +%s) - 90*3600 ))   # well past the 48h threshold

run() { (cd "$TMP" && CANARY_STATUS_CRON_DIR=".context/cron" bash "$SCRIPT" "$@" 2>&1); }
status_of() { run --json | python3 -c "
import json,sys
d=json.load(sys.stdin)
print(next((c['status'] for c in d['canaries'] if c['name']==sys.argv[1]),'ABSENT'))" "$1"; }

# (a) heartbeat only, stale, NO crontab anywhere -> NOT_SCHEDULED
touch -d "@$OLD" "$W/.solo-static-canary.heartbeat"
[ "$(status_of solo-static-canary)" = "NOT_SCHEDULED" ] \
  && ok "stale heartbeat with no crontab in git is NOT_SCHEDULED" \
  || bad "stale heartbeat with no crontab in git is NOT_SCHEDULED" "got $(status_of solo-static-canary)"

# (b) it must not be counted as a problem
run --quiet | grep -q 'solo-static-canary' \
  && bad "NOT_SCHEDULED is excluded from --quiet problem list" "it was listed" \
  || ok "NOT_SCHEDULED is excluded from --quiet problem list"

# (c) the summary must still add up — a silently-dropped bucket is the defect
#     this whole class is about
run --json | python3 -c "
import json,sys
s=json.load(sys.stdin)['summary']
tot=s['healthy']+s['firing']+s['stale']+s['no_heartbeat']+s['not_scheduled']
sys.exit(0 if tot==s['total'] else 1)" \
  && ok "summary buckets sum to total" || bad "summary buckets sum to total"

# (d) THE LOAD-BEARING LEG: crontab declared in git -> still STALE, never
#     downgraded. This is the T-2561 shipped-but-dark case.
touch -d "@$OLD" "$W/.declared-canary.heartbeat"
printf '# Installed to: /etc/cron.d/x\n0 5 * * * root bash scripts/check-x.sh --quiet >> .context/working/.declared-canary.log 2>> .context/working/.declared-canary.log.stderr\n' > "$C/declared.crontab"
[ "$(status_of declared-canary)" = "STALE" ] \
  && ok "declared-in-git but never installed is STILL STALE (T-2561 not masked)" \
  || bad "declared-in-git but never installed is STILL STALE (T-2561 not masked)" "got $(status_of declared-canary)"

# (e) a stale canary that HAS logged content is never downgraded
touch -d "@$OLD" "$W/.noisy-canary.heartbeat"
echo "something went wrong" > "$W/.noisy-canary.log"
touch -d "@$OLD" "$W/.noisy-canary.log"
[ "$(status_of noisy-canary)" = "STALE" ] \
  && ok "stale canary with log content is not downgraded" \
  || bad "stale canary with log content is not downgraded" "got $(status_of noisy-canary)"

# (f) fail-open: no cron dir at all -> old behaviour (STALE), never silent
[ "$( (cd "$TMP" && CANARY_STATUS_CRON_DIR="/nonexistent-cron-dir" bash "$SCRIPT" --json 2>/dev/null) | python3 -c "
import json,sys
d=json.load(sys.stdin)
print(next((c['status'] for c in d['canaries'] if c['name']=='solo-static-canary'),'ABSENT'))")" = "STALE" ] \
  && ok "absent cron dir fails OPEN to STALE (cannot tell -> keep the warning)" \
  || bad "absent cron dir fails OPEN to STALE"

echo ""
echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" = "0" ] || exit 1
