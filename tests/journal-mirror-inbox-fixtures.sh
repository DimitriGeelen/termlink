#!/usr/bin/env bash
# guard-layer: source
#
# T-3201 — fixtures for journal-mirror's topic enumeration.
#
# The mirror is the SOLE ingest point for the journal the arc-011 injector reads.
# A topic it does not enumerate is invisible to the entire rail — which is exactly
# how 49 AEF consults went unread for weeks after they moved from dm: to
# inbox:<circuit>/010-termlink. Measured before the fix: 0 rows on inbox:%,
# 2255 on dm:%.
#
# These run against the REAL script with a stub `termlink` on the TERMLINK_BIN
# seam, so they exercise the shipped jq selector rather than a copy of it.
set -uo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/journal-mirror.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
command -v jq       >/dev/null || { echo "TOOLING: jq missing" >&2; exit 2; }
command -v sqlite3  >/dev/null || { echo "TOOLING: sqlite3 missing" >&2; exit 2; }
command -v python3  >/dev/null || { echo "TOOLING: python3 missing" >&2; exit 2; }

TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

# Stub termlink. `channel list` answers per --prefix; `channel subscribe` returns
# exactly one envelope whose payload names the topic, so the journal tells us
# precisely which topics were enumerated.
cat > "$TMP/termlink" <<'STUB'
#!/usr/bin/env bash
sub=""; prefix=""; topic=""
[ "${1:-}" = "channel" ] && sub="${2:-}"
shift 2 2>/dev/null || true
if [ "$sub" = "subscribe" ]; then topic="${1:-}"; fi
while [ $# -gt 0 ]; do
    case "$1" in --prefix) prefix="${2:-}"; shift 2 ;; *) shift ;; esac
done
if [ "$sub" = "list" ]; then
    case "$prefix" in
      "dm:")    echo '{"topics":[{"name":"dm:aaa:bbb"}]}' ;;
      "inbox:") echo '{"topics":[
                    {"name":"inbox:cid/010-termlink"},
                    {"name":"inbox:cid/010-termlink/worker-1"},
                    {"name":"inbox:cid/999-Agentic-Engineering-Framework"},
                    {"name":"inbox:cid/999-Agentic-Engineering-Framework/e2e-dead-sender"},
                    {"name":"inbox:cid/832-Workflow-designer"}]}' ;;
      *)        echo '{"topics":[]}' ;;
    esac
    exit 0
fi
if [ "$sub" = "subscribe" ]; then
    b64=$(printf '%s' "body-for-$topic" | base64 -w0 2>/dev/null || printf '%s' "body-for-$topic" | base64)
    # `topic` is load-bearing: the inserter reads e.get("topic") and a row without
    # it lands under "" — which looks exactly like "nothing was ingested".
    printf '{"topic":"%s","offset":0,"ts":1,"msg_type":"note","sender_id":"s","payload_b64":"%s","metadata":{}}\n' "$topic" "$b64"
    exit 0
fi
exit 0
STUB
chmod +x "$TMP/termlink"

run_mirror() {
    local journal="$1"; shift
    rm -f "$journal"
    TERMLINK_BIN="$TMP/termlink" TERMLINK_JOURNAL_PATH="$journal" \
        FW_SIDECAR_SELF_PROJECT="${SELF_OVERRIDE:-010-termlink}" \
        bash "$SCRIPT" "$@" > "$TMP/out.txt" 2>&1
}
topics_in() { sqlite3 "$1" "select distinct topic from messages order by topic;" 2>/dev/null; }

echo "case 1: our own inbox topic is mirrored (the whole point)"
run_mirror "$TMP/j1.sqlite"
got=$(topics_in "$TMP/j1.sqlite")
grep -q "^inbox:cid/010-termlink$" <<<"$got" \
  && ok "inbox:<self> ingested" || bad "inbox:<self> NOT ingested — the T-3201 defect"

echo "case 2: a sub-address under us is mirrored"
grep -q "^inbox:cid/010-termlink/worker-1$" <<<"$got" \
  && ok "inbox:<self>/<sub> ingested" || bad "sub-address not ingested"

echo "case 3: OTHER projects' mailboxes are NOT mirrored"
for peer in "inbox:cid/999-Agentic-Engineering-Framework" \
            "inbox:cid/999-Agentic-Engineering-Framework/e2e-dead-sender" \
            "inbox:cid/832-Workflow-designer"; do
    if grep -q "^${peer}$" <<<"$got"; then
        bad "MIRRORED ANOTHER PROJECT'S MAIL: $peer"
    else
        ok "rejected $peer"
    fi
done

echo "case 4: dm: behaviour unchanged (this change is additive)"
grep -q "^dm:aaa:bbb$" <<<"$got" && ok "dm: still ingested" || bad "dm: REGRESSED"

echo "case 5: the summary counts every topic it scanned, not just dm:"
grep -q "3 topic(s) scanned" "$TMP/out.txt" \
  && ok "summary counts 3 (1 dm + 2 self-inbox)" \
  || bad "summary miscounts: $(grep -o '[0-9]* topic(s) scanned' "$TMP/out.txt")"

echo "case 6: self-identity is an env-overridable constant, not the checkout name"
SELF_OVERRIDE="999-Agentic-Engineering-Framework" run_mirror "$TMP/j2.sqlite"
got2=$(topics_in "$TMP/j2.sqlite")
grep -q "^inbox:cid/999-Agentic-Engineering-Framework$" <<<"$got2" \
  && ok "override selects the declared project" || bad "override ignored"
grep -q "^inbox:cid/010-termlink$" <<<"$got2" \
  && bad "override did not deselect the default" || ok "override deselects the default"

echo "case 7: --topic single-topic mode is unaffected by the new enumeration"
run_mirror "$TMP/j3.sqlite" --topic "inbox:cid/832-Workflow-designer"
got3=$(topics_in "$TMP/j3.sqlite")
[ "$got3" = "inbox:cid/832-Workflow-designer" ] \
  && ok "--topic still honoured verbatim (bypasses the self filter, as before)" \
  || bad "--topic mode changed: got '$got3'"

echo ""
echo "passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
