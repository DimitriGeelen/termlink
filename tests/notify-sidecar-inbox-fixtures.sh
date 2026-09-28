#!/usr/bin/env bash
# guard-layer: source
#
# T-3203 — fixtures for notify-sidecar's mail-topic enumeration.
#
# probe_mail writes last_mail_ts, and notify-injector keys on THAT flag advancing,
# not on the journal. So a topic missing from this enumeration cannot wake the
# session however well it is mirrored. T-3201 fixed journal-mirror's identical
# dm:-only blindness (journal went 0 -> 50 rows) and the rail still did not
# deliver, because the arrival flag is written here.
#
# These run the REAL script with a stub `termlink` on the TERMLINK_BIN seam, so
# they exercise the shipped jq selectors rather than a copy of them.
set -uo pipefail

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
SCRIPT="$PROJECT_ROOT/scripts/notify-sidecar.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL+1)); printf '  FAIL %s\n' "$1"; }

[ -r "$SCRIPT" ] || { echo "TOOLING: cannot read $SCRIPT" >&2; exit 2; }
command -v jq >/dev/null || { echo "TOOLING: jq missing" >&2; exit 2; }

TMP=$(mktemp -d); trap 'rm -rf "$TMP"' EXIT

SELF_FP="aaaabbbbccccdddd"

# Stub termlink.
#   channel list      — answers per --prefix
#   channel unread    — 1 unread per topic, and RECORDS the topic it was asked about
#                       in $PROBED. That file is the evidence of what was enumerated.
#   whoami / post / subscribe — enough to keep the cycle from erroring out.
cat > "$TMP/termlink" <<STUB
#!/usr/bin/env bash
PROBED="$TMP/probed.txt"
STUB
cat >> "$TMP/termlink" <<'STUB'
verb="${1:-}"; sub="${2:-}"
if [ "$verb" = "whoami" ]; then
    echo '{"identity_fingerprint":"aaaabbbbccccdddd"}'; exit 0
fi
if [ "$verb" = "channel" ] && [ "$sub" = "list" ]; then
    prefix=""
    while [ $# -gt 0 ]; do
        case "$1" in --prefix) prefix="${2:-}"; shift 2 ;; *) shift ;; esac
    done
    case "$prefix" in
      "dm:")    echo '{"topics":[
                    {"name":"dm:aaaabbbbccccdddd:9999"},
                    {"name":"dm:7777:8888"}]}' ;;
      "inbox:") echo '{"topics":[
                    {"name":"inbox:cid/010-termlink"},
                    {"name":"inbox:cid/010-termlink/worker-1"},
                    {"name":"inbox:cid/999-Agentic-Engineering-Framework"},
                    {"name":"inbox:cid/999-Agentic-Engineering-Framework/e2e-dead"},
                    {"name":"inbox:cid/832-Workflow-designer"}]}' ;;
      *)        echo '{"topics":[]}' ;;
    esac
    exit 0
fi
if [ "$verb" = "channel" ] && [ "$sub" = "unread" ]; then
    printf '%s\n' "${3:-}" >> "$PROBED"
    echo '{"unread_count":1,"last_offset":5}'
    exit 0
fi
# subscribe returns no content, so _auto_confirm_topic bails before posting.
exit 0
STUB
chmod +x "$TMP/termlink"

run_probe() {
    rm -f "$TMP/probed.txt"; : > "$TMP/probed.txt"
    TERMLINK_BIN="$TMP/termlink" \
    FW_SIDECAR_SELF_PROJECT="${SELF_OVERRIDE:-010-termlink}" \
        bash "$SCRIPT" --agent-id fixture-agent --self-fp "$SELF_FP" \
            --notify-dir "$TMP/notify" --once "$@" > "$TMP/out.txt" 2>&1
    PROBE_RC=$?
    sort -u "$TMP/probed.txt" 2>/dev/null
}

echo "case 1: our own inbox mailbox is probed (the T-3203 defect)"
got="$(run_probe)"
grep -qx "inbox:cid/010-termlink" <<<"$got" \
  && ok "inbox:<self> probed" \
  || bad "inbox:<self> NOT probed — last_mail_ts can never advance on inbox mail"

