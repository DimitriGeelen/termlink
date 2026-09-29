#!/usr/bin/env bash
# T-3263 — CI-skip fixture for scripts/check-fleet-recipient-agreement.sh.
#
# The skip must key on the absent PREREQUISITE (no readable fleet map), not on a
# proxy for it (no termlink on PATH). A release job builds a binary and has no fleet;
# with the old proxy the check errored rc 2 there.
#
#   S1  CI + termlink ON PATH + empty fleet status   → SKIP rc 0
#   S2  CI + no termlink on PATH + empty fleet status → SKIP rc 0 (unchanged)
#   S3  no CI + empty fleet status                    → no verdict rc 2
#   M1  mutant: restore the PATH-keyed skip           → S1 goes red
#
# Mock termlink via TERMLINK_BIN and a stub on PATH. No hub, no network.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/../scripts/check-fleet-recipient-agreement.sh"
[ -f "$SCRIPT" ] || { echo "FAIL: $SCRIPT not found"; exit 1; }
W="$(mktemp -d -t fleet-agree-ci.XXXXXX)" || { echo "FAIL: mktemp"; exit 1; }
trap 'rm -rf "$W"' EXIT
mkdir -p "$W/bin" "$W/nobin"
# A termlink whose `fleet status` lists no hubs (no hubs.toml) and exits 0.
printf '#!/usr/bin/env bash\necho "No hubs configured."\nexit 0\n' > "$W/bin/termlink"
chmod +x "$W/bin/termlink"
BASEPATH="/usr/bin:/bin"
fail=0
pass() { echo "PASS: $1"; }
bad()  { echo "FAIL: $1"; sed 's/^/      /' "$W/out" | head -6; fail=1; }

# run <script> <ci:0|1> <path-has-termlink:0|1> → rc
run() {
    local p="$W/nobin:$BASEPATH"; [ "$3" = 1 ] && p="$W/bin:$BASEPATH"
    if [ "$2" = 1 ]; then
        (cd "$HERE/.." && env -i PATH="$p" HOME="$W" CI=true TERMLINK_BIN="$W/bin/termlink" bash "$1") >"$W/out" 2>&1
    else
        (cd "$HERE/.." && env -i PATH="$p" HOME="$W" TERMLINK_BIN="$W/bin/termlink" bash "$1") >"$W/out" 2>&1
    fi
    echo $?
}
case_s1() { [ "$(run "$1" 1 1)" = 0 ] && grep -q 'SKIP' "$W/out"; }

case_s1 "$SCRIPT" && pass "S1 CI + termlink on PATH + no fleet → SKIP rc 0" || bad "S1"
rc="$(run "$SCRIPT" 1 0)"; [ "$rc" = 0 ] && grep -q SKIP "$W/out" && pass "S2 CI + no termlink on PATH + no fleet → SKIP rc 0" || bad "S2 rc=$rc"
rc="$(run "$SCRIPT" 0 1)"; [ "$rc" = 2 ] && grep -q 'no verdict' "$W/out" && pass "S3 no CI + no fleet → no verdict rc 2 (fail-closed)" || bad "S3 rc=$rc"

M="$W/mutant.sh"
sed 's/^if \[ -z "\$MAP" \] && \[ -n "\${CI:-}" \]; then$/if [ -z "$MAP" ] \&\& [ -n "${CI:-}" ] \&\& ! command -v termlink >\/dev\/null 2>\&1; then/' "$SCRIPT" > "$M"
if cmp -s "$SCRIPT" "$M"; then bad "M1 mutant did not apply"
elif case_s1 "$M"; then bad "M1 mutant (PATH-keyed skip) survived S1"
else pass "M1 mutant (PATH-keyed skip) killed by S1"; fi

echo
[ "$fail" -eq 0 ] && { echo "fleet-recipient-agreement-ci-skip-fixtures: ALL PASS"; exit 0; }
echo "fleet-recipient-agreement-ci-skip-fixtures: FAILURES"; exit 1
