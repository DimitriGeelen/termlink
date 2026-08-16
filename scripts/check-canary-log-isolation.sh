#!/usr/bin/env bash
# guard-layer: source --no-heartbeat
# check-canary-log-isolation.sh (T-2761, G-019 prevention for the T-2402-sibling class)
#
# THE DEFECT THIS GUARDS
#
# `scripts/agent-send.sh` escalates a never-acked send by appending a framed entry
# to the woken-but-silent canary log (T-2402 Stage 5 — a LOUD canary instead of a
# silent `exit 3`). The destination defaults to the operator's real
#
#     .context/working/.woken-but-silent-canary.log
#
# and is redirectable via TERMLINK_WOKEN_SILENT_LOG.
#
# Test scripts drive that give-up path ON PURPOSE — that is the behaviour under
# test. So every test that invokes the real agent-send.sh MUST redirect the log,
# or its own assertions write into the operator's signal channel.
#
# `scripts/test-agent-send.sh` has exported the override since T-2402 Stage 5.
# `scripts/test-agent-respond.sh` never did, and it drives the same real
# agent-send.sh against a deliberately-absent session — so its give-up is
# GUARANTEED, not incidental. Residue found in the production log:
#
#     woken-but-silent: no receipt for cid=cidB-962839 on topic=agent-respond-test-962839
#       recipient= session=no-such-session-962839
#
# `agent-respond-test-$$` and `no-such-session-$$` are that script's own fixtures.
#
# WHY THIS IS WORSE THAN ORDINARY TEST NOISE
#
# "Empty log = healthy" is a ONE-BIT channel. Once test residue is in the file the
# canary reads FIRING forever, so a subsequent GENUINE woken-but-silent event
# appends to an already-non-empty file and changes nothing an operator can see.
# The canary is not merely noisy — it is DEAF until someone truncates it by hand.
# That is precisely the harm T-2685 documents for the stderr-merge mistake,
# arriving through a different door: T-2685 guards the CRON job lines, this guards
# the TEST scripts. Same one-bit channel, same way to destroy it.
#
# It is also the recurring "hardened in one place, siblings not migrated" shape
# that check-busy-spin.sh (T-2672), T-2667 and T-2673 each exist to catch — here
# in the test layer, where no static check was looking.
#
# WHAT COUNTS AS REDIRECTED
#
# Any assignment of TERMLINK_WOKEN_SILENT_LOG in the same file, whether exported
# or set per-invocation. The check does not care about the destination path — only
# that the script does not silently inherit the production default.
#
# SCOPE. Test scripts only (`scripts/test-*.sh`, `tests/*.sh`). Production callers
# of agent-send.sh SHOULD write to the real log — that is the canary working as
# designed — so they are deliberately out of scope. This checks isolation of the
# test layer, not usage of the canary.
#
# Exit codes: 0 clean · 1 a test invokes agent-send.sh without redirecting
#             · 2 tooling error (fail-closed)
set -uo pipefail

SCRIPTS_DIR="${CANARY_ISOLATION_SCRIPTS_DIR:-scripts}"
TESTS_DIR="${CANARY_ISOLATION_TESTS_DIR:-tests}"
QUIET=0
FORMAT=human

usage() {
    cat <<'EOF'
check-canary-log-isolation.sh — a test that drives the real agent-send.sh must
redirect TERMLINK_WOKEN_SILENT_LOG, or its by-design give-up path writes into the
operator's real woken-but-silent canary log and leaves it permanently FIRING.

Correct:   export TERMLINK_WOKEN_SILENT_LOG="$tmp/woken-silent-canary.log"
Rejected:  (no assignment anywhere in a test that invokes agent-send.sh)

Usage: check-canary-log-isolation.sh [OPTIONS]
  --json          Emit {ok, firing[], checked, scanned}
  --quiet         Print only on firing (cron-friendly)
  --no-heartbeat  Accepted for guard-layer parity; this check writes no heartbeat
  -h, --help      This help

Test seams: CANARY_ISOLATION_SCRIPTS_DIR=<dir> (default scripts)
            CANARY_ISOLATION_TESTS_DIR=<dir>   (default tests)
Fixtures:   bash tests/canary-log-isolation-fixtures.sh

Exit: 0 clean · 1 an unredirected test · 2 tooling error
EOF
}

