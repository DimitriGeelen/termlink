#!/usr/bin/env bash
# tests/bvp-auto-confirm-fixtures.sh — T-3176
#
# guard-layer: source
#
# Fixtures for the BVP auto-confirm switch and its confirmation ledger, the LOCAL
# half of the operator's both-scopes ruling on T-3170 (upstream half filed at
# framework:pickup offset 193, building on 832-Workflow-designer's offset 168).
#
# THE LOAD-BEARING CASE IS NOT "confirm works with the switch on".
#
# A blanket CLAUDECODE bypass is the obvious implementation and is indistinguishable
# from the correct one on the happy path — both let `confirm` through. What separates
# them is the OTHER FOUR verbs, so Case 3 asserts that weight --set, driver --add,
# driver --remove and auto-promote --enable STILL refuse with the switch ON. If that
# case ever goes green-by-accident the bypass stopped being surgical, and nothing
# else here would notice.
#
# Every run writes its ledger to a scratch path via FW_BVP_TELEMETRY_PATH. That
# override exists because 832's did not: their first fixture run wrote rows into the
# real append-only ledger and those rows cannot be removed. Case 6 pins that the real
# ledger is never touched, so the protection itself is tested rather than assumed.
#
# Exit 0 = all assertions pass, 1 = a failure, 2 = tooling (fail-closed).

set -uo pipefail

PROJECT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FW_ROOT="$PROJECT/.agentic-framework"
BVP="$FW_ROOT/lib/bvp.sh"

PASS=0
FAIL=0

fail() { printf '  \033[0;31mFAIL\033[0m  %s\n' "$1"; FAIL=$((FAIL + 1)); }
ok()   { printf '  \033[0;32mok\033[0m    %s\n' "$1"; PASS=$((PASS + 1)); }

assert_contains() { # haystack needle label
    if grep -qF -- "$2" <<< "$1"; then ok "$3"; else
        fail "$3 — expected to find: $2"
        printf '        got: %s\n' "$(head -c 300 <<< "$1")"
    fi
}
assert_not_contains() {
    if grep -qF -- "$2" <<< "$1"; then
        fail "$3 — did NOT expect: $2"
    else ok "$3"; fi
}
assert_rc() { # actual expected label
    if [ "$1" = "$2" ]; then ok "$3"; else fail "$3 — rc $1, expected $2"; fi
}

