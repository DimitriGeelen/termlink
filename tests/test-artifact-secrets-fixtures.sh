#!/usr/bin/env bash
# guard-layer: source
# Fixtures for scripts/check-test-artifact-secrets.sh (arc-011 PH-0 6.2.1.a (6), SI-25, FC-17).
# Hermetic: temp dirs only; every planted secret is a synthetic string.
set -u
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
S="$HERE/../scripts/check-test-artifact-secrets.sh"
T="$(mktemp -d)"; trap 'rm -rf "$T"' EXIT
pass=0; fail=0
t() { # t <name> <expected-rc> <cmd...>
  local name="$1" want="$2"; shift 2
  "$@" >"$T/out" 2>&1; local rc=$?
  if [ "$rc" = "$want" ]; then pass=$((pass+1)); else fail=$((fail+1)); echo "FAIL $name: rc=$rc want=$want"; sed 's/^/    /' "$T/out"; fi
}
CANARY="FC17-CANARY-7f3a9c21d4e8"
HUBSEC="0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"

mk() { rm -rf "$T/$1"; mkdir -p "$T/$1"; }
mk clean; echo "test ok: 12 passed" > "$T/clean/run.log"; echo '{"status":"ready"}' > "$T/clean/status.json"
mk canary; echo "env dump: SOMEVAR=$CANARY" > "$T/canary/run.log"; echo fine > "$T/canary/b.log"
mk key; printf -- '-----BEGIN %s PRIVATE KEY-----\nabc\n' RSA > "$T/key/k.txt"
mk hub; echo "hub_secret=$HUBSEC" > "$T/hub/status.out"
mk lit; echo "frame payload $HUBSEC end" > "$T/lit/frame.bin"; echo "$HUBSEC" > "$T/hub.secret"
mk bin; printf '\x00\x01bin %s \x02' "$CANARY" > "$T/bin/frame.dat"
mk empty

t clean-passes 0 "$S" "$T/clean" --canary "$CANARY"
t canary-found 1 "$S" "$T/canary" --canary "$CANARY"
t canary-env 1 env TERMLINK_TEST_SECRET_CANARY="$CANARY" "$S" "$T/canary"
t canary-in-binary-frame 1 "$S" "$T/bin" --canary "$CANARY"
t private-key-found 1 "$S" "$T/key"
t hub-secret-shape-found 1 "$S" "$T/hub"
t secret-file-value-found 1 "$S" "$T/lit" --secret-file "$T/hub.secret"
t secret-file-absent-clean 0 "$S" "$T/clean" --secret-file "$T/hub.secret"
# value never echoed
"$S" "$T/canary" --canary "$CANARY" > "$T/o2" 2>&1; ! grep -q "$CANARY" "$T/o2" && grep -q "rule=canary" "$T/o2" && pass=$((pass+1)) || { fail=$((fail+1)); echo "FAIL value-withheld"; }
"$S" "$T/hub" > "$T/o3" 2>&1; ! grep -q "$HUBSEC" "$T/o3" && pass=$((pass+1)) || { fail=$((fail+1)); echo "FAIL hub-value-withheld"; }
# fail-closed
t missing-dir 2 "$S" "$T/nope"
t empty-dir 2 "$S" "$T/empty"
t no-args 2 "$S"
t bad-patterns 2 "$S" "$T/clean" --patterns "$T/nope"
t short-canary 2 "$S" "$T/clean" --canary abc
echo short > "$T/short.secret"
t short-secret-file 2 "$S" "$T/clean" --secret-file "$T/short.secret"
t unknown-flag 2 "$S" "$T/clean" --bogus
# json
t json-found 1 "$S" "$T/canary" --canary "$CANARY" --json
grep -q '"ok":false' "$T/out" && pass=$((pass+1)) || { fail=$((fail+1)); echo "FAIL json-shape"; }
# mutant: literal matching disabled -> the canary case must go clean (proves the case is load-bearing)
t mutant-no-canary-goes-clean 0 env SECRET_ARTIFACT_MUTANT_NO_CANARY=1 "$S" "$T/canary" --canary "$CANARY"

echo "pass=$pass fail=$fail"
[ "$fail" = 0 ]
