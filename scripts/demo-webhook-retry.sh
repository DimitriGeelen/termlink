#!/usr/bin/env bash
# T-2344 (arc-004 push-transport, Candidate B) — webhook RETRY-LOOP E2E regression
# demo against a FLAKY sink. Sibling of demo-webhook-fanout.sh (T-2343), which
# proves the direct-success path; THIS demo proves the T-2334 resilience chain:
#
#   dispatch -> 503 (Retryable via classify_outcome) -> schedule_retry (enqueue)
#     -> spawn_retry_loop drains every TERMLINK_WEBHOOK_RETRY_INTERVAL_MS
#     -> re-dispatch -> 204 -> webhook_retry_success_total increments
#
# Why: the retry mechanics have 18 unit tests, but NO prior smoke/demo ever drove
# a real hub through fail-then-recover (T-2336 + T-2343 test direct 204 only).
# PL-240 (T-2341): a background resilience loop can be unit-green yet UNREACHABLE
# in the shipped wiring — if spawn_retry_loop were unwired, every transient 5xx
# would silently dead-letter. This demo makes that regression fail a script.
#
# The sink returns 503 for the first FAIL_N POSTs, then 204 — and logs every
# request (status served + signature + raw body), so the demo can assert:
#   A. >=1 initial 503-served delivery (the retryable failure actually happened);
#   B. a LATER 204-served delivery of a correctly-signed payload (the background
#      retry loop re-dispatched — nothing else re-sends; fan_out fires once);
#   C. `hub status --governor --json` shows .governor.webhook_retry_success_total
#      >= 1 AND .governor.webhook_enqueued_total >= 1 (T-2335 counters move).
#
# Isolation contract: temp TERMLINK_RUNTIME_DIR + temp HOME + loopback sink;
# never touches the shared :9100 hub or ~/.termlink; teardown on exit.
#
# Usage:   scripts/demo-webhook-retry.sh
# Env:     TERMLINK_BIN             real termlink binary (default target/release/termlink)
#          DEMO_RETRY_HUB_PORT      isolated hub port (default 9202)
#          DEMO_RETRY_SINK_PORT     flaky sink port (default 8802)
# Exit:    0 PASS | 2 binary missing/pre-webhook | 3 hub/sink failed | 4 python3 missing
#          5 no initial 503 delivery | 6 no recovered 204 delivery | 7 HMAC mismatch
#          8 governor retry telemetry did not move
set -uo pipefail

BIN="${TERMLINK_BIN:-target/release/termlink}"
PORT="${DEMO_RETRY_HUB_PORT:-9202}"
SINK_PORT="${DEMO_RETRY_SINK_PORT:-8802}"
HUBADDR="127.0.0.1:${PORT}"
SIGNING_KEY="t2344-retry-demo-key-0123456789abcdef"
TOPIC="webhook-retry-demo-$$"
FAIL_N=2   # sink 503s the first N POSTs, then 204s

if [ ! -x "$BIN" ]; then
  echo "FATAL: termlink binary not found/executable at '$BIN'."; exit 2
fi
if [ "$(grep -a -c 'x-termlink-signature' "$BIN" 2>/dev/null || true)" -lt 1 ]; then
  echo "FATAL: '$BIN' has no webhook subsystem (predates arc-004 Candidate B / T-2332)."
  echo "  rebuild: cargo build --release -p termlink"; exit 2
fi
command -v python3 >/dev/null 2>&1 || { echo "SKIP: python3 required for the sink"; exit 4; }
command -v jq      >/dev/null 2>&1 || { echo "FATAL: jq required"; exit 3; }

RT="$(mktemp -d)"; HM="$(mktemp -d)"; HUBLOG="$(mktemp)"
CFG="$HM/webhooks.json"; SINKPY="$HM/sink.py"
REQLOG="$HM/sink-requests.ndjson"   # one JSON line per POST: {n, status, sig, body_b64}
HUB_PID=""; SINK_PID=""
cleanup() {
  [ -n "$SINK_PID" ] && kill "$SINK_PID" 2>/dev/null || true
  [ -n "$HUB_PID" ]  && kill "$HUB_PID"  2>/dev/null || true
  rm -rf "$RT" "$HM" "$HUBLOG" 2>/dev/null || true
}
trap cleanup EXIT

export TERMLINK_RUNTIME_DIR="$RT"
export HOME="$HM"

FAIL=0; RC=0
note_fail() { echo "FAIL: $1"; FAIL=1; RC="$2"; }
req_count() { local n; n=$(wc -l < "$REQLOG" 2>/dev/null | tr -d ' ') || true; echo "${n:-0}"; }

