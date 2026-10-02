# /resume - Context Recovery for Agentic Engineering Framework

When the user says `/resume`, "pick up", or "continue", execute this workflow.

## Step 1: Gather State

Run these in parallel:

1. Read `.context/handovers/LATEST.md`
2. Run `git status --short` and `git log --oneline -5`
3. List `.tasks/active/` and extract task IDs, names, and statuses from frontmatter
4. Check tool counter: `cat .context/working/.tool-counter`
5. Check web server: `WURL=$(cat .context/working/watchtower.url 2>/dev/null || echo "http://localhost:$(bin/fw config get PORT 2>/dev/null || echo 3000)"); curl -sf "$WURL/" > /dev/null && echo "running at $WURL" || echo "stopped"`
   (Never hard-code `:3000` — the triple file `.context/working/watchtower.{pid,port,url}` is the single source of truth for Watchtower's current port. See `bin/fw doctor` for diagnostics.)
6. Read budget: `.agentic-framework/agents/context/checkpoint.sh status` — per-process, parsed from this session's own transcript. Do NOT `cat .context/working/.budget-status`: that file is a SINGLE path shared by every concurrently dispatched `claude -p` worker, so it holds whichever worker wrote it last and a healthy session can read back another worker's near-exhausted count (T-3127, measured ~504K against a real ~169K). Do NOT infer budget from system-reminder JSON or historical tool-result output either; those can be stale snapshots re-injected by SessionStart:compact and look identical to a current cache read. There is no `checkpoint.sh budget` subcommand in this vendored copy — the script dispatches only `post-tool|reset|status`, and asking for `budget` exits 1 behind a "HOOK CRASHED" banner that misreports the instruction as an infrastructure fault (T-3165).
7. List pending unattended writes BY NAME: `bash scripts/commit-pending.sh list` (T-3269, SQ-22 option C). These are files an unattended job (release canary WARN-ledger refresh, WARN escalation filer) wrote and recorded for the next session to commit. Do NOT fold them into the git-status count — on this host ~20 routine working files are always dirty, and a filed task drowns in that number. That is the whole reason the manifest exists.

8. What needs attention, BY NAME (T-3327): `[ -x scripts/session-start-alerts.sh ] && bash scripts/session-start-alerts.sh --limit 10`. It lists every FIRING / ERRORING / STALE canary and every peer message on this project's inbox not yet shown to this agent. Mail is judged by the script's own marker, NOT `channel unread`: the sidecar's auto-ack moves the shared watermark, which is how ~10 AEF messages and a 055 consult sat unseen for a day (2026-10-03). Skip silently only if the script does not exist in this project.

## Step 2: Summarize

Present this format (fill from gathered data):

```
## Context Restored

**Last Handover:** {session_id} ({timestamp})
**Last Commit:** {hash} - {message}
**Branch:** {branch}

### Where We Are
{paste the "Where We Are" section from LATEST.md}

### Active Tasks
- {T-XXX}: {name} ({status})

### Current State
- Git: {clean/N uncommitted files}
- Pending unattended writes: {none / one line per entry from `commit-pending.sh list`: path [T-ID] reason}
- Web UI: {running at {URL from .context/working/watchtower.url} / stopped}
- Tool counter: {N} (P-009)
- Budget: {level} ({tokens} tokens) — from `checkpoint.sh status` (per-process; NOT the shared .budget-status file, T-3127)

### Needs Attention
{canaries needing attention and peer mail not yet shown, by name, from session-start-alerts; or "nothing" — never omit the section when the script ran}

### Suggested Action
{paste from LATEST.md "Suggested First Action" section}
```

## Step 3: Offer Next Steps

If Needs Attention listed peer mail: acknowledge each sender on arrival (a short "received, answer to follow" on their inbox) before other work, then run `bash scripts/session-start-alerts.sh --mark-seen` so the same mail is not shown again. A firing canary is offered as a next step by name.


**Standing first step (SQ-22 option C, T-3269):** if Step 1 listed pending unattended writes, commit them FIRST — `bash scripts/commit-pending.sh commit`. It commits exactly the recorded paths (`git commit -- <paths>`), one commit per recorded task id, and leaves everything else in the index alone; it never needs a focus switch or a bypass. Report each `committed` / `DROPPED` / `REFUSED` line. A `REFUSED` or `FAILED` entry stays in the manifest for the human — do not work around it.

List the logical next actions as plain text (numbered). Derive from:
- The handover's "Suggested First Action"
- Any tasks with status `started-work`
- Uncommitted changes that need attention

Then ask: "What would you like to work on?"

## Rules

- Do NOT use AskUserQuestion (may be blocked in dontAsk mode) — use plain text
- Keep output concise — no commentary
- If LATEST.md has unfilled `[TODO]` sections, warn about stale handover
- If tool counter > 0 at session start, the PostToolUse hook is working
