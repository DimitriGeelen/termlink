#!/usr/bin/env bash
# T-3438 — hermetic fixtures for scripts/arc011-w1-provision.sh (arc-011 W1).
# A fake termlink and a fake supervise script in a scratch tree; nothing touches
# /etc, /var/lib, a hub, systemd or a real key.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUT="$ROOT/scripts/arc011-w1-provision.sh"
T=$(mktemp -d)
trap 'rm -rf "$T"' EXIT
PASS=0; FAIL=0
ok() { PASS=$((PASS + 1)); echo "ok   $*"; }
bad() { FAIL=$((FAIL + 1)); echo "FAIL $*"; }
check() { if eval "$2"; then ok "$1"; else bad "$1"; fi; }

mkdir -p "$T/ident" "$T/proj/.context/working" "$T/bin"
echo "seed-main" > "$T/ident/claude-termlink.key"
echo "seed-alt" > "$T/ident/claude-termlink-alt.key"
echo "project_name: 010-termlink" > "$T/proj/.framework.yaml"

cat > "$T/bin/termlink" <<'EOF'
#!/usr/bin/env bash
case "$*" in
    "identity show --resolve --no-create --json")
        [ -f "$TERMLINK_IDENTITY_FILE" ] || { echo '{"ok":false}'; exit 3; }
        echo "{\"public_key_hex\":\"$(sha256sum < "$TERMLINK_IDENTITY_FILE" | cut -c1-64)\"}" ;;
    "sidecar") echo "usage: termlink sidecar hub-id --hub-socket <path>" >&2; exit 2 ;;
    "sidecar hub-id --hub-socket "*) [ -e "$FAKE_NO_HUB" ] && exit 2; echo feedfacefeedface ;;
    "sidecar status --json")
        a=$(basename "$TERMLINK_SIDECAR_STATUS_SOCKET" .status.sock)
        echo "{\"ok\":true,\"mode\":\"${FAKE_MODE:-production}\",\"agent\":\"$a\"}" ;;
    *) exit 9 ;;
esac
EOF
cat > "$T/bin/supervise" <<'EOF'
#!/usr/bin/env bash
echo "$*" >> "$FAKE_CALLS"
[ -e "$FAKE_START_FAIL" ] && exit 1
f="$W1_ETC/$2.env"
store=$(sed -n 's/^TERMLINK_SIDECAR_STORE=//p' "$f"); agent=$(sed -n 's/^AGENT=//p' "$f")
mkdir -p "$store/sidecar/$agent"
echo "{\"pid\":1,\"ts_ms\":$(( $(date +%s) * 1000 )),\"supervisor\":\"${FAKE_SUP:-systemd}\",\"keep_running\":\"on\",\"stale\":[]}" > "$store/sidecar/$agent/heartbeat.json"
EOF
chmod +x "$T/bin/termlink" "$T/bin/supervise"

export W1_PROJECT=010-termlink W1_PROJECT_DIR="$T/proj" W1_IDENT_DIR="$T/ident" W1_ETC="$T/etc" \
       W1_VARLIB="$T/varlib" W1_HUB_RT="$T/rt" W1_BIN="$T/bin/termlink" W1_HOME="$T/home" \
       W1_SUPERVISE="$T/bin/supervise" W1_STAMP=w1-20261013 W1_WAIT_SECS=3 \
       FAKE_CALLS="$T/calls" FAKE_START_FAIL="$T/start-fail" FAKE_NO_HUB="$T/no-hub"
pub() { sha256sum < "$T/ident/$1.key" | cut -c1-64; }

# 1 readiness
out=$(bash "$SUT" --check 2>&1); rc=$?
check "1a ready when keys, hub id and the verb are there" '[ $rc = 0 ] && ! grep -q "NOT READY" <<< "$out"'
touch "$T/no-hub"; out=$(bash "$SUT" --check 2>&1); rc=$?; rm -f "$T/no-hub"
check "1b a hub that gives no id is NOT READY" '[ $rc = 1 ] && grep -q "does not answer with an id" <<< "$out"'

