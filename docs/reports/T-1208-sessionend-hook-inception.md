# T-1208: SessionEnd hook design — inception

**Parent task:** T-174 (captured with placeholder ACs).
**Status:** Exploration plan drafted; awaiting user review before spikes.

## Problem Statement

Claude Code's `SessionEnd` hook fires on session termination with a `reason` field (e.g., `clear`, `logout`, `prompt_input_exit`, `other`). The goal: auto-trigger `fw handover` on every exit so no session ends without a handover document. Today the handover depends on agent discipline, the budget gate's auto-handover at critical, or the PreCompact hook — all partial. Sessions that end by `/exit`, terminal close, or API 500 still slip through.

## Known Claude Code bugs (T-174 mentions)

- **#17885** — SessionEnd doesn't fire on `/exit` in some versions.
- **#20197** — API 500 terminations skip SessionEnd.

Fallback needed: even if SessionEnd is wired, agent cannot rely on it as the sole trigger.

## Framework-existing position to override

`docs/claude-code-settings.md §Rec #3` said "No action. The gap is structural (Claude Code limitation). Existing mitigations are sufficient." User has now directed inception-and-build.

## Constraints

- **Best-effort only.** Cannot block session exit (Claude Code already exiting). Hook runs in "shutting down" state — can write files, cannot prompt user.
- **Must be idempotent.** If handover already ran (PreCompact, budget critical, agent-initiated), don't run again. Check `.context/handovers/LATEST.md` mtime vs session start.
- **Fast.** If the hook takes >10s, the session may be killed before it completes.
- **Framework-side.** Script lives under `agents/context/`, invoked via the `fw hook` dispatcher.

## Assumptions (to register)

- A1: `SessionEnd` fires reliably on `clear` and `logout` reasons — only `prompt_input_exit` (matching Claude Code `#17885`) is unreliable on the current installed version.
- A2: A "handover exists for this session" check (comparing `LATEST.md` frontmatter `session_id` against `.context/working/session.yaml`) is sufficient for idempotency.
- A3: Fallback for skipped SessionEnd is a cron-based "session went silent" check — runs every 15 min, looks at `.claude/sessions/*.jsonl` for sessions with no events in >30 min, generates handover for them.

## Exploration Plan

**Spike S1 — reason-field survey (1h):** Wire a no-op SessionEnd handler that just logs the `reason` field to `.context/working/.session-end-log`. Run 3 real session-end scenarios: `/clear`, terminal close via Ctrl+D, API error. Confirm which reasons actually fire the hook on current Claude Code.

**Spike S2 — handover trigger prototype (2h):** Implement session-end handler that (a) checks if LATEST.md already matches current session_id (skip), (b) runs `fw handover` in background, (c) logs result. Measure wall-clock time under both clean exit and simulated kill.

**Spike S3 — silent-session fallback (2h):** Cron job at 15-min cadence scans `.claude/sessions/` for session JSONLs whose last-modified is >30 min old AND whose session_id doesn't appear in `.context/handovers/*.md`. For matches: generate a "recovery handover" from the transcript. Test against a deliberately-killed session.

## Decision Framework (GO / NO-GO criteria)

**GO if:** S2 generates a handover on clean exits (100%) AND S3 recovers any session skipped by SessionEnd within 30 min (100%).
**NO-GO if:** SessionEnd is so unreliable (fires on <50% of exits) that the fallback becomes the primary mechanism — in that case retire the hook work and just build the silent-session cron.

## Implementation destination

Framework-side. Wire through `fw hook session-end` dispatcher.

## Dialogue Log

### 2026-04-24 — inception kickoff
- **Agent:** Drafted exploration plan before any spikes. Key insight: T-174's own description flags the known Claude Code bugs (#17885, #20197) — the hook CANNOT be the sole trigger. Design must include a silent-session fallback (S3) regardless of whether S1/S2 pass. Pending user direction on whether S3 is in-scope or should become its own follow-up inception.
- **Human:** (awaiting input)
