#!/usr/bin/env bash
# T-2343 (arc-004 push-transport, Candidate B) — webhook fan-out ISOLATED-HUB
# regression demo. Sibling of T-2342 (dm rail) for the OTHER arc-004 candidate.
#
# The webhook fan-out (S1-S6, T-2332..T-2337) is feature-complete and
# SECURITY-CRITICAL: outbound HTTP from the hub guarded by a deny-by-default
# exact-host SSRF allowlist + HMAC-SHA256 payload signing
# (`X-Termlink-Signature: sha256=<hex>`). It shipped with unit tests and a
# ONE-TIME manual smoke (T-2336) but — like the dm rail before T-2342 — no
# reusable reproducer. A silent regression here is a SECURITY defect, so this is
# the higher-stakes sibling. This demo drives the FULL fan-out path against an
# isolated hub + a local HTTP sink and verifies the signature over the wire.
#
# What it proves end-to-end (real hub, real HTTP, real HMAC — no stub):
#   A. POSITIVE: an isolated hub started with a TERMLINK_WEBHOOK_CONFIG target
#      (host allowlisted, topic filter) fans a `channel.post` on the matching
#      topic out to the local sink as a signed POST. The demo recomputes
#      HMAC-SHA256 over the RAW received body with the configured signing_key and
#      asserts it equals the `x-termlink-signature: sha256=<hex>` header
#      (exercises T-2332 sign_payload + T-2333 fan_out + channel.rs Ok-arm wiring).
#   B. TOPIC-FILTER negative: a post to a NON-matching topic delivers NOTHING to
#      the sink (the `topics` filter gates fan-out).
#   C. SSRF deny-by-default: `webhook test` against a NON-allowlisted host (the
#      169.254.169.254 cloud-metadata address) is refused LOUDLY (non-zero exit,
#      host-not-allowlisted) with NO delivery to the sink — the production
#      `webhook::dispatch` guard, at the CLI surface (PL-239).
#
# The signing_key is used as RAW UTF-8 BYTES (webhook.rs sign_payload:
# `HmacSha256::new_from_slice(signing_key.as_bytes())`), NOT hex-decoded — the
# recompute below mirrors the operator recipe's consumer-verify snippet.
#
# Isolation contract: temp TERMLINK_RUNTIME_DIR + temp HOME + a loopback sink;
# NEVER touches the shared :9100 hub or ~/.termlink. Hub + sink torn down on exit.
#
# Usage:   scripts/demo-webhook-fanout.sh
# Env:     TERMLINK_BIN            real termlink binary (default target/release/termlink)
#          DEMO_WEBHOOK_HUB_PORT   loopback TCP port for the isolated hub (default 9200)
#          DEMO_WEBHOOK_SINK_PORT  loopback TCP port for the HTTP sink (default 8799)
# Exit:    0 PASS | 2 binary missing | 3 hub/sink failed to start | 4 python3 missing
#          5 no signed delivery | 6 HMAC mismatch | 7 topic-filter leak (false delivery)
#          8 SSRF guard did not refuse
set -uo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_WEBHOOK_HUB_PORT:-9200}"
SINK_PORT="${DEMO_WEBHOOK_SINK_PORT:-8799}"
HUBADDR="127.0.0.1:${PORT}"
# Raw-string signing key (NOT hex-decoded by the hub — see header note).
SIGNING_KEY="t2343-demo-signing-key-do-not-reuse-0123456789abcdef"
MATCH_TOPIC="webhook-demo-$$"
NOMATCH_TOPIC="webhook-nomatch-$$"

if [ ! -x "$BIN" ]; then
  echo "FATAL: termlink binary not found/executable at '$BIN'."
  echo "  set TERMLINK_BIN=<path> or build: cargo build --release -p termlink"
  exit 2
fi
command -v python3 >/dev/null 2>&1 || { echo "SKIP: python3 required for the HTTP sink"; exit 4; }
command -v jq      >/dev/null 2>&1 || { echo "FATAL: jq required"; exit 3; }
# The fan-out path is load-bearing on the webhook subsystem (T-2332+). A binary
# that predates it will never sign/POST — fail LOUD here rather than as a mystery
# no-delivery (mirror of the dm.queued guard in demo-dm-rail-pushwake.sh).
if [ "$(grep -a -c 'x-termlink-signature' "$BIN" 2>/dev/null || true)" -lt 1 ]; then
  echo "FATAL: '$BIN' has no webhook subsystem (predates arc-004 Candidate B / T-2332)."
  echo "  rebuild: cargo build --release -p termlink"
  exit 2
