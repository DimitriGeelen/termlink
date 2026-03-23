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

# Extract SHA256 for each platform
SHA_AARCH64=$(echo "$CHECKSUMS" | grep "termlink-darwin-aarch64" | awk '{print $1}')
SHA_X86_64=$(echo "$CHECKSUMS" | grep "termlink-darwin-x86_64" | awk '{print $1}')
SHA_LINUX=$(echo "$CHECKSUMS" | grep "termlink-linux-x86_64" | awk '{print $1}')

if [ -z "$SHA_AARCH64" ] || [ -z "$SHA_X86_64" ] || [ -z "$SHA_LINUX" ]; then
    echo "ERROR: Could not extract all SHA256 hashes from checksums"
    echo "  aarch64: ${SHA_AARCH64:-MISSING}"
    echo "  x86_64:  ${SHA_X86_64:-MISSING}"
    echo "  linux:   ${SHA_LINUX:-MISSING}"
    exit 1
fi

# Update formula
sed -i.bak \
    -e "s/version \".*\"/version \"${VERSION#v}\"/" \
    -e "/aarch64/,/sha256/{s/sha256 \".*\"/sha256 \"${SHA_AARCH64}\"/}" \
    -e "/x86_64.*darwin/,/sha256/{s/sha256 \".*\"/sha256 \"${SHA_X86_64}\"/}" \
    -e "/linux/,/sha256/{s/sha256 \".*\"/sha256 \"${SHA_LINUX}\"/}" \
    "$FORMULA"

rm -f "${FORMULA}.bak"

echo "Updated ${FORMULA}:"
echo "  version:  ${VERSION#v}"
echo "  aarch64:  ${SHA_AARCH64}"
echo "  x86_64:   ${SHA_X86_64}"
echo "  linux:    ${SHA_LINUX}"
echo ""
echo "Next: review the formula, commit, and push to the tap repo"
