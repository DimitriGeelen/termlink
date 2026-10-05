#!/usr/bin/env bash
# guard-layer: source
#
# T-3065 — fixtures for the identity the auto-confirm receipt is SIGNED with.
#
# A receipt only helps if the sender can find it. `--await-ack` derives the recipient
# from the dm topic name (derive_dm_recipient) and polls channel.receipts for THAT
# sender_id, so a receipt signed by any other key satisfies nobody — measured: every
# --await-ack against this host exhausted and dead-lettered while auto-confirm was
# running and looking healthy.
#
# Relabelling via `channel post --sender-id` cannot substitute for signing: hub
# channel.rs:777 (T-1427) rejects a claimed sender_id that does not match the
# fingerprint derived from the signing pubkey. So these fixtures assert on WHICH
# IDENTITY the post is executed under, which is the only thing that moves sender_id.
#
# Runs the REAL script with a stub `termlink` on the TERMLINK_BIN seam.
set -uo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/notify-sidecar.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
command -v jq >/dev/null || { echo "TOOLING: jq missing" >&2; exit 2; }

TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

WANT_FP="aaaabbbbccccdddd"     # the dm-party fp the sidecar acks for
PERAGENT_FP="9999deadbeef9999" # what TERMLINK_AGENT_ID resolves to (the wrong key)

# Stub termlink.
#   agent identity --resolve : fp depends on which identity env is set — this is the
#                              whole mechanism under test, so the stub models the real
#                              precedence (IDENTITY_FILE > AGENT_ID > host default).
#   channel post             : records the identity env it ran under into $POSTED.
#   channel subscribe        : one content envelope at offset 7 so latest_off resolves.
cat > "$TMP/termlink" <<STUB
#!/usr/bin/env bash
POSTED="$TMP/posted.txt"
WANT_FP="$WANT_FP"
PERAGENT_FP="$PERAGENT_FP"
STUB
cat >> "$TMP/termlink" <<'STUB'
verb="${1:-}"; sub="${2:-}"
if [ "$verb" = "agent" ] && [ "$sub" = "identity" ]; then
    if [ -n "${TERMLINK_IDENTITY_FILE:-}" ]; then
        # only the fixture's "good key file" carries the wanted fp
        case "$TERMLINK_IDENTITY_FILE" in
          *goodkey*) echo "{\"fingerprint\":\"$WANT_FP\"}" ;;
          *)         echo '{"fingerprint":"0000000000000000"}' ;;
        esac
    elif [ -n "${TERMLINK_AGENT_ID:-}" ]; then
        echo "{\"fingerprint\":\"$PERAGENT_FP\"}"
    else
        echo "{\"fingerprint\":\"${FIXTURE_HOST_DEFAULT_FP:-0000000000000000}\"}"
    fi
    exit 0
fi
if [ "$verb" = "channel" ] && [ "$sub" = "list" ]; then
    prefix=""
    while [ $# -gt 0 ]; do case "$1" in --prefix) prefix="${2:-}"; shift 2 ;; *) shift ;; esac; done
    case "$prefix" in
      "dm:")    echo '{"topics":[{"name":"dm:aaaabbbbccccdddd:peer"}]}' ;;
      *)        echo '{"topics":[]}' ;;
    esac
    exit 0
fi
if [ "$verb" = "channel" ] && [ "$sub" = "unread" ]; then
    echo '{"unread_count":1,"last_offset":7}'; exit 0
fi
if [ "$verb" = "channel" ] && [ "$sub" = "subscribe" ]; then
    echo '{"topic":"t","offset":7,"ts":1,"msg_type":"note","sender_id":"peer","payload_b64":"aGk=","metadata":{}}'
    exit 0
fi
if [ "$verb" = "channel" ] && [ "$sub" = "post" ]; then
    # Record WHICH identity this post ran under. That is the assertion target.
    printf 'AGENT_ID=%s IDENTITY_FILE=%s\n' "${TERMLINK_AGENT_ID:-<unset>}" "${TERMLINK_IDENTITY_FILE:-<unset>}" >> "$POSTED"
    echo '{"ok":true,"offset":8}'; exit 0
fi
exit 0
STUB
chmod +x "$TMP/termlink"

