# /claim — reserve a unit of work on a topic (T-2097 skill-layer wrap)

WRITE-side daily verb for substrate primitive #1 (T-2019/T-2042).
Wraps `termlink channel claim` so an operator can reserve a unit of
work in one keystroke instead of remembering argument order. The
acquire side of the work-stealing primitive; pair with `/claims`
(T-2093) for the read side.

**This skill WRITES state.** Every successful invocation produces a
real claim envelope on the hub with a lease. Unlike the four
substrate-read skills (T-2092..T-2095) this is not free of
consequence — if you claim work you don't intend to do, another
worker can't pick it up until the lease expires.

The full substrate #1 daily-verb surface:

- **`/claims`** (T-2093) — READ ("what's claimed?")
- **`/claim`** (this skill) — WRITE ("reserve this unit")
- `termlink channel release` — release (CLI, low-friction enough as-is)
- `termlink channel claim-transfer` — cooperative handoff (T-2046; sibling pair with /find-idle dispatch flow)

**Invocation:**

| Form | Action |
|------|--------|
| `/claim <topic> <unit-id>` | Claim `<unit-id>` on `<topic>` |
| `/claim <topic>` | Auto-mint unit-id (random 16-hex) |
| `/claim <topic> <unit-id> --ttl-secs 300` | Override lease TTL (default per substrate impl) |
| `/claim <topic> <unit-id> --owner <id>` | Override owner (default: resolved from `/be-reachable` state) |
| `/claim <topic> <unit-id> --json` | Machine-readable envelope |

**Auto-resolution defaults** (mirror of T-1857 `/broadcast-chat`
sender-resolution chain):

- `--owner` resolves via:
  1. `--owner` flag (explicit)
  2. `$TERMLINK_AGENT_ID` env
  3. `~/.termlink/be-reachable.state` (set by `/be-reachable`)
  4. **Refuse** with hint if all three fail. Never invent an owner.

- `--unit-id` auto-mints a random 16-hex token when omitted (mirror of
  T-2049 client-msg-id pattern). When the operator wants a meaningful
  unit-id (task ID, file path), pass it explicitly.

## Step 1: Pre-flight

Run:

```
termlink channel claim --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
claim: `termlink` CLI not on PATH or substrate primitive #1 (channel.claim)
not available in this build. Ensure you're on a version with T-2019
shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Normalize:

- First positional → `<topic>`
- Second positional (if present) → `<unit-id>` (else auto-mint)
- Flag tokens (`--owner`, `--ttl-secs`, `--json`) → pass through

If `<topic>` is missing: **stop**. Print:

```
claim: topic required.

Usage:
  /claim <topic> [<unit-id>]              # reserve work on <topic>
  /claim <topic> --owner <id>             # explicit owner override
  /claim <topic> <unit-id> --ttl-secs N   # lease TTL override

If you don't know what topics exist:
  termlink topics                          # list local-hub topics
  /claims --all                            # see what's already claimed
```

Owner-resolution check (run before claim):

```
if [ -z "$EXPLICIT_OWNER" ] && [ -z "$TERMLINK_AGENT_ID" ] && [ ! -f ~/.termlink/be-reachable.state ]; then
  print refusal hint and stop
fi
```

Refusal hint:

```
claim: owner unresolved.

The claim's owner identifies who holds it (visible in /claims output
+ enforced by claim-transfer). Resolve via one of:

  /be-reachable                              # auto-establish session identity
  /claim <topic> <unit-id> --owner <id>      # explicit override
  export TERMLINK_AGENT_ID=<id>              # env override
```

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Surface refusal modes loudly

Substrate #1 has a known refusal taxonomy. Recognize each and surface
an actionable next-step hint rather than passing through the raw
error:

**CLAIM_ALREADY_HELD** (already claimed by another owner):

```
claim refused: <unit-id> on <topic> is already held.