# ---- flaky sink: 503 for the first FAIL_N POSTs, then 204; logs every request --
cat > "$SINKPY" <<'PY'
import base64, http.server, json, os, sys
LOG = os.environ["REQLOG"]; FAIL_N = int(os.environ["FAIL_N"])
count = {"n": 0}
class H(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        count["n"] += 1
        n = int(self.headers.get("content-length", 0) or 0)
        body = self.rfile.read(n) if n else b""
        status = 503 if count["n"] <= FAIL_N else 204
        with open(LOG, "a") as f:
            f.write(json.dumps({"n": count["n"], "status": status,
                                "sig": self.headers.get("x-termlink-signature", ""),
                                "body_b64": base64.b64encode(body).decode()}) + "\n")
        self.send_response(status); self.end_headers()
    def log_message(self, *a): pass
http.server.HTTPServer(("127.0.0.1", int(sys.argv[1])), H).serve_forever()
PY
: > "$REQLOG"
REQLOG="$REQLOG" FAIL_N="$FAIL_N" python3 "$SINKPY" "$SINK_PORT" >>"$HUBLOG" 2>&1 &
SINK_PID=$!
for _ in $(seq 1 50); do
  (exec 3<>"/dev/tcp/127.0.0.1/$SINK_PORT") 2>/dev/null && { exec 3>&- 3<&-; break; }
  sleep 0.1
done

# ---- config + isolated hub with a FAST retry interval -----------------------
cat > "$CFG" <<EOF
{
  "allowed_hosts": ["127.0.0.1"],
  "targets": [
    { "url": "http://127.0.0.1:${SINK_PORT}/hook", "signing_key": "${SIGNING_KEY}", "topics": ["${TOPIC}"] }
  ]
}
EOF
export TERMLINK_WEBHOOK_CONFIG="$CFG"
export TERMLINK_WEBHOOK_RETRY_INTERVAL_MS=500   # clamp floor is 250; default 2000
rm -f "$RT/hub.sock" "$RT/hub.pid" 2>/dev/null || true
"$BIN" hub start --tcp "$HUBADDR" >>"$HUBLOG" 2>&1 &
HUB_PID=$!
for _ in $(seq 1 100); do
  [ -s "$RT/hub.secret" ] && [ -s "$RT/hub.cert.pem" ] && break
  sleep 0.1
done
[ -s "$RT/hub.secret" ] || { echo "FATAL: isolated hub did not start"; cat "$HUBLOG"; exit 3; }
mkdir -p "$HM/.termlink"
cat > "$HM/.termlink/hubs.toml" <<EOF
[hubs.demo-retry]
address = "$HUBADDR"
secret_file = "$RT/hub.secret"
EOF

# ---- ONE post; the sink 503s it; ONLY the retry loop can produce the 204 ----
"$BIN" channel create "$TOPIC" --hub "$HUBADDR" >/dev/null 2>&1 || true
"$BIN" channel post "$TOPIC" --payload "retry-demo-$$" --hub "$HUBADDR" >/dev/null 2>&1

# Wait for: >=1 503-served request AND >=1 204-served request (recovery).
# Backoff for attempt 1 is ~2s (+/- jitter) then the 500ms drain tick — allow ~30s.
GOT_503=0; GOT_204=0
for _ in $(seq 1 300); do
  GOT_503=$(grep -c '"status": 503' "$REQLOG" 2>/dev/null) || GOT_503=0
  GOT_204=$(grep -c '"status": 204' "$REQLOG" 2>/dev/null) || GOT_204=0
  [ "$GOT_503" -ge 1 ] && [ "$GOT_204" -ge 1 ] && break
  sleep 0.1
done
if [ "$GOT_503" -lt 1 ]; then
  note_fail "sink never served a 503 — the flaky phase did not engage (posts seen: $(req_count))" 5
elif [ "$GOT_204" -lt 1 ]; then
  note_fail "no 204-served delivery after the 503s — the retry loop never re-dispatched (posts seen: $(req_count))" 6
fi

# ---- HMAC verification on the FINAL (204-served, retried) body --------------
HMAC_OK="n/a"
if [ "$FAIL" -eq 0 ]; then
  FINAL="$(grep '"status": 204' "$REQLOG" | tail -1)"
  HMAC_OK="$(SK="$SIGNING_KEY" LINE="$FINAL" python3 - <<'PY'
import base64, hashlib, hmac, json, os
row = json.loads(os.environ["LINE"])
key = os.environ["SK"].encode(); body = base64.b64decode(row["body_b64"])
expected = "sha256=" + hmac.new(key, body, hashlib.sha256).hexdigest()
print("yes" if hmac.compare_digest(expected, row["sig"]) else "no")
PY
)"
  [ "$HMAC_OK" = "yes" ] || note_fail "HMAC mismatch on the retried (204-served) delivery" 7
fi

# ---- governor telemetry: retry counters moved (T-2335) ----------------------
RETRY_OK="n/a"; ENQ="n/a"
if [ "$FAIL" -eq 0 ]; then
  GOV="$("$BIN" hub status --governor --json 2>/dev/null)"
  RETRY_OK="$(printf '%s' "$GOV" | jq -r '.governor.webhook_retry_success_total // "n/a"')"
  ENQ="$(printf '%s' "$GOV" | jq -r '.governor.webhook_enqueued_total // "n/a"')"
  if ! [ "$RETRY_OK" -ge 1 ] 2>/dev/null || ! [ "$ENQ" -ge 1 ] 2>/dev/null; then
    note_fail "governor retry telemetry did not move (retry_success_total=$RETRY_OK enqueued_total=$ENQ)" 8
  fi
fi

# ---- report ----------------------------------------------------------------
echo "=== arc-004 webhook retry-loop E2E demo (T-2344, proves T-2334 wiring) ==="
echo "binary:              $BIN"
echo "hub:                 $HUBADDR   (isolated, retry interval 500ms)"
echo "flaky sink:          http://127.0.0.1:$SINK_PORT/hook   (503 x$FAIL_N then 204)"
echo "deliveries:          503-served=$GOT_503  204-served=$GOT_204  total=$(req_count)"
echo "HMAC on retried:     $HMAC_OK   (signature verifies on the 204-served body)"
echo "governor telemetry:  webhook_retry_success_total=$RETRY_OK  webhook_enqueued_total=$ENQ"
echo
if [ "$FAIL" -eq 0 ]; then
  echo "RESULT: PASS — the first dispatch failed 503 (Retryable), the BACKGROUND retry"
  echo "        loop re-dispatched the same signed payload until the sink recovered (204),"
  echo "        and the T-2335 retry counters moved. spawn_retry_loop wiring is LIVE."
  exit 0
fi
echo "--- sink request log ---"; cat "$REQLOG" 2>/dev/null
echo "--- hub log (tail) ---"; tail -15 "$HUBLOG" 2>/dev/null
exit "${RC:-1}"
