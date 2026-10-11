#!/usr/bin/env bash
# T-3394: scripts/fleet-deploy-binary.sh chunk retry, error logging and abort cleanup.
# Hermetic: a fake `termlink` on PATH runs each "remote exec" command locally and can fail
# a named chunk a given number of times. No hub, no network, no real host state.
# The load-bearing case is the mutant: the pre-T-3394 loop (from git) must FAIL the
# transient case, otherwise this suite proves nothing.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPT="$ROOT/scripts/fleet-deploy-binary.sh"
TMP=$(mktemp -d -t fdcr-XXXXXX)
trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0
ok()  { echo "  PASS  $1"; PASS=$((PASS+1)); }
bad() { echo "  FAIL  $1: $2"; FAIL=$((FAIL+1)); }

mkdir -p "$TMP/bin"
cat > "$TMP/bin/termlink" <<'EOF'
#!/usr/bin/env bash
# Fake termlink: `remote list` and `remote exec`. FAKE_FAIL_CHUNK / FAKE_FAIL_TIMES make the
# send of that chunk fail that many times (counted in $FAKE_STATE), with an error on stderr.
[ "$1" = "remote" ] || exit 2
case "$2" in
  list) printf 'ID NAME\n-------\ntl-fake  fake-session\n'; exit 0;;
  exec)
    shift 2
    [ "$1" = "--timeout" ] && shift 2
    cmd="$3"
    if [ -n "${FAKE_FAIL_CHUNK:-}" ] && printf '%s' "$cmd" | grep -q "/$FAKE_FAIL_CHUNK'"; then
      n=$(cat "$FAKE_STATE" 2>/dev/null || echo 0)
      if [ "$n" -lt "${FAKE_FAIL_TIMES:-0}" ]; then
        echo $((n+1)) > "$FAKE_STATE"
        echo "fake: connection reset by peer" >&2
        exit 1
      fi
    fi
    bash -c "$cmd"; exit $?;;
esac
exit 2
EOF
chmod +x "$TMP/bin/termlink"

head -c 700 /dev/urandom > "$TMP/payload"      # 7 chunks of 100 bytes
SHA=$(sha256sum "$TMP/payload" | awk '{print $1}')

run() {  # $1 = script path; remaining env set by the caller
  rm -f "$TMP/state" "$TMP/dst"
  PATH="$TMP/bin:$PATH" FAKE_STATE="$TMP/state" FLEET_DEPLOY_BACKOFF="0 0" \
    bash "$1" fakehub --binary "$TMP/payload" --dst "$TMP/dst" --chunk-bytes 100 > "$TMP/out" 2>&1
}
leftover() { ls -d /tmp/.fleet-deploy.* 2>/dev/null | while read -r d; do [ -e "$d/c.0000" ] && echo "$d"; done; }
before_left=$(leftover | wc -l)

echo "== fleet-deploy-binary chunk retry (T-3394)"

# 1. A transient failure: chunk c.0002 fails once, is resent, and the binary arrives intact.
FAKE_FAIL_CHUNK=c.0002 FAKE_FAIL_TIMES=1 run "$SCRIPT"; rc=$?
if [ "$rc" = "0" ] && [ "$(sha256sum "$TMP/dst" 2>/dev/null | awk '{print $1}')" = "$SHA" ]; then
  ok "transient failure: chunk resent, binary assembled with the right sha"
else bad "transient recovery" "rc=$rc $(tail -5 "$TMP/out")"; fi
if grep -q "c.0002 attempt 1/3 FAILED (rc=1): fake: connection reset by peer" "$TMP/out" \
   && grep -q "sent 7/7 (retries=1)" "$TMP/out"; then
  ok "transient failure: attempt, rc and the remote error text are logged; retries counted"
else bad "transient logging" "$(grep -E 'attempt|sent' "$TMP/out")"; fi

# 2. A persistent failure: three attempts, then abort with the cause logged and staging removed.
FAKE_FAIL_CHUNK=c.0002 FAKE_FAIL_TIMES=99 run "$SCRIPT"; rc=$?
if [ "$rc" = "3" ] && grep -q "chunk c.0002 failed 3 times, aborting" "$TMP/out" \
   && grep -q "c.0002 attempt 3/3 FAILED" "$TMP/out"; then
  ok "persistent failure: aborts after 3 attempts of the same chunk, exit 3"
else bad "persistent abort" "rc=$rc $(tail -5 "$TMP/out")"; fi
if grep -q "removed remote staging dir" "$TMP/out" && [ "$(leftover | wc -l)" = "$before_left" ]; then
  ok "persistent failure: remote staging dir removed on abort"
else bad "abort cleanup" "$(grep -i staging "$TMP/out")"; fi
[ ! -e "$TMP/dst" ] && ok "persistent failure: no partial binary assembled" || bad "no partial" "dst exists"

# 3. No failure at all: unchanged behaviour.
run "$SCRIPT"; rc=$?
if [ "$rc" = "0" ] && grep -q "sent 7/7 (retries=0)" "$TMP/out"; then ok "clean run: all chunks, no retries"
else bad "clean run" "rc=$rc $(tail -3 "$TMP/out")"; fi

# 4. Mutant: the pre-T-3394 loop must FAIL the transient case (chunk skipped, never resent).
if git -C "$ROOT" show 5bf63128a:scripts/fleet-deploy-binary.sh > "$TMP/old.sh" 2>/dev/null; then
  FAKE_FAIL_CHUNK=c.0002 FAKE_FAIL_TIMES=1 run "$TMP/old.sh"; rc=$?
  if [ "$rc" != "0" ]; then ok "mutant (old loop) fails the transient case (rc=$rc): this suite can go red"
  else bad "mutant" "the old loop passed the transient case; the suite does not discriminate"; fi
  rm -rf $(leftover | tail -n +$((before_left+1))) 2>/dev/null
else bad "mutant" "could not read the pre-fix script from git"; fi

echo "fleet-deploy chunk retry fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = "0" ]
