# /closeable — list agent-closeable backlog tasks (T-2207)

Symmetric companion to `fw task verify` (which surfaces Human-AC pending). Answers
the agent-side question: **"Which agent-owned tasks have 0 unchecked Agent ACs and
are ready to close RIGHT NOW?"**

Read-only — does NOT close anything. Renders two sections so the operator (or
agent) can pick + close manually:

- **Full-close-ready** — Agent ✓ AND Human ✓ both at 0 unchecked → fully closeable.
- **Partial-complete-ready** — Agent ✓ but Human ✗ > 0 → close to partial-complete
  pending operator click.

Wraps `scripts/list-closeable.sh`. Pure local read of `.tasks/active/` — no auth,
no network, no state mutation. Safe anywhere.

**Invocation:**

| Form | Action |
|------|--------|
| `/closeable` | Render human-format table (two sections) |
| `/closeable --json` | Emit `{ok, full_close_ready[], partial_complete_ready[], summary}` JSON envelope |

## Step 1: Pre-flight

Run:

```
bash scripts/list-closeable.sh --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
closeable: wrapper not found at scripts/list-closeable.sh.
Ensure you're in the TermLink project root (cd /opt/termlink).
```

## Step 2: Pass-through

Forward all arguments verbatim to the script:

```
bash scripts/list-closeable.sh "$@"
```

The script handles its own validation, JSON mode, and empty-state messaging.

## Step 3: Empty-state framing

When the script reports zero closeable tasks (the affirmative-healthy state),
its own output already says so + suggests `fw task verify` / `/peers` as next
reads. Do NOT add additional narration — the user can act directly on the
suggestion.

## Step 4: Non-empty suggestion

When the script reports >0 closeable tasks, append (once, after the table):

```
Substrate-driven drain:
  bash scripts/orchestrator-backlog-drain.sh --live --limit 5
  → dispatches up to 5 ready units to LIVE workers (T-2204)
```

Only if `bash scripts/orchestrator-backlog-drain.sh --help` exists — otherwise
skip silently. The substrate kit is the natural next step for fleet-wide drain.

## Rules

- **Read-only by contract.** This skill never modifies `.tasks/`. The script's
  output suggests `fw task update T-XXX --status work-completed` per row — the
  operator chooses whether to run them.
- **Don't auto-close.** Even when `full_close_ready` is non-zero, the agent must
  NOT chain `fw task update` calls. Each closure has gates (RCA, recommendation
  block, verification commands) that the operator may want to review per task.
- **Pair with `/peers` when nothing's closeable.** An empty closeable list often
  means the agent has drained what it can — the next move is dispatching work
  to other workers (substrate orchestrator pattern).
- **No `AskUserQuestion`** — just run and report.

## Common patterns

**End-of-session check:**

```
/closeable
```

**Pipe into substrate orchestrator:**

```
/closeable --json | jq -r '.full_close_ready[].id'
```

**See pile health alongside human-AC queue:**

```
/closeable && fw task verify --compact
```

## Related

- T-2207 (this skill)
- `fw task verify` (the Human-AC counterpart)
- T-2204 (`scripts/orchestrator-backlog-drain.sh` — substrate-driven drain)
- T-2018 §6 substrate primitives — the dispatch surface this enables
- G-008 partial-complete backlog concern (the structural problem this attacks)
