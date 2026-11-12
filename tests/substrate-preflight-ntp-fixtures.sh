#!/usr/bin/env bash
# guard-layer: source
#
# T-3364 (P-23, CAND-13) — fixtures for substrate-preflight Check 9 (clock-sync).
# Hermetic: timedatectl / chronyc / ntpq are fake scripts in a temp dir reached
# only through TERMLINK_PREFLIGHT_TEST_CLOCK_TOOLS_DIR; the real host clock tools
# are never consulted. Covers synced, unsynced and unknown for every probe, the
# probe fall-through order, and the --clock-state line + exit code for P-24.
set -uo pipefail
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/substrate-preflight.sh"
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }
[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
command -v python3 >/dev/null || { echo "TOOLING: python3 missing" >&2; exit 2; }
TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

N=0
newtools() { N=$((N+1)); D="$TMP/tools$N"; mkdir -p "$D"; }
fake() { # name rc stdout
    printf '#!/bin/sh\ncat <<'"'"'EOT'"'"'\n%s\nEOT\nexit %s\n' "$3" "$2" > "$D/$1"
    chmod +x "$D/$1"
}

row() { # -> status|message of the clock-sync JSON row, plus |exit_code
    TERMLINK_PREFLIGHT_TEST_CLOCK_TOOLS_DIR="$D" \
    TERMLINK_PREFLIGHT_TEST_HUB_DIRS="$TMP/none" \
    TERMLINK_PREFLIGHT_TEST_HUB_STATUS_JSON="$TMP/none.json" \
        bash "$SCRIPT" --json --no-heartbeat 2>/dev/null | python3 -c '
import json,sys
d=json.load(sys.stdin)
r=[c for c in d["checks"] if c.get("name")=="clock-sync"]
print((r[0]["status"]+"|"+r[0]["message"]) if r else "MISSING|")'
}
state() { # -> "<line>|rc"
    local out rc
    out=$(TERMLINK_PREFLIGHT_TEST_CLOCK_TOOLS_DIR="$D" bash "$SCRIPT" --clock-state 2>/dev/null); rc=$?
    printf '%s|%s' "$out" "$rc"
}
: > "$TMP/none.json"

CHRONY_SYNCED='Reference ID    : C0A80101 (192.168.1.1)
Stratum         : 3
Leap status     : Normal'
CHRONY_UNSYNCED='Reference ID    : 00000000 ()
Stratum         : 0
Leap status     : Not synchronised'
NTPQ_SYNCED='     remote           refid      st t when poll reach   delay   offset  jitter
==============================================================================
*192.168.1.1     .GPS.            1 u   33   64  377    0.512    0.031   0.010'
NTPQ_UNSYNCED='     remote           refid      st t when poll reach   delay   offset  jitter
==============================================================================
 192.168.1.1     .INIT.          16 u    -   64    0    0.000    0.000   0.000'

echo "== timedatectl"
newtools; fake timedatectl 0 yes
case "$(row)" in pass\|*"NTP-synchronised (source: timedatectl)"*) ok "timedatectl yes: PASS (green)" ;; *) bad "timedatectl yes: $(row)" ;; esac
case "$(state)" in "clock_sync=synced source=timedatectl|0") ok "--clock-state synced, rc 0" ;; *) bad "state synced: $(state)" ;; esac

newtools; fake timedatectl 0 no
case "$(row)" in warn\|*"NOT NTP-synchronised (source: timedatectl)"*) ok "timedatectl no: WARN (red)" ;; *) bad "timedatectl no: $(row)" ;; esac
case "$(state)" in "clock_sync=unsynced source=timedatectl|1") ok "--clock-state unsynced, rc 1" ;; *) bad "state unsynced: $(state)" ;; esac
rc=0; TERMLINK_PREFLIGHT_TEST_CLOCK_TOOLS_DIR="$D" TERMLINK_PREFLIGHT_TEST_HUB_DIRS="$TMP/none" \
    TERMLINK_PREFLIGHT_TEST_HUB_STATUS_JSON="$TMP/none.json" \
    bash "$SCRIPT" --no-heartbeat > "$TMP/human.out" 2>&1 || rc=$?
if [ "$rc" -ge 1 ] && grep -q '^\[WARN\] clock-sync' "$TMP/human.out"; then ok "unsynced: human output [WARN] clock-sync, exit >= 1"; else bad "human unsynced: rc=$rc"; fi

echo "== fall-through: timedatectl fails (container, no timedated) -> chronyc"
newtools; fake timedatectl 1 "Failed to connect to bus"; fake chronyc 0 "$CHRONY_SYNCED"
case "$(row)" in pass\|*"source: chronyc"*) ok "chronyc Normal: PASS" ;; *) bad "chronyc synced: $(row)" ;; esac
newtools; fake chronyc 0 "$CHRONY_UNSYNCED"
case "$(row)" in warn\|*"source: chronyc"*) ok "chronyc Not synchronised: WARN" ;; *) bad "chronyc unsynced: $(row)" ;; esac
newtools; fake chronyc 1 "506 Cannot talk to daemon"; fake ntpq 0 "$NTPQ_SYNCED"
case "$(row)" in pass\|*"source: ntpq"*) ok "chronyc down -> ntpq '*' peer: PASS" ;; *) bad "ntpq synced: $(row)" ;; esac
newtools; fake ntpq 0 "$NTPQ_UNSYNCED"
case "$(row)" in warn\|*"source: ntpq"*) ok "ntpq no selected peer: WARN" ;; *) bad "ntpq unsynced: $(row)" ;; esac

echo "== unknown (could not look) is SKIP, never PASS"
newtools
case "$(row)" in skip\|*unchecked*) ok "no tools: SKIP unchecked" ;; *) bad "no tools: $(row)" ;; esac
case "$(state)" in "clock_sync=unknown source=none|2") ok "--clock-state unknown, rc 2" ;; *) bad "state unknown: $(state)" ;; esac
newtools; fake timedatectl 0 "garbage"; fake ntpq 0 "ntpq: read: Connection refused"
case "$(row)" in skip\|*) ok "unparseable answers: SKIP (not a guess)" ;; *) bad "garbage: $(row)" ;; esac

echo "== negative control: the WARN comes from the probe, not the fixture"
# Same unsynced fake, but placed where the seam does not look: the check must
# not see it (SKIP). If the check ignored the seam and read the host, or always
# warned, this would not be SKIP.
newtools; mkdir -p "$D/elsewhere"; ( D="$D/elsewhere"; fake timedatectl 0 no )
case "$(row)" in skip\|*) ok "control: unsynced fake outside the seam dir is not read" ;; *) bad "control: $(row)" ;; esac

echo; echo "  passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ]
