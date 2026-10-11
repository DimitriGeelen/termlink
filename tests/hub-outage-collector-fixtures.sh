#!/usr/bin/env bash
# T-3383 fixtures — collector, summariser, liveness canary. Hermetic: HUBOUT_TEST_PROBES
# feeds canned probe results; no real hub, no network, scratch dir only.
# guard-layer fixtures run via naming convention.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
C="${HUBOUT_COLLECT:-$ROOT/scripts/collect-hub-outages.sh}"
S="${HUBOUT_SUMMARISE:-$ROOT/scripts/summarise-hub-outages.sh}"
K="${HUBOUT_CANARY:-$ROOT/scripts/check-hub-outage-collector-freshness.sh}"
TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   $*"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL $*"; }
chk() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }
export HUBOUT_HUBS_TOML="$TMP/hubs.toml" HUBOUT_SAMPLES="$TMP/s.ndjson" HUBOUT_HEARTBEAT="$TMP/hb" HUBOUT_CANARY_HEARTBEAT="$TMP/chb"
cat > "$HUBOUT_HUBS_TOML" <<T
[hubs.alpha]
address = "10.0.0.1:9100"
secret_file = "/x"

[hubs.beta]
address = "10.0.0.2:9100"
T
printf 'alpha 0 12 ok\nbeta 1 5001 timeout\n' > "$TMP/p1"

echo "collector"
HUBOUT_NOW=1000 HUBOUT_TEST_PROBES="$TMP/p1" bash "$C"; rc=$?
chk "C1 exit 0 even with an unreachable hub (PL-219)" '[ $rc -eq 0 ]'
chk "C2 one line per hub" '[ "$(wc -l < "$HUBOUT_SAMPLES")" -eq 2 ]'
chk "C3 alpha reachable w/ latency" 'grep -q "\"hub\":\"alpha\".*\"reachable\":true.*\"latency_ms\":12" "$HUBOUT_SAMPLES"'
chk "C4 beta unreachable w/ error class" 'grep -q "\"hub\":\"beta\".*\"reachable\":false.*\"error\":\"timeout\"" "$HUBOUT_SAMPLES"'
chk "C5 lines are valid JSON" 'python3 -c "import json,sys;[json.loads(l) for l in open(sys.argv[1])]" "$HUBOUT_SAMPLES"'
chk "C6 heartbeat written on exit" '[ -s "$HUBOUT_HEARTBEAT" ]'
rm -f "$HUBOUT_HEARTBEAT"
HUBOUT_NOW=1060 HUBOUT_TEST_PROBES="$TMP/p1" bash "$C" --no-heartbeat
chk "C7 --no-heartbeat skips heartbeat" '[ ! -e "$HUBOUT_HEARTBEAT" ]'
HUBOUT_HUBS_TOML="$TMP/none" bash "$C" 2>/dev/null; rc=$?
chk "C8 missing hubs.toml is tooling rc 2" '[ $rc -eq 2 ]'

echo "summariser"
# beta: ok@0,60 ; down@120,180,240 ; up@300 -> 180s ; down@360,420 then end -> OPEN 60s ; alpha always up
: > "$HUBOUT_SAMPLES"
emit() { printf '{"ts":%s,"hub":"%s","addr":"%s","reachable":%s,"latency_ms":1,"error":"x"}\n' "$1" "$2" "$3" "$4" >> "$HUBOUT_SAMPLES"; }
for t in 0 60 120 180 240 300 360 420; do
    emit $t alpha 10.0.0.1:9100 true
    case $t in 0|60|300) r=true ;; *) r=false ;; esac
    emit $t beta 10.0.0.2:9100 $r
