#!/usr/bin/env bash
# TermLink Agent Mesh — Dispatch a task to a worker agent
# Usage: dispatch.sh [--isolate] [--worker-name NAME] [--timeout SECS] "prompt text"
#
# Options:
#   --isolate       Create a git worktree per worker for filesystem isolation.
#                   Each worker gets its own branch (mesh-{worker-name}) and
#                   CARGO_TARGET_DIR. Worktree is cleaned up on exit.
#   --worker-name   Worker session name (default: mesh-worker-$$)
#   --timeout       Worker timeout in seconds (default: $TERMLINK_DISPATCH_TIMEOUT or 120)
#
# Flow:
#   1. Ensures hub is running
#   2. (If --isolate) Creates git worktree on a new branch
#   3. Spawns a worker agent (Claude Code via agent-wrapper.sh)
#   4. Worker executes, writes result to stdout
#   5. Orchestrator reads result
#   6. Cleanup (worktree removal if --isolate)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# --- Parse arguments ---
ISOLATE=false
WORKER_NAME="mesh-worker-$$"
TIMEOUT="${TERMLINK_DISPATCH_TIMEOUT:-120}"
CUSTOM_CMD=""
PROMPT=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --isolate)
            ISOLATE=true
            shift
            ;;
        --worker-name)
            WORKER_NAME="$2"
            shift 2
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        --command)
            CUSTOM_CMD="$2"
            shift 2
            ;;
        *)
            PROMPT="$1"
            shift
            ;;
    esac
done

if [ -z "$PROMPT" ] && [ -z "$CUSTOM_CMD" ]; then
    echo "Usage: dispatch.sh [--isolate] [--worker-name NAME] [--timeout SECS] [--command CMD] \"prompt text\"" >&2
    exit 1
fi

RESULT_FILE="/tmp/termlink-mesh-result-${WORKER_NAME}.txt"
WORKTREE_DIR=""
BRANCH_NAME="mesh-${WORKER_NAME}"
WORKDIR="$PROJECT_ROOT"

# --- Auto-commit worktree changes ---
auto_commit_worktree() {
    if [ ! -d "$WORKTREE_DIR" ]; then return; fi

    cd "$WORKTREE_DIR"
    if git diff --quiet && git diff --cached --quiet && [ -z "$(git ls-files --others --exclude-standard)" ]; then
        echo "No changes to commit in worktree" >&2
        return 1  # Signal: no commits, safe to remove
    fi

    echo "Auto-committing worker changes..." >&2
    git add -A
    git commit -m "mesh(${WORKER_NAME}): auto-commit worker changes" --no-gpg-sign 2>&1 >&2
    echo "Committed on branch: $BRANCH_NAME" >&2
    return 0  # Signal: commits exist, preserve branch
}

# --- Cleanup trap ---
cleanup() {
    rm -f "$RESULT_FILE"
    if [ "$ISOLATE" = true ] && [ -n "$WORKTREE_DIR" ] && [ -d "$WORKTREE_DIR" ]; then
        if auto_commit_worktree; then
            # Worker made changes — remove worktree but keep branch for merge
            echo "Preserving branch $BRANCH_NAME (has commits). Removing worktree only." >&2
            git -C "$PROJECT_ROOT" worktree remove --force "$WORKTREE_DIR" 2>/dev/null || true
        else
            # No changes — clean up completely
            echo "Cleaning up worktree: $WORKTREE_DIR" >&2
            git -C "$PROJECT_ROOT" worktree remove --force "$WORKTREE_DIR" 2>/dev/null || true
            git -C "$PROJECT_ROOT" branch -d "$BRANCH_NAME" 2>/dev/null || true
        fi
    fi
}
trap cleanup EXIT

# --- Step 1: Ensure hub is running ---
if ! termlink info 2>/dev/null | grep -q "Hub socket:"; then
    echo "ERROR: Cannot check hub status" >&2
    exit 1
fi

