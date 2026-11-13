#!/usr/bin/env bash
# render-formula.sh — render the Homebrew formula for one release (T-3754).
#
# Usage: homebrew/render-formula.sh <version> <checksums.txt> [template.rb]
#
#   <version>        X.Y.Z or vX.Y.Z (the release tag); anything else is refused.
#   <checksums.txt>  the release's `sha256sum *` output: "<64-hex>  <artifact>" per line.
#   [template.rb]    default: homebrew/Formula/termlink.rb next to this script.
#
# Writes the rendered formula to stdout. The template's `version "..."` line gets
# <version>. Every `sha256 "..."` line gets the hash of the artifact named in the
# nearest `url "...releases/download/.../<artifact>"` line above it.
#
# Fail-closed: a non-semver version, an artifact missing from checksums.txt, a hash
# that is not 64 lowercase hex, a sha256 line with no url above it, or a template
# with no url/sha256 pairs → exit 1 with a reason on stderr and NOTHING on stdout.
# Unreadable inputs or bad usage → exit 2. A half-rendered formula is never printed.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
    echo "usage: $0 <version> <checksums.txt> [template.rb]" >&2
    exit 2
fi

version="${1#v}"
checksums="$2"
template="${3:-$here/Formula/termlink.rb}"

if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "render-formula: version '$1' is not X.Y.Z or vX.Y.Z" >&2
    exit 1
fi
for f in "$checksums" "$template"; do
    if [ ! -r "$f" ] || [ ! -f "$f" ]; then
        echo "render-formula: cannot read $f" >&2
        exit 2
    fi
done
# An empty checksums file would make awk read the template as pass 1 (FNR == NR).
if ! grep -q '[^[:space:]]' "$checksums"; then
    echo "render-formula: checksums file $checksums is empty" >&2
    exit 1
fi

awk -v version="$version" '
    # Pass 1: checksums.txt. sha256sum may prefix the name with "*" (binary mode).
    FNR == NR {
        if (NF == 0) next
        name = $2; sub(/^\*/, "", name)
        sums[name] = $1
        next
    }
    # Pass 2: the template.
    {
        line = $0
        if (line ~ /^[ \t]*version "[^"]*"/) {
            indent = line; sub(/[^ \t].*$/, "", indent)
            line = indent "version \"" version "\""
            versions++
        } else if (line ~ /^[ \t]*url "[^"]*releases\/download\/[^"]*"/) {
            u = line; sub(/^[ \t]*url "/, "", u); sub(/".*$/, "", u)
            n = split(u, parts, "/"); artifact = parts[n]
            urls++
        } else if (line ~ /^[ \t]*sha256 "/) {
            indent = line; sub(/[^ \t].*$/, "", indent)
            if (artifact == "") { err = "sha256 at template line " FNR " has no url line above it"; exit 1 }
            if (!(artifact in sums)) { err = "artifact " artifact " is not in the checksums file"; exit 1 }
            h = sums[artifact]
            if (h !~ /^[0-9a-f]+$/ || length(h) != 64) { err = "hash for " artifact " is not 64 lowercase hex: " h; exit 1 }
            line = indent "sha256 \"" h "\""
            artifact = ""
            shas++
        }
        out[++lines] = line
    }
    END {
        if (err == "" && versions != 1) err = "template has " versions + 0 " version lines, expected 1"
        if (err == "" && (urls == 0 || urls != shas)) err = "template has " urls + 0 " url lines and " shas + 0 " sha256 lines; expected equal and > 0"
        if (err != "") { print "render-formula: " err > "/dev/stderr"; exit 1 }
        for (i = 1; i <= lines; i++) print out[i]
    }
' "$checksums" "$template"