# ---------------------------------------------------------------- tooling guards
# Fail-closed: a suite that cannot run must never report clean (T-2818 / T-3105).
[ -f "$BVP" ] || { echo "TOOLING: $BVP not found" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || { echo "TOOLING: python3 missing" >&2; exit 2; }
python3 -c 'import yaml' 2>/dev/null || { echo "TOOLING: PyYAML missing" >&2; exit 2; }

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

# ---------------------------------------------------------------- scratch project
mk_project() { # $1 = dir
    local d="$1"
    mkdir -p "$d/.tasks/active" "$d/.context" "$d/policy"
    cat > "$d/.framework.yaml" <<'YAML'
project_name: fixture-project
version: 1.6.29
YAML
    # Mirrors the real shape: a list of timestamped proposal entries, newest last,
    # with the estimator's rationale carrying the per-driver no-signal markers.
    cat > "$d/.tasks/active/T-9001-fixture.md" <<'MD'
---
id: T-9001
name: "Fixture task"
status: started-work
workflow_type: build
owner: agent
bvp_scores_proposed:
  - ts: '2026-09-27T00:00:00Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 4
      D3: 2
    rationale: D1=2 (no-signal); D2=4 (body:structural-gate); D3=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-9001
MD
    # A second task where EVERY driver is no-signal — the 16%-of-449 shape that
    # amendment (a) exists to keep visible.
    cat > "$d/.tasks/active/T-9002-allnosignal.md" <<'MD'
---
id: T-9002
name: "Fixture task, fully no-signal"
status: started-work
workflow_type: build
owner: agent
bvp_scores_proposed:
  - ts: '2026-09-27T00:00:00Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-9002
MD
    cp -r "$PROJECT/policy/." "$d/policy/" 2>/dev/null || true
}

# Drive bvp.sh exactly as an agent session would: CLAUDECODE=1, no human flag.
run_bvp() { # env assignments come from the caller's exported vars
    env PROJECT_ROOT="$PROJ" \
        FRAMEWORK_ROOT="$FW_ROOT" \
        CLAUDECODE=1 \
        FW_BVP_TELEMETRY_PATH="$LEDGER" \
        ${SWITCH+BVP_AUTO_CONFIRM="$SWITCH"} \
        bash "$BVP" "$@" 2>&1
}

echo "=== T-3176: BVP auto-confirm switch + confirmation ledger ==="

# ================================================================= Case 1
echo
echo "Case 1 — switch OFF (absent): confirm still refuses, exactly as today"
PROJ="$SCRATCH/c1"; LEDGER="$SCRATCH/c1.ndjson"; mk_project "$PROJ"; unset SWITCH
out="$(run_bvp confirm T-9001)"; rc=$?
assert_rc "$rc" 1 "refuses with rc 1"
assert_contains "$out" "agents must not invoke" "prints the §ACD refusal"
assert_contains "$out" "--i-am-human" "still offers the human override"
if [ -s "$LEDGER" ]; then fail "a refused confirm wrote a ledger row"; else
    ok "a refused confirm writes no ledger row"; fi
if grep -q 'bvp_scores:' "$PROJ/.tasks/active/T-9001-fixture.md"; then
    fail "a refused confirm wrote bvp_scores to the task"
else ok "a refused confirm leaves the task untouched"; fi

# ================================================================= Case 2
echo
echo "Case 2 — switch ON: confirm proceeds and is attributed to the agent, not \$USER"
PROJ="$SCRATCH/c2"; LEDGER="$SCRATCH/c2.ndjson"; mk_project "$PROJ"; SWITCH=1
out="$(run_bvp confirm T-9001)"; rc=$?
assert_rc "$rc" 0 "confirms with rc 0"
task="$(cat "$PROJ/.tasks/active/T-9001-fixture.md")"
assert_contains "$task" "confirmed_by: agent:auto (BVP_AUTO_CONFIRM)" \
    "amendment (b): confirmed_by names the auto path, never \$USER"
assert_not_contains "$task" "confirmed_by: ${USER:-nobody}" \
    "amendment (b): the OS account never appears on the auto path"
assert_contains "$task" "D2: 4" "the proposed scores were promoted"

# ================================================================= Case 3  ★
echo
echo "Case 3 ★ LOAD-BEARING — switch ON, the other four verbs STILL refuse"
echo "          (this is what distinguishes a surgical opening from a blanket bypass)"
PROJ="$SCRATCH/c3"; LEDGER="$SCRATCH/c3.ndjson"; mk_project "$PROJ"; SWITCH=1
for spec in \
    "weight|--set|D1=4|--rationale|a rationale long enough to pass the length check" \
    "driver|--add|F-NEW|--rationale|a rationale long enough to pass the length check" \
    "driver|--remove|F-NEW|--rationale|a rationale long enough to pass the length check" \
    "auto-promote|--enable|--rationale|a rationale long enough to pass the length check"
do
    IFS='|' read -r -a argv <<< "$spec"
    vout="$(run_bvp "${argv[@]}")"; vrc=$?
    label="${argv[0]} ${argv[1]}"
    if [ "$vrc" -eq 0 ]; then
        fail "$label was ALLOWED with the switch on — the bypass is not surgical"
    else
        ok "$label still refuses (rc $vrc)"
    fi
    assert_contains "$vout" "agents must not invoke" "$label prints the §ACD refusal"
done

# ================================================================= Case 4
echo
echo "Case 4 — the ledger row: promote semantics are what make it meaningful"
PROJ="$SCRATCH/c4"; LEDGER="$SCRATCH/c4.ndjson"; mk_project "$PROJ"; SWITCH=1
run_bvp confirm T-9001 >/dev/null
if [ ! -s "$LEDGER" ]; then
    fail "no ledger row was written for a successful confirm"
else
    ok "one row written for a successful confirm"
    row="$(head -1 "$LEDGER")"
    assert_contains "$row" '"path": "auto"' "row records the auto path"
    assert_contains "$row" '"proposal_existed": true' "row records that a proposal existed"
    assert_contains "$row" '"proposer_exact": true' \
        "unchanged promotion with no overrides logs proposer_exact"
    assert_contains "$row" '"no_signal_count": 2' \
        "amendment (a): 2 of 3 drivers were no-signal"
    assert_contains "$row" '"driver_count": 3' "driver_count recorded alongside it"
    assert_contains "$row" '"all_no_signal": false' \
        "a partially-evidenced proposal is not flagged all_no_signal"
    assert_contains "$row" '"estimator": "bvp-estimator-v1-heuristic"' "estimator carried through"
    python3 -c "import json,sys; json.loads(open('$LEDGER').readline())" 2>/dev/null \
        && ok "row is parseable JSON" || fail "row is not parseable JSON"
fi

# ================================================================= Case 5
echo
echo "Case 5 — amendment (a): the all-no-signal shape is visible, not flattering"
PROJ="$SCRATCH/c5"; LEDGER="$SCRATCH/c5.ndjson"; mk_project "$PROJ"; SWITCH=1
run_bvp confirm T-9002 >/dev/null
row="$(head -1 "$LEDGER" 2>/dev/null || echo '')"
assert_contains "$row" '"all_no_signal": true' \
    "every driver no-signal is flagged"
assert_contains "$row" '"proposer_exact": true' \
    "...and it STILL logs proposer_exact — which is exactly why the flag is needed:"
echo "        without all_no_signal this row reads as the estimator being right"
echo "        about a task it never assessed, inflating any accuracy figure."

# ================================================================= Case 6
echo
echo "Case 6 — the real ledger is never written by a fixture run"
real="$PROJECT/.context/telemetry/bvp-confirmations.ndjson"
before="$(wc -c < "$real" 2>/dev/null || echo 0)"
PROJ="$SCRATCH/c6"; LEDGER="$SCRATCH/c6.ndjson"; mk_project "$PROJ"; SWITCH=1
run_bvp confirm T-9001 >/dev/null
after="$(wc -c < "$real" 2>/dev/null || echo 0)"
if [ "$before" = "$after" ]; then
    ok "FW_BVP_TELEMETRY_PATH kept the real append-only ledger untouched"
else
    fail "a fixture run wrote into the REAL ledger ($before -> $after bytes) — unremovable"
fi

# ================================================================= Case 7
echo
echo "Case 7 — fail-closed: only a recognised truthy value opens the gate"
for bad in 0 false FALSE no off "" "yes-ish" "TRUE " garbage; do
    PROJ="$SCRATCH/c7"; LEDGER="$SCRATCH/c7.ndjson"
    rm -rf "$PROJ"; mk_project "$PROJ"; SWITCH="$bad"
    out="$(run_bvp confirm T-9001)"; rc=$?
    case "$bad" in
        "TRUE ")  # trailing space is stripped, so this one legitimately opens
            assert_rc "$rc" 0 "switch='TRUE ' (stripped+lowered) opens the gate" ;;
        *)
            if [ "$rc" -eq 0 ]; then
                fail "switch='$bad' OPENED the gate — must fail closed"
            else
                ok "switch='$bad' keeps the gate closed"
            fi ;;
    esac
