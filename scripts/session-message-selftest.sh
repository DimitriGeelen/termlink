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
# THE OUTCOMES THAT ARE BYTE-IDENTICAL FROM THE SENDER'S SIDE
#   DELIVERED    sentinel is a `user` turn                  -> the rail works        (0)
#   BLOCKED      sentinel is queued, target is at rest      -> DELIVERY WAS FINE     (1)
#   ENQUEUED     sentinel is queued, target is still busy   -> may yet drain; retry  (1)
#   UNDELIVERED  sentinel is absent from the transcript     -> the rail is broken    (1)
#
# The BLOCKED/UNDELIVERED split is the whole point. Collapsing them is the T-2875
# misdiagnosis — a working rail declared impossible because the target could not act.
#
# HOW "BLOCKED" IS ACTUALLY DETECTED, AND HOW IT IS NOT
# It is read from the RECEIVER'S TRANSCRIPT: a `queue-operation` carrying the sentinel
# with no matching `user` turn means the message was accepted and never drained.
#
# It is deliberately NOT read from a `waitingFor` field in `claude agents --json`.
# That field DOES NOT EXIST — measured across the live fleet, 0 of 56 agents carried
# it. An earlier draft of this script depended on it, which made the entire blocked
# branch dead code. Nor can `state` carry the signal on its own: `state:"blocked"` is
# the ordinary RESTING state of a finished background session (43 of those same 56),
# so keying off it would classify almost every broken rail as "target merely stuck" —
# the exact inversion of the bug this prover exists to prevent. `state`/`status` are
# used only to ANNOTATE an already-queued message as at-rest vs still-busy, never to
# decide whether delivery happened.
#
# WHAT MAKES A TARGET REACHABLE (measured, T-2876)
# A session is addressable by SendMessage only if its `claude agents --json` record
# carries a `status` field. That set matched the SendMessage peer list exactly (15 of
# 58 records). The other 43 records are listed but unreachable — `claude --bg` produces
# exactly such a record: it appears in `agents --json` with a `state` and NO `status`,
# it never writes a transcript, and SendMessage answers "No agent named ... is
# reachable". So `prepare --spawn` GATES on reachability and fails closed rather than
# handing back an address that cannot receive anything; `prepare --existing` targets a
# session that is already reachable, which is the realistic way to run the live pass.
#
# NOT marked `# guard-layer: source`. `prepare` spawns a real background Claude session
# and `cleanup` stops it — side effects the guard-layer runner must not have. `assert`
# alone is a pure read and is safe anywhere.
#
# Usage:
#   scripts/session-message-selftest.sh prepare --existing MATCH [--sentinel TEXT] [--json]
#   scripts/session-message-selftest.sh prepare --spawn [--sentinel TEXT] [--dir DIR] [--json]
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
EXISTING=""; SPAWN=0; READY_SECS=60

while [ $# -gt 0 ]; do
  case "$1" in
    --sentinel)     SENTINEL="${2:-}"; shift 2 ;;
    --dir)          DIR="${2:-}"; shift 2 ;;
    --session-id)   SESSION_ID="${2:-}"; shift 2 ;;
    --id)           SHORT_ID="${2:-}"; shift 2 ;;
    --timeout-secs) TIMEOUT_SECS="${2:-}"; shift 2 ;;
    --existing)     EXISTING="${2:-}"; shift 2 ;;
    --spawn)        SPAWN=1; shift ;;
    --ready-secs)   READY_SECS="${2:-}"; shift 2 ;;
    --json)         JSON=1; shift ;;
    -h|--help)      sed -n '1,68p' "$0"; exit $RC_OK ;;
    *) echo "session-message-selftest: unknown arg: $1" >&2; exit $RC_TOOLING ;;
  esac
done

die_tooling() { echo "session-message-selftest: TOOLING: $*" >&2; exit $RC_TOOLING; }

command -v python3 >/dev/null 2>&1 || die_tooling "python3 not found"

