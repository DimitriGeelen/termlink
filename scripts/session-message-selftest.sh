#!/usr/bin/env bash
# session-message-selftest.sh — prove Claude-session -> Claude-session messaging end to end.
#
# The fourth prover. The existing three cover the charter's other rails:
#   comms-selftest.sh    (T-2482) TermLink discover + exchange durable messages
#   session-selftest.sh  (T-2485) TermLink control terminal sessions (PTY inject)
#   substrate-smoke.sh   (T-2151) claim work
# None of them touch the rail Claude Code sessions actually use to reach each other,
# which is the rail T-2875 found a peer agent had declared structurally impossible
# while it was working the whole time.
#
# WHY IT ASSERTS ON THE RECEIVER
# SendMessage returns {"success":true} the moment the message is queued. That value
# says nothing about whether the target received or acted on it. Three findings in two
# days share this exact shape: a hub reported "injected" while the PTY got nothing
# (T-2873); a config looked authoritative and was never read (T-2874); a send returned
# success while the target sat blocked on a permission prompt (T-2875). So this script
# never trusts the sender. It reads the RECEIVER's own transcript.
#
# THE THREE OUTCOMES THAT LOOK IDENTICAL FROM THE SENDER
#   DELIVERED   sentinel present as a `user` turn  -> the rail works
#   ENQUEUED    sentinel present only as `queue-operation` -> accepted, not yet drained
#   BLOCKED     absent, and the target is stuck on a permission prompt -> DELIVERY IS
#               FINE; the target cannot act. Reported separately because calling this
#               "not delivered" is the misdiagnosis this script exists to prevent.
#   UNDELIVERED absent, target not blocked -> the rail is broken
#
# NOT marked `# guard-layer: source`. `prepare` spawns a real background Claude session
# and `cleanup` stops it — side effects the guard-layer runner must not have. `assert`
# alone is a pure read and is safe anywhere.
#
# Usage:
#   scripts/session-message-selftest.sh prepare [--sentinel TEXT] [--dir DIR] [--json]
#   scripts/session-message-selftest.sh assert --session-id ID --sentinel TEXT \
#                                              [--id SHORT_ID] [--timeout-secs N] [--json]
#   scripts/session-message-selftest.sh cleanup --id SHORT_ID
#
# The send between prepare and assert has no CLI verb (verified: `claude --help` has no
# send/message/peer subcommand), so it must be an agent tool call. This script does not
# pretend to perform it — `prepare` prints the exact recipient and sentinel to use.
#
# Exit: 0 proven | 1 broken (names the stage) | 2 tooling error (fail-closed)

set -uo pipefail

RC_OK=0; RC_BROKEN=1; RC_TOOLING=2

SUB="${1:-}"; [ $# -gt 0 ] && shift

SENTINEL=""; DIR=""; SESSION_ID=""; SHORT_ID=""; TIMEOUT_SECS=120; JSON=0

while [ $# -gt 0 ]; do
  case "$1" in
    --sentinel)     SENTINEL="${2:-}"; shift 2 ;;
    --dir)          DIR="${2:-}"; shift 2 ;;
    --session-id)   SESSION_ID="${2:-}"; shift 2 ;;
    --id)           SHORT_ID="${2:-}"; shift 2 ;;
    --timeout-secs) TIMEOUT_SECS="${2:-}"; shift 2 ;;
    --json)         JSON=1; shift ;;
    -h|--help)      sed -n '1,40p' "$0"; exit $RC_OK ;;
    *) echo "session-message-selftest: unknown arg: $1" >&2; exit $RC_TOOLING ;;
  esac
done

die_tooling() { echo "session-message-selftest: TOOLING: $*" >&2; exit $RC_TOOLING; }

command -v python3 >/dev/null 2>&1 || die_tooling "python3 not found"

# `claude` is only needed for the subcommands that shell out to it; `assert` can run
# against a fixture transcript with no binary present at all.
need_claude() { command -v claude >/dev/null 2>&1 || die_tooling "claude not found on PATH"; }

agents_json() {
  # Test seam (PL-213): canned `claude agents --json`.
  if [ -n "${SESSION_MSG_TEST_AGENTS_JSON:-}" ]; then
    [ -f "$SESSION_MSG_TEST_AGENTS_JSON" ] || die_tooling "test agents json not found: $SESSION_MSG_TEST_AGENTS_JSON"
    cat "$SESSION_MSG_TEST_AGENTS_JSON"
  else
    need_claude
    timeout 60 claude agents --json 2>/dev/null
  fi
}