# 2 apply
out=$(bash "$SUT" --apply 2>&1); rc=$?
check "2a apply succeeds" '[ $rc = 0 ] && ! grep -q FAILED <<< "$out"'
check "2b key register holds each public key" '[ "$(cat "$T/etc/keys/claude-termlink.pub")" = "$(pub claude-termlink)" ] && [ "$(cat "$T/etc/keys/claude-termlink-alt.pub")" = "$(pub claude-termlink-alt)" ]'
check "2c instance ids are <agent>.<stamp> (plain names)" '[ "$(cat "$T/etc/instances/claude-termlink")" = claude-termlink.w1-20261013 ]'
check "2d production marker in each store root, owner-only dir" '[ "$(head -1 "$T/varlib/claude-termlink/.termlink-sidecar-root")" = "termlink-sidecar-production v1" ] && [ "$(stat -c %a "$T/varlib/claude-termlink")" = 700 ]'
check "2e config: roster is the partner key, book maps partner to its instance, main pinned" \
    'python3 -c "import json,sys; c=json.load(open(\"$T/etc/claude-termlink.json\")); sys.exit(0 if c[\"roster\"]==[\"$(pub claude-termlink-alt)\"] and c[\"book\"]==dict(peers={\"claude-termlink-alt\":\"claude-termlink-alt.w1-20261013\"},main=\"claude-termlink\") and c[\"hub_id\"]==\"feedfacefeedface\" and c[\"key_register\"].endswith(\"keys/claude-termlink.pub\") and c[\"hub\"]==dict(type=\"unix\",path=\"$T/rt/hub.sock\") else 1)"'
check "2f per-agent settings files name the project, agent, config and store" 'grep -qx "AGENT=claude-termlink-alt" "$T/etc/010-termlink--claude-termlink-alt.env" && grep -qx "TERMLINK_SIDECAR_STORE=$T/varlib/claude-termlink-alt" "$T/etc/010-termlink--claude-termlink-alt.env"'
check "2g admin-socket list names both admin sockets" '[ "$(wc -l < "$T/proj/.context/working/sidecar-admin-sockets.list")" = 2 ] && grep -qx "$T/rt/sidecar/claude-termlink.admin.sock" "$T/proj/.context/working/sidecar-admin-sockets.list"'
check "2h SIDECAR_KEEP_RUNNING on in the project settings" 'grep -qx "SIDECAR_KEEP_RUNNING: on" "$T/proj/.framework.yaml"'
check "2i both sidecars started through the supervise script" 'grep -qx "start 010-termlink--claude-termlink" "$T/calls" && grep -qx "start 010-termlink--claude-termlink-alt" "$T/calls"'
check "2j each verified running in production" '[ "$(grep -c "running in production mode under systemd" <<< "$out")" = 2 ]'

# 3 idempotent; a later stamp never re-mints an instance id
out=$(W1_STAMP=w1-20991231 bash "$SUT" --apply 2>&1); rc=$?
check "3a a second apply writes nothing new" '[ $rc = 0 ] && ! grep -q "written and verified" <<< "$out" && ! grep -q FAILED <<< "$out"'
check "3b the instance id is kept, not re-minted" '[ "$(cat "$T/etc/instances/claude-termlink")" = claude-termlink.w1-20261013 ]'

# 4 a changed key is refused, nothing rewritten
cp "$T/etc/claude-termlink.json" "$T/cfg.before"
echo "seed-other" > "$T/ident/claude-termlink.key"
out=$(bash "$SUT" --check 2>&1); rc=$?
check "4a check: a key that differs from the register is NOT READY" '[ $rc = 1 ] && grep -q "holds a different key" <<< "$out"'
out=$(bash "$SUT" --apply 2>&1); rc=$?
check "4b apply refuses and leaves the config as it was" '[ $rc = 1 ] && grep -q FAILED <<< "$out" && cmp -s "$T/cfg.before" "$T/etc/claude-termlink.json"'
echo "seed-main" > "$T/ident/claude-termlink.key"

# 5 a missing key is NOT READY (never minted)
mv "$T/ident/claude-termlink-alt.key" "$T/alt.key"
out=$(bash "$SUT" --check 2>&1); rc=$?
check "5a a missing key file is NOT READY and is not created" '[ $rc = 1 ] && grep -q "is missing" <<< "$out" && [ ! -e "$T/ident/claude-termlink-alt.key" ]'
mv "$T/alt.key" "$T/ident/claude-termlink-alt.key"

# 6 start and verification failures are FAILED, never OK
touch "$T/start-fail"; out=$(bash "$SUT" --apply 2>&1); rc=$?; rm -f "$T/start-fail"
check "6a a sidecar that does not start is FAILED" '[ $rc = 1 ] && grep -q "did not start" <<< "$out"'
out=$(FAKE_SUP=none bash "$SUT" --apply 2>&1); rc=$?
check "6b running without the expected supervisor is FAILED" '[ $rc = 1 ] && grep -q "no fresh heartbeat under supervisor systemd" <<< "$out"'
out=$(FAKE_MODE=estate bash "$SUT" --apply 2>&1); rc=$?
check "6c a sidecar not in production mode is FAILED" '[ $rc = 1 ] && grep -q "does not answer in production mode" <<< "$out"'

echo "---"
echo "arc011-w1-provision fixtures: $PASS passed, $FAIL failed"
[ "$FAIL" = 0 ]
