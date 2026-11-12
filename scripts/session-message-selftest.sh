#!/usr/bin/env bash
# session-message-selftest.sh — prove agent-session -> agent-session messaging end to end.
#
# T-2876 built this prover for Claude Code's vendor SendMessage rail. T-3711 (arc-011 P-27,
# B-26, E-74) re-bases it on the TermLink rail, because B-18 (P-28) will deny SendMessage and
# the deny must never switch off a working prover without its replacement in place (ARC4
# 6.10.2.a, NID). The SendMessage rail stays available as `--rail sendmessage` (rollback)
# until this one is proven in the field.
#
# The siblings cover the charter's other rails:
#   comms-selftest.sh    (T-2482) TermLink discover + exchange durable messages
#   session-selftest.sh  (T-2485) TermLink control terminal sessions (PTY inject)
#   substrate-smoke.sh   (T-2151) claim work
#
# WHY IT ASSERTS ON THE RECEIVER
# A sender's "success" says nothing about whether the target received the message. Both
# rails lie in the same way: SendMessage returns {"success":true} once queued, and
# `fw sidecar send` returns {"delivered": true, "state": "HUB_ACCEPTED"} with
# "receiver": "none-registered" (measured T-3711, live). Earlier findings share this shape:
# a hub reported "injected" while the PTY got nothing (T-2873); a config looked
# authoritative and was never read (T-2874); a send returned success while the target sat
# blocked on a permission prompt (T-2875). So this script never trusts the sender. It
# reads the RECEIVER's own transcript.
#
# WHAT COUNTS AS DELIVERY (the receiver's transcript, harness-written records only)
#   a `user` turn whose message content carries the sentinel (SendMessage drain; PTY inject)
#   an `attachment` record of type `hook_additional_context` carrying it (the TermLink rail:
#     the receiver's hook surfaces the peer mail, and Claude Code writes that record; the
#     same evidence the arc-011 sidecar finalizer uses, crates/termlink-sidecar adapter)
# Never: an assistant turn, a tool result, a `hook_success` echo of a hook's stdout, or an
# `api-request-blob` (each can carry the sentinel without the model having been given it).
#
# THE OUTCOMES THAT ARE BYTE-IDENTICAL FROM THE SENDER'S SIDE
#   DELIVERED    sentinel reached the receiver's context        -> the rail works        (0)
#   BLOCKED      sentinel is queued, target is at rest          -> DELIVERY WAS FINE     (1)
#   ENQUEUED     sentinel is queued, target is still busy       -> may yet drain; retry  (1)
#   UNDELIVERED  sentinel is in neither transcript nor queue    -> the rail is broken    (1)
#
# "Queued" means held on the RECEIVER's side and not yet in its context: a `queue-operation`
# record in its transcript (SendMessage), or a consult on its own TermLink inbox topic
# (`fw sidecar inbox --peek` run as the receiver's agent id). On the TermLink rail an inbox
# that cannot be read is TOOLING: without it ENQUEUED and UNDELIVERED cannot be told apart.
#
# The BLOCKED/UNDELIVERED split is the whole point. Collapsing them is the T-2875
# misdiagnosis — a working rail declared impossible because the target could not act.
#
# HOW "BLOCKED" IS ACTUALLY DETECTED, AND HOW IT IS NOT
# From the receiver's queue (above), annotated with the target's state. It is deliberately
# NOT read from a `waitingFor` field in `claude agents --json`. That field DOES NOT EXIST —
# measured across the live fleet, 0 of 56 agents carried it. An earlier draft depended on
# it, which made the entire blocked branch dead code. Nor can `state` carry the signal on
# its own: `state:"blocked"` is the ordinary RESTING state of a finished background session
# (43 of those same 56), so keying off it would classify almost every broken rail as
# "target merely stuck". `state`/`status` only ANNOTATE an already-queued message.
#
# WHAT MAKES A SENDMESSAGE TARGET REACHABLE (measured, T-2876)
# Only a `claude agents --json` record carrying a `status` field (15 of 58; matched the
# SendMessage peer list exactly). `prepare --existing` gates on that and fails closed.
#
# THE TERMLINK LIVE PASS (`prove`)
# Allocates a throwaway agent id and session id, sends the sentinel with
# `fw sidecar send --level agent --notice`, then wakes a headless probe session
# (`claude -p --session-id`) that owns that inbox; its UserPromptSubmit hook surfaces the
# consult. It asserts the sentinel on the probe's transcript (DELIVERED rc 0) and a control
# sentinel that was never sent on the same transcript (UNDELIVERED rc 1), and appends both
# to the record file. `verify-record` is the P-28 gate: it passes only on such a pair made
# by this exact script (sha256), so an edit to the prover needs a fresh live pass.
#
# NOT marked `# guard-layer: source`: prepare/wake/prove/cleanup start or stop real
# sessions and send real messages. `assert` and `verify-record` alone are pure reads.
#
# Usage:
#   scripts/session-message-selftest.sh prove [--record FILE] [--model M] [--timeout-secs N] [--json]
#   scripts/session-message-selftest.sh send --to AGENT --sentinel TEXT [--json]
#   scripts/session-message-selftest.sh wake --to AGENT --session-id ID [--model M]
#   scripts/session-message-selftest.sh assert --session-id ID --sentinel TEXT [--rail termlink|sendmessage]
#          [--to AGENT] [--id SHORT_ID] [--timeout-secs N] [--record FILE --role probe|control --run-id R] [--json]
#   scripts/session-message-selftest.sh verify-record [--record FILE] [--json]
#   scripts/session-message-selftest.sh prepare --existing MATCH [--sentinel TEXT] [--json]   (sendmessage rail)
#   scripts/session-message-selftest.sh prepare --spawn [--sentinel TEXT] [--dir DIR] [--json] (sendmessage rail)
#   scripts/session-message-selftest.sh cleanup --id SHORT_ID
#
# Exit: 0 proven | 1 broken (names the stage) | 2 tooling error (fail-closed)

