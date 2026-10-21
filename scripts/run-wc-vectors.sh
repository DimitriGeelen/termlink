#!/usr/bin/env bash
# Runs AEF's shared worker-contract conformance vectors (wc/0.3, CC-01..CC-25) against the
# TermLink hub backend (T-3517).
#
# The vector file is AEF's. It is fetched at a pinned AEF commit into a temp dir that is removed
# on exit; nothing is written into this repo, and vectors.yaml is never committed here. The
# adapter is crates/termlink-hub/tests/wc_conformance.rs (with wc_conformance/hub.rs). Vectors are
# DATA: the script reads them through the adapter and never runs anything from them.
#
# Usage: scripts/run-wc-vectors.sh [--ref <aef-commit>]     (default ref: 25f56ed68)
#        History: afe974124 first published; b4c168aa9 fixed CC-11/12 (question asked first);
#        25f56ed68 added seen_alive_epoch preconditions for confirm_end (CC-01/02/03/06/09/25).
# Exit:  0 no case fails (not-implemented cases are listed, not hidden)
#        1 at least one case fails against the hub backend
#        2 tooling: fetch failed, build failed, or the adapter did not reach its summary line
#
# Set TERMLINK_WC_CASE=CC-NN to run a single case, e.g. to reproduce a finding.

set -uo pipefail

REF="25f56ed68"
while [ $# -gt 0 ]; do
    case "$1" in
        --ref) REF="${2:?--ref needs an AEF commit}"; shift 2 ;;
        -h|--help) sed -n '2,15p' "$0"; exit 0 ;;
        *) echo "run-wc-vectors: unknown argument: $1" >&2; exit 2 ;;
    esac
done

URL="https://raw.githubusercontent.com/DimitriGeelen/agentic-engineering-framework/${REF}/tests/conformance/worker-contract/vectors.yaml"
REPO="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)" || { echo "run-wc-vectors: mktemp failed" >&2; exit 2; }
trap 'rm -rf "$TMP"' EXIT

if ! curl -fsSL --max-time 60 "$URL" -o "$TMP/vectors.yaml"; then
    echo "run-wc-vectors: tooling: could not fetch $URL" >&2
    echo "run-wc-vectors: no vectors, so nothing was judged (not a pass)" >&2
    exit 2
fi
echo "vectors: AEF ${REF} (${URL})"

OUT="$TMP/run.out"
(
    cd "$REPO" && \
    TERMLINK_WC_VECTORS="$TMP/vectors.yaml" \
        cargo test -p termlink-hub --test wc_conformance -- --nocapture --test-threads=1
) > "$OUT" 2>&1
RC=$?

grep -E '^(backend:|  CC-|         |summary:)' "$OUT"

SUMMARY="$(grep '^summary: ' "$OUT" | tail -1)"
if [ -z "$SUMMARY" ]; then
    echo "run-wc-vectors: tooling: the adapter did not reach its summary line (rc=$RC)" >&2
    grep -E 'error(\[|:)|panicked' "$OUT" | head -20 >&2
    exit 2
fi

FAILS="$(printf '%s\n' "$SUMMARY" | sed -n 's/.* fail=\([0-9]*\) .*/\1/p')"
if [ "${FAILS:-x}" = "0" ] && [ "$RC" -eq 0 ]; then
    exit 0
fi
if [ "${FAILS:-0}" -gt 0 ] 2>/dev/null; then
    exit 1
fi
echo "run-wc-vectors: tooling: adapter exited rc=$RC with fail=${FAILS:-?}" >&2
exit 2