fi
BIN_ABS="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")"

RT="$(mktemp -d)"; HM="$(mktemp -d)"; HUBLOG="$(mktemp)"
CFG="$HM/webhooks.json"
SINKPY="$HM/sink.py"
SIG_FILE="$HM/sink-sig.txt"       # latest received x-termlink-signature header
BODY_FILE="$HM/sink-body.bin"     # latest received RAW body bytes
COUNT_FILE="$HM/sink-count"        # one byte appended per received POST
HUB_PID=""; SINK_PID=""
cleanup() {
  [ -n "$SINK_PID" ] && kill "$SINK_PID" 2>/dev/null || true
  [ -n "$HUB_PID" ]  && kill "$HUB_PID"  2>/dev/null || true
  rm -rf "$RT" "$HM" "$HUBLOG" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT"
export HOME="$HM"
export TERMLINK_BIN="$BIN_ABS"

FAIL=0; RC=0
note_fail() { echo "FAIL: $1"; FAIL=1; RC="$2"; }
sink_count() { local n; n=$(wc -c < "$COUNT_FILE" 2>/dev/null | tr -d ' ') || true; echo "${n:-0}"; }

# ---- local HTTP sink (captures signature + raw body of each POST) -----------
cat > "$SINKPY" <<'PY'
import http.server, os, sys
SIG = os.environ["SIG_FILE"]; BODY = os.environ["BODY_FILE"]; COUNT = os.environ["COUNT_FILE"]
class H(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        n = int(self.headers.get("content-length", 0) or 0)
        body = self.rfile.read(n) if n else b""
        sig = self.headers.get("x-termlink-signature", "")
        with open(SIG, "w") as f: f.write(sig)
        with open(BODY, "wb") as f: f.write(body)
        with open(COUNT, "ab") as f: f.write(b"1")
        self.send_response(204); self.end_headers()
    def log_message(self, *a): pass
http.server.HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PY
: > "$COUNT_FILE"
SIG_FILE="$SIG_FILE" BODY_FILE="$BODY_FILE" COUNT_FILE="$COUNT_FILE" \
  python3 "$SINKPY" "$SINK_PORT" >>"$HUBLOG" 2>&1 &
SINK_PID=$!
for _ in $(seq 1 50); do
  (exec 3<>"/dev/tcp/127.0.0.1/$SINK_PORT") 2>/dev/null && { exec 3>&- 3<&-; break; }
  sleep 0.1
done

# ---- webhook config: allowlist 127.0.0.1 only, target filters MATCH_TOPIC ----
cat > "$CFG" <<EOF
{
  "allowed_hosts": ["127.0.0.1"],
  "targets": [
    { "url": "http://127.0.0.1:${SINK_PORT}/hook", "signing_key": "${SIGNING_KEY}", "topics": ["${MATCH_TOPIC}"] }
  ]
}
EOF
export TERMLINK_WEBHOOK_CONFIG="$CFG"   # hub reads this at startup (webhook::init)

# ---- isolated hub (inherits TERMLINK_WEBHOOK_CONFIG) ------------------------
rm -f "$RT/hub.sock" "$RT/hub.pid" 2>/dev/null || true
"$BIN" hub start --tcp "$HUBADDR" >>"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do
  [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && break
  sleep 0.1
done
[ -s "$RT/hub.secret" ] || { echo "FATAL: isolated hub did not start"; cat "$HUBLOG"; exit 3; }
# hubs.toml so CLI posts can mint a TCP token.
mkdir -p "$HM/.termlink"
cat > "$HM/.termlink/hubs.toml" <<EOF
[hubs.demo-webhook]
address = "$HUBADDR"
secret_file = "$RT/hub.secret"
EOF
WEBHOOK_ENABLED="$("$BIN" hub status --governor --json 2>/dev/null | jq -r '.governor.webhook_enabled // .webhook_enabled // "n/a"' 2>/dev/null)"

# ---- A. POSITIVE: matching post fans out a signed POST to the sink ----------
"$BIN" channel create "$MATCH_TOPIC" --hub "$HUBADDR" >/dev/null 2>&1 || true
CNT0="$(sink_count)"
"$BIN" channel post "$MATCH_TOPIC" --payload "webhook-fanout-$$" --hub "$HUBADDR" >/dev/null 2>&1
for _ in $(seq 1 200); do   # up to ~10s
  [ "$(sink_count)" -gt "$CNT0" ] && break
  sleep 0.05
done
if [ "$(sink_count)" -le "$CNT0" ]; then
  note_fail "matching post did NOT deliver to the webhook sink (no signed POST)" 5
fi

# ---- HMAC verification over the RAW received body --------------------------
HMAC_OK="n/a"
if [ "$FAIL" -eq 0 ]; then
  RECV_SIG="$(cat "$SIG_FILE" 2>/dev/null)"
  HMAC_OK="$(SK="$SIGNING_KEY" BF="$BODY_FILE" RS="$RECV_SIG" python3 - <<'PY'
import hmac, hashlib, os
key = os.environ["SK"].encode()
body = open(os.environ["BF"], "rb").read()
expected = "sha256=" + hmac.new(key, body, hashlib.sha256).hexdigest()
print("yes" if hmac.compare_digest(expected, os.environ["RS"]) else "no")
PY
)"
  if [ "$HMAC_OK" != "yes" ]; then
    note_fail "HMAC mismatch — X-Termlink-Signature does not verify over the received body (sig='$RECV_SIG')" 6
  fi
fi

# ---- B. TOPIC-FILTER negative: non-matching post delivers nothing -----------
if [ "$FAIL" -eq 0 ]; then
  "$BIN" channel create "$NOMATCH_TOPIC" --hub "$HUBADDR" >/dev/null 2>&1 || true
  CNT_PRE_NEG="$(sink_count)"
  "$BIN" channel post "$NOMATCH_TOPIC" --payload "should-not-fan-out-$$" --hub "$HUBADDR" >/dev/null 2>&1
  sleep 3
  if [ "$(sink_count)" -ne "$CNT_PRE_NEG" ]; then
    note_fail "non-matching topic '$NOMATCH_TOPIC' delivered to the sink (topic filter leaked)" 7
  fi
fi

# ---- C. SSRF deny-by-default: non-allowlisted host refused, no delivery ------
SSRF_OUT=""; SSRF_RC=0
if [ "$FAIL" -eq 0 ]; then
  CNT_PRE_SSRF="$(sink_count)"
  SSRF_OUT="$("$BIN" webhook test --url "http://169.254.169.254/latest/meta-data/" \
      --signing-key "$SIGNING_KEY" --topic demo --config "$CFG" 2>&1)" && SSRF_RC=0 || SSRF_RC=$?
  # must refuse (non-zero) AND mention the allowlist AND deliver nothing
  if [ "$SSRF_RC" -eq 0 ]; then
    note_fail "webhook test to 169.254.169.254 SUCCEEDED — SSRF guard did not refuse" 8
  elif ! printf '%s' "$SSRF_OUT" | grep -qiE 'allowlist|allowed|ssrf|not allow'; then
    note_fail "webhook test refused (rc=$SSRF_RC) but message lacks an allowlist reason: $SSRF_OUT" 8
  elif [ "$(sink_count)" -ne "$CNT_PRE_SSRF" ]; then
    note_fail "SSRF path still delivered to the sink (guard fired AFTER a network call)" 8
  fi
fi

# ---- report ----------------------------------------------------------------
echo "=== arc-004 webhook fan-out isolated-hub demo (T-2343, proves T-2332/T-2333) ==="
echo "binary:              $BIN"
echo "hub:                 $HUBADDR   (isolated, TERMLINK_WEBHOOK_CONFIG loaded=$WEBHOOK_ENABLED)"
echo "sink:                http://127.0.0.1:$SINK_PORT/hook   (loopback, torn down on exit)"
echo "match topic:         $MATCH_TOPIC   (target topic filter)"
echo "positive delivery:   sink POSTs ${CNT0:-?} -> $(sink_count)   (>=1 signed POST on the matching topic)"
echo "HMAC verified:       $HMAC_OK   (recomputed sha256 over raw body == X-Termlink-Signature)"
echo "topic-filter negative: non-matching '$NOMATCH_TOPIC' -> no new delivery"
echo "SSRF refused:        rc=$SSRF_RC  $(printf '%s' "$SSRF_OUT" | head -1)"
echo
if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS — a matching channel.post fanned out a signed POST whose HMAC verifies"
  echo "        over the wire; a non-matching topic delivered nothing (filter holds); and a"
  echo "        non-allowlisted host was refused by the SSRF guard with no network delivery."
  exit 0
fi
echo "--- hub/sink log (tail) ---"; tail -20 "$HUBLOG" 2>/dev/null
exit "${RC:-1}"
