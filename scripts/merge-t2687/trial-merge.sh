#!/usr/bin/env bash
# Trial-merge origin/main into a DETACHED scratch worktree and capture the
# conflict set. Never touches main, never touches the real branch.
set -uo pipefail
T=/root/.claude/jobs/d638a35c/tmp/merge-trial
cd "$T" || exit 2

git merge --no-commit --no-ff origin/main > /root/.claude/jobs/d638a35c/tmp/merge-out.txt 2>&1
rc=$?
echo "merge exit: $rc"
echo
echo "=== conflicted paths (git ls-files -u) ==="
git diff --name-only --diff-filter=U > /root/.claude/jobs/d638a35c/tmp/conflicted-paths.txt
n=$(grep -c . /root/.claude/jobs/d638a35c/tmp/conflicted-paths.txt || echo 0)
echo "count: $n"
cat /root/.claude/jobs/d638a35c/tmp/conflicted-paths.txt
