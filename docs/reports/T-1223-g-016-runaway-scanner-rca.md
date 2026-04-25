# T-1223: G-016 root cause — DRY_RUN=0 bootstrap source

**Task:** T-1223 (inception, NO-GO 2026-04-24)
**Status:** WORK-COMPLETED — root cause traced; mitigations in place
**Backfilled:** 2026-04-25 (T-1258, audit cleanup)

## Problem statement

G-016 fired when the silent-session scanner ran destructively against
`/opt/termlink` at 2026-04-24T11:38:23Z, generating 92 handover commits
in 30 minutes before manual kill. T-1222 capped the blast radius
(per-invocation cap, default=10) but deferred identifying *what
launched the scanner*. Without knowing the trigger, we couldn't tell
whether the compound flaw could recur via a different path.

## Assumptions tested

| ID  | Assumption                                                            | Outcome    |
| --- | --------------------------------------------------------------------- | ---------- |
| A1  | A Claude Code hook or cron entry fired the scanner.                   | Rejected   |
| A2  | The invocation explicitly set `DRY_RUN=0` to bypass the safety default. | Rejected (guard didn't exist yet) |
| A3  | The bootstrap source is recoverable from transcripts / hook configs.  | Confirmed  |

## Findings

### F1 — Zero cron / hook trigger (A1 rejected)

- No `/etc/cron.d/*`, user crontab, or project `cron-registry.yaml` entry
  installs the scanner.
- `settings.json` PostToolUse / SessionStart arrays contain zero
  references to `session-silent-scanner`.
- The claim "Invoked via cron every 15 min" in the scanner docstring is
  aspirational — no cron entry exists.

### F2 — No DRY_RUN=0 bypass (A2 rejected)

At 11:38:22Z the scanner file had **no DRY_RUN variable at all**. The
guard was added by Edit at 11:41:24Z — 3 minutes AFTER runaway started.
So the destructive default was the only mode available; no explicit
`DRY_RUN=0` flag was set.

### F3 — Bootstrap source: smoke-test dispatcher command (A3 confirmed)

Exact trigger Bash tool_use at 2026-04-24T11:38:22.806Z in session
`d938f9cf-…`:

- First leg: `echo` stub JSON piped into `hook session-end`
- Second leg: `hook session-silent-scanner 2>&1 | head -3`

The second leg invoked the agent dispatcher directly against real
`/opt/termlink`. The dispatcher (`bin/fw` line 3889:
`exec bash "$_hook_script" "$@"`) is a pass-through — it does NOT
sandbox, does NOT set `DRY_RUN=1`. Combined with the scanner's
then-missing DRY_RUN guard, the invocation recovered the full backlog
of stale transcripts.

The `| head -3` suffix explains why the session appeared to exit
quickly while the scanner kept running: `head` closes its stdin after
3 lines, but the scanner's `subprocess.run(handover)` loop writes to
its own stdout (not through head). The scanner continued ~30 min
until manually killed.

### F4 — Compound failure mode (three flaws, two fixed)

| Flaw                                                          | Status                       |
| ------------------------------------------------------------- | ---------------------------- |
| Scanner had no DRY_RUN safety default                         | Fixed (prior session, default=1) |
| Scanner had no per-invocation cap                             | Fixed (T-1222, default=10)   |
| Dispatcher is pass-through (no DRY_RUN guard at entry)        | Unchanged                    |

Residual risk: if a future agent invokes the dispatcher from a project
with `DRY_RUN=0` explicitly set in environment, or if an agent edits
the scanner to remove the DRY_RUN default, the cap alone limits blast
radius to 10 commits/invocation — annoying but not catastrophic.

## Decision: NO-GO on further structural fix

**Rationale:** With (1)+(2) in place, the dispatcher can no longer
produce a 92-commit storm — even if invoked directly with `DRY_RUN=0`,
the cap bounds damage to 10 commits per invocation. The hypothetical
remaining failure mode (agent edits the DRY_RUN default out) is caught
by code review on the scanner file, not by dispatcher guards.

A structural fix to the dispatcher (e.g., require explicit `DRY_RUN=0`
opt-in at the `fw hook` verb level) would touch a framework-wide
pattern affecting all 20+ hook scripts. That's disproportionate to the
residual risk.

## Scope fence

- **IN:** Trace the trigger. Assess whether current mitigations are sufficient.
- **OUT:** Rewriting the hook dispatcher (framework-wide change requiring
  its own inception); retiring the `fw hook` verb pattern.

## Secondary action (no task needed)

Behavioral note added to learnings: don't smoke-test scanner-style
dispatchers against real `PROJECT_ROOT`; use stub test with sandbox or
explicit `DRY_RUN=1` env override.

## References

- T-1222 (prior cap fix, default=10).
- G-016 in `.context/project/concerns.yaml`.
- T-1258 (this artifact backfill).
