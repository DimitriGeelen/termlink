# The main lease: one main session per project (T-3654, OD-18 / R-67, R-68)

## What "main" is

Every project has at most one **main**: one Claude Code *session* (not an agent id, not a
host) that surfaces, claims and answers the project's role mail. Other sessions of the same
agent leave that mail unread for main and say who main is.

Main is a lease at the project's home hub, built on the ordinary claim primitives (no new hub
protocol):

| piece | where |
|---|---|
| the lease | a hub claim on offset 0 of topic `main:<project>` |
| holder | claimer `main;s=<session>;a=<agent>;h=<host>;p=<pid>` |
| audit trail and generation | signed `main.lease.v1` records on the same topic (acquire, release, lapse, pin); retention: newest 1000 |
| generation | the highest acquire generation in the log; +1 on every holder change, never reused |

Rules (R-67.a / R-67.a1):

1. The holder renews; a busy holder keeps main. Only the session renews, never its sidecar
   (a sidecar proves a process, not a turn: R-67.c).
2. Another session takes over only after **lapse + quiet period**, and not before the
   **cooldown** since the last generation change. A clean release skips the quiet period.
3. No automatic take-back: a holder that lost main finds it held and is told
   (`lost main at generation N to <session>`). A lapsed holder may resume while nobody else
   took it (same generation).
4. **Operator pin first**: a pinned session takes main at its next claim, even from a live
   holder; the override is recorded (`reason: pin-override`, `from: <old holder>`).
5. Duplicate holders at one generation (only after a restored or partitioned log) resolve by
   pin, else the longest uninterrupted holder, else the lowest session id.
6. On takeover, open claims the old holder left on `--obligation-topic` topics are recorded
   in the acquire record as `class: uncertain, to: operator`.
7. Fail-closed: a hub that cannot be read is **unknown**, never main.

Constants (T-3654 Decisions): lease 15 min, quiet period 5 min, cooldown 10 min. Worst-case
takeover after a holder dies: 20 minutes. Tests shorten them with
`TERMLINK_MAIN_TEST_LEASE_MS` / `_QUIET_MS` / `_COOLDOWN_MS` (test seams only).

## How to see it

```
termlink main status [--project P] [--json]       # holder, generation, since, pin, eligible_at, recent records
termlink main am-i --session <id> [--json]         # exit 0 main, 1 not-main (holder printed), 2 unknown
```

The sidecar's `termlink sidecar status --json` carries the same answer as `main.lease`
(next to `main.pinned` and `main.liveness`); MCP: `termlink_sidecar_main` (whole `main`
object) and `termlink_main_status {project, session?}`.

Defaults: `--project` from `$TERMLINK_MAIN_PROJECT`, else `project_name` in `.framework.yaml`
above the cwd; `--hub` the local hub; `--session` from `--session-from-stdin` (Claude Code
hook input JSON; an empty `session_id` counts as absent), else `$CLAUDE_SESSION_ID`, else `$CLAUDE_CODE_SESSION_ID`
(what Claude Code exports to its shells), else `$TERMLINK_SESSION_ID` **only outside Claude Code**. Inside Claude Code
(`$CLAUDECODE` set) the TermLink session id is a different identity, so with no Claude session id the verbs answer
unknown (exit 2) rather than guess (T-3668).

## How to pin (operator)

```
termlink main pin --to <session-id> --reason "<why>"     # the session takes main at its next claim
termlink main pin --clear --reason "<why>"
```

Every pin is a record on `main:<project>` (who, why, when) and shows in `status`.

## How hooks ask (the interface for AEF T-5014 and ring20)

| hook | command | meaning of the exit code |
|---|---|---|
| SessionStart | `termlink main claim --session-from-stdin` | 0 this session is main; 1 another is (line names it); 2 unknown |
| UserPromptSubmit / PostToolUse | `termlink main claim --session-from-stdin` (renews when held, cheap) | same |
| before surfacing peer mail | `termlink main am-i --session-from-stdin` (read-only) | surface only on 0; on 1 print the one line; on 2 do not surface |
| SessionEnd | `termlink main release --session-from-stdin` | clean shutdown hands main on without the quiet period |

`--json` gives `{verdict, holder{session,agent,host,pid}, generation, since_ms, claimed_until_ms,
pinned, vacant_since_ms, eligible_at_ms, lost, recent[...]}`. Any session that renews on
UserPromptSubmit keeps main while it is being used; an idle session lapses after the lease and
another session takes over after the quiet period.

## What the sidecar does for non-main sessions (R-67.a1 (4), minimum)

Configure `main_project` in the sidecar config (or `$TERMLINK_MAIN_PROJECT`). The sidecar reads
the lease every 15 s (it never writes it). When the lease names another session **of this
agent** as main, `pull` from a non-main session hands out no **role** mail: it stays stored for
main, and the session gets one line, once per generation:
`TermLink: this session is not main of P; role mail is left unread for main S (generation N).`

Mail of a conversation bound to that exact session is still handed to it (P-5, T-3695; ARC4
7.1.a). A conversation is bound to the first session that sends or replies in it, or that has
one of its messages handed over; the binding never moves. Main still reads every message, so
mail bound to a session that has ended is not stranded.

Rollback to the earlier all-mail withhold (T-3654) is one step: create the flag file
`withhold-all` in the sidecar's store directory (next to `delivery-off`); delete it to undo. It is
read at every pull, needs no restart, and anything at that path that cannot be read counts as set.
While it is set, bound mail held back from a non-main session counts toward "mail waiting".

Fail direction: an unknown, stale or vacant lease, or a main that is another agent's session,
leaves delivery as before, so a lease problem can never stop all delivery.

Not built yet: forwarding to a main behind another sidecar/host and telling the sender
"delivered via X; main is Y (generation N)" (R-67.a1 (4) full form); AEF's own receiver hook
gating (AEF T-5014).