# `claude` is only needed for the subcommands that shell out to it; `assert` can run
# against a fixture transcript with no binary present at all.
need_claude() { command -v claude >/dev/null 2>&1 || die_tooling "claude not found on PATH"; }

agents_json() {
  # Test seam (PL-213): canned `claude agents --json`. A seam path that does not
  # exist is a TOOLING fault, never a silent fallback to the live binary — a
  # fixture run that quietly probed the real fleet would prove nothing.
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
#
# Emits the reachable recipient name + sentinel for the send, and the sessionId the
# assert step reads. Two modes:
#   --existing MATCH  resolve an ALREADY-reachable session (id prefix or name substring)
#   --spawn           start a fresh probe target, then WAIT for it to become reachable
#
# Both gate on the same measured predicate: a `status` field in the agent's record.
# Handing back an unreachable address would make the send fail for a reason that has
# nothing to do with the rail under test.

# Emit {session_id,name,id} for the one reachable agent matching $1, else fail.
resolve_reachable() {
  local match="$1" agents_out
  agents_out=$(agents_json) || exit $RC_TOOLING
  printf '%s' "$agents_out" | SMS_MATCH="$match" python3 -c '
import json,os,sys
want=os.environ["SMS_MATCH"]
try: d=json.load(sys.stdin)
except Exception: sys.exit(3)
if not isinstance(d,list): sys.exit(3)
# Reachable == the record carries a `status`. Measured T-2876: that set is exactly
# the SendMessage peer list; records without it are listed but unaddressable.
hits=[a for a in d if a.get("status")
      and (str(a.get("id") or "").startswith(want) or want.lower() in str(a.get("name") or "").lower())]
if not hits: sys.exit(4)
if len(hits) > 1:
    sys.stderr.write("matches %d reachable agents: %s\n" %
                     (len(hits), ", ".join(str(a.get("name"))[:40] for a in hits)))
    sys.exit(5)
a=hits[0]
print(json.dumps({"session_id":a.get("sessionId") or "","name":a.get("name") or "",
                  "id":a.get("id") or "","status":a.get("status")}))
'
}

# Poll until $1 (id prefix) is reachable AND has a transcript, or time out.
await_reachable() {
  local id="$1" deadline=$(( $(date +%s) + READY_SECS )) meta="" root
  root=$(transcript_root)
  while :; do
    meta=$(resolve_reachable "$id" 2>/dev/null) && {
      local sid
      sid=$(printf '%s' "$meta" | python3 -c 'import json,sys;print(json.load(sys.stdin)["session_id"])')
      # Reachable is necessary but not sufficient: assert reads the transcript, so a
      # target with no transcript on disk is not yet observable.
      if [ -n "$sid" ] && compgen -G "$root/*/$sid.jsonl" >/dev/null 2>&1; then
        printf '%s' "$meta"; return 0
      fi
    }
    [ "$(date +%s)" -ge "$deadline" ] && return 1
    sleep 3
  done
}

cmd_prepare() {
  [ -n "$SENTINEL" ] || SENTINEL="SMS-PROBE-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  case "$READY_SECS" in ''|*[!0-9]*) die_tooling "--ready-secs must be an integer" ;; esac

  if [ -n "$EXISTING" ] && [ "$SPAWN" = "1" ]; then
    die_tooling "--existing and --spawn are mutually exclusive"
  fi
  if [ -z "$EXISTING" ] && [ "$SPAWN" != "1" ]; then
    die_tooling "prepare needs --existing MATCH (target a reachable session) or --spawn (start one)"
  fi

  local meta id
  if [ -n "$EXISTING" ]; then
    meta=$(resolve_reachable "$EXISTING")
    case $? in
      0) : ;;
      4) die_tooling "no REACHABLE agent matches '$EXISTING' (a listed-but-unreachable session carries no 'status')" ;;
      5) die_tooling "'$EXISTING' is ambiguous — narrow it" ;;
      *) die_tooling "could not read 'claude agents --json'" ;;
    esac
    id=$(printf '%s' "$meta" | python3 -c 'import json,sys;print(json.load(sys.stdin)["id"])')
  else
    need_claude
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
    id=$(printf '%s' "$out" | sed 's/\x1b\[[0-9;]*m//g' | grep -oE '[0-9a-f]{8}' | head -1)
    [ -n "$id" ] || die_tooling "could not parse session id from: $(echo "$out" | tail -3)"

    meta=$(await_reachable "$id") || die_tooling \
      "spawned session $id never became reachable within ${READY_SECS}s (no 'status' in 'claude agents --json' and/or no transcript on disk). Measured T-2876: 'claude --bg' produces exactly this — a listed but unaddressable record. Use --existing to target a session that is already reachable."
  fi

  local sid name
  sid=$(printf '%s' "$meta" | python3 -c 'import json,sys;print(json.load(sys.stdin)["session_id"])')
  name=$(printf '%s' "$meta" | python3 -c 'import json,sys;print(json.load(sys.stdin)["name"])')
  [ -n "$sid" ] || die_tooling "resolved target has no sessionId"

  if [ "$JSON" = "1" ]; then
    SMS_ID="$id" SMS_SID="$sid" SMS_NAME="$name" SMS_SENT="$SENTINEL" SMS_DIR="$DIR" python3 -c '
