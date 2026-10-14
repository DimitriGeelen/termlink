#!/usr/bin/env bash
# T-3442 (arc-011 P9) — hermetic fixtures for install.sh: the checksum record is
# kept (SI-27), the sidecar tools are installed from the binary, the previous
# binary is kept and --rollback restores it, and a bad checksum installs nothing.
# A fake "release" of shell-script artifacts served over file:// into a scratch
# prefix; no network, no real termlink, no host paths.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }
command -v curl >/dev/null || { echo "TOOLING: curl needed"; exit 2; }

# The artifact name install.sh picks on this host.
if [ -r /etc/alpine-release ] || ! (ldd --version 2>&1 | grep -qi 'glibc\|gnu libc'); then ART=termlink-linux-x86_64-static; else ART=termlink-linux-x86_64; fi
[ "$(uname -m)" = x86_64 ] || { echo "TOOLING: fixture assumes x86_64"; exit 2; }

mkrel() {  # mkrel <dir> <version-label>
    mkdir -p "$1"
    cat > "$1/$ART" <<EOF
#!/bin/sh
case "\$1 \$2 \$3" in
    "sidecar tools install") mkdir -p "\$5/share/termlink/sidecar"; echo "$2" > "\$5/share/termlink/sidecar/tools-from"; exit 0 ;;
esac
[ "\$1" = "--version" ] && echo "termlink $2"
EOF
    (cd "$1" && sha256sum "$ART" > checksums.txt)
}
mkrel "$T/rel/v1" 1.0.0
mkrel "$T/rel/v2" 2.0.0
P="$T/prefix"; mkdir -p "$P/bin"
inst() { TERMLINK_INSTALL_BASE_URL="file://$T/rel/$1" sh "$ROOT/install.sh" --prefix="$P" 2>&1; }

out=$(inst v1); rc=$?
check "1a install succeeds" '[ $rc = 0 ]'
check "1b the binary is v1" '[ "$("$P/bin/termlink" --version)" = "termlink 1.0.0" ]'
check "1c the release checksum record is kept (SI-27)" 'cmp -s "$T/rel/v1/checksums.txt" "$P/share/termlink/checksums.txt"'
check "1d the sidecar tools were installed from the new binary" '[ "$(cat "$P/share/termlink/sidecar/tools-from")" = 1.0.0 ]'
check "1e a first install has nothing to keep" '[ ! -e "$P/bin/termlink.prev" ]'

out=$(inst v2); rc=$?
check "2a upgrade to v2 succeeds" '[ $rc = 0 ] && [ "$("$P/bin/termlink" --version)" = "termlink 2.0.0" ]'
check "2b the replaced v1 binary and its record are kept" '[ "$("$P/bin/termlink.prev" --version)" = "termlink 1.0.0" ] && cmp -s "$T/rel/v1/checksums.txt" "$P/share/termlink/checksums.txt.prev"'
check "2c the record now describes v2, and the tools came from v2" 'cmp -s "$T/rel/v2/checksums.txt" "$P/share/termlink/checksums.txt" && [ "$(cat "$P/share/termlink/sidecar/tools-from")" = 2.0.0 ]'

out=$(sh "$ROOT/install.sh" --rollback --prefix="$P" 2>&1); rc=$?
check "3a --rollback restores v1" '[ $rc = 0 ] && [ "$("$P/bin/termlink" --version)" = "termlink 1.0.0" ]'
check "3b ... with its checksum record, and reinstalls its tools" 'cmp -s "$T/rel/v1/checksums.txt" "$P/share/termlink/checksums.txt" && [ "$(cat "$P/share/termlink/sidecar/tools-from")" = 1.0.0 ]'
out=$(sh "$ROOT/install.sh" --rollback --prefix="$P" 2>&1); rc=$?
check "3c a second --rollback has nothing left and says so" '[ $rc != 0 ] && grep -q "nothing to roll back" <<< "$out"'

# 4 a release whose checksum does not match installs nothing.
mkrel "$T/rel/bad" 9.9.9; echo "# tampered" >> "$T/rel/bad/$ART"
out=$(inst bad); rc=$?
check "4a a checksum mismatch refuses" '[ $rc != 0 ] && grep -q "checksum verification failed" <<< "$out"'
check "4b and leaves the installed binary and record untouched" '[ "$("$P/bin/termlink" --version)" = "termlink 1.0.0" ] && cmp -s "$T/rel/v1/checksums.txt" "$P/share/termlink/checksums.txt"'

echo "---"
echo "install.sh fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