run_sidecar() {
    rm -f "$TMP/posted.txt"; : > "$TMP/posted.txt"
    rm -rf "$TMP/notify"; mkdir -p "$TMP/notify"
    TERMLINK_BIN="$TMP/termlink" FIXTURE_HOST_DEFAULT_FP="${HOSTFP:-0000000000000000}" \
    TERMLINK_IDENTITY_DIR="${IDDIR:-$TMP/none}" \
        bash "$SCRIPT" --agent-id fixture-agent --self-fp "$WANT_FP" \
            --notify-dir "$TMP/notify" --auto-confirm --once \
            > "$TMP/out.txt" 2> "$TMP/err.txt"
    RC=$?
}

echo "case 1: per-agent identity ALREADY matches self-fp -> post keeps it (no regression)"
PERAGENT_FP="$WANT_FP"
cat > "$TMP/termlink.bak" < "$TMP/termlink"
sed -i "s/^PERAGENT_FP=.*/PERAGENT_FP=\"$WANT_FP\"/" "$TMP/termlink"
run_sidecar
if grep -q 'AGENT_ID=fixture-agent' "$TMP/posted.txt"; then
    ok "signs under the per-agent identity when that already matches"
else
    bad "expected a post under the per-agent identity, got: $(cat "$TMP/posted.txt")"
fi
cp "$TMP/termlink.bak" "$TMP/termlink"; chmod +x "$TMP/termlink"
PERAGENT_FP="9999deadbeef9999"

echo "case 2: per-agent MISMATCHES, host default matches -> post drops TERMLINK_AGENT_ID"
HOSTFP="$WANT_FP" run_sidecar
if grep -q 'AGENT_ID=<unset>' "$TMP/posted.txt"; then
    ok "receipt posted under the HOST DEFAULT identity (the T-3065 fix)"
else
    bad "receipt still signed by the wrong key: $(cat "$TMP/posted.txt")"
fi

echo "case 3: neither matches -> REFUSE loudly, post nothing, write no guard"
HOSTFP="0000000000000000" run_sidecar
[ -s "$TMP/posted.txt" ] && bad "posted a receipt that satisfies nobody" \
                         || ok "no receipt posted when no identity matches"
grep -q 'REFUSING to auto-confirm' "$TMP/err.txt" \
  && ok "refusal is loud on stderr" || bad "refusal was silent — indistinguishable from working"
if ls "$TMP/notify"/.*acked >/dev/null 2>&1; then
    bad "wrote an offset guard despite not acking — suppresses the retry forever"
else
    ok "no offset guard written on refusal"
fi

echo "case 4: the refusal names the agent and the fp so it is actionable"
grep -q "$WANT_FP" "$TMP/err.txt" && ok "names the self-fp" || bad "refusal does not name the fp"
grep -q 'fixture-agent' "$TMP/err.txt" && ok "names the agent" || bad "refusal does not name the agent"

echo "case 5: an explicit per-agent key file is used when it carries the wanted fp"
mkdir -p "$TMP/iddir"; : > "$TMP/iddir/fixture-agent.key"
# stub keys on the filename containing 'goodkey', so point the dir entry at one
rm -f "$TMP/iddir/fixture-agent.key"; mkdir -p "$TMP/goodkeydir"; : > "$TMP/goodkeydir/fixture-agent.key"
mv "$TMP/goodkeydir" "$TMP/goodkey-dir" 2>/dev/null || true
IDDIR="$TMP/goodkey-dir" HOSTFP="0000000000000000" run_sidecar
if grep -q 'IDENTITY_FILE=.*goodkey' "$TMP/posted.txt"; then
    ok "receipt posted under the explicit per-agent key file"
else
    bad "key-file branch not used: $(cat "$TMP/posted.txt")"
fi

echo "case 6: the offset guard is keyed by IDENTITY, so a signing change re-arms it"
# Pre-seed a guard under the OLD (wrong-identity) name at a high offset. The fix must
# still ack, because the corrective receipt is a different identity. This is the
# difference between the fix working and it silently doing nothing on every topic that
# was already 'acked' under the broken key.
HOSTFP="$WANT_FP"
rm -f "$TMP/posted.txt"; : > "$TMP/posted.txt"
rm -rf "$TMP/notify"; mkdir -p "$TMP/notify"
old_guard="$TMP/notify/.fixture-agent.dm_aaaabbbbccccdddd_peer.acked"
printf '999\n' > "$old_guard"
TERMLINK_BIN="$TMP/termlink" FIXTURE_HOST_DEFAULT_FP="$WANT_FP" TERMLINK_IDENTITY_DIR="$TMP/none" \
    bash "$SCRIPT" --agent-id fixture-agent --self-fp "$WANT_FP" \
        --notify-dir "$TMP/notify" --auto-confirm --once >/dev/null 2>&1
