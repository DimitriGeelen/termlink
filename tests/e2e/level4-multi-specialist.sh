#!/usr/bin/env bash
# =============================================================================
# Level 4: Multi-Specialist — Parallel agents with different roles
# =============================================================================
# Orchestrator spawns 3 specialists in parallel:
#   - reviewer:  analyzes code quality
#   - tester:    identifies test gaps
#   - documenter: writes module documentation
#
# Each specialist gets a different file and task. All run concurrently
# with their own fresh context windows. Orchestrator fans out tasks,
# waits for all completions, and synthesizes results.
#
# Usage: ./tests/e2e/level4-multi-specialist.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERMLINK="$PROJECT_ROOT/target/debug/termlink"
CLAUDE="/Users/dimidev32/.local/bin/claude"
WATCHER="$SCRIPT_DIR/specialist-watcher.sh"
RUNTIME_DIR=$(mktemp -d)

# Result paths
REVIEW_RESULT="$RUNTIME_DIR/review-result.md"
TEST_RESULT="$RUNTIME_DIR/test-result.md"
DOC_RESULT="$RUNTIME_DIR/doc-result.md"
SYNTHESIS="$RUNTIME_DIR/synthesis.md"

# Target files for each specialist
REVIEW_FILE="$PROJECT_ROOT/crates/termlink-session/src/manager.rs"
TEST_FILE="$PROJECT_ROOT/crates/termlink-cli/src/main.rs"
DOC_FILE="$PROJECT_ROOT/crates/termlink-protocol/src/data.rs"

source "$SCRIPT_DIR/e2e-helpers.sh"
trap cleanup_all EXIT

echo "============================================="
echo "  Level 4: Multi-Specialist (3 parallel)"
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

# Spawn 3 specialists in parallel
echo "--- Spawn 3 specialists ---"

for ROLE in reviewer tester documenter; do
    echo "  Spawning $ROLE..."
    spawn_tracked \
        --name "$ROLE" --roles "$ROLE" \
        --wait --wait-timeout 15 \
        -- bash "$WATCHER" "$TERMLINK" "$RUNTIME_DIR" "$CLAUDE" "$ROLE"
    echo "  $ROLE ready"
done

sleep 3
echo "All 3 specialists running"
echo ""

# Verify all specialists are alive
echo "--- Health check ---"
for ROLE in reviewer tester documenter; do
    if TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" ping "$ROLE" 2>/dev/null; then
        echo "  $ROLE: alive"
    else
        echo "  $ROLE: DEAD"; exit 1
    fi
done
echo ""

# Fan out: delegate tasks to all 3 simultaneously
echo "--- Fan out: delegate 3 tasks ---"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit reviewer task.delegate \
    --payload "{\"request_id\":\"req-review\",\"action\":\"Review this file for code quality issues: error handling, potential panics, unsafe patterns, and suggest improvements. Be specific with line references.\",\"scope\":{\"file\":\"$REVIEW_FILE\",\"result_path\":\"$REVIEW_RESULT\"}}"
echo "  reviewer: task delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit tester task.delegate \
    --payload "{\"request_id\":\"req-test\",\"action\":\"Analyze this CLI file and identify the top 5 commands that lack test coverage or have weak tests. For each, suggest a specific test case.\",\"scope\":{\"file\":\"$TEST_FILE\",\"result_path\":\"$TEST_RESULT\"}}"
echo "  tester:   task delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit documenter task.delegate \
    --payload "{\"request_id\":\"req-doc\",\"action\":\"Write concise module-level documentation: purpose, key structs/enums, public API, and usage example.\",\"scope\":{\"file\":\"$DOC_FILE\",\"result_path\":\"$DOC_RESULT\"}}"
echo "  documenter: task delegated"

echo ""

# Fan in: wait for all 3 completions
echo "--- Fan in: wait for completions ---"

# Wait for all 3 in a single polling loop (they're running in parallel)
TIMEOUT=300
ELAPSED=0
GOT_REVIEW=false GOT_TEST=false GOT_DOC=false

while [ $ELAPSED -lt $TIMEOUT ]; do
    EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator --topic task.completed 2>/dev/null || true)

    if [ "$GOT_REVIEW" = false ] && echo "$EVENTS" | grep -q "req-review"; then
        GOT_REVIEW=true; echo "  reviewer completed (${ELAPSED}s)"
    fi
    if [ "$GOT_TEST" = false ] && echo "$EVENTS" | grep -q "req-test"; then
        GOT_TEST=true; echo "  tester completed (${ELAPSED}s)"
    fi
    if [ "$GOT_DOC" = false ] && echo "$EVENTS" | grep -q "req-doc"; then
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
for ROLE in reviewer tester documenter; do
    touch "$RUNTIME_DIR/shutdown"
done

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

# Show all events
echo "--- Orchestrator event bus ---"
EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator 2>/dev/null || true)
echo "$EVENTS"
echo ""

ACCEPTED=$(echo "$EVENTS" | grep -c "task.accepted" || true)
COMPLETED=$(echo "$EVENTS" | grep -c "task.completed" || true)

echo "Events: $ACCEPTED accepted, $COMPLETED completed"
echo "Results: $RESULTS_OK/$RESULTS_TOTAL files written"
echo ""

if [ "$RESULTS_OK" -eq 3 ] && [ "$COMPLETED" -ge 3 ]; then
    echo "============================================="
    echo "  LEVEL 4 PASSED — Multi-Specialist"
    echo "============================================="
    echo ""
    echo "  3 Claude Code specialists ran in parallel:"
    echo "    - reviewer:   code quality analysis"
    echo "    - tester:     test gap identification"
    echo "    - documenter: module documentation"
    echo "  Each with its own terminal and context window"
    echo "  All communicated via TermLink event bus"
    echo "  Results written to disk, events correlated"
elif [ "$RESULTS_OK" -gt 0 ]; then
    echo "=== LEVEL 4 PARTIAL ==="
    echo "$RESULTS_OK/3 specialists completed"
else
    echo "=== LEVEL 4 FAILED ==="
fi
