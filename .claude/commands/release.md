# /release — release a claim (T-2098 skill-layer wrap)

WRITE-side daily verb for substrate primitive #1 (T-2019/T-2042). Wraps
`termlink channel release <claim-id>` so an operator can free a claim
in one keystroke instead of recalling the CLI's positional argument.
The release side of the work-stealing primitive — the natural pair to
`/claim` (T-2097).

**This skill WRITES state.** Every successful invocation removes a
real claim envelope from the hub. Once released, other workers can
pick up the freed unit on their next `/find-idle` → `/claim` cycle.

The full substrate #1 daily-verb surface:

- **`/claims`** (T-2093) — READ ("what's claimed?")
- **`/claim`** (T-2097) — WRITE ACQUIRE ("reserve this unit")
- **`/release`** (this skill) — WRITE RELEASE ("free this claim")
- `termlink channel claim-transfer` — cooperative handoff (T-2046; sibling pair with /find-idle dispatch flow)

**Invocation:**

| Form | Action |
|------|--------|
| `/release <claim-id>` | Release the claim by id |
| `/release <claim-id> --by <id>` | Override identity (default: resolved from `/be-reachable` state) |
| `/release <claim-id> --json` | Machine-readable envelope |

**Auto-resolution defaults** (mirror of T-1857 `/broadcast-chat`
sender-resolution chain, same as T-2097 `/claim`):

- `--by` resolves via:
  1. `--by` flag (explicit)
  2. `$TERMLINK_AGENT_ID` env
  3. `~/.termlink/be-reachable.state` (set by `/be-reachable`)
  4. **Refuse** with hint if all three fail. Never invent an owner.

The `--by` field must match the claim's current owner (the hub
enforces this via CLAIM_NOT_OWNED -32017). Auto-resolution is the
right default precisely because the operator usually IS the owner —
they're releasing their own claim.

## Step 1: Pre-flight

Run:

```
termlink channel release --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
release: `termlink` CLI not on PATH or substrate primitive #1 release
not available in this build. Ensure you're on a version with T-2019
shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Normalize:

- First positional → `<claim-id>`
- Flag tokens (`--by`, `--json`) → pass through

If `<claim-id>` is missing: **stop**. Print:

```
release: claim-id required.

Usage:
  /release <claim-id>                    # release the claim
  /release <claim-id> --by <id>          # explicit identity override
  /release <claim-id> --json             # machine-readable

If you don't know your claim-id:
  /claims <topic>                        # see active claims (find yours)
  /claims --all                          # fleet-wide view
```

Owner-resolution check (run before release):

```
if [ -z "$EXPLICIT_BY" ] && [ -z "$TERMLINK_AGENT_ID" ] && [ ! -f ~/.termlink/be-reachable.state ]; then
  print refusal hint and stop
fi
```

Refusal hint:

```
release: identity unresolved.

The `--by` field identifies who is releasing and MUST match the claim's
current owner (the hub refuses CLAIM_NOT_OWNED otherwise). Resolve via:

  /be-reachable                              # auto-establish session identity
  /release <claim-id> --by <id>              # explicit override
  export TERMLINK_AGENT_ID=<id>              # env override
```

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Surface refusal modes loudly

Substrate #1 release has a known refusal taxonomy. Recognize each and
surface an actionable next-step hint rather than passing through the
raw error:

**CLAIM_NOT_FOUND** (-32018 or similar — claim id never existed or
already released):

```
release refused: <claim-id> not found.

Likely cause: already released, or the claim expired naturally.
Inspect: /claims <topic>                              # current state
Audit:   termlink channel claims-history --since 1    # recent events (T-2074)
```

**CLAIM_NOT_OWNED** (-32017 — claim exists but `--by` is not the holder):

```
release refused: <claim-id> is not held by <by-identity>.

Inspect: /claims <topic>                              # see who actually holds it
Cooperative handoff: termlink channel claim-transfer  # T-2046 (if you got the claim
                                                        through legitimate handoff,
                                                        the prior owner needs to
                                                        release-or-transfer)
