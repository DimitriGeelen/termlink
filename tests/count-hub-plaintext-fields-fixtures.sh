#!/usr/bin/env bash
# guard-layer fixtures for scripts/count-hub-plaintext-fields.sh (T-3427). Hermetic: copies the
# real anchor files into a temp root; never touches a hub.
set -uo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
S="$HERE/scripts/count-hub-plaintext-fields.sh"
fails=0; n=0
# The fixture copies scripts/agent-send.sh as text and never runs it; the redirect
# keeps any future run away from the real woken-but-silent canary log anyway (T-3443).
WS_TMP="$(mktemp -d)"; trap 'rm -rf "$WS_TMP"' EXIT
export TERMLINK_WOKEN_SILENT_LOG="$WS_TMP/woken-silent-canary.log"
ok() { n=$((n+1)); if [ "$2" = "$3" ]; then echo "ok $n $1"; else echo "not ok $n $1 (want '$3' got '$2')"; fails=$((fails+1)); fi; }

mk() { # build a fixture root from the real tree's anchor files
    local r; r="$(mktemp -d)"
    for f in crates/termlink-hub/src/channel.rs scripts/agent-send.sh scripts/agent-respond.sh \
             scripts/chat-arc-broadcast.sh scripts/notify-ack-read.sh \
             crates/termlink-cli/src/commands/channel.rs crates/termlink-sidecar/src/wire.rs \
             crates/termlink-sidecar/src/send.rs crates/termlink-hub/src/artifact.rs; do
        mkdir -p "$r/$(dirname "$f")"; cp "$HERE/$f" "$r/$f"
    done
    echo "$r"
}

R="$(mk)"
out="$(bash "$S" --root "$R" --quiet)"; rc=$?
ok "baseline total is 9 (8 + artifact store, T-3432)" "$out" "9"; ok "baseline rc" "$rc" "0"
bash "$S" --root "$R" --max 9 >/dev/null 2>&1; ok "within --max 9 passes" "$?" "0"

# an added free-text metadata key raises the count and trips --max
printf '\ntermlink channel post t --metadata summary="$S"\n' >> "$R/scripts/agent-send.sh"
out="$(bash "$S" --root "$R" --quiet)"; ok "added metadata key raises total" "$out" "10"
bash "$S" --root "$R" --max 9 >/dev/null 2>&1; ok "grown count fires --max (rc 1)" "$?" "1"

# a routing-only key does not raise it
R2="$(mk)"; printf '\ntermlink channel post t --metadata conversation_id="$C"\n' >> "$R2/scripts/agent-send.sh"
ok "routing key does not raise" "$(bash "$S" --root "$R2" --quiet)" "9"

# a removed rail lowers it
R3="$(mk)"; sed -i 's/self.body.clone().into()/String::new().into()/' "$R3/crates/termlink-sidecar/src/wire.rs"
ok "removed sidecar body lowers" "$(bash "$S" --root "$R3" --quiet)" "8"

# fail-closed: empty root and bad root are tooling errors, never 0
E="$(mktemp -d)"; bash "$S" --root "$E" >/dev/null 2>&1; ok "empty root rc 2" "$?" "2"
bash "$S" --root /nonexistent-x >/dev/null 2>&1; ok "missing root rc 2" "$?" "2"
bash "$S" --max abc >/dev/null 2>&1; ok "bad --max rc 2" "$?" "2"
ok "json carries total" "$(bash "$S" --root "$R3" --json | grep -c '"total":8')" "1"
# T-3432: removing the artifact store's write path lowers the count (the anchor is load-bearing)
R4="$(mk)"; sed -i 's/store.put_streaming(staging_id, offset, &chunk, is_final, expected_sha256)/store.gone()/' "$R4/crates/termlink-hub/src/artifact.rs"
ok "removed artifact store write lowers" "$(bash "$S" --root "$R4" --quiet)" "8"
rm -rf "$R" "$R2" "$R3" "$R4" "$E"
[ "$fails" -eq 0 ] && { echo "all $n passed"; exit 0; } || { echo "$fails failed"; exit 1; }