while [ $# -gt 0 ]; do
    case "$1" in
        --json)  FORMAT=json; shift ;;
        --quiet) QUIET=1; shift ;;
        --no-heartbeat) shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-canary-log-isolation: unknown arg: $1" >&2; exit 2 ;;
    esac
done

if [ ! -d "$SCRIPTS_DIR" ]; then
    echo "check-canary-log-isolation: scripts dir not found: $SCRIPTS_DIR" >&2
    exit 2
fi

firing=(); checked=0; scanned=0

# Candidate test scripts: scripts/test-*.sh plus everything under tests/.
# A missing tests/ dir is not an error (a repo may keep all tests in scripts/).
candidates=()
for f in "$SCRIPTS_DIR"/test-*.sh; do
    [ -e "$f" ] && candidates+=("$f")
done
if [ -d "$TESTS_DIR" ]; then
    for f in "$TESTS_DIR"/*.sh; do
        [ -e "$f" ] && candidates+=("$f")
    done
fi

if [ "${#candidates[@]}" -eq 0 ]; then
    # No test scripts at all means the scan is not measuring what it claims.
    # Fail closed rather than report a vacuous clean census (T-2747 lesson).
    echo "check-canary-log-isolation: no test scripts found under $SCRIPTS_DIR or $TESTS_DIR" >&2
    exit 2
fi

for f in "${candidates[@]}"; do
    scanned=$((scanned+1))
    # Does this script invoke the real agent-send.sh? Comments are stripped first:
    # prose ABOUT agent-send.sh is not a call to it.
    body="$(sed 's/#.*//' "$f" 2>/dev/null)" || {
        echo "check-canary-log-isolation: cannot read $f" >&2
        exit 2
    }
    printf '%s' "$body" | grep -q 'agent-send\.sh' || continue

    checked=$((checked+1))
    # Redirected if the file assigns TERMLINK_WOKEN_SILENT_LOG anywhere —
    # exported, or set inline on a single invocation. Either isolates the log.
    if printf '%s' "$body" | grep -qE 'TERMLINK_WOKEN_SILENT_LOG[[:space:]]*='; then
        continue
    fi
    firing+=("$f|invokes agent-send.sh but never assigns TERMLINK_WOKEN_SILENT_LOG — its give-up path appends to the real .woken-but-silent-canary.log")
done

n=${#firing[@]}

if [ "$FORMAT" = json ]; then
    printf '{"ok":%s,"firing":[' "$([ "$n" -eq 0 ] && echo true || echo false)"
    i=0
    while [ "$i" -lt "$n" ]; do
        [ "$i" -eq 0 ] || printf ','
        e="${firing[$i]}"
        printf '{"file":%s,"why":%s}' \
            "$(printf '%s' "${e%%|*}" | jq -R .)" \
            "$(printf '%s' "${e#*|}" | jq -R .)"
        i=$((i+1))
    done
    printf '],"checked":%d,"scanned":%d}\n' "$checked" "$scanned"
    [ "$n" -eq 0 ] && exit 0 || exit 1
fi

if [ "$n" -eq 0 ]; then
    [ "$QUIET" -eq 1 ] || echo "canary-log isolation: clean — $checked of $scanned test script(s) invoke agent-send.sh, all redirect TERMLINK_WOKEN_SILENT_LOG"
    exit 0
fi

echo "canary-log isolation: $n test script(s) write into the real woken-but-silent canary log"
for e in "${firing[@]}"; do
    echo "  ${e%%|*}"
    echo "    ↳ ${e#*|}"
done
echo
echo "Remediation: add, after the tmp dir is created and trapped —"
echo '    export TERMLINK_WOKEN_SILENT_LOG="$tmp/woken-silent-canary.log"'
echo "Then truncate the polluted log so the canary can signal again:"
echo "    : > .context/working/.woken-but-silent-canary.log"
exit 1
