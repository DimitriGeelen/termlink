#!/usr/bin/env bash
# tests/pickup-failopen-fixtures.sh — T-2687 regression fixtures.
#
# Pins the four fail-open refusals added to .agentic-framework/lib/pickup.sh:
#
#   1. pickup_dedup_hash    refuses an unreadable envelope
#   2. pickup_dedup_hash    refuses an all-empty extraction (the constant-digest bug)
#   3. pickup_record_dedup  refuses to append a row it cannot compute
#   4. pickup_dedup_check   degrades to "not a duplicate" (never silently drops)
#   5. pickup_create_inception refuses an empty payload.summary / source.project
#   6. pickup_process_one   stops on that refusal (no dedup row, envelope stays put)
#
# ...plus the happy path, so the guards cannot be "fixed" by refusing everything.
#
# Hub-independent and binary-independent (PL-213): everything runs against fixture
# envelopes under a scratch PROJECT_ROOT. No network, no live hub, no real `fw`.
#
# Usage: bash tests/pickup-failopen-fixtures.sh
# Exit:  0 = all pass, 1 = a fixture regressed.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PICKUP_LIB="$REPO_ROOT/.agentic-framework/lib/pickup.sh"

# The exact digest the pre-T-2687 code emitted for an all-empty extraction:
# sha256("||"). If this ever comes back out of pickup_dedup_hash, the fail-open
# path has been reintroduced.
POISON_DIGEST="565d240f5343e625ae579a4d45a770f1f02c6368b5ed4d06da4fbe6f47c28866"

PASS=0
FAIL=0

ok()   { PASS=$((PASS + 1)); printf '  PASS  %s\n' "$1"; }
bad()  { FAIL=$((FAIL + 1)); printf '  FAIL  %s\n' "$1" >&2; [ -n "${2:-}" ] && printf '          %s\n' "$2" >&2; }

if [ ! -r "$PICKUP_LIB" ]; then
    echo "pickup-failopen-fixtures: cannot read $PICKUP_LIB" >&2
    exit 2
fi

SCRATCH="$(mktemp -d)"
trap 'rm -rf "$SCRATCH"' EXIT

export PROJECT_ROOT="$SCRATCH"
# shellcheck source=/dev/null
source "$PICKUP_LIB"
pickup_ensure_dirs
mkdir -p "$PICKUP_AUTO_DEFERRED"

# --- fixtures ---------------------------------------------------------------

GOOD="$PICKUP_INBOX/good.yaml"
cat > "$GOOD" <<'EOF'
pickup_id: P-900
version: 1
type: bug-report
source:
  project: "fixture-project"
  task_id: "T-001"
  agent: "fixture"
  timestamp: "2026-08-20T00:00:00Z"
payload:
  summary: "a real summary that must survive extraction"
  detail: "detail body"
  priority: high
  tags: [fixture]
EOF

# Keys present (so key-presence validation passes) but values empty — the shape
# that used to hash to the poison constant and mint "Pickup:  (from )".
EMPTY="$PICKUP_INBOX/empty-values.yaml"
cat > "$EMPTY" <<'EOF'
pickup_id: P-901
version: 1
type:
source:
  project:
payload:
  summary:
EOF

# Valid type + project, but an empty summary only.
NOSUM="$PICKUP_INBOX/no-summary.yaml"
cat > "$NOSUM" <<'EOF'
pickup_id: P-902
version: 1
type: bug-report
source:
  project: "fixture-project"
payload:
  summary:
EOF

# Valid type + summary, but an empty project only.
NOPROJ="$PICKUP_INBOX/no-project.yaml"
cat > "$NOPROJ" <<'EOF'
pickup_id: P-903
version: 1
type: bug-report
source:
  project:
payload:
  summary: "has a summary but no source project"
EOF

MISSING="$PICKUP_INBOX/does-not-exist.yaml"

echo "T-2687 pickup fail-open fixtures"
echo

# --- 1. pickup_dedup_hash refuses an unreadable envelope ---------------------

if out=$(pickup_dedup_hash "$MISSING" 2>&1); then
    bad "dedup_hash refuses unreadable envelope" "returned 0 with output: $out"
elif [ "$out" = "$POISON_DIGEST" ]; then
    bad "dedup_hash refuses unreadable envelope" "emitted the poison digest"
else
    ok "dedup_hash refuses unreadable envelope (non-zero, no digest)"
fi

# --- 2. pickup_dedup_hash refuses an all-empty extraction --------------------

if out=$(pickup_dedup_hash "$EMPTY" 2>&1); then
    bad "dedup_hash refuses all-empty extraction" "returned 0 with output: $out"
elif printf '%s' "$out" | grep -q "$POISON_DIGEST"; then
    bad "dedup_hash refuses all-empty extraction" "emitted the poison digest"
else
    ok "dedup_hash refuses all-empty extraction (the constant-digest bug)"
fi

# --- 3. happy path still hashes ---------------------------------------------

