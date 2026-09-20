#!/usr/bin/env bash
# guard-layer: source
#
# T-2996: prove the property the invocation-audit sink DEPENDS ON, across real
# processes — not threads.
#
# `crates/termlink-hub/src/invocation_audit.rs` claims that concurrent writers
# in DIFFERENT PROCESSES cannot lose records, because each record is a single
# short line written with O_APPEND, and POSIX makes such a write atomic. Its
# Rust unit test only exercises threads inside one process, and the process-local
# Mutex it holds is precisely what does NOT generalise across processes. So the
# unit test cannot substantiate the cross-process claim; this fixture does.
#
# Leg 2 is the load-bearing half. T-2982 established the standard: a harness
# that cannot reproduce the defect proves nothing about the fix. So this also
# runs the REJECTED shape — the unlocked read-modify-TRUNCATE-write used by
# `.agentic-framework/lib/hook-telemetry.sh::_fw_telemetry_increment` — under
# the identical load. If leg 2 stops losing records, the design rationale in the
# module doc has become false and should be re-derived, not trusted.
set -uo pipefail

WRITERS=8
PER_WRITER=60
EXPECTED=$((WRITERS * PER_WRITER))
passed=0; failed=0
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

ok()   { echo "  ok   $1"; passed=$((passed+1)); }
bad()  { echo "  FAIL $1"; failed=$((failed+1)); }

echo "== leg 1: O_APPEND line writes across $WRITERS PROCESSES must lose nothing =="
for run in 1 2 3; do
    sink="$TMP/append-$run.jsonl"
    : > "$sink"
    for w in $(seq 1 "$WRITERS"); do
        (
            for _ in $(seq 1 "$PER_WRITER"); do
                # `>>` is O_APPEND, the same flag OpenOptions::append(true) sets.
                printf '{"ts":0,"surface":"mcp","name":"tool_%s"}\n' "$w" >> "$sink"
            done
        ) &
    done
    wait
    got=$(grep -c . "$sink")
    torn=$(grep -cv '^{"ts":0,"surface":"mcp","name":"tool_[0-9]*"}$' "$sink" || true)
    echo "   run $run: expected=$EXPECTED observed=$got torn_lines=$torn"
    if [ "$got" -ne "$EXPECTED" ] || [ "$torn" -ne 0 ]; then
        bad "O_APPEND lost or tore records (run $run)"
        leg1_bad=1
    fi
done
[ "${leg1_bad:-0}" = "0" ] && ok "cross-process O_APPEND: exact count, zero torn lines, 3/3 runs"

echo "== leg 2: the REJECTED read-modify-truncate-write shape must LOSE records =="
# Mirrors _fw_telemetry_increment: read the file, compute, truncate, write back.
cat > "$TMP/rmw.sh" <<'RMW'
#!/usr/bin/env bash
f="$1"; key="$2"
for _ in $(seq 1 "$3"); do
    if [ -f "$f" ]; then
        mapfile -t lines < "$f"
        found=0
        for i in "${!lines[@]}"; do
            k="${lines[i]%%=*}"
            if [ "$k" = "$key" ]; then
                v="${lines[i]#*=}"; lines[i]="$key=$((v+1))"; found=1; break
            fi
        done
        [ "$found" = "0" ] && lines+=("$key=1")
        printf '%s\n' "${lines[@]}" > "$f"   # truncate-on-open: the defect
    else
        printf '%s=1\n' "$key" > "$f"
    fi
done
RMW
chmod +x "$TMP/rmw.sh"
leg2_reproduced=0
for run in 1 2 3; do
    counter="$TMP/counter-$run"
    : > "$counter"
    for _ in $(seq 1 "$WRITERS"); do
        "$TMP/rmw.sh" "$counter" "calls" "$PER_WRITER" &
    done
    wait
    observed=$(awk -F= '/^calls=/ {s+=$2} END {print s+0}' "$counter")
    echo "   run $run: expected=$EXPECTED observed=$observed"
    [ "$observed" -lt "$EXPECTED" ] && leg2_reproduced=1
done
if [ "$leg2_reproduced" = "1" ]; then
    ok "read-modify-truncate-write loses records under the same load (the defect reproduces)"
else
    bad "harness cannot go red: the rejected shape did NOT lose records — re-derive the design rationale"
fi

echo
echo "passed=$passed failed=$failed"
[ "$failed" -eq 0 ] && echo "ALL ASSERTIONS PASSED" && exit 0
exit 1
