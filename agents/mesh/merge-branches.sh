#!/usr/bin/env bash
# TermLink Agent Mesh — Merge worktree branches onto main
# Usage: merge-branches.sh [--no-test] [--no-cleanup] branch1 branch2 ...
#
# Sequentially rebases and merges each branch onto main.
# Runs the test suite after each merge to catch breakage early.
# Stops on first conflict or test failure.
#
# Options:
#   --no-test     Skip test suite after each merge (faster, less safe)
#   --no-cleanup  Keep branches after merging (default: delete merged branches)
#
# Designed to run after dispatch.sh --isolate, which creates mesh-* branches.
# To find pending branches: git branch --list 'mesh-*'

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$PROJECT_ROOT"

# --- Parse arguments ---
RUN_TESTS=true
CLEANUP=true
BRANCHES=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-test)
            RUN_TESTS=false
            shift
            ;;
        --no-cleanup)
            CLEANUP=false
            shift
            ;;
        --auto)
            # Auto-discover mesh-* branches
            while IFS= read -r b; do
                BRANCHES+=("$b")
            done < <(git branch --list 'mesh-*' | sed 's/^[* ]*//')
            shift
            ;;
        -*)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
        *)
            BRANCHES+=("$1")
            shift
            ;;
    esac
done

if [ ${#BRANCHES[@]} -eq 0 ]; then
    echo "Usage: merge-branches.sh [--no-test] [--no-cleanup] [--auto] branch1 branch2 ..." >&2
    echo "" >&2
    echo "Pending mesh branches:" >&2
    git branch --list 'mesh-*' 2>/dev/null || echo "  (none)" >&2
    exit 1
fi

# Resolve cargo
if [ -n "${CARGO_BIN:-}" ]; then
    CARGO="$CARGO_BIN"
elif command -v cargo >/dev/null 2>&1; then
    CARGO="$(command -v cargo)"
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
    CARGO="$HOME/.cargo/bin/cargo"
else
    echo "ERROR: cargo not found" >&2
    exit 1
fi

# --- Verify we're on main ---
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "ERROR: Must be on main branch (currently on: $CURRENT_BRANCH)" >&2
    exit 1
fi

# --- Verify clean working tree ---
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "ERROR: Working tree has uncommitted changes. Commit or stash first." >&2
    exit 1
fi

# --- Verify all branches exist ---
for branch in "${BRANCHES[@]}"; do
    if ! git rev-parse --verify "$branch" >/dev/null 2>&1; then
        echo "ERROR: Branch '$branch' does not exist" >&2
        exit 1
    fi
done

echo "=== Merge Orchestration ===" >&2
echo "Branches to merge: ${BRANCHES[*]}" >&2
echo "Test after merge: $RUN_TESTS" >&2
echo "Cleanup branches: $CLEANUP" >&2
echo "" >&2

MERGED=0
FAILED_BRANCH=""

for branch in "${BRANCHES[@]}"; do
    echo "--- [$((MERGED + 1))/${#BRANCHES[@]}] Merging: $branch ---" >&2

    # Count commits on branch
    COMMIT_COUNT=$(git rev-list "main..$branch" --count 2>/dev/null || echo "0")
    echo "  Commits: $COMMIT_COUNT" >&2

    if [ "$COMMIT_COUNT" -eq 0 ]; then
        echo "  Skipping (no commits ahead of main)" >&2
        if [ "$CLEANUP" = true ]; then
            git branch -d "$branch" 2>/dev/null || true
        fi
        continue
    fi

    # Rebase onto main
    echo "  Rebasing onto main..." >&2
    if ! git rebase main "$branch" 2>&1 | tail -3 >&2; then
        echo "  CONFLICT during rebase of $branch" >&2
        echo "  Run: git rebase --abort  (to undo)" >&2
        echo "  Then resolve manually and re-run without this branch" >&2
        FAILED_BRANCH="$branch"
        git rebase --abort 2>/dev/null || true
        break
    fi

    # Switch back to main and merge (fast-forward since we just rebased)
    git checkout main 2>&1 >&2
    if ! git merge --ff-only "$branch" 2>&1 | tail -3 >&2; then
        echo "  ERROR: Fast-forward merge failed for $branch" >&2
        FAILED_BRANCH="$branch"
        break
    fi

    # Run tests
    if [ "$RUN_TESTS" = true ]; then
        echo "  Running test suite..." >&2
        if ! "$CARGO" test --workspace 2>&1 | grep -q "test result: ok"; then
            echo "  TEST FAILURE after merging $branch" >&2
            echo "  Revert with: git reset --hard HEAD~$COMMIT_COUNT" >&2
            FAILED_BRANCH="$branch"
            break
        fi
        echo "  Tests passed" >&2
    fi

    # Cleanup branch
    if [ "$CLEANUP" = true ]; then
        git branch -d "$branch" 2>/dev/null || true
        echo "  Branch deleted" >&2
    fi

    MERGED=$((MERGED + 1))
    echo "" >&2
done

# --- Summary ---
echo "=== Summary ===" >&2
echo "Merged: $MERGED / ${#BRANCHES[@]}" >&2

if [ -n "$FAILED_BRANCH" ]; then
    REMAINING=$((${#BRANCHES[@]} - MERGED))
    echo "Failed on: $FAILED_BRANCH ($REMAINING remaining)" >&2
    echo "Fix the conflict/failure, then re-run with remaining branches" >&2
    exit 1
fi

echo "All branches merged successfully" >&2