set -uo pipefail

RC_OK=0; RC_BROKEN=1; RC_TOOLING=2
SELF="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
ROOT="$(cd "$(dirname "$SELF")/.." && pwd)"
DEFAULT_RECORD="$ROOT/.context/project/session-message-prover.jsonl"

SUB="${1:-}"; [ $# -gt 0 ] && shift

SENTINEL=""; DIR=""; SESSION_ID=""; SHORT_ID=""; TIMEOUT_SECS=120; JSON=0
EXISTING=""; SPAWN=0; READY_SECS=60; RAIL="termlink"; TO=""; RECORD=""; ROLE=""; RUN_ID=""
MODEL="claude-haiku-5-5"

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
    --rail)         RAIL="${2:-}"; shift 2 ;;
    --to)           TO="${2:-}"; shift 2 ;;
    --record)       RECORD="${2:-}"; shift 2 ;;
    --role)         ROLE="${2:-}"; shift 2 ;;
    --run-id)       RUN_ID="${2:-}"; shift 2 ;;
    --model)        MODEL="${2:-}"; shift 2 ;;
    --json)         JSON=1; shift ;;
    -h|--help)      sed -n '1,/^# Exit:/p' "$SELF"; exit $RC_OK ;;
    *) echo "session-message-selftest: unknown arg: $1" >&2; exit $RC_TOOLING ;;
  esac
done

die_tooling() { echo "session-message-selftest: TOOLING: $*" >&2; exit $RC_TOOLING; }

command -v python3 >/dev/null 2>&1 || die_tooling "python3 not found"
case "$RAIL" in termlink|sendmessage) : ;; *) die_tooling "--rail must be termlink or sendmessage" ;; esac
case "$TIMEOUT_SECS" in ''|*[!0-9]*) die_tooling "--timeout-secs must be an integer" ;; esac