transcript_root() {
  if [ -n "${SESSION_MSG_TEST_TRANSCRIPT_DIR:-}" ]; then
    echo "$SESSION_MSG_TEST_TRANSCRIPT_DIR"
  else
    echo "$HOME/.claude/projects"
  fi
}

# ── prepare ──────────────────────────────────────────────────────────────────────
cmd_prepare() {
  need_claude
  [ -n "$SENTINEL" ] || SENTINEL="SMS-PROBE-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  [ -n "$DIR" ] || DIR="${TMPDIR:-/tmp}/session-msg-selftest-$$"
  mkdir -p "$DIR" || die_tooling "cannot create scratch dir: $DIR"

  # The target is deliberately told to do NOTHING. Assertion is on its transcript, so
  # the prover measures the RAIL rather than the target's willingness to obey — and it
  # cannot be defeated by a target that lacks permission to act (the T-2875 false
  # negative, which is precisely what made the rail look broken when it was not).
  local out rc
  out=$(cd "$DIR" && timeout 120 claude --bg \
        'You are a passive probe target for a messaging self-test. Reply with the single word READY and then do nothing further. Do not write files. Do not run commands.' 2>&1)
  rc=$?
  [ $rc -eq 0 ] || die_tooling "claude --bg failed (rc=$rc): $(echo "$out" | tail -3)"

  # `claude --bg` prints "backgrounded · <id>" with ANSI colour; strip it before matching.
  local id
  id=$(printf '%s' "$out" | sed 's/\x1b\[[0-9;]*m//g' | grep -oE 'backgrounded[^0-9a-f]*[0-9a-f]{8}' | grep -oE '[0-9a-f]{8}$' | head -1)
  [ -n "$id" ] || die_tooling "could not parse session id from: $(echo "$out" | tail -3)"

  # Resolve the full sessionId (the transcript filename) and the addressable name.
  local meta
  meta=$(agents_json | SMS_ID="$id" python3 -c '
import json,os,sys
want=os.environ["SMS_ID"]
try: d=json.load(sys.stdin)
except Exception: sys.exit(3)
for a in d:
    if str(a.get("id","")).startswith(want):
        print(json.dumps({"session_id":a.get("sessionId"),"name":a.get("name"),"cwd":a.get("cwd")})); sys.exit(0)
sys.exit(4)') || die_tooling "spawned session $id did not appear in 'claude agents --json'"

  local sid name
  sid=$(printf '%s' "$meta" | python3 -c 'import json,sys;print(json.load(sys.stdin)["session_id"] or "")')
  name=$(printf '%s' "$meta" | python3 -c 'import json,sys;print(json.load(sys.stdin)["name"] or "")')
  [ -n "$sid" ] || die_tooling "session $id has no sessionId yet"

  if [ "$JSON" = "1" ]; then
    printf '{"ok":true,"stage":"PREPARE","id":"%s","session_id":"%s","name":"%s","sentinel":"%s","dir":"%s"}\n' \
      "$id" "$sid" "$name" "$SENTINEL" "$DIR"
  else
    echo "PREPARE=PASS  target spawned"
    echo "  id:          $id"
    echo "  session_id:  $sid"
    echo "  name:        ${name:-(not named yet)}"
    echo "  sentinel:    $SENTINEL"
    echo
    echo "Now SEND (agent tool call — there is no CLI verb for this):"
    echo "  SendMessage to=\"${name:-<name from ListAgents>}\" message=\"$SENTINEL\""
    echo
    echo "Then ASSERT:"
    echo "  bash scripts/session-message-selftest.sh assert --session-id $sid --sentinel $SENTINEL --id $id"
  fi
  return $RC_OK
}

# ── assert ───────────────────────────────────────────────────────────────────────
cmd_assert() {
  [ -n "$SESSION_ID" ] || die_tooling "assert requires --session-id"
  [ -n "$SENTINEL" ]   || die_tooling "assert requires --sentinel"
  case "$TIMEOUT_SECS" in ''|*[!0-9]*) die_tooling "--timeout-secs must be an integer" ;; esac

  local root; root=$(transcript_root)
  [ -d "$root" ] || die_tooling "transcript root not found: $root"

  local deadline=$(( $(date +%s) + TIMEOUT_SECS ))
  local verdict="" detail=""

  while :; do
    verdict=$(SMS_ROOT="$root" SMS_SID="$SESSION_ID" SMS_SENT="$SENTINEL" python3 -c '
import glob,json,os,sys
root=os.environ["SMS_ROOT"]; sid=os.environ["SMS_SID"]; sent=os.environ["SMS_SENT"]
files=glob.glob(os.path.join(root,"*",sid+".jsonl"))
if not files:
    print("NO_TRANSCRIPT"); sys.exit(0)
user=False; queued=False
for f in files:
    try: fh=open(f,encoding="utf-8",errors="replace")
    except OSError: print("UNREADABLE"); sys.exit(0)
    with fh:
        for line in fh:
            if sent not in line: continue
            try: d=json.loads(line)
            except Exception: continue
            t=d.get("type")
            if t=="user": user=True
            elif t=="queue-operation": queued=True
print("DELIVERED" if user else ("ENQUEUED" if queued else "ABSENT"))
') || die_tooling "transcript scan failed"

    case "$verdict" in
      DELIVERED) break ;;
      UNREADABLE) die_tooling "transcript exists but is unreadable for session $SESSION_ID" ;;
    esac
    [ "$(date +%s)" -ge "$deadline" ] && break
    sleep 3
  done

  # A missing transcript after the timeout is a tooling fault, not a rail verdict:
  # we cannot see the receiver, so we must not claim anything about delivery.
  if [ "$verdict" = "NO_TRANSCRIPT" ]; then
    die_tooling "no transcript for session $SESSION_ID under $root — cannot observe the receiver"
  fi

  # Only when the sentinel is absent do we ask whether the target is merely stuck.
  # Delivered-but-blocked is a DIFFERENT fault from not-delivered and must not be
  # collapsed into it — collapsing them is the misdiagnosis this prover prevents.
  local blocked_note=""
  if [ "$verdict" = "ABSENT" ] && [ -n "$SHORT_ID" ]; then
    blocked_note=$(agents_json 2>/dev/null | SMS_ID="$SHORT_ID" python3 -c '
import json,os,sys
want=os.environ["SMS_ID"]
try: d=json.load(sys.stdin)
except Exception: sys.exit(0)
for a in d:
    if str(a.get("id","")).startswith(want):
        w=a.get("waitingFor"); st=a.get("state")
        if w or st=="blocked": print(w or st)
        break
')
    [ -n "$blocked_note" ] && verdict="BLOCKED" && detail="$blocked_note"
  fi

  local rc stage
  case "$verdict" in
    DELIVERED) rc=$RC_OK;     stage="DELIVER" ;;
    ENQUEUED)  rc=$RC_BROKEN; stage="DRAIN" ;;
    BLOCKED)   rc=$RC_BROKEN; stage="TARGET" ;;
    *)         rc=$RC_BROKEN; stage="DELIVER"; verdict="UNDELIVERED" ;;
  esac

  if [ "$JSON" = "1" ]; then
    printf '{"ok":%s,"stage":"%s","verdict":"%s","detail":"%s","session_id":"%s","sentinel":"%s"}\n' \
      "$([ $rc -eq 0 ] && echo true || echo false)" "$stage" "$verdict" "$detail" "$SESSION_ID" "$SENTINEL"
  else
    case "$verdict" in
      DELIVERED)   echo "DELIVER=PASS  sentinel is a user turn in the receiver's transcript — the rail works" ;;
      ENQUEUED)    echo "DRAIN=FAIL    accepted but never drained: sentinel is only a queue-operation, no user turn" ;;
      BLOCKED)     echo "TARGET=FAIL   DELIVERY IS FINE — the target is stuck: ${detail}"
                   echo "              Do not read this as a messaging failure. Clear the prompt or relaunch the"
                   echo "              target with permissions that let it act." ;;
      UNDELIVERED) echo "DELIVER=FAIL  sentinel never reached the receiver's transcript within ${TIMEOUT_SECS}s" ;;
    esac
  fi
  return $rc
}

# ── cleanup ──────────────────────────────────────────────────────────────────────
cmd_cleanup() {
  [ -n "$SHORT_ID" ] || die_tooling "cleanup requires --id"
  need_claude
  timeout 60 claude stop "$SHORT_ID" >/dev/null 2>&1
  echo "CLEANUP=done  stopped $SHORT_ID"
  return $RC_OK
}

case "$SUB" in
  prepare) cmd_prepare ;;
  assert)  cmd_assert ;;
  cleanup) cmd_cleanup ;;
  ""|-h|--help) sed -n '1,40p' "$0"; exit $RC_OK ;;
  *) echo "session-message-selftest: unknown subcommand: $SUB (want prepare|assert|cleanup)" >&2; exit $RC_TOOLING ;;
esac