done
out="$(bash "$S" --json)"
chk "S1 alpha has no outages" 'echo "$out" | python3 -c "import json,sys;d=json.load(sys.stdin);sys.exit(0 if d[\"hubs\"][\"10.0.0.1:9100\"][\"outages\"]==0 else 1)"'
chk "S2 beta closed outage = 180s (120->300)" 'echo "$out" | python3 -c "import json,sys;d=json.load(sys.stdin);i=d[\"hubs\"][\"10.0.0.2:9100\"][\"intervals\"];sys.exit(0 if i[0][\"start\"]==120 and i[0][\"end\"]==300 and i[0][\"duration_s\"]==180 and not i[0][\"open\"] else 1)"'
chk "S3 trailing outage is OPEN 60s" 'echo "$out" | python3 -c "import json,sys;d=json.load(sys.stdin);i=d[\"hubs\"][\"10.0.0.2:9100\"][\"intervals\"];sys.exit(0 if i[1][\"open\"] and i[1][\"duration_s\"]==60 else 1)"'
chk "S4 max=180 and p95=180" 'echo "$out" | python3 -c "import json,sys;d=json.load(sys.stdin);sys.exit(0 if d[\"all\"][\"max_s\"]==180 and d[\"all\"][\"p95_s\"]==180 else 1)"'
chk "S5 H-1 covered at 3600 limit" 'echo "$out" | grep -q "\"h1_covered\": true"'
out2="$(bash "$S" --json --limit 100)"
chk "S6 H-1 NOT covered at limit 100" 'echo "$out2" | python3 -c "import json,sys;d=json.load(sys.stdin);sys.exit(0 if d[\"all\"][\"h1_covered\"] is False else 1)"'
# collector gap: down@0, then silence 1000s, then up@1000 -> must not be one 1000s outage
: > "$HUBOUT_SAMPLES"; emit 0 beta 10.0.0.2:9100 false; emit 60 beta 10.0.0.2:9100 false; emit 1000 beta 10.0.0.2:9100 true
out3="$(bash "$S" --json)"
chk "S7 sampling gap closes the run, flagged gap, not 1000s" 'echo "$out3" | python3 -c "import json,sys;d=json.load(sys.stdin);i=d[\"hubs\"][\"10.0.0.2:9100\"][\"intervals\"][0];sys.exit(0 if i[\"gap\"] and i[\"duration_s\"]==60 else 1)"'
# two hub names, one address -> one hub
: > "$HUBOUT_SAMPLES"; emit 0 a 1.1.1.1:9 true; emit 0 b 1.1.1.1:9 true
chk "S8 shared address deduped" 'bash "$S" --json | python3 -c "import json,sys;d=json.load(sys.stdin);sys.exit(0 if len(d[\"hubs\"])==1 and d[\"hubs\"][\"1.1.1.1:9\"][\"samples\"]==1 else 1)"'
bash "$S" --samples "$TMP/nope" 2>/dev/null; rc=$?
chk "S9 unreadable samples is rc 2" '[ $rc -eq 2 ]'

echo "canary"
: > "$HUBOUT_SAMPLES"; emit 1000 alpha 10.0.0.1:9100 true; echo 1000 > "$HUBOUT_HEARTBEAT"
HUBOUT_NOW=1100 bash "$K" --quiet >/dev/null; rc=$?
chk "K1 fresh heartbeat + samples = healthy" '[ $rc -eq 0 ]'
HUBOUT_NOW=2000 bash "$K" --quiet >/dev/null; rc=$?
chk "K2 stale heartbeat fires" '[ $rc -eq 1 ]'
echo 1990 > "$HUBOUT_HEARTBEAT"; HUBOUT_NOW=2000 bash "$K" --quiet >/dev/null; rc=$?
chk "K3 fresh heartbeat but stale samples fires" '[ $rc -eq 1 ]'
rm -f "$HUBOUT_HEARTBEAT"; HUBOUT_NOW=1100 bash "$K" --quiet >/dev/null; rc=$?
chk "K4 missing heartbeat fires" '[ $rc -eq 1 ]'
echo 1000 > "$HUBOUT_HEARTBEAT"; rm -f "$HUBOUT_CANARY_HEARTBEAT"
HUBOUT_NOW=1100 bash "$K" --quiet >/dev/null
chk "K5 canary writes its own heartbeat" '[ -s "$HUBOUT_CANARY_HEARTBEAT" ]'
echo "mutant"
if [ -z "${HUBOUT_MUTANT:-}" ]; then
    # M1: summariser that ignores the sampling-gap rule must turn S7 red
    sed 's/ts - prev > maxgap/False/' "$S" > "$TMP/sum_m1.sh"; chmod +x "$TMP/sum_m1.sh"
    # M2: canary that never fires must turn K2 red
    sed 's/-gt "\$lim"/-gt 99999999/g' "$K" > "$TMP/can_m2.sh"; chmod +x "$TMP/can_m2.sh"
    # M3: collector writing reachable=true always must turn C4 red
    sed 's/r=false; \[ "\$rc" = 0 \] \&\& r=true/r=true/' "$C" > "$TMP/col_m3.sh"; chmod +x "$TMP/col_m3.sh"
    for m in "M1 S7 HUBOUT_SUMMARISE=$TMP/sum_m1.sh" "M2 K2 HUBOUT_CANARY=$TMP/can_m2.sh" "M3 C4 HUBOUT_COLLECT=$TMP/col_m3.sh"; do
        set -- $m
        mout="$(env HUBOUT_MUTANT=1 "$3" bash "${BASH_SOURCE[0]}" 2>&1)"
        if grep -q "FAIL $2 " <<< "$mout"; then ok "$1 mutant turns $2 red"; else bad "$1 mutant not caught by $2"; fi
    done
fi
echo "PASS: $PASS  FAIL: $FAIL"
[ "$FAIL" -eq 0 ]