import json,os
print(json.dumps({"ok":True,"stage":"PREPARE","id":os.environ["SMS_ID"],
                  "session_id":os.environ["SMS_SID"],"name":os.environ["SMS_NAME"],
                  "sentinel":os.environ["SMS_SENT"],"dir":os.environ["SMS_DIR"]}))'
  else
    echo "PREPARE=PASS  target is reachable"
    echo "  id:          ${id:-(interactive session — no short id)}"
    echo "  session_id:  $sid"
    echo "  name:        $name"
    echo "  sentinel:    $SENTINEL"
    echo
    echo "Now SEND (agent tool call — there is no CLI verb for this):"
    echo "  SendMessage to=\"$name\" message=\"$SENTINEL\""
    echo
    echo "Then ASSERT:"
    echo "  bash scripts/session-message-selftest.sh assert --session-id $sid --sentinel $SENTINEL${id:+ --id $id}"
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
  local verdict=""

  while :; do
    # The sentinel is matched against the DECODED text of each record, not against
    # the raw line: a sentinel containing a quote or backslash is JSON-escaped on
    # disk and a raw substring test would silently miss it. Only `user` and
    # `queue-operation` records are considered, so an assistant turn that merely
    # echoes the sentinel can never be read as delivery.
    verdict=$(SMS_ROOT="$root" SMS_SID="$SESSION_ID" SMS_SENT="$SENTINEL" python3 -c '
import glob,json,os,sys
root=os.environ["SMS_ROOT"]; sid=os.environ["SMS_SID"]; sent=os.environ["SMS_SENT"]
files=glob.glob(os.path.join(root,"*",sid+".jsonl"))
if not files:
    print("NO_TRANSCRIPT"); sys.exit(0)
esc=json.dumps(sent, ensure_ascii=False)[1:-1]   # how it appears inside a JSON string

def text_of(d):
    t=d.get("type")
    if t=="queue-operation":
        c=d.get("content")
        return c if isinstance(c,str) else ""
    if t=="user":
        c=(d.get("message") or {}).get("content")
        if isinstance(c,str): return c
        if isinstance(c,list):
            out=[]
            for b in c:
                if isinstance(b,str): out.append(b)
                elif isinstance(b,dict):
                    for k in ("text","content"):
                        if isinstance(b.get(k),str): out.append(b[k])
            return "\n".join(out)
    return ""

user=False; queued=False
for f in files:
    try: fh=open(f,encoding="utf-8",errors="replace")
    except OSError: print("UNREADABLE"); sys.exit(0)
    with fh:
        for line in fh:
            if sent not in line and esc not in line: continue
            try: d=json.loads(line)
            except Exception: continue
            if sent not in text_of(d): continue
            if d.get("type")=="user": user=True
            elif d.get("type")=="queue-operation": queued=True
print("DELIVERED" if user else ("ENQUEUED" if queued else "ABSENT"))
') || die_tooling "transcript scan failed"

    case "$verdict" in
      DELIVERED)  break ;;
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

  # A QUEUED-but-never-drained message means delivery succeeded and the target did
  # not act. Annotate with the target's own state to separate "stuck, go clear it"
  # from "still working, retry" — but only as detail. The verdict is already decided
  # by the transcript above; no agents field can promote or demote it.
  local detail=""
  if [ "$verdict" = "ENQUEUED" ] && [ -n "$SHORT_ID" ]; then
    local agents_out
    agents_out=$(agents_json) || exit $RC_TOOLING
    # Unparseable agents output is a TOOLING fault, not a verdict. Swallowing it
    # would leave `detail` empty, which reads as "target at rest" and silently
    # reports BLOCKED — a confident answer derived from data we failed to read.
    detail=$(printf '%s' "$agents_out" | SMS_ID="$SHORT_ID" python3 -c '
import json,os,sys
want=os.environ["SMS_ID"]
try: d=json.load(sys.stdin)
except Exception: sys.exit(3)
if not isinstance(d,list): sys.exit(3)
for a in d:
    if str(a.get("id","")).startswith(want):
        st=a.get("state") or "unknown"; stat=a.get("status") or "unknown"
        print("state=%s status=%s" % (st,stat)); sys.exit(0)
print("state=unknown status=unknown (target not listed)")
') || die_tooling "could not read target state from 'claude agents --json'"
    case "$detail" in
      *busy*|*working*) : ;;                    # still turning; may yet drain
      *) verdict="BLOCKED" ;;                   # at rest with the message still queued
    esac
  fi

  local rc stage
  case "$verdict" in
    DELIVERED) rc=$RC_OK;     stage="DELIVER" ;;
    BLOCKED)   rc=$RC_BROKEN; stage="TARGET" ;;
    ENQUEUED)  rc=$RC_BROKEN; stage="DRAIN" ;;
    *)         rc=$RC_BROKEN; stage="DELIVER"; verdict="UNDELIVERED" ;;
  esac

  if [ "$JSON" = "1" ]; then
    SMS_OK=$([ $rc -eq 0 ] && echo 1 || echo 0) \
    SMS_STAGE="$stage" SMS_VERDICT="$verdict" SMS_DETAIL="$detail" \
    SMS_SID="$SESSION_ID" SMS_SENT="$SENTINEL" python3 -c '