Inspect: /claims <topic>                              # see who holds it
Cooperative handoff: termlink channel claim-transfer  # T-2046
Wait for natural release: claim leases expire — re-run /claim later.
```

**AUTH_FAIL** (-32001 or similar):

```
claim refused: authentication failure.

Likely cause: hub rotated its secret (substrate-related: T-1052/T-1053).
Diagnose: termlink fleet doctor
Heal: termlink fleet reauth <profile> --bootstrap-from auto
```

**RATE_LIMITED** (-32008):

```
claim refused: rate-limited by hub governor.

The hub is throttling per-sender requests. Inspect:
  /governor --only-pressured
  termlink hub status --governor

Wait `retry_after_ms` (see error payload) and retry.
```

**HUB_AT_CAPACITY** (-32019):

```
claim refused: hub at connection capacity.

The hub refused a new connection (rare under normal operation).
Inspect: /governor
Wait `retry_after_ms` and retry.
```

**Other / unknown errors:** surface stderr verbatim with no editorialization.

## Step 5: Render success

For default human-format output, render:

```
claim acquired:
  topic:      <topic>
  unit_id:    <unit-id>          (auto-minted)  [if auto-minted]
  owner:      <owner>            (from /be-reachable)  [if resolved]
  claim_id:   <claim-id from response>
  lease_ttl:  <secs>
  expires_at: <RFC3339>

Next steps:
- Do the work.
- Release when done: termlink channel release <claim-id>
- Or hand off: termlink channel claim-transfer --claim-id <claim-id> --to-owner <other> --by <owner>
```

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render.

## Step 6: Owner-not-resolved success-side hint

When success uses auto-resolved owner (not explicit), prepend a one-line
confirmation:

```
(using owner=<id> resolved from ~/.termlink/be-reachable.state)
```

This is observability — the operator knows who they're claiming as
without having to read state files separately.

## Rules

- **Writes state.** Unlike the four substrate-read skills, every
  invocation produces a real envelope. Don't auto-retry on transient
  errors — surface the refusal and let the operator decide.
- **Never invent an owner.** If owner-resolution fails, refuse with a
  hint. Silent owner-defaults break claim accountability.
- **Loud refusals.** Each known error class gets an actionable
  next-step. Don't pass through raw `Err(-32017): ...` lines.
- **No `AskUserQuestion`** — just run and report.
- **Auto-mint is opt-in.** When the operator provides a unit-id,
  respect it byte-for-byte. The auto-mint is only when omitted.

## Common patterns

**Reserve work tied to a focus task:**

```
/claim work-queue T-1234
```

The task ID becomes the unit-id — visible in `/claims` output for
other workers to understand what's in flight.

**Quick anonymous reservation:**

```
/claim work-queue
```

Auto-mints a 16-hex unit-id. Use when you just need exclusive access
to "the next work item" and don't care about labeling.

**Explicit owner (multi-session scenarios):**

```
/claim work-queue T-1234 --owner claude-orchestrator
```

Useful when the claim should be visible as held by an orchestrator
identity, not the calling session.

**Cooperative handoff after claim:**

```
/claim work-queue T-1234                                          # acquire
/find-idle --capability rust                                       # find worker
termlink channel claim-transfer --claim-id <id> --to-owner <peer> --by <self>
```

This is the substrate primitive #3 (T-2046) atomic handoff pattern.

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #1
  WRITE-side at the daily-verb tier.
- T-2019 / T-2042 — substrate primitive #1 implementation (the
  underlying `channel.claim` RPC).
- T-2046 — `channel claim-transfer` (cooperative handoff after claim).
- T-2093 / `/claims` — sibling READ-side skill.
- T-1857 / `/broadcast-chat` — sender-resolution pattern reused here.
- T-1841 / `/be-reachable` — auto-establishes the identity this skill
  resolves owner from.
- T-2049 — client-msg-id pattern (the auto-mint inspiration).
- T-2092 / `/find-idle` — the natural pre-claim verb (find worker, then
  claim+transfer).
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