HUB_SOCK=$(termlink info 2>/dev/null | grep "Hub socket:" | awk '{print $3}')
if [ ! -S "$HUB_SOCK" ]; then
    echo "Starting hub..." >&2
    termlink hub &
    sleep 1
fi

# --- Step 2: Create worktree if --isolate ---
if [ "$ISOLATE" = true ]; then
    WORKTREE_DIR=$(mktemp -d /tmp/termlink-worktree-XXXXX)
    echo "Creating worktree: $WORKTREE_DIR (branch: $BRANCH_NAME)" >&2

    # Delete branch if it exists from a previous failed run
    git -C "$PROJECT_ROOT" branch -D "$BRANCH_NAME" 2>/dev/null || true

    git -C "$PROJECT_ROOT" worktree add -b "$BRANCH_NAME" "$WORKTREE_DIR" HEAD 2>&1 >&2
    WORKDIR="$WORKTREE_DIR"

    export CARGO_TARGET_DIR="${WORKTREE_DIR}/target"
    echo "CARGO_TARGET_DIR=$CARGO_TARGET_DIR" >&2
fi

# --- Step 3: Spawn worker agent ---
echo "Dispatching to worker: $WORKER_NAME" >&2
echo "Workdir: $WORKDIR" >&2
echo "Prompt: ${PROMPT:0:80}..." >&2

if [ -n "$CUSTOM_CMD" ]; then
    # Custom command mode: run user-provided command instead of Claude agent
    echo "Using custom command: $CUSTOM_CMD" >&2
    termlink run \
        -n "$WORKER_NAME" \
        -t "worker,agent-mesh" \
        --timeout "$TIMEOUT" \
        -- bash -c "cd '$WORKDIR' && $CUSTOM_CMD" > "$RESULT_FILE" 2>/dev/null &
    WORKER_PID=$!
else
    # Default: wrap prompt with standard mesh worker instructions
    source "$SCRIPT_DIR/prompt-template.sh"
    WRAPPED_PROMPT=$(cd "$WORKDIR" && wrap_prompt "$PROMPT" "$WORKER_NAME")

    termlink run \
        -n "$WORKER_NAME" \
        -t "worker,agent-mesh" \
        --timeout "$TIMEOUT" \
        -- "$SCRIPT_DIR/agent-wrapper.sh" "$WRAPPED_PROMPT" "$WORKDIR" > "$RESULT_FILE" 2>/dev/null &
    WORKER_PID=$!
fi

# --- Step 4: Wait for completion ---
echo "Worker PID: $WORKER_PID (timeout: ${TIMEOUT}s)" >&2

if wait $WORKER_PID 2>/dev/null; then
    echo "Worker completed successfully" >&2
else
    EXIT_CODE=$?
    echo "Worker failed (exit $EXIT_CODE)" >&2
    exit $EXIT_CODE
fi

# --- Step 5: Output result ---
if [ -f "$RESULT_FILE" ] && [ -s "$RESULT_FILE" ]; then
    cat "$RESULT_FILE"
else
    echo "ERROR: No result produced" >&2
    exit 1
fi

# --- Step 6: Report worktree branch if --isolate ---
if [ "$ISOLATE" = true ]; then
    echo "" >&2
    echo "=== Worktree Branch ===" >&2
    echo "Branch: $BRANCH_NAME" >&2
    if git -C "$PROJECT_ROOT" rev-parse --verify "$BRANCH_NAME" >/dev/null 2>&1; then
        COMMIT_COUNT=$(git -C "$PROJECT_ROOT" rev-list "main..$BRANCH_NAME" --count 2>/dev/null || echo "0")
        echo "Commits: $COMMIT_COUNT" >&2
        echo "To merge: git merge $BRANCH_NAME" >&2
        echo "To inspect: git log main..$BRANCH_NAME --oneline" >&2
    else
        echo "Branch cleaned up (no changes)" >&2
    fi
fi
