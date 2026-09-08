#!/usr/bin/env bash
# comms-selftest-fixtures.sh (T-2696) — guard-layer entry point for the comms
# prover's hermetic harness.
#
# The harness itself is scripts/test-comms-selftest.sh (T-2482): it drives the
# DISCOVER/SEND/CONSUME stages of comms-selftest.sh entirely on canned fixtures
# (TERMLINK_DIAGNOSE_TEST_PRESENCE_JSON + COMMS_SELFTEST_TEST_SEND_RC) — no live
# hub, no peer, no network. But it sits in scripts/, and the guard-layer runner
# (T-2684) picks fixture suites up by the tests/*fixtures*.sh naming convention —
# so it was hermetic AND executed by nothing (the exact T-2683 shipped-but-dark
# class this repo keeps refinding). This thin wrapper puts it where the runner
# and the T-2686 CI job look. Deliberately exec-thin: two copies of the case
# list would drift, and the copy that drifts is the one nobody runs.
set -u

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec bash "$SELF_DIR/../scripts/test-comms-selftest.sh" "$@"
