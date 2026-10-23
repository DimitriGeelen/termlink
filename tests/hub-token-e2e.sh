#!/usr/bin/env bash
# T-3543 — live proof that a node holding ONLY a hub token (no hub secret) can use the channel
# rail over TCP, and is refused above the token's scope.
#
# Starts a throwaway hub (own TERMLINK_RUNTIME_DIR, free 127.0.0.1 port) and a throwaway client
# HOME, so neither the host's hub nor its ~/.termlink/hubs.toml is touched. Needs a built binary:
#   TERMLINK_BIN=target/debug/termlink bash tests/hub-token-e2e.sh
# Not a fixture suite (it binds a port and runs a hub), so the guard layer does not run it.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
B="${TERMLINK_BIN:-$ROOT/target/debug/termlink}"
[ -x "$B" ] || { echo "hub-token-e2e: no binary at $B" >&2; exit 2; }
W="$(mktemp -d "${TMPDIR:-/var/tmp}/tl-hubtoken.XXXXXX")"
HUBPID=""
cleanup() { [ -n "$HUBPID" ] && kill "$HUBPID" 2>/dev/null; wait 2>/dev/null; rm -rf "$W"; }
trap cleanup EXIT
pass=0; fail=0
ok()  { echo "ok   $1"; pass=$((pass+1)); }
bad() { echo "FAIL $1"; fail=$((fail+1)); }

PORT="$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1])')"
RT="$W/rt"; NODE="$W/node"; mkdir -p "$RT" "$NODE/.termlink"
hub_env() { env TERMLINK_RUNTIME_DIR="$RT" "$@"; }
node()    { env -u TERMLINK_RUNTIME_DIR HOME="$NODE" TERMLINK_RUNTIME_DIR="$W/node-rt" "$B" "$@"; }

hub_env "$B" hub start --tcp "127.0.0.1:$PORT" > "$W/hub.log" 2>&1 &
HUBPID=$!
for _ in $(seq 1 100); do [ -s "$RT/hub.secret" ] && [ -S "$RT/hub.sock" ] && break; sleep 0.1; done
[ -s "$RT/hub.secret" ] || { echo "hub-token-e2e: hub did not start"; cat "$W/hub.log"; exit 2; }

# On the hub host: create the topic (channel.create needs execute) and mint an interact token.
hub_env "$B" channel create tok-e2e --hub "$RT/hub.sock" > "$W/create.out" 2>&1 \
    && ok "1 hub host creates the topic over its unix socket" || bad "1 create: $(cat "$W/create.out")"
hub_env "$B" hub token --out "$W/node.token" --scope interact --ttl 600 --json > "$W/mint.out" 2>&1 \
    && grep -q '"scope":"interact"' "$W/mint.out" && ok "2 hub token mints an interact token" || bad "2 mint: $(cat "$W/mint.out")"
grep -q "$(head -c 20 "$W/node.token")" "$W/mint.out" && bad "3 mint output contains the token" || ok "3 the token is never printed"
[ "$(stat -c %a "$W/node.token")" = 600 ] && ok "4 token file is mode 600" || bad "4 mode $(stat -c %a "$W/node.token")"
hub_env "$B" hub token --out "$W/x.token" --scope execute > "$W/exec-mint.out" 2>&1 \
    && bad "5 execute token minted" || { grep -q refusing "$W/exec-mint.out" && ok "5 execute scope is refused at mint" || bad "5: $(cat "$W/exec-mint.out")"; }

# The node: a profile with ONLY token_file. It never sees hub.secret.
cp "$W/node.token" "$NODE/.termlink/hub.token"; chmod 600 "$NODE/.termlink/hub.token"
node remote profile add lab "127.0.0.1:$PORT" --token-file "$NODE/.termlink/hub.token" --scope interact > "$W/add.out" 2>&1 \
    && ok "6 profile add --token-file" || bad "6 add: $(cat "$W/add.out")"
grep -q "secret" "$NODE/.termlink/hubs.toml" && bad "7 node hubs.toml holds a secret" || ok "7 node hubs.toml holds no secret"
node remote profile list --json > "$W/list.out" 2>&1 && grep -q '"secret_type":"token"' "$W/list.out" \
    && ok "8 profile list shows type token" || bad "8: $(cat "$W/list.out")"

node channel post tok-e2e "hello from a token node" --hub "127.0.0.1:$PORT" > "$W/post.out" 2>&1 \
    && ok "9 channel post --hub works with only the token" || bad "9 post: $(cat "$W/post.out")"
node channel subscribe tok-e2e --hub "127.0.0.1:$PORT" --json --limit 10 > "$W/sub.out" 2>&1 \
    && grep -q '"offset":0' "$W/sub.out" && ok "10 channel subscribe --hub reads it back" || bad "10 sub: $(cat "$W/sub.out")"
node channel list --hub "127.0.0.1:$PORT" > "$W/clist.out" 2>&1 && grep -q tok-e2e "$W/clist.out" \
    && ok "11 channel list --hub works" || bad "11 list: $(cat "$W/clist.out")"

node remote exec lab any-session "id" > "$W/exec.out" 2>&1 \
    && bad "12 exec allowed with an interact token" || { grep -qi "permission denied" "$W/exec.out" && ok "12 exec is refused (scope)" || bad "12: $(cat "$W/exec.out")"; }
node remote inject lab any-session "x" > "$W/inj.out" 2>&1 \
    && bad "13 inject allowed with an interact token" || { grep -qi "permission denied" "$W/inj.out" && ok "13 inject is refused (scope)" || bad "13: $(cat "$W/inj.out")"; }
node channel create other-topic --hub "127.0.0.1:$PORT" > "$W/c2.out" 2>&1 \
    && bad "14 channel create allowed with an interact token" || ok "14 channel create needs the hub host"

# An expired token is refused naming the file, never printing the token.
hub_env "$B" hub token --out "$W/dead.token" --scope interact --ttl 1 > /dev/null 2>&1; sleep 2
cp "$W/dead.token" "$NODE/.termlink/hub.token"; chmod 600 "$NODE/.termlink/hub.token"
node channel post tok-e2e "late" --hub "127.0.0.1:$PORT" > "$W/dead.out" 2>&1
if grep -q "expired" "$W/dead.out" && grep -q "hub.token" "$W/dead.out" && ! grep -qF "$(cat "$W/dead.token")" "$W/dead.out"; then
    ok "15 an expired token is refused naming the file, token not printed"
else bad "15: $(cat "$W/dead.out")"; fi

echo "---"
echo "hub-token e2e: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
