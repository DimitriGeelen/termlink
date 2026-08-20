#!/usr/bin/env bash
# For each conflicted path, report how the two sides actually differ, so the
# resolution is a reading rather than a guess.
set -uo pipefail
T=/root/.claude/jobs/d638a35c/tmp/merge-trial
cd "$T" || exit 2

while IFS= read -r p; do
    [ -n "$p" ] || continue
    ours=$(git show ":2:$p" 2>/dev/null | wc -l)
    theirs=$(git show ":3:$p" 2>/dev/null | wc -l)
    # Identical content on both sides? (add/add of the same bytes)
    oh=$(git show ":2:$p" 2>/dev/null | sha256sum | cut -c1-12)
    th=$(git show ":3:$p" 2>/dev/null | sha256sum | cut -c1-12)
    same="differ"
    [ "$oh" = "$th" ] && same="IDENTICAL"
    printf '%-62s ours=%-6s main=%-6s %s\n' "${p:0:62}" "$ours" "$theirs" "$same"
done < /root/.claude/jobs/d638a35c/tmp/conflicted-paths.txt
