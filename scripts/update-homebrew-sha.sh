#!/usr/bin/env bash
# Update SHA256 hashes in Homebrew formula from GitHub release checksums
# Usage: ./scripts/update-homebrew-sha.sh v0.1.0

set -euo pipefail

VERSION="${1:?Usage: $0 <version-tag>  (e.g., v0.1.0)}"
FORMULA="homebrew/Formula/termlink.rb"
REPO="DimitriGeelen/termlink"
BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"

echo "Fetching checksums for ${VERSION}..."

# Download checksums file from release
CHECKSUMS=$(curl -sfL "${BASE_URL}/checksums.txt") || {
    echo "ERROR: Could not download checksums from ${BASE_URL}/checksums.txt"
    echo "Has the release been published?"
    exit 1
}

echo "$CHECKSUMS"
echo ""

# Extract SHA256 for each platform. Linux x86_64 uses the musl static variant
# (T-1135) so brew install works on LXC containers that lack glibc.
SHA_DARWIN_ARM=$(echo "$CHECKSUMS" | grep " termlink-darwin-aarch64$" | awk '{print $1}')
SHA_DARWIN_X86=$(echo "$CHECKSUMS" | grep " termlink-darwin-x86_64$" | awk '{print $1}')
SHA_LINUX_X86=$(echo "$CHECKSUMS" | grep " termlink-linux-x86_64-static$" | awk '{print $1}')
SHA_LINUX_ARM=$(echo "$CHECKSUMS" | grep " termlink-linux-aarch64$" | awk '{print $1}')

MISSING=0
for var in "darwin-aarch64:$SHA_DARWIN_ARM" "darwin-x86_64:$SHA_DARWIN_X86" \
           "linux-x86_64-static:$SHA_LINUX_X86" "linux-aarch64:$SHA_LINUX_ARM"; do
    name="${var%%:*}"
    hash="${var#*:}"
    if [ -z "$hash" ]; then
        echo "  MISSING: $name"
        MISSING=1
    fi
done
if [ "$MISSING" -eq 1 ]; then
    echo "ERROR: Could not extract all SHA256 hashes from checksums"
    exit 1
fi

# Update formula — match each platform's url line and update the sha256 on the following line
# The formula structure uses url/sha256 pairs within on_macos/on_linux blocks.
# Note: the linux x86_64 url is the -static variant, so the match is unique.
sed -i.bak \
    -e "s/version \".*\"/version \"${VERSION#v}\"/" \
    -e "/termlink-darwin-aarch64/{n;s/sha256 \".*\"/sha256 \"${SHA_DARWIN_ARM}\"/;}" \
    -e "/termlink-darwin-x86_64/{n;s/sha256 \".*\"/sha256 \"${SHA_DARWIN_X86}\"/;}" \
    -e "/termlink-linux-aarch64/{n;s/sha256 \".*\"/sha256 \"${SHA_LINUX_ARM}\"/;}" \
    -e "/termlink-linux-x86_64-static/{n;s/sha256 \".*\"/sha256 \"${SHA_LINUX_X86}\"/;}" \
    "$FORMULA"

rm -f "${FORMULA}.bak"

echo "Updated ${FORMULA}:"
echo "  version:              ${VERSION#v}"
echo "  darwin-aarch64:       ${SHA_DARWIN_ARM}"
echo "  darwin-x86_64:        ${SHA_DARWIN_X86}"
echo "  linux-x86_64-static:  ${SHA_LINUX_X86}"
echo "  linux-aarch64:        ${SHA_LINUX_ARM}"
echo ""
echo "Next: review the formula, commit, and push to the tap repo"