echo "case 2: a sub-address under us is probed"
grep -qx "inbox:cid/010-termlink/worker-1" <<<"$got" \
  && ok "inbox:<self>/<sub> probed" || bad "sub-address not probed"

echo "case 3: another project's mailbox is NOT probed"
for peer in "inbox:cid/999-Agentic-Engineering-Framework" \
            "inbox:cid/999-Agentic-Engineering-Framework/e2e-dead" \
            "inbox:cid/832-Workflow-designer"; do
    if grep -qx "$peer" <<<"$got"; then
        bad "PROBED ANOTHER PROJECT'S MAILBOX: $peer"
    else
        ok "rejected $peer"
    fi
done

echo "case 4: dm: selection is unchanged and still keyed on the FINGERPRINT"
grep -qx "dm:aaaabbbbccccdddd:9999" <<<"$got" \
  && ok "dm: topic containing self fp still probed" || bad "dm: REGRESSED"
grep -qx "dm:7777:8888" <<<"$got" \
  && bad "probed a dm: topic this fp is not party to" \
  || ok "dm: topic without self fp still rejected"

echo "case 5: the two selectors are keyed differently, and inbox does not use the fp"
# The inbox topics carry no fingerprint at all. If someone 'tidies' the inbox arm to
# reuse the dm fp predicate it matches nothing — and looks exactly like a working fix.
inbox_hits=$(grep -c '^inbox:' <<<"$got" || true)
[ "$inbox_hits" -eq 2 ] \
  && ok "exactly the 2 self-addressed inbox topics (fp predicate would yield 0)" \
  || bad "expected 2 self inbox topics, got $inbox_hits"

echo "case 6: self-identity is an env-overridable constant, not the checkout name"
got2="$(SELF_OVERRIDE="999-Agentic-Engineering-Framework" run_probe)"
grep -qx "inbox:cid/999-Agentic-Engineering-Framework" <<<"$got2" \
  && ok "override selects the declared project" || bad "override ignored"
grep -qx "inbox:cid/010-termlink" <<<"$got2" \
  && bad "override did not deselect the default" || ok "override deselects the default"

echo "case 7: the constant is declared, never derived from the path (T-2815/T-2816)"
if grep -nE '^[^#]*basename' "$SCRIPT" | grep -q 'SELF_PROJECT'; then
    bad "SELF_PROJECT derived via basename — wrong inside a worktree"
else
    ok "SELF_PROJECT is not basename-derived"
fi
grep -q 'FW_SIDECAR_SELF_PROJECT' "$SCRIPT" \
  && ok "env override present" || bad "no FW_SIDECAR_SELF_PROJECT override"

echo "case 8: TERMLINK_NOTIFY_TEST_TOPICS seam still short-circuits enumeration"
rm -f "$TMP/probed.txt"; : > "$TMP/probed.txt"
TERMLINK_BIN="$TMP/termlink" TERMLINK_NOTIFY_TEST_TOPICS="dm:only:this" \
    bash "$SCRIPT" --agent-id fixture-agent --self-fp "$SELF_FP" \
        --notify-dir "$TMP/notify2" --once >/dev/null 2>&1
seam="$(sort -u "$TMP/probed.txt" 2>/dev/null)"
[ "$seam" = "dm:only:this" ] \
  && ok "test seam overrides both arms (no live enumeration leaked in)" \
  || bad "seam broken, probed: '$seam'"

echo "case 9: mirror and sidecar read the SAME self-identity variable"
# If these two ever disagree, one half mirrors mail the other refuses to notice —
# which is the exact shape of the bug this task closes.
if grep -q 'FW_SIDECAR_SELF_PROJECT' "$PROJECT_ROOT/scripts/journal-mirror.sh"; then
    ok "journal-mirror.sh reads the same constant"
else
    bad "journal-mirror.sh uses a DIFFERENT self-identity source — halves can diverge"
fi

echo ""
echo "passed: $PASS   failed: $FAIL"
[ "$FAIL" -eq 0 ] || exit 1
exit 0
