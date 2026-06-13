# /claim — reserve an offset on a topic (T-2097 / T-2100-fixed skill-layer wrap)

WRITE-side daily verb for substrate primitive #1 (T-2019/T-2042/T-2032).
Wraps `termlink channel claim` so an operator can reserve an offset
of work in one keystroke instead of remembering positional + flag
order. The acquire side of the work-stealing primitive; pair with
`/claims` (T-2093) for the read side.

**This skill WRITES state.** Every successful invocation produces a
real claim envelope on the hub with a lease. Unlike the four
substrate-read skills (T-2092..T-2095) this is not free of
consequence — if you claim work you don't intend to do, another
worker can't pick it up until the lease expires.

**Substrate claim model is offset-based.** A claim is exclusive
ownership of a specific OFFSET on a specific TOPIC for `ttl_ms`. The
operator must know which offset they're claiming (typically from
`channel subscribe` or `/claims` showing unread offsets). There is
NO `unit-id` concept — the offset IS the unit. T-2049's client-msg-id
auto-mint pattern is for `channel post`, not `channel claim`; they
are distinct surfaces.

The full substrate #1 daily-verb surface:

- **`/claims`** (T-2093) — READ ("what's claimed?")
- **`/claim`** (this skill) — WRITE ACQUIRE ("reserve this offset")
- **`/release`** (T-2098) — WRITE COMPLETE/RETRY
- **`/claim-transfer`** (T-2099) — WRITE COOPERATIVE HANDOFF

**Invocation:**

| Form | Action |
|------|--------|
| `/claim <topic> <offset>` | Claim offset `<offset>` on `<topic>` |
| `/claim <topic> <offset> --ttl-ms 60000` | Override lease TTL (default 30000ms = 30s; hub-clamped to 1h) |
| `/claim <topic> <offset> --claimer <id>` | Override claimer (default: resolved from `/be-reachable` state) |
| `/claim <topic> <offset> --json` | Machine-readable envelope |

**Auto-resolution defaults** (mirror of T-1857 `/broadcast-chat`
sender-resolution chain):

- `--claimer` resolves via:
  1. `--claimer` flag (explicit)
  2. `$TERMLINK_AGENT_ID` env
  3. `~/.termlink/be-reachable.state` (set by `/be-reachable`)
  4. **Refuse** with hint if all three fail. Never invent a claimer.

## Step 1: Pre-flight

Run:

```
termlink channel claim --help >/dev/null 2>&1
```

The skill validates against the actual CLI surface:

```
Usage: termlink channel claim [OPTIONS] --claimer <CLAIMER> <TOPIC> <OFFSET>

Arguments:
  <TOPIC>   Topic name (must already exist — `channel create` first)
  <OFFSET>  Offset within the topic to exclusively claim

Options:
      --claimer <CLAIMER>  Worker identifier — your stable identity
      --ttl-ms <TTL_MS>    Lease TTL in milliseconds (default 30000; hub-clamped to 1h)
      --hub <HUB>          Target hub address
      --json               Output as JSON
```

If --help exit non-zero: **stop**. Print:

```
claim: `termlink` CLI not on PATH or substrate primitive claim not
available in this build. Ensure you're on a version with T-2029
shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Normalize:

- First positional → `<topic>`
- Second positional → `<offset>` (must be a non-negative integer)
- Flag tokens (`--claimer`, `--ttl-ms`, `--hub`, `--json`) → pass through

If `<topic>` is missing: **stop**. Print:

```
claim: topic required.

Usage:
  /claim <topic> <offset>                # reserve offset on topic
  /claim <topic> <offset> --claimer <id> # explicit claimer override
  /claim <topic> <offset> --ttl-ms N     # lease TTL in milliseconds

If you don't know what topics exist:
  termlink topics                        # list local-hub topics
  /claims --all                          # see what's already claimed
```

If `<offset>` is missing: **stop**. Print:

```
claim: offset required.

The substrate claim model is offset-based — you reserve a specific
envelope by its offset on the topic's log. To find an offset:

  termlink channel subscribe <topic>     # see envelopes + offsets
  /claims <topic>                        # see what's already claimed (avoid these)

Usage: /claim <topic> <offset>
```

If `<offset>` is not a non-negative integer: **stop**. Print:

```
claim: <offset> must be a non-negative integer (substrate claims are offset-based,
not unit-id-based). See `termlink channel subscribe <topic>` for valid offsets.
```

Claimer-resolution check (run before claim):

```
if [ -z "$EXPLICIT_CLAIMER" ] && [ -z "$TERMLINK_AGENT_ID" ] && [ ! -f ~/.termlink/be-reachable.state ]; then
  print refusal hint and stop
fi
```

Refusal hint:

```
claim: claimer identity unresolved.

The `--claimer` field identifies who is holding the claim — gates
ownership for renew/release. Resolve via one of:

  /be-reachable                              # auto-establish session identity
  /claim <topic> <offset> --claimer <id>     # explicit override
  export TERMLINK_AGENT_ID=<id>              # env override
