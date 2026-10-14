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
ROLLBACK=0

for arg in "$@"; do
    case "$arg" in
        --dry-run)    DRY_RUN=1 ;;
        --version=*)  VERSION="${arg#*=}" ;;
        --prefix=*)   PREFIX="${arg#*=}" ;;
        --rollback)   ROLLBACK=1 ;;
        -h|--help)
            cat <<'EOF'
termlink installer

Usage:
  curl -fsSL https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh | sh
  sh install.sh [--version=vX.Y.Z] [--prefix=DIR] [--dry-run]
  sh install.sh --rollback [--prefix=DIR]

Options:
  --version=VERSION   install a specific release tag (default: latest)
  --prefix=DIR        install prefix; binary lands at DIR/bin/termlink (default: /usr/local)
  --dry-run           show what would happen without downloading or writing
  --rollback          restore the binary (and its checksum record) the last
                      install replaced, then reinstall its sidecar tools
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

if [ -n "${TERMLINK_INSTALL_BASE_URL:-}" ]; then
    # Test seam (T-3442): install from a local "release" (e.g. file:///srv/rel).
    base_url="$TERMLINK_INSTALL_BASE_URL"
elif [ "$VERSION" = "latest" ]; then
    base_url="https://github.com/${REPO}/releases/latest/download"
else
    base_url="https://github.com/${REPO}/releases/download/${VERSION}"
fi
art_url="${base_url}/${artifact}"
sum_url="${base_url}/checksums.txt"
dest="${PREFIX}/bin/termlink"
share="${PREFIX}/share/termlink"

# as_owner CMD...: run directly when the prefix is writable, else through sudo.
as_owner() {
    if [ -w "$PREFIX" ] 2>/dev/null || { [ -d "$PREFIX/bin" ] && [ -w "$PREFIX/bin" ]; }; then "$@"
    elif command -v sudo >/dev/null 2>&1; then sudo "$@"
    else "$@"; fi
}

# T-3442 (arc-011 P9, R-39/SI-27): after the binary, install the sidecar's tools
# (supervise script, unit template, backstop crontab) from the binary itself.
install_tools() {
    if "$dest" sidecar tools install --prefix "$PREFIX" >/dev/null 2>&1 \
       || as_owner "$dest" sidecar tools install --prefix "$PREFIX" >/dev/null 2>&1; then
        log "sidecar tools: ${share}/sidecar/"
    else
        log "note: this termlink has no 'sidecar tools install' (older release); sidecar tools not installed"
    fi
}

if [ "$ROLLBACK" = "1" ]; then
    [ -e "$dest.prev" ] || die "nothing to roll back: $dest.prev not found"
    as_owner mv "$dest.prev" "$dest"
    if [ -e "$share/checksums.txt.prev" ]; then as_owner mv "$share/checksums.txt.prev" "$share/checksums.txt"
    else as_owner rm -f "$share/checksums.txt"; fi
    log "rolled back: $dest ($("$dest" --version 2>/dev/null || echo unknown))"
    install_tools
    exit 0
fi

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

# Keep what this install replaces, for --rollback.
if [ -e "$dest" ]; then
    as_owner cp -p "$dest" "$dest.prev" || die "cannot keep the previous binary as $dest.prev"
    [ -e "$share/checksums.txt" ] && as_owner cp -p "$share/checksums.txt" "$share/checksums.txt.prev"
fi

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
# SI-27: keep the release checksum record; `termlink sidecar status` re-verifies
# the running binary against it on every call.
as_owner mkdir -p "$share"
as_owner cp "$tmp/checksums.txt" "$share/checksums.txt" || die "cannot write $share/checksums.txt"
log "checksums: $share/checksums.txt"
install_tools
case ":$PATH:" in
    *":$PREFIX/bin:"*) : ;;
    *) log "note: $PREFIX/bin is not in your PATH — add it or use --prefix=\$HOME/.local" ;;
esac
