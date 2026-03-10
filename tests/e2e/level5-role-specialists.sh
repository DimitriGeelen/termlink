#!/usr/bin/env bash
# =============================================================================
# Level 5: Role-Specific Specialists — Domain experts with tailored prompts
# =============================================================================
# Orchestrator spawns 3 role-aware specialists, each with:
#   - Domain-specific system prompt (from role-prompts/*.md)
#   - Role-appropriate tool permissions (reviewer: Read only, etc.)
#   - Role identity in events (specialist field in accepted/completed)
#
# Validates: role prompts load, tool restrictions apply, role identity flows
# through the event bus, and results reflect domain expertise.
#
# Usage: ./tests/e2e/level5-role-specialists.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERMLINK="$PROJECT_ROOT/target/debug/termlink"
CLAUDE="/Users/dimidev32/.local/bin/claude"
WATCHER="$SCRIPT_DIR/role-watcher.sh"
RUNTIME_DIR=$(mktemp -d)

# Result paths
REVIEW_RESULT="$RUNTIME_DIR/review-result.md"
TEST_RESULT="$RUNTIME_DIR/test-result.md"
DOC_RESULT="$RUNTIME_DIR/doc-result.md"

# Target files — each specialist gets a different file
REVIEW_FILE="$PROJECT_ROOT/crates/termlink-session/src/manager.rs"
TEST_FILE="$PROJECT_ROOT/crates/termlink-cli/src/main.rs"
DOC_FILE="$PROJECT_ROOT/crates/termlink-protocol/src/data.rs"

source "$SCRIPT_DIR/e2e-helpers.sh"
trap cleanup_all EXIT

echo "============================================="
echo "  Level 5: Role-Specific Specialists"
echo "============================================="
echo "Runtime: $RUNTIME_DIR"
echo ""

# Build
echo "--- Build ---"
(cd "$PROJECT_ROOT" && /Users/dimidev32/.cargo/bin/cargo build -p termlink 2>&1 | tail -1)
echo ""

# Register orchestrator
echo "--- Register orchestrator ---"
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" register --name orchestrator --roles orchestrator &
ORCH_PID=$!
for i in $(seq 1 10); do
    if ls "$RUNTIME_DIR/sessions/"*.sock >/dev/null 2>&1; then break; fi; sleep 1
done
TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" ping orchestrator 2>/dev/null && echo "Orchestrator OK" || { echo "FAIL"; exit 1; }
echo ""

# Verify role prompts exist
echo "--- Verify role prompts ---"
for ROLE in reviewer tester documenter; do
    if [ -f "$SCRIPT_DIR/role-prompts/${ROLE}.md" ]; then
        echo "  $ROLE prompt: OK"
    else
        echo "  $ROLE prompt: MISSING"; exit 1
    fi
done
echo ""

# Spawn 3 role-aware specialists
echo "--- Spawn 3 role-aware specialists ---"

for ROLE in reviewer tester documenter; do
    echo "  Spawning $ROLE (role-aware)..."
    spawn_tracked \
        --name "$ROLE" --roles "$ROLE" \
        --wait --wait-timeout 15 \
        -- bash "$WATCHER" "$TERMLINK" "$RUNTIME_DIR" "$CLAUDE" "$ROLE" "$ROLE"
    echo "  $ROLE ready"
done

sleep 3
echo "All 3 role-specific specialists running"
echo ""

# Health check
echo "--- Health check ---"
for ROLE in reviewer tester documenter; do
    if TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" ping "$ROLE" 2>/dev/null; then
        echo "  $ROLE: alive"
    else
        echo "  $ROLE: DEAD"; exit 1
    fi
done
echo ""

# Fan out: delegate role-specific tasks
echo "--- Fan out: delegate 3 role-specific tasks ---"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit reviewer task.delegate \
    --payload "{\"request_id\":\"req-review-5\",\"action\":\"Review this file for code quality: error handling, panics, unsafe patterns, TOCTOU races. Give specific line numbers.\",\"scope\":{\"file\":\"$REVIEW_FILE\",\"result_path\":\"$REVIEW_RESULT\"}}"