Operator-bypass: termlink channel claim-force-release # Tier-0, last-resort
```

**AUTH_FAIL** (-32001 or similar):

```
release refused: authentication failure.

Likely cause: hub rotated its secret (substrate-related: T-1052/T-1053).
Diagnose: termlink fleet doctor
Heal: termlink fleet reauth <profile> --bootstrap-from auto
```

**RATE_LIMITED** (-32008):

```
release refused: rate-limited by hub governor.

Inspect: /governor --only-pressured
Wait `retry_after_ms` (see error payload) and retry.
```

**Other / unknown errors:** surface stderr verbatim with no editorialization.

## Step 5: Render success

For default human-format output, render:

```
claim released:
  claim_id:  <claim-id>
  topic:     <topic from response>
  unit_id:   <unit-id from response>
  by:        <by>                    (from /be-reachable)  [if auto-resolved]
  released_at: <RFC3339 from response>

The unit is now free. Next workers will see it on their next
/find-idle → /claim cycle.
```

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render.

## Step 6: Owner-not-resolved success-side hint

When success uses auto-resolved identity (not explicit), prepend a
one-line confirmation:

```
(using by=<id> resolved from ~/.termlink/be-reachable.state)
```

This is observability — the operator confirms they released as
themselves, not some stale env var.

## Rules

- **Writes state.** Unlike the four substrate-read skills, every
  invocation removes a real envelope. Don't auto-retry on transient
  errors — surface the refusal and let the operator decide.
- **Never invent an identity.** If `--by` resolution fails, refuse
  with a hint. The hub will refuse silently-defaulted identities via
  CLAIM_NOT_OWNED anyway — better to refuse loudly client-side.
- **Loud refusals.** Each known error class gets an actionable
  next-step. Don't pass through raw `Err(-32017): ...` lines.
- **No `AskUserQuestion`** — just run and report.
- **No bulk-release.** This skill releases ONE claim by id. For
  fleet-wide cleanup, the operator drops to CLI. Bulk operations are
  not daily-verb-shaped.

## Common patterns

**Standard release after work completion:**

```
/claim work-queue T-1234         # acquire (T-2097)
... do the work ...
/release <claim-id>              # release this skill
```

The `<claim-id>` is in `/claim`'s success output.

**Release after a transfer (the new owner releases when done):**

```
# After T-2046 transfer
termlink channel claim-transfer --claim-id <id> --to-owner self --by orchestrator
... do the work ...
/release <claim-id>              # using self identity from /be-reachable
```

**Release when you've forgotten the claim-id:**

```
/claims work-queue                # find your unit_id in the active list
/release <claim-id>               # paste the id from above
```

**Release with explicit identity (multi-session scenarios):**

```
/release <claim-id> --by claude-orchestrator
```

Useful when the calling session is acting on behalf of an
orchestrator identity that holds the claim.

## Not in scope

- **Bulk release.** No `/release --all` or `/release --topic <T>`.
  Daily-verb means one unit per invocation. Operators wanting fleet
  cleanup use the CLI directly.
- **Force release.** Releasing claims you don't own is Tier-0
  (`termlink channel claim-force-release`). This skill never bypasses
  ownership — it would silently undermine claim accountability.
- **Auto-release on session end.** Out of scope; explicit release is
  intentional. A session that crashes leaves the lease to expire
  naturally (the substrate handles this — see T-2042).

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #1
  RELEASE-side at the daily-verb tier.
- T-2019 / T-2042 — substrate primitive #1 implementation (the
  underlying `channel.release` RPC + lease lifecycle).
- T-2046 — `channel claim-transfer` (cooperative handoff alternative
  to release-then-re-claim).
- T-2093 / `/claims` — sibling READ-side skill (find your claim-id).
- T-2097 / `/claim` — sibling ACQUIRE-side skill (the natural pair).
- T-1857 / `/broadcast-chat` — sender-resolution pattern reused here.
- T-1841 / `/be-reachable` — auto-establishes the identity this skill
  resolves `--by` from.
- T-2074 / `channel claims-history` — retrospective audit (when you
  want to know "did I already release this?").
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
