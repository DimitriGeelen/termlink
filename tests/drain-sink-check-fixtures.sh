#!/usr/bin/env bash
# drain-sink-check-fixtures.sh (T-2531) — load-bearing tests for
# scripts/check-drain-sink-caps.sh. No live binary; pure fixtures.
#
# Proves the detector (a) FIRES on an unbounded `.output()` child drain,
# (b) FIRES on an unbounded `.read_to_end`, (c) does NOT fire on a
# `.take(N).read_to_end` bounded read, (d) skips a `//`-comment mention,
# (e) respects the allowlist by `relpath::fn::sink` signature, (f) FIRES on
# `.collect::<Vec<u8>>()`. Run: bash tests/drain-sink-check-fixtures.sh
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.." || exit 2

CHECK=scripts/check-drain-sink-caps.sh
[ -f "$CHECK" ] || { echo "FAIL: $CHECK not found"; exit 1; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
FIX="$TMP/fixture.rs"
EMPTY_ALLOW="$TMP/empty-allowlist"
: > "$EMPTY_ALLOW"

pass=0 fail=0
ok()  { echo "  ok: $1"; pass=$((pass+1)); }
bad() { echo "  FAIL: $1"; fail=$((fail+1)); }

run() { bash "$CHECK" --root "$TMP" --allowlist "${1:-$EMPTY_ALLOW}" --no-heartbeat --json 2>/dev/null; }

# --- (a) unbounded .output() child drain -> MUST fire ------------------------
cat > "$FIX" <<'RS'
async fn run_peer_cmd(peer_cmd: &str) -> Vec<u8> {
    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c").arg(peer_cmd);
    let out = cmd.output().await.unwrap();
    out.stdout
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":false' && printf '%s' "$out" | grep -q '"fn":"run_peer_cmd"' && printf '%s' "$out" | grep -q '"sink":"output"'; then
    ok "(a) unbounded cmd.output() fires with the enclosing fn name"
else bad "(a) expected fire on cmd.output(); got: $out"; fi

# --- (b) unbounded .read_to_end -> MUST fire ---------------------------------
cat > "$FIX" <<'RS'
async fn drain(mut sock: TcpStream) -> Vec<u8> {
    let mut buf = Vec::new();
    sock.read_to_end(&mut buf).await.unwrap();
    buf
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":false' && printf '%s' "$out" | grep -q '"sink":"read_to_end"'; then
    ok "(b) unbounded read_to_end fires"
else bad "(b) expected fire on read_to_end; got: $out"; fi

# --- (c) .take(N).read_to_end bounded read -> MUST NOT fire ------------------
cat > "$FIX" <<'RS'
async fn drain_bounded(mut sock: TcpStream) -> Vec<u8> {
    let mut buf = Vec::new();
    sock.take(1_048_576).read_to_end(&mut buf).await.unwrap();
    buf
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":true'; then
    ok "(c) .take(N).read_to_end bounded read does not fire"
else bad "(c) expected clean on .take()-bounded read; got: $out"; fi

# --- (d) comment mention -> MUST NOT fire ------------------------------------
cat > "$FIX" <<'RS'
async fn documented() -> Vec<u8> {
    // we deliberately avoid cmd.output() here; see execute_capped instead
    Vec::new()
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":true'; then
    ok "(d) a //-comment mention of cmd.output() is skipped"
else bad "(d) expected clean on comment-only mention; got: $out"; fi

# --- (e) allowlist ack by relpath::fn::sink -> MUST NOT fire -----------------
cat > "$FIX" <<'RS'
async fn trusted_probe() -> Vec<u8> {
    let mut cmd = tokio::process::Command::new(&exe);
    let out = cmd.output().await.unwrap();
    out.stdout
}
RS
ALLOW="$TMP/allow"
echo "$TMP/fixture.rs::trusted_probe::output" > "$ALLOW"
out="$(run "$ALLOW")"
if printf '%s' "$out" | grep -q '"ok":true'; then
    ok "(e) allowlisted relpath::fn::sink signature suppresses the site"
else bad "(e) expected clean on allowlisted site; got: $out"; fi

# --- (f) .collect::<Vec<u8>>() -> MUST fire ----------------------------------
cat > "$FIX" <<'RS'
fn slurp(stream: impl Iterator<Item = u8>) -> Vec<u8> {
    stream.collect::<Vec<u8>>()
}
RS
out="$(run)"
if printf '%s' "$out" | grep -q '"ok":false' && printf '%s' "$out" | grep -q '"sink":"collect_vec_u8"'; then
    ok "(f) unbounded collect::<Vec<u8>>() fires"
else bad "(f) expected fire on collect::<Vec<u8>>(); got: $out"; fi

echo ""
echo "drain-sink fixtures: $pass passed, $fail failed"
[ "$fail" -eq 0 ] || exit 1
