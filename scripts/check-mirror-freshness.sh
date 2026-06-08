#!/usr/bin/env bash
# T-1140 / T-1696 — Detect OneDev → GitHub mirror drift (G-007 / G-058 mitigation).
#
# The CI flow depends on OneDev auto-mirroring main + tags to GitHub via
# its PushRepository buildspec job. When the mirror stalls, recent commits
# on OneDev never reach GitHub, and Actions (release, install-check) silently
# stop firing. This script makes that drift visible by comparing the two
# HEADs side-by-side AND by checking the most-recent local tag exists on
# GitHub (the specific failure mode G-058 exposed — tag-mirror can fail
# independently of branch-mirror).
#
# Exit codes:
#   0  — synced (or GitHub ahead, which should not happen but is not an incident)
#   1  — drift detected (OneDev ahead of GitHub on HEAD or tag)
#   2  — network/tooling error (could not read one of the refs)
#
# Usage:
#   check-mirror-freshness.sh          # human-readable, one-shot
#   check-mirror-freshness.sh --json   # JSON for scripting
#   check-mirror-freshness.sh --quiet  # only print on drift (cron-friendly)

set -eu

FORMAT=human
QUIET=0
HEARTBEAT=1
GITHUB_URL="https://github.com/DimitriGeelen/termlink.git"

for arg in "$@"; do
    case "$arg" in
        --json)  FORMAT=json ;;
        --quiet) QUIET=1 ;;
        --no-heartbeat) HEARTBEAT=0 ;;
        -h|--help)
            sed -n '2,20p' "$0"
            exit 0
            ;;
        *) echo "unknown arg: $arg" >&2; exit 2 ;;
    esac
done

# T-1723 heartbeat: prove this canary ran, even on synced/error cycles.
# scripts/check-canary-aliveness.sh stats this file's mtime; if stale,
# the canary itself is broken (cron didn't load, script crashed, etc.).
# Placed BEFORE the network calls so a network error still leaves a heartbeat.
# --no-heartbeat suppresses the touch so the meta-canary can probe drift
# without side-effecting the very signal it's checking.
HEARTBEAT_FILE="${HEARTBEAT_FILE:-.context/working/.release-mirror-canary.heartbeat}"
if [ "$HEARTBEAT" = 1 ]; then
    mkdir -p "$(dirname "$HEARTBEAT_FILE")" 2>/dev/null || true
    touch -- "$HEARTBEAT_FILE" 2>/dev/null || true
fi

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

# T-1696: tag drift — most-recent local tag must exist on GitHub.
# Mirror jobs can lose tags independently of branches; G-058's release
# tags (v0.10.0, v0.11.0, v0.11.1) all missed GitHub for 16 days while
# main was the only thing being checked.
latest_tag=$(git describe --tags --abbrev=0 2>/dev/null || true)
tag_status=skipped
if [ -n "$latest_tag" ]; then
    if git ls-remote --tags "$GITHUB_URL" "refs/tags/$latest_tag" 2>/dev/null \
        | grep -q "refs/tags/$latest_tag$"; then
        tag_status=synced
    else
        tag_status=missing
        if [ "$status" = synced ]; then
            status=drift
        fi
    fi
fi

# T-2052: structural drift diagnosis — scan the github_head..origin_head range
# for blobs over GitHub's 100MB pre-receive limit (GH001). When this fires,
# the mirror job's silent rejection is the explained root cause: the operator
# should clean up the offending blob from history rather than rotate the PAT.
# Falls back gracefully if range is unresolvable (diverged or shallow clone).
oversize_hint=""
if [ "$status" = drift ] && [ -n "$github_head" ] && [ -n "$origin_head" ]; then
    if git merge-base --is-ancestor "$github_head" "$origin_head" 2>/dev/null; then
        # GitHub's per-file limit is 100 MiB (104857600 bytes). Use 100 MB
        # (decimal) as a slightly conservative trigger: any blob ≥100 MB will
        # also fail the 100 MiB hard limit, and operators reading the message
        # think in "100MB" terms.
        BIG_BYTES=100000000
        oversize_hint=$(
            git rev-list "$github_head..$origin_head" 2>/dev/null \
                | while read -r sha; do
                    git diff-tree --no-commit-id --name-only -r "$sha" 2>/dev/null \
                        | while read -r path; do
                            [ -z "$path" ] && continue
                            sz=$(git cat-file -s "$sha:$path" 2>/dev/null || echo 0)
                            if [ "$sz" -ge "$BIG_BYTES" ] 2>/dev/null; then
                                printf '%s %s %s\n' "$sha" "$sz" "$path"
                                break
                            fi
                        done
                done | head -n 5
        )
    fi
fi

if [ "$FORMAT" = json ]; then
    if [ -n "$oversize_hint" ]; then
        # Best-effort JSON: just count hits + the first one (avoid embedded newlines).
        oversize_count=$(echo "$oversize_hint" | wc -l | tr -d ' ')
        first_oversize=$(echo "$oversize_hint" | head -n 1)
        printf '{"status":"%s","behind":"%s","origin":"%s","github":"%s","latest_tag":"%s","tag_status":"%s","oversize_blobs":%s,"oversize_first":"%s"}\n' \
            "$status" "$behind" "$origin_head" "$github_head" "$latest_tag" "$tag_status" \
            "$oversize_count" "$first_oversize"
    else
        printf '{"status":"%s","behind":"%s","origin":"%s","github":"%s","latest_tag":"%s","tag_status":"%s","oversize_blobs":0}\n' \
            "$status" "$behind" "$origin_head" "$github_head" "$latest_tag" "$tag_status"
    fi
elif [ "$QUIET" = 1 ] && [ "$status" = synced ]; then
    :
else
    echo "GitHub mirror: $status"
    echo "  origin (OneDev): $origin_head"
    echo "  GitHub:          $github_head"
    if [ "$status" = drift ]; then
        if [ "$behind" != "0" ] && [ "$behind" != "?" ]; then
            echo "  GitHub is $behind commit(s) behind origin"
        fi
        if [ "$tag_status" = missing ]; then
            echo "  Latest tag $latest_tag is NOT on GitHub (tag mirror broken)"
        fi
        if [ -n "$oversize_hint" ]; then
            echo ""
            echo "  Likely cause — oversize blob(s) over GitHub's 100MB pre-receive limit:"
            echo "$oversize_hint" | while read -r sha sz path; do
                mb=$(awk -v b="$sz" 'BEGIN{printf "%.1f", b/1048576}')
                printf '    %s  %s  %s MiB\n' "$sha" "$path" "$mb"
            done
            echo ""
            echo "  Diagnose history: bash .agentic-framework/agents/git/lib/large-file-scan.sh scan-tree"
            echo "  Clean recipe: git rm --cached <path> and BFG / git-filter-repo to scrub history"
            echo "  Origin: T-2052 (G-058 root-cause class — silent mirror rejection on file size)"
        fi
    elif [ "$status" = diverged ]; then
        echo "  GitHub and origin have diverged — manual investigation needed"
    fi
fi

case "$status" in
    synced)   exit 0 ;;
    drift|diverged) exit 1 ;;
esac