# `claude` is only needed for the subcommands that shell out to it; `assert` can run
# against a fixture transcript with no binary present at all.
need_claude() { command -v claude >/dev/null 2>&1 || die_tooling "claude not found on PATH"; }

fw_bin() {
  if [ -x "$ROOT/.agentic-framework/bin/fw" ]; then echo "$ROOT/.agentic-framework/bin/fw"
  elif command -v fw >/dev/null 2>&1; then command -v fw
  else return 1; fi
}

script_sha() { sha256sum "$SELF" 2>/dev/null | cut -d' ' -f1; }

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

# The receiver's own TermLink inbox, read AS the receiver (its agent id is also its
# session key, so the circuit collapses to inbox:<hub>/<project>/<agent>). Peek only:
# the cursor never moves, so reading it cannot consume the probe.
inbox_json() {
  if [ -n "${SESSION_MSG_TEST_INBOX_JSON:-}" ]; then
    [ -f "$SESSION_MSG_TEST_INBOX_JSON" ] || die_tooling "test inbox json not found: $SESSION_MSG_TEST_INBOX_JSON"
    cat "$SESSION_MSG_TEST_INBOX_JSON"
  else
    local fw; fw=$(fw_bin) || die_tooling "fw not found (needed to read the receiver's inbox)"
    (cd "$ROOT" && env -u TERMLINK_SESSION FW_SIDECAR_AGENT_ID="$1" FW_FOCUS_SESSION_KEY="$1" \
       timeout 60 "$fw" sidecar inbox --peek --json 2>/dev/null)
  fi
}

transcript_root() {
  if [ -n "${SESSION_MSG_TEST_TRANSCRIPT_DIR:-}" ]; then
    echo "$SESSION_MSG_TEST_TRANSCRIPT_DIR"
  else
    echo "$HOME/.claude/projects"
  fi
}

# ── prepare (sendmessage rail) ───────────────────────────────────────────────────
#
# Emits the reachable recipient name + sentinel for the send, and the sessionId the
# assert step reads. Two modes:
#   --existing MATCH  resolve an ALREADY-reachable session (id prefix or name substring)
#   --spawn           start a fresh probe target, then WAIT for it to become reachable
# Both gate on the same measured predicate: a `status` field in the agent's record.
# On the TermLink rail use `prove`, which allocates and wakes its own target.

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
    # The target is told to do NOTHING: assertion is on its transcript, so the prover
    # measures the RAIL, not the target's willingness to obey (the T-2875 false negative).
    local out rc
    out=$(cd "$DIR" && timeout 120 claude --bg \
          'You are a passive probe target for a messaging self-test. Reply with the single word READY and then do nothing further. Do not write files. Do not run commands.' 2>&1)
    rc=$?
    [ $rc -eq 0 ] || die_tooling "claude --bg failed (rc=$rc): $(echo "$out" | tail -3)"
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
    echo "Now SEND (sendmessage rail: an agent tool call — there is no CLI verb for it):"
    echo "  SendMessage to=\"$name\" message=\"$SENTINEL\""
    echo
    echo "Then ASSERT:"
    echo "  bash scripts/session-message-selftest.sh assert --rail sendmessage --session-id $sid --sentinel $SENTINEL${id:+ --id $id}"
  fi
  return $RC_OK
}

# ── send (termlink rail) ─────────────────────────────────────────────────────────
# Posts the sentinel to AGENT's own TermLink inbox as a notice (no reply expected).
# The sender's answer decides only whether the SEND stage ran; it never decides
# delivery — that is `assert`'s job, on the receiver's transcript.
do_send() { # $1 = to, $2 = sentinel; prints the sender's raw answer; returns its rc
  if [ -n "${SESSION_MSG_TEST_SEND_CMD:-}" ]; then
    SMS_TO="$1" SMS_SENT="$2" bash -c "$SESSION_MSG_TEST_SEND_CMD"
  else
    local fw; fw=$(fw_bin) || return 127
    (cd "$ROOT" && "$fw" sidecar send --to "$1" --level agent --notice --body "$2" --json 2>&1)
  fi
}

