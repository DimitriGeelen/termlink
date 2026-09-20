#!/usr/bin/env bash
# tests/hook-telemetry-race-fixtures.sh — T-2982 / L-023 / C-20
#
# Proves the lost-update + key-duplication race in the VENDORED
# lib/hook-telemetry.sh::_fw_telemetry_increment, and proves that an
# flock-serialised variant eliminates it.
#
# The vendored function is NOT patched here (G-062 — a local fix is erased by
# the next re-vendor). This harness is the evidence that travels with the
# upstream filing, and the local regression net if the fix ever lands.
#
# Load-bearing property: leg 1 must go RED against the unfixed function. A
# harness that cannot reproduce the defect proves nothing about the fix.

set -uo pipefail

FW_LIB="${FW_LIB:-/opt/termlink/.agentic-framework/lib/hook-telemetry.sh}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

WRITERS=8
PER=60
EXPECT=$((WRITERS * PER))

# ---------------------------------------------------------------- fixed impl
# The proposed upstream fix: serialise the whole read-modify-write on a
# sidecar lock file. Same file format, same readers, no contract change.
cat > "$TMP/fixed.sh" <<'IMPL'
_fw_telemetry_increment_locked() {
    local file="$1" key="$2" lockfd
    local lock="${file}.lock"
    exec {lockfd}>>"$lock" 2>/dev/null || { _fw_telemetry_increment "$file" "$key"; return 0; }
    flock -w 5 "$lockfd" 2>/dev/null
    _fw_telemetry_increment "$file" "$key"
    exec {lockfd}>&-
}
IMPL

WRITER="$TMP/writer.sh"
cat > "$WRITER" <<EOF
#!/usr/bin/env bash
source "$FW_LIB"
source "$TMP/fixed.sh"
fn="\$1"; file="\$2"; n="\$3"; key="\$4"
for i in \$(seq 1 "\$n"); do "\$fn" "\$file" "\$key"; done
EOF
chmod +x "$WRITER"

run_race() {  # $1=impl-fn $2=file -> prints observed total
    rm -f "$2" "$2.lock"
    local p
    for p in $(seq 1 $WRITERS); do "$WRITER" "$1" "$2" "$PER" check-active-task & done
    wait
    # sum ALL matching lines — hook-threshold.py's reader semantics, which is
    # what makes duplicate keys visible rather than silently hidden.
    awk -F= '/^check-active-task=/ {s+=$2} END {print s+0}' "$2" 2>/dev/null
}

count_dupes() { awk -F= '/^[a-z-]+=/ {c[$1]++} END {for (k in c) if (c[k]>1) n++; print n+0}' "$1" 2>/dev/null; }

echo "== leg 1: the unfixed vendored function must LOSE increments =="
lost_any=0; dupe_any=0
for run in 1 2 3; do
    got="$(run_race _fw_telemetry_increment "$TMP/unfixed")"
    d="$(count_dupes "$TMP/unfixed")"
    printf '   run %s: expected=%s observed=%s duplicate_keys=%s\n' "$run" "$EXPECT" "$got" "$d"
    [ "$got" -lt "$EXPECT" ] && lost_any=1
    [ "${d:-0}" -gt 0 ] && dupe_any=1
done
if [ "$lost_any" = "1" ]; then
    ok "unfixed impl loses increments under $WRITERS concurrent writers (the defect reproduces)"
else
    bad "unfixed impl did NOT lose increments — harness cannot go red, so it proves nothing"
fi

echo "== leg 2: the flock-serialised fix must lose NOTHING =="
clean=1
for run in 1 2 3; do
    got="$(run_race _fw_telemetry_increment_locked "$TMP/fixed")"
    d="$(count_dupes "$TMP/fixed")"
    printf '   run %s: expected=%s observed=%s duplicate_keys=%s\n' "$run" "$EXPECT" "$got" "$d"
    [ "$got" != "$EXPECT" ] && clean=0
    [ "${d:-0}" -gt 0 ] && clean=0
done
if [ "$clean" = "1" ]; then
    ok "flock-serialised impl: exact count, zero duplicate keys, 3/3 runs"
else
    bad "flock-serialised impl still lost increments or duplicated keys"
fi

echo "== leg 3: per-call overhead vs the T-1626 5ms budget =="
source "$FW_LIB"; source "$TMP/fixed.sh"
bench() {  # $1=fn $2=file -> ms per call
    rm -f "$2" "$2.lock"; local n=200 s e
    s=$(date +%s%N); for i in $(seq 1 $n); do "$1" "$2" bench >/dev/null 2>&1; done; e=$(date +%s%N)
    awk -v d="$((e-s))" -v n="$n" 'BEGIN{printf "%.3f", d/n/1000000}'
}
u="$(bench _fw_telemetry_increment "$TMP/bu")"
l="$(bench _fw_telemetry_increment_locked "$TMP/bl")"
printf '   unfixed=%sms  flock=%sms  budget=5.000ms\n' "$u" "$l"
if awk -v v="$l" 'BEGIN{exit !(v < 5.0)}'; then
    ok "flock variant stays inside the 5ms per-fire budget the comment cites"
else
    bad "flock variant exceeds the 5ms budget ($l ms) — fix needs a lock-free design"
fi

echo
printf 'passed=%s failed=%s\n' "$PASS" "$FAIL"
if [ "$FAIL" -eq 0 ]; then echo "ALL ASSERTIONS PASSED"; exit 0; fi
exit 1
