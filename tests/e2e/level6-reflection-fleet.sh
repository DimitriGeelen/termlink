#!/usr/bin/env bash
# =============================================================================
# Level 6: Reflection Fleet — 10 agents review the project
# =============================================================================
# 10 specialists each analyze a different aspect of the TermLink project:
#   1. architecture    — crate structure, dependency graph, modularity
#   2. protocol        — protocol design, wire format, extensibility
#   3. session         — session management, lifecycle, liveness
#   4. cli-ux          — CLI ergonomics, help text, error messages
#   5. test-coverage   — test suite quality, coverage gaps
#   6. e2e-suite       — e2e test design, reliability, patterns
#   7. event-schema    — delegation event convention, schema quality
#   8. watcher-pattern — specialist-watcher reliability, scalability
#   9. security        — security posture, trust boundaries, input validation
#  10. enhancement     — top enhancement opportunities, roadmap suggestions
#
# Usage: ./tests/e2e/level6-reflection-fleet.sh
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TERMLINK="$PROJECT_ROOT/target/debug/termlink"
CLAUDE="/Users/dimidev32/.local/bin/claude"
WATCHER="$SCRIPT_DIR/specialist-watcher.sh"
RUNTIME_DIR=$(mktemp -d)

# 10 agent names
AGENTS=(arch proto session cli-ux test-cov e2e-suite event-schema watcher-pat security enhance)

source "$SCRIPT_DIR/e2e-helpers.sh"
trap cleanup_all EXIT

echo "============================================="
echo "  Level 6: Reflection Fleet (10 agents)"
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

# Spawn 10 specialists
echo "--- Spawn 10 specialists ---"
for AGENT in "${AGENTS[@]}"; do
    echo "  Spawning $AGENT..."
    spawn_tracked \
        --name "$AGENT" --roles analyst \
        --wait --wait-timeout 15 \
        -- bash "$WATCHER" "$TERMLINK" "$RUNTIME_DIR" "$CLAUDE" "$AGENT"
    echo "  $AGENT ready"
done

sleep 3
echo "All 10 specialists running"
echo ""

# Health check
echo "--- Health check ---"
ALIVE=0
for AGENT in "${AGENTS[@]}"; do
    if TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" ping "$AGENT" 2>/dev/null; then
        echo "  $AGENT: alive"
        ALIVE=$((ALIVE + 1))
    else
        echo "  $AGENT: DEAD"
    fi
done
echo "  $ALIVE/10 alive"
echo ""

# Fan out: delegate reflection tasks
echo "--- Fan out: 10 reflection tasks ---"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit arch task.delegate \
    --payload "{\"request_id\":\"req-arch\",\"action\":\"Analyze the overall crate architecture of this Rust workspace. Read Cargo.toml files in the root and each crate. Assess: modularity, dependency direction, separation of concerns, and whether the crate boundaries are well-drawn. Note successes and areas for improvement.\",\"scope\":{\"file\":\"$PROJECT_ROOT/Cargo.toml\",\"result_path\":\"$RUNTIME_DIR/result-arch.md\"}}"