cmd_send() {
  [ -n "$TO" ]       || die_tooling "send requires --to AGENT"
  [ -n "$SENTINEL" ] || die_tooling "send requires --sentinel"
  local out rc state
  out=$(do_send "$TO" "$SENTINEL"); rc=$?
  [ $rc -eq 127 ] && die_tooling "fw not found (cannot send on the TermLink rail)"
  state=$(printf '%s' "$out" | python3 -c '
import json,sys
st=""
for line in sys.stdin.read().splitlines():
    try: d=json.loads(line)
    except Exception: continue
    if isinstance(d,dict) and "state" in d: st=str(d.get("state"))
print(st)')
  if [ $rc -ne 0 ] || [ "$state" = "REFUSED" ] || [ -z "$state" ]; then
    echo "SEND=FAIL     the TermLink send did not post (rc=$rc state=${state:-none}): $(printf '%s' "$out" | tail -1)"
    return $RC_BROKEN
  fi
  echo "SEND=PASS     posted to $TO (sender says state=$state; not trusted for delivery)"
  return $RC_OK
}

# ── wake (termlink rail) ─────────────────────────────────────────────────────────
# Runs one headless turn of a probe session that owns AGENT's inbox, so its
# UserPromptSubmit hook surfaces the consult into its transcript. The probe is told to
# do nothing: the assertion is on its transcript, never on what it says.
do_wake() { # $1 = to, $2 = session id
  if [ -n "${SESSION_MSG_TEST_WAKE_CMD:-}" ]; then
    SMS_TO="$1" SMS_SID="$2" bash -c "$SESSION_MSG_TEST_WAKE_CMD"
  else
    need_claude
    (cd "$ROOT" && env -u CLAUDECODE -u CLAUDE_CODE_SESSION_ID -u CLAUDE_CODE_CHILD_SESSION \
        -u CLAUDE_CODE_MESSAGING_SOCKET -u CLAUDE_CODE_MESSAGING_TOKEN -u CLAUDE_PID \
        -u CLAUDE_CODE_ENTRYPOINT -u FW_DISPATCHED_WORKER -u TERMLINK_SESSION \
        FW_SIDECAR_AGENT_ID="$1" FW_FOCUS_SESSION_KEY="$1" \
        timeout 300 claude -p --session-id "$2" --model "$MODEL" \
        'You are a passive probe target for a messaging self-test. Reply with the single word READY and then do nothing further. Do not run commands. Do not write files.' \
        </dev/null >/dev/null 2>&1)
  fi
}

cmd_wake() {
  [ -n "$TO" ]         || die_tooling "wake requires --to AGENT"
  [ -n "$SESSION_ID" ] || die_tooling "wake requires --session-id"
  do_wake "$TO" "$SESSION_ID" || die_tooling "the probe session did not run (rc=$?)"
  echo "WAKE=done     probe session $SESSION_ID ran one turn as $TO"
  return $RC_OK
}

# ── assert ───────────────────────────────────────────────────────────────────────
write_record() { # $1 verdict $2 rc $3 stage
  [ -n "$RECORD" ] || return 0
  mkdir -p "$(dirname "$RECORD")" 2>/dev/null || die_tooling "cannot create the record dir for $RECORD"
  SMS_V="$1" SMS_RC="$2" SMS_STAGE="$3" SMS_RAIL="$RAIL" SMS_ROLE="${ROLE:-probe}" SMS_RUN="$RUN_ID" \
  SMS_SID="$SESSION_ID" SMS_SENT="$SENTINEL" SMS_TO="$TO" SMS_SHA="$(script_sha)" python3 -c '
import json,os,datetime
e=os.environ
print(json.dumps({"ts":datetime.datetime.now(datetime.timezone.utc).isoformat(),
  "rail":e["SMS_RAIL"],"role":e["SMS_ROLE"],"run_id":e["SMS_RUN"],"verdict":e["SMS_V"],
  "rc":int(e["SMS_RC"]),"stage":e["SMS_STAGE"],"session_id":e["SMS_SID"],"to":e["SMS_TO"],
  "sentinel":e["SMS_SENT"],"script_sha256":e["SMS_SHA"]}))' >> "$RECORD" \
    || die_tooling "could not append to the record $RECORD"
}

cmd_assert() {
  [ -n "$SESSION_ID" ] || die_tooling "assert requires --session-id"
  [ -n "$SENTINEL" ]   || die_tooling "assert requires --sentinel"
  case "$ROLE" in ''|probe|control) : ;; *) die_tooling "--role must be probe or control" ;; esac

  local root; root=$(transcript_root)
  [ -d "$root" ] || die_tooling "transcript root not found: $root"

  local deadline=$(( $(date +%s) + TIMEOUT_SECS ))
  local verdict=""

  while :; do
    # The sentinel is matched against the DECODED text of each record, not against
    # the raw line: a sentinel containing a quote or backslash is JSON-escaped on
    # disk and a raw substring test would silently miss it. Only harness-written
    # records count: `user` message content, a `hook_additional_context` attachment,
    # and (queued, not delivered) a `queue-operation`.
    verdict=$(SMS_ROOT="$root" SMS_SID="$SESSION_ID" SMS_SENT="$SENTINEL" python3 -c '
import glob,json,os,sys
root=os.environ["SMS_ROOT"]; sid=os.environ["SMS_SID"]; sent=os.environ["SMS_SENT"]
files=glob.glob(os.path.join(root,"*",sid+".jsonl"))
if not files:
    print("NO_TRANSCRIPT"); sys.exit(0)
esc=json.dumps(sent, ensure_ascii=False)[1:-1]   # how it appears inside a JSON string

def strings(c):
    if isinstance(c,str): return [c]
    out=[]
    if isinstance(c,list):
        for b in c:
            if isinstance(b,str): out.append(b)
            elif isinstance(b,dict):
                for k in ("text","content"):
                    if isinstance(b.get(k),str): out.append(b[k])
    return out

def text_of(d):
    t=d.get("type")
    if t=="queue-operation":
        c=d.get("content")
        return c if isinstance(c,str) else ""
    if t=="user":
        return "\n".join(strings((d.get("message") or {}).get("content")))
    if t=="attachment":
        a=d.get("attachment") or {}
        # Only the record the harness writes when a hook hands text to the model.
        # A hook_success echo of the hook stdout exists even when the harness
        # dropped the output, so it proves nothing.
        if a.get("type")=="hook_additional_context":
            return "\n".join(strings(a.get("content")))
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
            if d.get("type") in ("user","attachment"): user=True
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

  # TermLink rail: a sentinel absent from the transcript may still be waiting in the
  # receiver's own inbox. Without reading it ENQUEUED and UNDELIVERED would collapse,
  # so an inbox that cannot be read is TOOLING, never UNDELIVERED.
  if [ "$verdict" = "ABSENT" ] && [ "$RAIL" = "termlink" ]; then
    [ -n "$TO" ] || die_tooling "the termlink rail needs --to AGENT (the receiver's inbox) to tell ENQUEUED from UNDELIVERED"
    local inbox_out q
    inbox_out=$(inbox_json "$TO") || die_tooling "could not read the receiver's inbox ($TO)"
    q=$(printf '%s' "$inbox_out" | SMS_SENT="$SENTINEL" python3 -c '
import json,os,sys
sent=os.environ["SMS_SENT"]
try: d=json.load(sys.stdin)
except Exception: sys.exit(3)
if not isinstance(d,dict) or not isinstance(d.get("consults"),list): sys.exit(3)
print("QUEUED" if any(isinstance(c,dict) and sent in str(c.get("body") or "") for c in d["consults"]) else "NONE")
') || die_tooling "the receiver's inbox answer is not the expected JSON ($TO)"
    [ "$q" = "QUEUED" ] && verdict="ENQUEUED"
  fi

  # A QUEUED-but-never-drained message means delivery succeeded and the target did
  # not act. Annotate with the target's own state to separate "stuck, go clear it"
  # from "still working, retry" — but only as detail. No agents field can promote or
  # demote the verdict decided above.
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
  write_record "$verdict" "$rc" "$stage"

  if [ "$JSON" = "1" ]; then
    SMS_OK=$([ $rc -eq 0 ] && echo 1 || echo 0) \
    SMS_STAGE="$stage" SMS_VERDICT="$verdict" SMS_DETAIL="$detail" SMS_RAIL="$RAIL" \
    SMS_SID="$SESSION_ID" SMS_SENT="$SENTINEL" python3 -c '
import json,os
print(json.dumps({"ok":os.environ["SMS_OK"]=="1","stage":os.environ["SMS_STAGE"],
                  "verdict":os.environ["SMS_VERDICT"],"detail":os.environ["SMS_DETAIL"],
                  "rail":os.environ["SMS_RAIL"],
                  "session_id":os.environ["SMS_SID"],"sentinel":os.environ["SMS_SENT"]}))'
  else
    case "$verdict" in
      DELIVERED)
        echo "DELIVER=PASS  DELIVERED — the sentinel reached the receiver's context (its transcript); the rail works" ;;
      BLOCKED)
        echo "TARGET=FAIL   BLOCKED — DELIVERY IS FINE. The sentinel reached the receiver's queue and the"
        echo "              target never drained it (${detail:-target state unknown})."
        echo "              Do not read this as a messaging failure. Clear the target's prompt, or relaunch"
        echo "              it with permissions that let it act." ;;
      ENQUEUED)
        echo "DRAIN=FAIL    ENQUEUED — DELIVERY IS FINE. The sentinel is queued for the receiver and the target"
        echo "              has not taken it yet (${detail:-target state unknown}); it may yet drain. Re-run"
        echo "              assert with a longer --timeout-secs, or wake the target, before treating this as a fault." ;;
      UNDELIVERED)
        echo "DELIVER=FAIL  UNDELIVERED — sentinel reached neither the receiver's transcript nor its queue within ${TIMEOUT_SECS}s" ;;
    esac
  fi
  return $rc
}

