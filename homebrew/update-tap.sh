#!/usr/bin/env bash
# update-tap.sh — publish the rendered Homebrew formula to the tap, then prove it (T-3754).
#
# Called by the release workflow's homebrew-tap job after the GitHub release exists.
# Environment:
#   TAG             release tag (vX.Y.Z). Any other shape (pre-release) is skipped, exit 0.
#   CHECKSUMS       path to the release's checksums.txt.
#   TAP_TOKEN       token that can push to the tap (repo secret HOMEBREW_TAP_TOKEN).
#   TAP_REPO        owner/name of the tap (default DimitriGeelen/homebrew-termlink).
#   TAP_PUSH_URL    override of the authenticated push URL (tests use a local bare repo).
#   TAP_READ_URL    override of the UNAUTHENTICATED read-back URL; default is the public
#                   https URL, which is what `brew tap` itself clones.
#   WORKDIR         scratch dir (default: mktemp).
#
# Steps: render (fail-closed) → clone tap → refuse to downgrade a newer tap → commit
# Formula/termlink.rb only if it changed → push → clone the tap again WITHOUT the token
# and require its formula to equal what was rendered. The last line on success is
#   tap formula version == vX.Y.Z
# Exit 0 published and proven (or nothing to do), 1 not published / not provable
# (each with a ::error:: line naming the cause), 2 tooling (bad inputs, render error).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TAP_REPO="${TAP_REPO:-DimitriGeelen/homebrew-termlink}"
export GIT_TERMINAL_PROMPT=0

err() { echo "::error::homebrew-tap: $*" >&2; }

tag="${TAG:-}"
if ! [[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "::notice::homebrew-tap: '$tag' is not vX.Y.Z; the tap is only updated for plain releases"
    exit 0
fi
version="${tag#v}"

if [ -z "${TAP_TOKEN:-}" ] && [ -z "${TAP_PUSH_URL:-}" ]; then
    err "repository secret HOMEBREW_TAP_TOKEN is not set, so the tap cannot be updated. Add a token with Contents: read and write on $TAP_REPO (T-3754)."
    exit 1
fi
if [ -z "${CHECKSUMS:-}" ] || [ ! -r "$CHECKSUMS" ]; then
    err "checksums file '${CHECKSUMS:-}' is not readable"
    exit 2
fi

push_url="${TAP_PUSH_URL:-https://x-access-token:${TAP_TOKEN:-}@github.com/${TAP_REPO}.git}"
read_url="${TAP_READ_URL:-https://github.com/${TAP_REPO}.git}"
work="${WORKDIR:-$(mktemp -d)}"
mkdir -p "$work"

if ! bash "$here/render-formula.sh" "$tag" "$CHECKSUMS" > "$work/termlink.rb"; then
    err "render-formula.sh refused the release inputs (see the line above)"
    exit 2
fi

rm -rf "$work/tap"
if ! git clone --quiet "$push_url" "$work/tap" 2> "$work/clone.err"; then
    # Never echo the URL: it carries the token.
    err "cannot clone the tap $TAP_REPO with HOMEBREW_TAP_TOKEN. Does the repo exist, and can the token read and write it?"
    exit 1
fi
cd "$work/tap"
branch="$(git symbolic-ref --short HEAD 2>/dev/null || echo main)"

current=""
if [ -f Formula/termlink.rb ]; then
    current="$(sed -n 's/^  version "\(.*\)"$/\1/p' Formula/termlink.rb)"
fi
if [ -n "$current" ] && [ "$current" != "$version" ] \
   && [ "$(printf '%s\n%s\n' "$current" "$version" | sort -V | tail -1)" = "$current" ]; then
    echo "::notice::homebrew-tap: the tap already has $current, newer than $version; not downgrading"
    exit 0
fi

mkdir -p Formula
cp "$work/termlink.rb" Formula/termlink.rb
git add Formula/termlink.rb
if git diff --cached --quiet; then
    echo "homebrew-tap: the tap already has this formula for $tag; nothing to push"
else
    git -c user.name="termlink release" -c user.email="41898282+github-actions[bot]@users.noreply.github.com" \
        commit --quiet -m "termlink $version (release $tag, rendered by T-3754 homebrew-tap job)"
    if ! git push --quiet origin "HEAD:refs/heads/$branch" 2> "$work/push.err"; then
        err "push to $TAP_REPO branch $branch failed. Does HOMEBREW_TAP_TOKEN have Contents: write on it?"
        exit 1
    fi
    echo "homebrew-tap: pushed Formula/termlink.rb $version to $TAP_REPO ($branch)"
fi

# Read-back, without the token: this is what `brew tap` will see.
cd "$work"
rm -rf readback
if ! git clone --quiet --depth 1 --branch "$branch" "$read_url" readback 2> "$work/read.err"; then
    err "cannot read $TAP_REPO without credentials. brew tap needs the tap repo to be PUBLIC."
    exit 1
fi
got="$(sed -n 's/^  version "\(.*\)"$/\1/p' readback/Formula/termlink.rb 2>/dev/null || true)"
if [ "$got" != "$version" ]; then
    err "tap formula version is '${got:-missing}', expected $version for $tag"
    exit 1
fi
if ! cmp -s "$work/termlink.rb" readback/Formula/termlink.rb; then
    err "tap formula has version $version but differs from the rendered one (hashes?)"
    exit 1
fi
echo "tap formula version == $tag"