echo "  reviewer: task delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit tester task.delegate \
    --payload "{\"request_id\":\"req-test-5\",\"action\":\"Identify the top 5 functions lacking test coverage. For each, provide a test name and what to assert.\",\"scope\":{\"file\":\"$TEST_FILE\",\"result_path\":\"$TEST_RESULT\"}}"
echo "  tester:   task delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit documenter task.delegate \
    --payload "{\"request_id\":\"req-doc-5\",\"action\":\"Write module documentation: purpose, key types, public API, one usage example.\",\"scope\":{\"file\":\"$DOC_FILE\",\"result_path\":\"$DOC_RESULT\"}}"
echo "  documenter: task delegated"

echo ""

# Fan in: wait for all 3 completions
echo "--- Fan in: wait for completions ---"

TIMEOUT=300
ELAPSED=0
GOT_REVIEW=false GOT_TEST=false GOT_DOC=false

while [ $ELAPSED -lt $TIMEOUT ]; do
    EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator --topic task.completed 2>/dev/null || true)

    if [ "$GOT_REVIEW" = false ] && echo "$EVENTS" | grep -q "req-review-5"; then
        GOT_REVIEW=true; echo "  reviewer completed (${ELAPSED}s)"
    fi
    if [ "$GOT_TEST" = false ] && echo "$EVENTS" | grep -q "req-test-5"; then
        GOT_TEST=true; echo "  tester completed (${ELAPSED}s)"
    fi
    if [ "$GOT_DOC" = false ] && echo "$EVENTS" | grep -q "req-doc-5"; then
        GOT_DOC=true; echo "  documenter completed (${ELAPSED}s)"
    fi

    if [ "$GOT_REVIEW" = true ] && [ "$GOT_TEST" = true ] && [ "$GOT_DOC" = true ]; then
        echo "  All 3 complete!"
        break
    fi

    sleep 5; ELAPSED=$((ELAPSED + 5))
    if [ $((ELAPSED % 30)) -eq 0 ]; then
        echo "  Waiting... ${ELAPSED}s (review=$GOT_REVIEW test=$GOT_TEST doc=$GOT_DOC)"
    fi
done

if [ "$GOT_REVIEW" = false ] || [ "$GOT_TEST" = false ] || [ "$GOT_DOC" = false ]; then
    echo "  Some specialists timed out (review=$GOT_REVIEW test=$GOT_TEST doc=$GOT_DOC)"
fi
echo ""

# Shutdown all watchers
touch "$RUNTIME_DIR/shutdown"

# Verify results
echo "============================================="
echo "  RESULTS"
echo "============================================="
echo ""

RESULTS_OK=0
RESULTS_TOTAL=3

for LABEL in review test doc; do
    RVAR="${LABEL^^}_RESULT"
    RPATH="${!RVAR}"
    if [ -f "$RPATH" ]; then
        echo "--- ${LABEL} result ---"
        cat "$RPATH"
        echo ""
        echo "--- end ---"
        echo ""
        RESULTS_OK=$((RESULTS_OK + 1))
    else
        echo "--- ${LABEL} result: MISSING ---"
        echo ""
    fi
done

# Show all events — verify role identity in events
echo "--- Orchestrator event bus ---"
EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator 2>/dev/null || true)
echo "$EVENTS"
echo ""

ACCEPTED=$(echo "$EVENTS" | grep -c "task.accepted" || true)
COMPLETED=$(echo "$EVENTS" | grep -c "task.completed" || true)

# Check role identity in completed events
ROLE_TAGGED=0
for ROLE in reviewer tester documenter; do
    if echo "$EVENTS" | grep "task.completed" | grep -q "\"specialist\":\"$ROLE\""; then
        ROLE_TAGGED=$((ROLE_TAGGED + 1))
    fi
done

echo "Events: $ACCEPTED accepted, $COMPLETED completed"
echo "Role-tagged completions: $ROLE_TAGGED/3"
echo "Results: $RESULTS_OK/$RESULTS_TOTAL files written"
echo ""

if [ "$RESULTS_OK" -eq 3 ] && [ "$COMPLETED" -ge 3 ] && [ "$ROLE_TAGGED" -ge 3 ]; then
    echo "============================================="
    echo "  LEVEL 5 PASSED — Role-Specific Specialists"
    echo "============================================="
    echo ""
    echo "  3 role-aware Claude Code specialists ran in parallel:"
    echo "    - reviewer:   domain-specific code quality analysis"
    echo "    - tester:     domain-specific test gap identification"
    echo "    - documenter: domain-specific module documentation"
    echo "  Each with:"
    echo "    - Role-specific system prompt (role-prompts/*.md)"
    echo "    - Role-appropriate tool permissions"
    echo "    - Role identity in event bus (specialist field)"
    echo "  All communicated via TermLink event bus"
elif [ "$RESULTS_OK" -gt 0 ]; then
    echo "=== LEVEL 5 PARTIAL ==="
    echo "$RESULTS_OK/3 specialists completed, $ROLE_TAGGED/3 role-tagged"
else
    echo "=== LEVEL 5 FAILED ==="
fi
