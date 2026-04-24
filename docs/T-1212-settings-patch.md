# T-1212 — human-gated settings.json + cron install

T-1212 ships two artifacts:

1. **SessionEnd hook handler** (`session-end.sh`) — B-005 gated.
2. **Silent-session scanner** (`session-silent-scanner.sh`) — system-gated
   (cron install).

## 1. `.claude/settings.json` — SessionEnd block

Append this entry to `.claude/settings.json` alongside the other hooks:

```json
    "SessionEnd": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".agentic-framework/bin/fw hook session-end"
          }
        ]
      }
    ]
```

### Verify

```
# Stub payload — handler logs telemetry + (since no LATEST.md session match)
# spawns fw handover in background, returns exit 0 fast
echo '{"session_id":"S-STUB","reason":"logout","transcript_path":"/nonexistent"}' \
  | .agentic-framework/bin/fw hook session-end
echo "exit=$?"   # expect: exit=0
tail -1 .context/working/.session-end-log
# expect a JSON line with session_id=S-STUB and reason=logout
```

## 2. Cron install — silent-session scanner

The scanner walks `$HOME/.claude/projects/*.jsonl`, finds session transcripts
with `mtime > 30 min` AND no matching handover file, then triggers
`fw handover` for each (best-effort recovery).

**DANGER — scanner defaults to DRY_RUN=1** after the T-1212 development
incident (8 spurious commits from a smoke-test that scanned production state).
Cron stanza MUST explicitly opt in with `DRY_RUN=0`.

### Recommended cron stanza (system-wide)

Write `/etc/cron.d/fw-session-silent` (root-owned, permissions 0644):

```cron
# Every 15 min — recover handovers for sessions that skipped SessionEnd
# (Claude Code #17885 /exit bug, #20197 API 500 kill, SIGKILL, laptop sleep).
# DRY_RUN=0 opts in to real `fw handover` invocation.
*/15 * * * * root cd /opt/termlink && DRY_RUN=0 .agentic-framework/bin/fw hook session-silent-scanner >/dev/null 2>&1
```

Or per-user via `crontab -e`:

```cron
*/15 * * * * cd /opt/termlink && DRY_RUN=0 .agentic-framework/bin/fw hook session-silent-scanner >/dev/null 2>&1
```

### Verify — always dry-run first

```
# Safe: dry-run mode (default)
.agentic-framework/bin/fw hook session-silent-scanner
tail -5 .context/working/.session-silent-scanner.log
# expect: "DRY-RUN would-recover session=..." lines for each candidate, plus
# "scan-end candidates=N dry_run=1"

# Live: only after confirming candidates look correct
DRY_RUN=0 .agentic-framework/bin/fw hook session-silent-scanner
```

### Tuning

- `SESSION_SILENT_THRESHOLD_MIN=45` — require 45 min idle before recovery
  (default 30).
- `CLAUDE_PROJECTS_DIR=/path/to/claude/projects` — non-default transcript dir.

## Recovery-handover convention

When `fw handover` is invoked by the scanner, the following env vars are set:

- `RECOVERED=1`
- `RECOVERED_SESSION_ID=<jsonl stem>`
- `RECOVERED_AGE_MIN=<integer>`
- `RECOVERED_TRANSCRIPT=<path to .jsonl>`

A future enhancement to `fw handover` should detect these and prepend a
`[recovered, no agent context]` banner to the generated file. Today, the
scanner just triggers handover; the banner is a future follow-up.