# ── prove (termlink rail live pass, with its negative control) ───────────────────
cmd_prove() {
  [ "$RAIL" = "termlink" ] || die_tooling "prove runs the TermLink rail only"
  [ -n "$RECORD" ] || RECORD="$DEFAULT_RECORD"
  local stamp; stamp="$(date -u +%Y%m%dT%H%M%SZ)-$$"
  RUN_ID="run-$stamp"
  TO="sms-probe-$stamp"
  SESSION_ID=$(python3 -c 'import uuid;print(uuid.uuid4())')
  local probe="SMS-PROBE-$stamp" control="SMS-CONTROL-$stamp"
  echo "PROVE         run=$RUN_ID to=$TO session=$SESSION_ID record=$RECORD"

  SENTINEL="$probe"
  cmd_send; local src=$?
  [ $src -eq 0 ] || { echo "PROVE=FAIL    the send stage failed; nothing to assert"; return $RC_BROKEN; }
  do_wake "$TO" "$SESSION_ID" || die_tooling "the probe session did not run"

  ROLE=probe
  ( SENTINEL="$probe" cmd_assert ); local prc=$?
  [ $prc -eq $RC_TOOLING ] && die_tooling "the probe assertion could not observe the receiver"

  # The never-sent control, on the SAME transcript: it must come back UNDELIVERED
  # (rc 1). A control that reads DELIVERED or queued means the prover is vacuous.
  ROLE=control; local saved_to="$TO"; TIMEOUT_SECS=0
  ( SENTINEL="$control" cmd_assert ); local crc=$?
  TO="$saved_to"
  [ $crc -eq $RC_TOOLING ] && die_tooling "the control assertion could not observe the receiver"

  if [ $prc -eq 0 ] && [ $crc -eq 1 ] && control_undelivered "$RUN_ID"; then
    echo "PROVE=PASS    probe DELIVERED rc 0, never-sent control UNDELIVERED rc 1 (recorded)"
    return $RC_OK
  fi
  if [ $prc -eq 0 ]; then
    echo "PROVE=TOOLING the control did not read UNDELIVERED (rc=$crc): the prover cannot be trusted"
    return $RC_TOOLING
  fi
  echo "PROVE=FAIL    the probe was not delivered (rc=$prc); control rc=$crc"
  return $RC_BROKEN
}