if good_hash=$(pickup_dedup_hash "$GOOD" 2>/dev/null); then
    if printf '%s' "$good_hash" | grep -Eq '^[0-9a-f]{64}$'; then
        if [ "$good_hash" = "$POISON_DIGEST" ]; then
            bad "dedup_hash happy path" "a well-formed envelope hashed to the poison digest"
        else
            ok "dedup_hash happy path returns a real 64-hex digest"
        fi
    else
        bad "dedup_hash happy path" "not a 64-hex digest: $good_hash"
    fi
else
    bad "dedup_hash happy path" "refused a well-formed envelope (over-refusal)"
fi

# --- 4. pickup_record_dedup refuses to write an uncomputable row -------------

: > "$PICKUP_DEDUP_LOG"
if pickup_record_dedup "$MISSING" 2>/dev/null; then
    bad "record_dedup refuses uncomputable hash" "returned 0"
elif [ -s "$PICKUP_DEDUP_LOG" ]; then
    bad "record_dedup refuses uncomputable hash" "ledger was written anyway"
else
    ok "record_dedup refuses uncomputable hash and leaves the ledger untouched"
fi

# happy path still records exactly one row
if pickup_record_dedup "$GOOD" 2>/dev/null && [ "$(wc -l < "$PICKUP_DEDUP_LOG")" -eq 1 ]; then
    ok "record_dedup happy path appends exactly one ledger row"
else
    bad "record_dedup happy path" "expected 1 ledger row, got $(wc -l < "$PICKUP_DEDUP_LOG")"
fi

# --- 5. pickup_dedup_check fails safe (not-a-duplicate) ----------------------

# A poison row in the ledger must NOT swallow an unrelated unreadable envelope.
echo "2026-08-20T00:00:00Z|${POISON_DIGEST}|poison.yaml" >> "$PICKUP_DEDUP_LOG"
if pickup_dedup_check "$MISSING" 2>/dev/null; then
    bad "dedup_check fails safe on uncomputable hash" "classified it as a DUPLICATE (would be silently dropped)"
else
    ok "dedup_check fails safe on uncomputable hash (not a duplicate)"
fi

# and a genuinely-repeated envelope is still caught
if pickup_dedup_check "$GOOD" 2>/dev/null; then
    ok "dedup_check still detects a real repeat"
else
    bad "dedup_check still detects a real repeat" "missed a hash already in the ledger"
fi

# --- 6. pickup_create_inception refuses empty summary / project --------------

# Stub `fw` so a guard regression would be caught here rather than shelling out.
FW_CALLED="$SCRATCH/fw-called"
fw() { echo "called with: $*" >> "$FW_CALLED"; echo "File: $SCRATCH/stub-task.md"; }

: > "$FW_CALLED"
if pickup_create_inception "$NOSUM" >/dev/null 2>&1; then
    bad "create_inception refuses empty payload.summary" "returned 0"
elif [ -s "$FW_CALLED" ]; then
    bad "create_inception refuses empty payload.summary" "still invoked fw task create"
else
    ok "create_inception refuses empty payload.summary (no task created)"
fi

: > "$FW_CALLED"
if pickup_create_inception "$NOPROJ" >/dev/null 2>&1; then
    bad "create_inception refuses empty source.project" "returned 0"
elif [ -s "$FW_CALLED" ]; then
    bad "create_inception refuses empty source.project" "still invoked fw task create"
else
    ok "create_inception refuses empty source.project (no task created)"
fi

# the refusal names the offending field so the operator can act
err=$(pickup_create_inception "$NOSUM" 2>&1 >/dev/null)
if printf '%s' "$err" | grep -q "payload.summary"; then
    ok "create_inception refusal names the missing field"
else
    bad "create_inception refusal names the missing field" "stderr was: $err"
fi

# guard does not over-refuse a well-formed envelope
: > "$FW_CALLED"
pickup_create_inception "$GOOD" >/dev/null 2>&1
if [ -s "$FW_CALLED" ]; then
    ok "create_inception happy path reaches task creation"
else
    bad "create_inception happy path" "guard blocked a well-formed envelope (over-refusal)"
fi

# --- 7. pickup_process_one stops on refusal ---------------------------------

: > "$PICKUP_DEDUP_LOG"
: > "$FW_CALLED"
STUCK="$PICKUP_INBOX/stuck.yaml"
cp "$NOSUM" "$STUCK"
if pickup_process_one "$STUCK" false >/dev/null 2>&1; then
    bad "process_one propagates the refusal" "returned 0"
elif [ -s "$PICKUP_DEDUP_LOG" ]; then
    bad "process_one propagates the refusal" "recorded a dedup row anyway"
elif [ ! -f "$STUCK" ]; then
    bad "process_one propagates the refusal" "envelope was moved out of the inbox"
else
    ok "process_one propagates the refusal (no dedup row, envelope stays inspectable)"
fi

# --- summary ----------------------------------------------------------------

echo
echo "pickup-failopen-fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
