#!/usr/bin/env bash
# T-1140 — Detect OneDev → GitHub mirror drift (G-007 mitigation).
#
# The CI flow depends on OneDev auto-mirroring main + tags to GitHub via
# its PushRepository buildspec job. When the mirror stalls, recent commits
# on OneDev never reach GitHub, and Actions (release, install-check) silently
# stop firing. This script makes that drift visible by comparing the two
# HEADs side-by-side.
#
# Exit codes:
#   0  — synced (or GitHub ahead, which should not happen but is not an incident)
#   1  — drift detected (OneDev ahead of GitHub)
#   2  — network/tooling error (could not read one of the refs)
#
# Usage:
#   check-mirror-freshness.sh          # human-readable, one-shot
#   check-mirror-freshness.sh --json   # JSON for scripting
#   check-mirror-freshness.sh --quiet  # only print on drift (cron-friendly)

set -eu

FORMAT=human
QUIET=0
GITHUB_URL="https://github.com/DimitriGeelen/termlink.git"

for arg in "$@"; do
    case "$arg" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        -h|--help)
            sed -n '2,20p' "$0"
            exit 0
            ;;
        *) echo "unknown arg: $arg" >&2; exit 2 ;;
    esac
done

die() {
    if [ "$FORMAT" = json ]; then
        printf '{"status":"error","message":"%s"}\n' "$1"
    else
        echo "error: $1" >&2
    fi
    exit 2
}

origin_head=$(git ls-remote origin HEAD 2>/dev/null | awk '{print $1}' | head -n1) \
    || die "failed to read origin HEAD"
[ -n "$origin_head" ] || die "origin HEAD empty"

github_head=$(git ls-remote "$GITHUB_URL" HEAD 2>/dev/null | awk '{print $1}' | head -n1) \
    || die "failed to read GitHub HEAD"
[ -n "$github_head" ] || die "GitHub HEAD empty"

if [ "$origin_head" = "$github_head" ]; then
    status=synced
    behind=0
elif git merge-base --is-ancestor "$github_head" "$origin_head" 2>/dev/null; then
    status=drift
    behind=$(git rev-list --count "$github_head..$origin_head" 2>/dev/null || echo "?")
else
    status=diverged
    behind=unknown
fi

if [ "$FORMAT" = json ]; then
    printf '{"status":"%s","behind":"%s","origin":"%s","github":"%s"}\n' \
        "$status" "$behind" "$origin_head" "$github_head"
elif [ "$QUIET" = 1 ] && [ "$status" = synced ]; then
    :
else
    echo "GitHub mirror: $status"
    echo "  origin (OneDev): $origin_head"
    echo "  GitHub:          $github_head"
    if [ "$status" = drift ]; then
        echo "  GitHub is $behind commit(s) behind origin"
    elif [ "$status" = diverged ]; then
        echo "  GitHub and origin have diverged — manual investigation needed"
    fi
fi

case "$status" in
    synced)   exit 0 ;;
    drift|diverged) exit 1 ;;
esac