control_undelivered() { # the control's own record line says UNDELIVERED for this run
  SMS_REC="$RECORD" SMS_RUN="$1" python3 -c '
import json,os,sys
ok=False
for l in open(os.environ["SMS_REC"],encoding="utf-8"):
    try: d=json.loads(l)
    except Exception: continue
    if d.get("run_id")==os.environ["SMS_RUN"] and d.get("role")=="control" and d.get("verdict")=="UNDELIVERED": ok=True
sys.exit(0 if ok else 1)'
}

# ── verify-record (the P-28 gate's reader) ───────────────────────────────────────
# Exit 0 only when the record holds one run, on the TermLink rail, made by THIS script
# (same sha256), with the probe DELIVERED rc 0 and its never-sent control UNDELIVERED
# rc 1 on the same session. 1 = no such run; 2 = the record cannot be read.
cmd_verify_record() {
  [ -n "$RECORD" ] || RECORD="$DEFAULT_RECORD"
  if [ ! -e "$RECORD" ]; then
    echo "VERIFY=FAIL   no prover record at $RECORD (no live pass yet)"; return $RC_BROKEN
  fi
  [ -r "$RECORD" ] || die_tooling "record unreadable: $RECORD"
  local out rc
  out=$(SMS_REC="$RECORD" SMS_SHA="$(script_sha)" python3 -c '
import json,os,sys
runs={}
try: lines=open(os.environ["SMS_REC"],encoding="utf-8").read().splitlines()
except OSError as e: print("unreadable: %s" % e); sys.exit(2)
for n,l in enumerate(lines,1):
    if not l.strip(): continue
    try: d=json.loads(l)
    except Exception: print("line %d is not JSON" % n); sys.exit(2)
    if d.get("rail")!="termlink" or d.get("script_sha256")!=os.environ["SMS_SHA"] or not d.get("run_id"): continue
    runs.setdefault(d["run_id"],[]).append(d)
good=[]
for rid,es in runs.items():
    p=[e for e in es if e.get("role")=="probe" and e.get("verdict")=="DELIVERED" and e.get("rc")==0]
    c=[e for e in es if e.get("role")=="control" and e.get("verdict")=="UNDELIVERED" and e.get("rc")==1]
    if p and c and any(x["session_id"]==y["session_id"] and x["sentinel"]!=y["sentinel"] for x in p for y in c):
        good.append((p[0]["ts"],rid))
if not good: print("no run of this script (sha %s) has a DELIVERED probe with an UNDELIVERED control" % os.environ["SMS_SHA"][:12]); sys.exit(1)
good.sort(); print("%s at %s" % (good[-1][1], good[-1][0]))')
  rc=$?
  case $rc in
    0) echo "VERIFY=PASS   $out"; return $RC_OK ;;
    1) echo "VERIFY=FAIL   $out"; return $RC_BROKEN ;;
    *) die_tooling "record unreadable: $out" ;;
  esac
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
  send)    cmd_send ;;
  wake)    cmd_wake ;;
  assert)  cmd_assert ;;
  prove)   cmd_prove ;;
  verify-record) cmd_verify_record ;;
  cleanup) cmd_cleanup ;;
  ""|-h|--help) sed -n '1,/^# Exit:/p' "$SELF"; exit $RC_OK ;;
  *) echo "session-message-selftest: unknown subcommand: $SUB (want prepare|send|wake|assert|prove|verify-record|cleanup)" >&2; exit $RC_TOOLING ;;
esac