```

## Step 3: Run the verb

Construct the command:

```
termlink channel claim --claimer <resolved-claimer> [--ttl-ms <ms>] [--json] <topic> <offset>
```

Execute via Bash. Capture stdout + stderr + exit code.

## Step 4: Surface refusal modes loudly

Substrate claim has a known refusal taxonomy. Recognize each and
surface an actionable next-step hint:

**CLAIM_CONFLICT** (offset already claimed by another worker):

```
claim refused: offset <N> on <topic> is already held.

Inspect: /claims <topic>                              # see who holds it
Cooperative handoff: /claim-transfer (T-2099)         # only the current holder can hand off
Wait for natural release: claim leases expire — re-run /claim later.
Pick a different offset: termlink channel subscribe <topic>
```

**AUTH_FAIL**:

```
claim refused: authentication failure.

Likely cause: hub rotated its secret (substrate-related: T-1052/T-1053).
Diagnose: termlink fleet doctor
Heal: termlink fleet reauth <profile> --bootstrap-from auto
```

**RATE_LIMITED** (-32008):

```
claim refused: rate-limited by hub governor.

Inspect: /governor --only-pressured
Wait `retry_after_ms` (see error payload) and retry.
```

**HUB_AT_CAPACITY** (-32019):

```
claim refused: hub at connection capacity.

Inspect: /governor
Wait `retry_after_ms` and retry.
```

**Other / unknown errors:** surface stderr verbatim with no editorialization.

## Step 5: Render success

For default human-format output, render:

```
claim acquired:
  topic:        <topic>
  offset:       <offset>
  claimer:      <claimer>           (from /be-reachable)  [if auto-resolved]
  claim_id:     <claim-id from response>
  ttl_ms:       <ms>
  claimed_until: <RFC3339>

Next steps:
- Do the work at offset <N>.
- Release when done:   /release <claim-id>           # default: --ack (completed)
- Or release for retry: /release <claim-id> --retry  # cursor stays; next worker gets it
- Or hand off:         /claim-transfer <claim-id> <new-owner>
- Or renew if running long: termlink channel renew --claim-id <claim-id> --claimer <id>
```

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render.

## Step 6: Claimer-resolved success-side hint

When success uses auto-resolved claimer (not explicit), prepend a
one-line confirmation:

```
(using claimer=<id> resolved from ~/.termlink/be-reachable.state)
```

This is observability — the operator confirms they claimed as
themselves, not some stale env var.

## Rules

- **Writes state.** Unlike the four substrate-read skills, every
  invocation produces a real envelope. Don't auto-retry on transient
  errors — surface the refusal and let the operator decide.
- **Never invent a claimer.** If claimer-resolution fails, refuse
  with a hint. Silent defaults break claim accountability.
- **Loud refusals.** Each known error class gets an actionable
  next-step. Don't pass through raw `Err(-32017): ...` lines.
- **No `AskUserQuestion`** — just run and report.
- **Offset is required.** The substrate model is offset-based — there
  is no `unit-id` to auto-mint. The operator must pick an offset.
- **Default TTL is 30000ms.** Pass `--ttl-ms` to override. The hub
  clamps to 1h max.

## Common patterns

**Reserve work at a known offset:**

```
termlink channel subscribe work-queue --limit 5      # see recent envelopes + offsets
/claim work-queue 42                                  # reserve offset 42
```

**Reserve with extended TTL for long-running work:**

```
/claim work-queue 42 --ttl-ms 600000   # 10 minutes
```

For work that may take >30s, set TTL appropriately — OR renew
periodically with `termlink channel renew`.

**Explicit claimer (multi-session scenarios):**

```
/claim work-queue 42 --claimer claude-orchestrator
```

Useful when the claim should be visible as held by an orchestrator
identity, not the calling session.

**Cooperative handoff after claim (orchestrator → worker):**

```
/claim work-queue 42                                          # orchestrator acquires
/find-idle --capability rust                                   # find worker
/claim-transfer <claim-id> <worker-id>                         # atomic handoff (T-2099)
```

This is the substrate primitive #3 atomic handoff pattern — eliminates
the release-then-claim race window.

## Not in scope

- **Auto-pick offset.** The skill won't pick "the next unclaimed
  offset" for you — that requires reading state and racing against
  other workers. Operators pick deliberately from `channel subscribe`
  output.
- **Bulk claim.** No `/claim --topic <T> --range a..b`. Daily-verb
  means one offset per invocation.

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #1
  ACQUIRE-side at the daily-verb tier.
- T-2029 — substrate `channel.claim` JSON-RPC verb (the underlying
  implementation).
- T-2032 — substrate primitive #1 CLI surface (claim/renew/release
  wrappers; the parent task that shipped these verbs).
- T-2046 — `channel claim-transfer` (cooperative handoff after claim).
- T-2093 / `/claims` — sibling READ-side skill.
- T-2098 / `/release` — natural completion verb for this skill.
- T-2099 / `/claim-transfer` — atomic cooperative handoff.
- T-1857 / `/broadcast-chat` — sender-resolution pattern reused here.
- T-1841 / `/be-reachable` — auto-establishes the identity this skill
  resolves claimer from.
- T-2092 / `/find-idle` — the natural pre-claim verb (find worker, then
  claim+transfer).
- T-2100 — fix-up task that corrected this skill's flag names and
  offset-based model (origin: shipped initially with wrong shorthand
  extrapolated from CLAUDE.md row).
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