echo "  1/10 arch delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit proto task.delegate \
    --payload "{\"request_id\":\"req-proto\",\"action\":\"Review the protocol design. Read the control plane (lib.rs, rpc.rs) and data plane (data.rs) files. Assess: wire format quality, extensibility, versioning strategy, error handling, and whether the control/data plane split is sound.\",\"scope\":{\"file\":\"$PROJECT_ROOT/crates/termlink-protocol/src/lib.rs\",\"result_path\":\"$RUNTIME_DIR/result-proto.md\"}}"
echo "  2/10 proto delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit session task.delegate \
    --payload "{\"request_id\":\"req-session\",\"action\":\"Review session management. Read manager.rs and context.rs. Assess: session lifecycle (register/deregister), liveness checking, state machine correctness, cleanup on failure, and race conditions.\",\"scope\":{\"file\":\"$PROJECT_ROOT/crates/termlink-session/src/manager.rs\",\"result_path\":\"$RUNTIME_DIR/result-session.md\"}}"
echo "  3/10 session delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit cli-ux task.delegate \
    --payload "{\"request_id\":\"req-cli\",\"action\":\"Review CLI user experience. Read main.rs. Assess: command naming, help text quality, error messages, discoverability, consistency across subcommands, and whether the 26-command surface area is manageable.\",\"scope\":{\"file\":\"$PROJECT_ROOT/crates/termlink-cli/src/main.rs\",\"result_path\":\"$RUNTIME_DIR/result-cli.md\"}}"
echo "  4/10 cli-ux delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit test-cov task.delegate \
    --payload "{\"request_id\":\"req-testcov\",\"action\":\"Analyze test suite quality. Read the test files in each crate. Assess: unit test coverage, integration test presence, test isolation, flaky test risks, missing edge cases, and overall test strategy.\",\"scope\":{\"file\":\"$PROJECT_ROOT/crates/termlink-session/src/manager.rs\",\"result_path\":\"$RUNTIME_DIR/result-testcov.md\"}}"
echo "  5/10 test-cov delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit e2e-suite task.delegate \
    --payload "{\"request_id\":\"req-e2e\",\"action\":\"Review the e2e test suite design. Read all level*.sh files and specialist-watcher.sh and role-watcher.sh. Assess: progressive difficulty design, reliability, timeout handling, cleanup, reusability of patterns, and what levels are missing.\",\"scope\":{\"file\":\"$PROJECT_ROOT/tests/e2e/level1-echo.sh\",\"result_path\":\"$RUNTIME_DIR/result-e2e.md\"}}"
echo "  6/10 e2e-suite delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit event-schema task.delegate \
    --payload "{\"request_id\":\"req-evschema\",\"action\":\"Review the event delegation schema convention. Read the convention doc. Assess: schema completeness, field naming, extensibility, whether it covers failure cases well, and compare to industry patterns (CloudEvents, CNCF).\",\"scope\":{\"file\":\"$PROJECT_ROOT/docs/conventions/agent-delegation-events.md\",\"result_path\":\"$RUNTIME_DIR/result-evschema.md\"}}"
echo "  7/10 event-schema delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit watcher-pat task.delegate \
    --payload "{\"request_id\":\"req-watcher\",\"action\":\"Review the specialist watcher pattern. Read specialist-watcher.sh and role-watcher.sh. Assess: reliability (what if claude crashes?), scalability, error recovery, cursor management, shutdown handling, and resource cleanup.\",\"scope\":{\"file\":\"$PROJECT_ROOT/tests/e2e/specialist-watcher.sh\",\"result_path\":\"$RUNTIME_DIR/result-watcher.md\"}}"
echo "  8/10 watcher-pat delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit security task.delegate \
    --payload "{\"request_id\":\"req-security\",\"action\":\"Security review of the TermLink system. Read lib.rs and manager.rs. Assess: trust boundaries (who can emit events to whom?), input validation on RPC payloads, socket permissions, command injection risks in spawn, and whether --dangerously-skip-permissions in e2e tests is acceptable.\",\"scope\":{\"file\":\"$PROJECT_ROOT/crates/termlink-protocol/src/lib.rs\",\"result_path\":\"$RUNTIME_DIR/result-security.md\"}}"
echo "  9/10 security delegated"

TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" emit enhance task.delegate \
    --payload "{\"request_id\":\"req-enhance\",\"action\":\"Read the project Cargo.toml and the main CLI (main.rs). Propose the top 5 enhancement opportunities for TermLink. Consider: what would make this production-ready? What features are missing for real multi-agent orchestration? What are the biggest pitfalls to avoid? Be specific and actionable.\",\"scope\":{\"file\":\"$PROJECT_ROOT/Cargo.toml\",\"result_path\":\"$RUNTIME_DIR/result-enhance.md\"}}"
echo "  10/10 enhance delegated"

echo ""

# Fan in: wait for all 10 completions
echo "--- Fan in: wait for completions ---"

TIMEOUT=600
ELAPSED=0
declare -A GOT
for AGENT in arch proto session cli testcov e2e evschema watcher security enhance; do
    GOT[$AGENT]=false
done

REQ_IDS="req-arch req-proto req-session req-cli req-testcov req-e2e req-evschema req-watcher req-security req-enhance"
REQ_NAMES=(arch proto session cli testcov e2e evschema watcher security enhance)

COMPLETED_COUNT=0

while [ $ELAPSED -lt $TIMEOUT ]; do
    EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator --topic task.completed 2>/dev/null || true)

    IDX=0
    for REQ_ID in $REQ_IDS; do
        NAME="${REQ_NAMES[$IDX]}"
        if [ "${GOT[$NAME]}" = false ] && echo "$EVENTS" | grep -q "$REQ_ID"; then
            GOT[$NAME]=true
            COMPLETED_COUNT=$((COMPLETED_COUNT + 1))
            echo "  $NAME completed (${ELAPSED}s) [$COMPLETED_COUNT/10]"
        fi
        IDX=$((IDX + 1))
    done

    if [ "$COMPLETED_COUNT" -ge 10 ]; then
        echo "  All 10 complete!"
        break
    fi

    sleep 5; ELAPSED=$((ELAPSED + 5))
    if [ $((ELAPSED % 30)) -eq 0 ]; then
        echo "  Waiting... ${ELAPSED}s ($COMPLETED_COUNT/10 done)"
    fi
done

echo ""

# Shutdown
touch "$RUNTIME_DIR/shutdown"

# Collect results
echo "============================================="
echo "  RESULTS"
echo "============================================="
echo ""

RESULTS_OK=0
RESULT_FILES=(result-arch result-proto result-session result-cli result-testcov result-e2e result-evschema result-watcher result-security result-enhance)
RESULT_LABELS=("Architecture" "Protocol" "Session" "CLI UX" "Test Coverage" "E2E Suite" "Event Schema" "Watcher Pattern" "Security" "Enhancements")

for i in $(seq 0 9); do
    RFILE="${RESULT_FILES[$i]}"
    RLABEL="${RESULT_LABELS[$i]}"
    RPATH="$RUNTIME_DIR/${RFILE}.md"
    if [ -f "$RPATH" ]; then
        echo "=== $RLABEL ==="
        cat "$RPATH"
        echo ""
        echo "--- end $RLABEL ---"
        echo ""
        RESULTS_OK=$((RESULTS_OK + 1))
        # Copy to project for persistence
        cp "$RPATH" "$PROJECT_ROOT/docs/reports/reflection-${RFILE}.md" 2>/dev/null || true
    else
        echo "=== $RLABEL: MISSING ==="
        echo ""
    fi
done

# Event summary
echo "--- Event Summary ---"
EVENTS=$(TERMLINK_RUNTIME_DIR="$RUNTIME_DIR" "$TERMLINK" events orchestrator 2>/dev/null || true)
ACCEPTED=$(echo "$EVENTS" | grep -c "task.accepted" || true)
COMPLETED=$(echo "$EVENTS" | grep -c "task.completed" || true)
echo "Events: $ACCEPTED accepted, $COMPLETED completed"
echo "Results: $RESULTS_OK/10 files written"
echo ""

if [ "$RESULTS_OK" -ge 10 ]; then
    echo "============================================="
    echo "  LEVEL 6 PASSED — Reflection Fleet"
    echo "============================================="
    echo "  10 Claude Code agents ran in parallel"
    echo "  Each analyzed a different project aspect"
    echo "  Reports saved to docs/reports/reflection-*.md"
elif [ "$RESULTS_OK" -gt 0 ]; then
    echo "=== LEVEL 6 PARTIAL ==="
    echo "$RESULTS_OK/10 agents completed"
else
    echo "=== LEVEL 6 FAILED ==="
fi
