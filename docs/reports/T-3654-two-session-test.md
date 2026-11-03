# Two-session test — second (non-main) session report

Date: 2026-11-02

## (a) Peer / sidecar mail in starting context

NONE. No "Sidecar receiver", "[sidecar]", PEER-DATA block, or any peer message appeared in this
session's context. The text TWO-SESSION-TEST did not appear anywhere in the context. The only
injected context was the standard gitStatus/userEmail/attribution system reminders and the
review-worker brief (T-3580) itself.

## (b) Command outputs

`termlink main status`:
```
not-main: main of 010-termlink is 10c2f139-7c28-4c84-b41d-df672cbe37c4@dimitrimintdev pid 1958984 (agent claude-termlink) (generation 1)
rc=1
```

`echo '{"session_id":"'"$CLAUDE_SESSION_ID"'"}' | termlink main am-i --session-from-stdin; echo rc=$?`:
```
not-main: held by main 10c2f139-7c28-4c84-b41d-df672cbe37c4@dimitrimintdev pid 1958984 (agent claude-termlink) (generation 1)
rc=1
```

## (c) Session id

Unknown — CLAUDE_SESSION_ID was EMPTY in this session's shell environment, so am-i was fed
`{"session_id":""}`.

---
# Run 2 (22:04Z): fresh interactive session in /opt/termlink started while test message 1fdac1b0 was pending
Result: main 10c2f139 received TWO-SESSION-TEST-2 at its next prompt; the second session saw no peer mail, am-i rc 1. Caveat: the second session was started by fw termlink dispatch; a hand-opened second session is not separately tested. Two earlier probes (spawned via session-message-selftest) were invalid: one refuses cross-session messages, one ran outside the project folder.
# Two-session test — second (non-main) session, run 2

Date: 2026-11-02

## (a) Peer / sidecar mail in context
No. My start context had no "Sidecar receiver", "[sidecar]" or PEER-DATA block, and no peer message
of any kind. The text TWO-SESSION-TEST-2 did not appear anywhere in my context except in my own
task brief. The only system context was Claude Code's normal gitStatus/user-email block and the
commit attribution reminder.

## (b) Command outputs

`termlink main status`:
```
not-main: main of 010-termlink is 10c2f139-7c28-4c84-b41d-df672cbe37c4@dimitrimintdev pid 1958984 (agent claude-termlink) (generation 1)
rc=1
```

`echo '{"session_id":"'"$CLAUDE_SESSION_ID"'"}' | termlink main am-i --session-from-stdin; echo rc=$?`:
```
not-main: held by main 10c2f139-7c28-4c84-b41d-df672cbe37c4@dimitrimintdev pid 1958984 (agent claude-termlink) (generation 1)
rc=1
```

## (c) Session id
Unknown. CLAUDE_SESSION_ID was empty in this session, so am-i got `{"session_id":""}`.

---
# Caveat added 2026-11-03 (T-3668)
The `am-i` / `status` outputs above (rc 1) do NOT prove the second session was answered as not-main.
CLAUDE_SESSION_ID was empty, so the CLI fell back to $TERMLINK_SESSION_ID (the TermLink `tl-…` id), and it would
have said not-main from the main session too. The evidence that stands is (a): no peer mail reached the second
session. T-3668 fixed the resolution: inside Claude Code the CLI now reads $CLAUDE_CODE_SESSION_ID and never the
TermLink id.