import json,os
print(json.dumps({"ok":os.environ["SMS_OK"]=="1","stage":os.environ["SMS_STAGE"],
                  "verdict":os.environ["SMS_VERDICT"],"detail":os.environ["SMS_DETAIL"],
                  "session_id":os.environ["SMS_SID"],"sentinel":os.environ["SMS_SENT"]}))'
  else
    case "$verdict" in
      DELIVERED)
        echo "DELIVER=PASS  DELIVERED — sentinel is a user turn in the receiver's transcript; the rail works" ;;
      BLOCKED)
        echo "TARGET=FAIL   BLOCKED — DELIVERY IS FINE. The sentinel reached the receiver's queue and the"
        echo "              target never drained it (${detail:-target state unknown})."
        echo "              Do not read this as a messaging failure. Clear the target's prompt, or relaunch"
        echo "              it with permissions that let it act." ;;
      ENQUEUED)
        echo "DRAIN=FAIL    ENQUEUED — DELIVERY IS FINE. The sentinel is queued and the target is still"
        echo "              busy (${detail:-target state unknown}); it may yet drain. Re-run assert with a"
        echo "              longer --timeout-secs before treating this as a fault." ;;
      UNDELIVERED)
        echo "DELIVER=FAIL  UNDELIVERED — sentinel never reached the receiver's transcript within ${TIMEOUT_SECS}s" ;;
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
  ""|-h|--help) sed -n '1,68p' "$0"; exit $RC_OK ;;
  *) echo "session-message-selftest: unknown subcommand: $SUB (want prepare|assert|cleanup)" >&2; exit $RC_TOOLING ;;
esac