if [ -s "$TMP/posted.txt" ]; then
    ok "stale wrong-identity guard does not suppress the corrective re-ack"
else
    bad "old guard suppressed the re-ack — the fix would be inert on already-acked topics"
fi

echo "case 7: --sender-id relabelling is NOT used (the hub would reject it, T-1427)"
if grep -nE '^[^#]*--sender-id' "$SCRIPT" >/dev/null 2>&1; then
    bad "script passes --sender-id; hub channel.rs:777 rejects a mismatched claim"
else
    ok "no --sender-id relabelling attempted"
fi

# T-3346 cases. run_sidecar_as <as-identity> [keep-notify] — the stub keys a matching
# fingerprint on a key-file path containing 'goodkey', so the as-identity NAME carries it.
run_sidecar_as() {
    rm -f "$TMP/posted.txt"; : > "$TMP/posted.txt"
    [ "${2:-}" = keep ] || { rm -rf "$TMP/notify"; mkdir -p "$TMP/notify"; }
    TERMLINK_BIN="$TMP/termlink" FIXTURE_HOST_DEFAULT_FP="0000000000000000" \
    TERMLINK_IDENTITY_DIR="$TMP/asdir" \
        bash "$SCRIPT" --agent-id fixture-agent --self-fp "$WANT_FP" --as-identity "$1" \
            --notify-dir "$TMP/notify" --auto-confirm --once \
            > "$TMP/out.txt" 2> "$TMP/err.txt"
    RC=$?
}
mkdir -p "$TMP/asdir"; : > "$TMP/asdir/goodkey-owner.key"; : > "$TMP/asdir/wrong-owner.key"

echo "case 8 (T-3346): --as-identity finds the key stored under another agent's name"
run_sidecar_as goodkey-owner
if grep -q "IDENTITY_FILE=$TMP/asdir/goodkey-owner.key" "$TMP/posted.txt"; then
    ok "receipt signed with <identities>/<as-identity>.key"
else
    bad "as-identity key not used: $(cat "$TMP/posted.txt")"
fi
grep -q 'REFUSING' "$TMP/err.txt" && bad "refused although the as-identity key matches" \
                                  || ok "no refusal when the as-identity key matches"

echo "case 9 (T-3346): --as-identity naming a key with the WRONG fp still refuses"
run_sidecar_as wrong-owner
[ -s "$TMP/posted.txt" ] && bad "signed with a key whose fp does not match self-fp" \
                         || ok "a name match alone never earns a signature"
grep -q 'REFUSING' "$TMP/err.txt" && ok "refusal still loud" || bad "refusal went silent"

echo "case 10 (T-3346): the refusal is logged once per fp, not once per cycle"
run_sidecar_as wrong-owner keep
grep -q 'REFUSING' "$TMP/err.txt" && bad "refusal repeated on the next cycle (the 32,205-line log)" \
                                  || ok "second cycle with the same fp is quiet"
[ "$(cat "$TMP/notify/.fixture-agent.refusal" 2>/dev/null)" = "$WANT_FP" ] \
  && ok "marker records the refused fp" || bad "no refusal marker written"
printf 'ffffffffffffffff\n' > "$TMP/notify/.fixture-agent.refusal"
run_sidecar_as wrong-owner keep
grep -q 'REFUSING' "$TMP/err.txt" && ok "a changed fp logs the refusal again" \
                                  || bad "changed fp stayed silent"

echo "case 11 (T-3346): the shipped conf maps claude-termlink-alt to the claude-termlink key"
if grep -qE '^claude-termlink-alt[[:space:]].*--as-identity[[:space:]]+claude-termlink([[:space:]]|$)' \
        "$PROJECT_ROOT/.context/cron/notify-sidecar-agents.conf"; then
    ok "notify-sidecar-agents.conf carries --as-identity claude-termlink for -alt"
else
    bad "conf line for claude-termlink-alt lacks --as-identity claude-termlink"
fi

echo ""
echo "passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
