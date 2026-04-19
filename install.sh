#!/bin/sh
# termlink installer — curl-pipe bootstrap
#
#   curl -fsSL https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh | sh
#
# Detects the host target, downloads the matching binary from GitHub Releases,
# verifies its sha256 against the release's checksums.txt, and installs it to
# /usr/local/bin (or $PREFIX/bin).

set -eu

REPO="DimitriGeelen/termlink"
VERSION="${TERMLINK_VERSION:-latest}"
PREFIX="${PREFIX:-/usr/local}"
DRY_RUN="${DRY_RUN:-0}"

for arg in "$@"; do
    case "$arg" in
        --dry-run)    DRY_RUN=1 ;;
        --version=*)  VERSION="${arg#*=}" ;;
        --prefix=*)   PREFIX="${arg#*=}" ;;
        -h|--help)
            cat <<'EOF'
termlink installer

Usage:
  curl -fsSL https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh | sh
  sh install.sh [--version=vX.Y.Z] [--prefix=DIR] [--dry-run]

Options:
  --version=VERSION   install a specific release tag (default: latest)
  --prefix=DIR        install prefix; binary lands at DIR/bin/termlink (default: /usr/local)
  --dry-run           show what would happen without downloading or writing
  -h, --help          this help

Environment:
  TERMLINK_VERSION    same as --version
  PREFIX              same as --prefix
  DRY_RUN=1           same as --dry-run

Supported targets:
  darwin-aarch64, darwin-x86_64,
  linux-x86_64 (glibc), linux-x86_64-static (musl), linux-aarch64
EOF
            exit 0
            ;;
        *) printf 'error: unknown argument: %s\n' "$arg" >&2; exit 2 ;;
    esac
done

log() { printf '  %s\n' "$*" >&2; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

os_raw="$(uname -s)"
arch_raw="$(uname -m)"

case "$os_raw" in
    Darwin) os="darwin" ;;
    Linux)  os="linux"  ;;
    *) die "unsupported OS: $os_raw (supported: Darwin, Linux)" ;;
esac

case "$arch_raw" in
    x86_64|amd64)  arch="x86_64"  ;;
    aarch64|arm64) arch="aarch64" ;;
    *) die "unsupported architecture: $arch_raw (supported: x86_64, aarch64)" ;;
esac

case "$os-$arch" in
    darwin-aarch64) artifact="termlink-darwin-aarch64" ;;
    darwin-x86_64)  artifact="termlink-darwin-x86_64"  ;;
    linux-aarch64)  artifact="termlink-linux-aarch64"  ;;
    linux-x86_64)
        # Prefer the musl static variant when glibc isn't obviously present
        # (LXC minimal images, Alpine, busybox). Static also works on glibc hosts.
        if [ -r /etc/alpine-release ] \
           || ! (ldd --version 2>&1 | grep -qi 'glibc\|gnu libc'); then
            artifact="termlink-linux-x86_64-static"
        else
            artifact="termlink-linux-x86_64"
        fi
        ;;
    *) die "no prebuilt binary for $os-$arch" ;;
esac

if [ "$VERSION" = "latest" ]; then
    base_url="https://github.com/${REPO}/releases/latest/download"
else
    base_url="https://github.com/${REPO}/releases/download/${VERSION}"
fi
art_url="${base_url}/${artifact}"
sum_url="${base_url}/checksums.txt"
dest="${PREFIX}/bin/termlink"

log "target:   $os-$arch → $artifact"
log "version:  $VERSION"
log "source:   $art_url"
log "dest:     $dest"

if [ "$DRY_RUN" = "1" ]; then
    log "dry-run: would install $artifact to $dest"
    exit 0
fi

if command -v curl >/dev/null 2>&1; then
    fetch() { curl -fsSL -o "$1" "$2"; }
elif command -v wget >/dev/null 2>&1; then
    fetch() { wget -q -O "$1" "$2"; }
else
    die "need curl or wget installed"
fi

if command -v sha256sum >/dev/null 2>&1; then
    sha_check() { ( cd "$1" && sha256sum -c expected ); }
elif command -v shasum >/dev/null 2>&1; then
    sha_check() { ( cd "$1" && shasum -a 256 -c expected ); }
else
    die "need sha256sum or shasum installed"
fi

tmp="$(mktemp -d 2>/dev/null || mktemp -d -t termlink)"
trap 'rm -rf "$tmp"' EXIT INT TERM HUP

log "downloading binary..."
fetch "$tmp/$artifact" "$art_url" \
    || die "failed to download $art_url"

log "downloading checksums..."
fetch "$tmp/checksums.txt" "$sum_url" \
    || die "failed to download $sum_url"

# checksums.txt lines look like:  <hash>  termlink-<triple>
grep "  ${artifact}\$" "$tmp/checksums.txt" > "$tmp/expected" \
    || die "checksum line for $artifact not found in checksums.txt"

log "verifying sha256..."
sha_check "$tmp" >/dev/null 2>&1 \
    || die "checksum verification failed — refusing to install"

chmod +x "$tmp/$artifact"

if [ -w "$PREFIX/bin" ] 2>/dev/null; then
    mv "$tmp/$artifact" "$dest"
elif [ ! -e "$PREFIX/bin" ] && mkdir -p "$PREFIX/bin" 2>/dev/null; then
    mv "$tmp/$artifact" "$dest"
elif command -v sudo >/dev/null 2>&1; then
    log "elevating with sudo to write $dest"
    sudo mkdir -p "$PREFIX/bin"
    sudo mv "$tmp/$artifact" "$dest"
    sudo chmod +x "$dest"
else
    die "$PREFIX/bin not writable and sudo not available — try PREFIX=\$HOME/.local"
fi

log "installed: $dest"
case ":$PATH:" in
    *":$PREFIX/bin:"*) : ;;
    *) log "note: $PREFIX/bin is not in your PATH — add it or use --prefix=\$HOME/.local" ;;
esac
