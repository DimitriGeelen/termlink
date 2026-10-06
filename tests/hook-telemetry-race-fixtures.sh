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

# T-3234: resolve from this checkout, not the literal origin-host path. With the
# literal, a CI checkout (or any other clone) sourced a file that did not exist
# there, and on the origin host a clone silently tested the MAIN checkout's copy.
FW_LIB="${FW_LIB:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/.agentic-framework/lib/hook-telemetry.sh}"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

WRITERS=8
PER=60
EXPECT=$((WRITERS * PER))

# ---------------------------------------------------------------- T-3370
# AEF 1.8.3 LANDED the fix this harness was evidence for (AEF T-3371: the vendored
# _fw_telemetry_increment now serialises its read-modify-write with flock). Per this
# file's header, the harness now becomes the local REGRESSION NET: the vendored
# function itself must lose nothing. The former leg 2 (our own proposed flock
# wrapper) is gone: it wrapped a function that now takes the same lock itself, so
# every call waited out its 5 s timeout and then ran unlocked — slow and lossy,
# measured 200/480 on 2026-10-06.

WRITER="$TMP/writer.sh"
cat > "$WRITER" <<EOF
#!/usr/bin/env bash
source "$FW_LIB"
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

echo "== leg 0: the vendored function carries the upstream lock (AEF T-3371) =="
if grep -q 'flock' "$FW_LIB"; then ok "vendored hook-telemetry.sh serialises with flock"
else bad "vendored hook-telemetry.sh has no flock — the T-3371 fix is gone (re-vendor regression?)"; fi

echo "== leg 1: the vendored function must lose NOTHING under contention =="
clean=1
for run in 1 2 3; do
    got="$(run_race _fw_telemetry_increment "$TMP/vendored")"
    d="$(count_dupes "$TMP/vendored")"
    printf '   run %s: expected=%s observed=%s duplicate_keys=%s\n' "$run" "$EXPECT" "$got" "$d"
    [ "$got" != "$EXPECT" ] && clean=0
    [ "${d:-0}" -gt 0 ] && clean=0
done
if [ "$clean" = "1" ]; then
    ok "vendored impl: exact count, zero duplicate keys under $WRITERS writers, 3/3 runs"
else
    bad "vendored impl lost increments or duplicated keys — the T-2982 race is back"
fi

echo "== leg 2: per-call overhead vs the T-1626 5ms budget =="
source "$FW_LIB"
bench() {  # $1=fn $2=file -> ms per call
    rm -f "$2" "$2.lock"; local n=200 s e
    s=$(date +%s%N); for i in $(seq 1 $n); do "$1" "$2" bench >/dev/null 2>&1; done; e=$(date +%s%N)
    awk -v d="$((e-s))" -v n="$n" 'BEGIN{printf "%.3f", d/n/1000000}'
}
v="$(bench _fw_telemetry_increment "$TMP/bv")"
printf '   vendored(flock)=%sms  budget=5.000ms\n' "$v"
if awk -v v="$v" 'BEGIN{exit !(v < 5.0)}'; then
    ok "vendored flock impl stays inside the 5ms per-fire budget"
else
    bad "vendored flock impl exceeds the 5ms budget ($v ms)"
fi

echo
printf 'passed=%s failed=%s\n' "$PASS" "$FAIL"
if [ "$FAIL" -eq 0 ]; then echo "ALL ASSERTIONS PASSED"; exit 0; fi
exit 1