done

# ================================================================= Case 8
echo
echo "Case 8 — the human path is never removed, switch on or off"
for sw in 0 1; do
    PROJ="$SCRATCH/c8-$sw"; LEDGER="$SCRATCH/c8-$sw.ndjson"; mk_project "$PROJ"; SWITCH="$sw"
    out="$(run_bvp confirm T-9001 --i-am-human)"; rc=$?
    assert_rc "$rc" 0 "--i-am-human works with switch=$sw"
    task="$(cat "$PROJ/.tasks/active/T-9001-fixture.md")"
    assert_not_contains "$task" "agent:auto" \
        "an explicit human confirm is NOT attributed to the agent (switch=$sw)"
    row="$(head -1 "$LEDGER" 2>/dev/null || echo '')"
    assert_contains "$row" '"path": "human"' \
        "human confirmations are recorded too, so the ledger can compare the two"
done

# ================================================================= Case 9 (mutant)
echo
echo "Case 9 — mutant: an auto_ok that ignores the switch must be caught by Case 3"
mut="$SCRATCH/bvp-mutant.sh"
sed 's/^    if auto_ok:$/    if True:/' "$BVP" > "$mut"
if ! grep -q '^    if True:' "$mut"; then
    fail "MUTATION SETUP BROKEN — the auto_ok branch was not substituted"
    echo "        (a mutant that was never applied reads identically to one nothing caught)"
else
    ok "mutant applied (auto_ok branch forced open)"
    PROJ="$SCRATCH/c9"; LEDGER="$SCRATCH/c9.ndjson"; mk_project "$PROJ"; SWITCH=1
    mrc=0
    env PROJECT_ROOT="$PROJ" FRAMEWORK_ROOT="$FW_ROOT" CLAUDECODE=1 \
        FW_BVP_TELEMETRY_PATH="$LEDGER" BVP_AUTO_CONFIRM=1 \
        bash "$mut" weight --set D1=4 --rationale "a rationale long enough to pass" \
        >/dev/null 2>&1 || mrc=$?
    if [ "$mrc" -eq 0 ]; then
        ok "mutant KILLED — blanket bypass lets weight --set through, Case 3 would go red"
    else
        fail "mutant SURVIVED — Case 3 cannot distinguish surgical from blanket"
    fi
fi

# ---------------------------------------------------------------- summary
echo
echo "=== SUMMARY ==="
echo "Pass: $PASS"
echo "Fail: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
