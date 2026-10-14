#!/usr/bin/env bash
# T-3442 (arc-011 PH-2 row P9; R-39.e, SI-27, exit 5) — clean-host install,
# upgrade, rollback and restart test.
#
# Builds a local "release" from the musl binary (v1 = the build, v2 = the same
# build with a trailing marker so it has a different checksum and still runs),
# serves it on 127.0.0.1 over HTTP, and runs tests/arc011-clean-host/inner.sh in
# a fresh Alpine container (no /opt/termlink checkout, no Rust) using the REAL
# install.sh. Not a guard-layer member: it needs Docker, network for apk, and a
# musl build. Run it by hand after changing install.sh or the sidecar tools.
#
# Usage: bash tests/arc011-clean-host.sh [--binary PATH]   (default: the musl local-fast build)
# Exit: 0 all steps passed · 1 a step failed · 2 tooling (no docker/binary/python3).
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/target/x86_64-unknown-linux-musl/local-fast/termlink"
[ "${1:-}" = "--binary" ] && BIN="$2"
IMAGE="${CLEAN_HOST_IMAGE:-alpine:3}"
command -v docker >/dev/null || { echo "TOOLING: docker not found"; exit 2; }
command -v python3 >/dev/null || { echo "TOOLING: python3 not found"; exit 2; }
[ -x "$BIN" ] || { echo "TOOLING: no musl binary at $BIN (cargo build --profile local-fast --target x86_64-unknown-linux-musl -p termlink)"; exit 2; }
file "$BIN" 2>/dev/null | grep -q 'statically linked' || echo "note: $BIN may not be static"

T=$(mktemp -d)
SRV=""
cleanup() { [ -n "$SRV" ] && kill "$SRV" 2>/dev/null; rm -rf "$T"; }
trap cleanup EXIT
ART=termlink-linux-x86_64-static
mkdir -p "$T/rel/v1" "$T/rel/v2"
cp "$BIN" "$T/rel/v1/$ART"
cp "$BIN" "$T/rel/v2/$ART"; printf '\n# clean-host v2 marker %s\n' "$(date +%s)" >> "$T/rel/v2/$ART"
for v in v1 v2; do (cd "$T/rel/$v" && sha256sum "$ART" > checksums.txt); done
cut -d' ' -f1 "$T/rel/v1/checksums.txt" > "$T/expect-v1.sha"
cut -d' ' -f1 "$T/rel/v2/checksums.txt" > "$T/expect-v2.sha"

PORT=$(python3 -c 'import socket; s=socket.socket(); s.bind(("127.0.0.1",0)); print(s.getsockname()[1])')
python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$T/rel" >/dev/null 2>&1 &
SRV=$!
for _ in $(seq 1 20); do python3 -c "import urllib.request; urllib.request.urlopen('http://127.0.0.1:$PORT/v1/checksums.txt')" 2>/dev/null && break; sleep 0.25; done

echo "clean host: $IMAGE, release served at http://127.0.0.1:$PORT (v1 $(cut -c1-12 "$T/expect-v1.sha"), v2 $(cut -c1-12 "$T/expect-v2.sha"))"
docker run --rm --network host \
    -v "$ROOT/install.sh:/work/install.sh:ro" \
    -v "$ROOT/tests/arc011-clean-host/inner.sh:/work/inner.sh:ro" \
    -v "$T/expect-v1.sha:/srv/expect-v1.sha:ro" -v "$T/expect-v2.sha:/srv/expect-v2.sha:ro" \
    "$IMAGE" sh /work/inner.sh "http://127.0.0.1:$PORT"
rc=$?
[ "$rc" = 0 ] && echo "clean-host test: PASS" || echo "clean-host test: FAIL (rc=$rc)"
exit $([ "$rc" = 0 ] && echo 0 || echo 1)
