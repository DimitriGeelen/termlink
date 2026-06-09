# /claim-transfer — atomic cooperative claim handoff (T-2099 skill-layer wrap)

WRITE-side daily verb for substrate primitive #3 (T-2046). Wraps
`termlink channel claim-transfer` so an operator can hand a claim's
ownership to another agent in one keystroke instead of typing three
long `--flag VALUE` pairs.

**This is the orchestrator-to-worker handoff verb.** The whole point
of substrate primitive #3 is the atomicity: the lease moves from old
owner to new owner inside the hub with zero gap. Compared with
release-then-claim, claim-transfer eliminates the race window during
which a third party could steal the unit.

The full substrate #1 daily-verb surface — NOW COMPLETE:

- **`/claims`** (T-2093) — READ ("what's claimed?")
- **`/claim`** (T-2097) — WRITE ACQUIRE ("reserve this unit")
- **`/release`** (T-2098) — WRITE RELEASE ("free this claim")
- **`/claim-transfer`** (this skill) — WRITE COOPERATIVE HANDOFF ("hand it off atomically")

The natural orchestrator chain in one slash-command sequence:

```
/find-idle --capability rust          # who's free?
/claim work-queue T-1234              # orchestrator reserves
/claim-transfer <claim-id> <worker>   # atomic handoff
... worker does work ...
/release <claim-id>                   # worker frees when done
```

**Invocation:**

| Form | Action |
|------|--------|
| `/claim-transfer <claim-id> <to-owner>` | Hand `<claim-id>` to `<to-owner>` |
| `/claim-transfer <claim-id> <to-owner> --by <id>` | Override identity (default: resolved) |
| `/claim-transfer <claim-id> <to-owner> --reason "..."` | Annotate transfer (visible in audit) |
| `/claim-transfer <claim-id> <to-owner> --json` | Machine-readable envelope |

**Auto-resolution defaults** (mirror of T-1857 chain, same as
`/claim` and `/release`):

- `--by` resolves via:
  1. `--by` flag (explicit)
  2. `$TERMLINK_AGENT_ID` env
  3. `~/.termlink/be-reachable.state` (set by `/be-reachable`)
  4. **Refuse** with hint if all three fail. Never invent an owner.

The `--by` field must equal the claim's current holder (the hub
enforces this via CLAIM_NOT_OWNED -32017). Auto-resolution is right
because the operator usually IS the current holder — they're handing
off their own claim.

## Step 1: Pre-flight

Run:

```
termlink channel claim-transfer --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
claim-transfer: `termlink` CLI not on PATH or substrate primitive #3
(channel.claim-transfer) not available in this build. Ensure you're
on a version with T-2046 shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Normalize:

- First positional → `<claim-id>`
- Second positional → `<to-owner>`
- Flag tokens (`--by`, `--reason`, `--json`) → pass through

If `<claim-id>` is missing: **stop**. Print:

```
claim-transfer: claim-id required.

Usage:
  /claim-transfer <claim-id> <to-owner>             # cooperative handoff
  /claim-transfer <claim-id> <to-owner> --by <id>   # explicit identity
  /claim-transfer <claim-id> <to-owner> --reason ".."  # annotate

If you don't know your claim-id:
  /claims <topic>                                    # see active claims
  /claims --all                                      # fleet-wide view
```

If `<to-owner>` is missing: **stop**. Print:

```
claim-transfer: to-owner required.

Usage:
  /claim-transfer <claim-id> <to-owner>

If you don't know who's free to receive:
  /find-idle [--capability X]                        # idle agents (T-2092)
  /peers --all                                       # all LIVE listeners
```

Owner-resolution check (run before transfer):

```
if [ -z "$EXPLICIT_BY" ] && [ -z "$TERMLINK_AGENT_ID" ] && [ ! -f ~/.termlink/be-reachable.state ]; then
  print refusal hint and stop
fi
```

Refusal hint:

```
claim-transfer: identity unresolved.

The `--by` field identifies who is handing off and MUST equal the
claim's current holder (the hub refuses CLAIM_NOT_OWNED otherwise).
Resolve via one of:

  /be-reachable                              # auto-establish session identity
  /claim-transfer <c> <to> --by <id>         # explicit override
  export TERMLINK_AGENT_ID=<id>              # env override
```

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr +
exit code.

## Step 4: Surface refusal modes loudly

Substrate #3 has a known refusal taxonomy. Recognize each and surface
an actionable next-step hint rather than passing through the raw
error:

**CLAIM_NOT_FOUND** (-32018 or similar — claim id never existed or
expired):

```
claim-transfer refused: <claim-id> not found.

Likely cause: claim expired naturally before handoff, or never existed.
Inspect: /claims <topic>                              # current state
Audit:   termlink channel claims-history --since 1    # recent events (T-2074)

If lease expired, the unit is unclaimed — the receiver can /claim it directly.
```

**CLAIM_NOT_OWNED** (-32017 — claim exists but `--by` is not the
holder):

```
claim-transfer refused: <claim-id> is not held by <by-identity>.

A cooperative handoff requires the CURRENT holder to volunteer.
Inspect: /claims <topic>                              # see who actually holds it

If you must take ownership without the holder's cooperation:
  termlink channel claim-force-release <claim-id>     # Tier-0, last-resort,
                                                        operator bypass.
                                                        Logs the override.
```

**This distinction matters.** Cooperative claim-transfer preserves
claim accountability — the handoff is in the audit log as the original
holder's choice. claim-force-release is the operator's override and
shows up as such. Don't conflate the two.

**AUTH_FAIL** (-32001 or similar):

```
claim-transfer refused: authentication failure.

Likely cause: hub rotated its secret (substrate-related: T-1052/T-1053).
Diagnose: termlink fleet doctor
Heal: termlink fleet reauth <profile> --bootstrap-from auto
```

**RATE_LIMITED** (-32008):

```
claim-transfer refused: rate-limited by hub governor.

Inspect: /governor --only-pressured
Wait `retry_after_ms` (see error payload) and retry.
```

**Other / unknown errors:** surface stderr verbatim with no editorialization.

## Step 5: Render success

For default human-format output, render:

```
claim transferred:
  claim_id:   <claim-id>
  topic:      <topic from response>
  unit_id:    <unit-id from response>
  from:       <by>                    (from /be-reachable)  [if auto-resolved]
  to:         <to-owner>
  transferred_at: <RFC3339 from response>
  reason:     <reason>                [if provided]

The lease atomically moved to <to-owner>. The receiver can now /release
when done (or further transfer if delegating).
```

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render.

## Step 6: Identity-resolved success-side hint

When success uses auto-resolved identity (not explicit), prepend a
one-line confirmation:

```
(using by=<id> resolved from ~/.termlink/be-reachable.state)
```

This is observability — the operator confirms they handed off as
themselves, not some stale env var.

## Rules

- **Writes state.** Atomic at the hub layer (T-2046 guarantee) — the
  lease moves from `--by` to `--to-owner` with zero gap. But it IS a
  state mutation. Don't auto-retry on transient errors.
- **Cooperative only.** This skill never bypasses ownership — it
  ALWAYS asserts `--by` is the current holder. For
  operator-bypass-without-holder-cooperation use Tier-0
  `claim-force-release`. The two verbs are intentionally distinct.
- **Never invent an identity.** If `--by` resolution fails, refuse
  with a hint. The hub will refuse silently-defaulted identities via
  CLAIM_NOT_OWNED anyway — better to refuse loudly client-side.
- **Loud refusals.** Each known error class gets an actionable
  next-step. Don't pass through raw `Err(-32017): ...` lines.
- **No `AskUserQuestion`** — just run and report.
- **Positional, not flag-only.** The CLI takes `--claim-id` and
  `--to-owner` as flags; this skill takes them positionally. The skill
  layer is the friction-reduction layer — that's its whole purpose.

## Common patterns

**Orchestrator-to-worker handoff (the canonical pattern):**

```
/find-idle --capability rust       # who's free? Returns claude-worker-3
/claim work-queue T-1234           # orchestrator reserves; returns claim-id abc123
/claim-transfer abc123 claude-worker-3   # atomic handoff — worker now owns it
```

The unit goes from "orchestrator holding" to "worker holding" with no
gap. A separate orchestrator competing for the same unit cannot
race-steal during the transition.

**Delegate to a peer (no orchestrator):**

```
/claim deploy-queue T-5678         # I take it
... realize I'm wrong agent for the job ...
/claim-transfer <claim-id> claude-deploy-bot --reason "wrong toolchain"
```

The receiver gets ownership; the audit log shows the reason.

**Multi-hop delegation chain:**

```
/claim work-queue T-1234           # orchestrator picks it up
/claim-transfer <id> coordinator   # to a specialized coordinator
... coordinator narrows the scope ...
/claim-transfer <id> claude-worker # to actual worker
... work happens ...
/release <id>                      # worker frees when done
```

Each hop is atomic; each hop is audited.

**When cooperative handoff fails — the operator override:**

```
/claim-transfer <id> claude-worker
# Output: claim-transfer refused: <id> is not held by <me>.
#         (Holder is currently claude-stale, last heartbeat 4h ago.)

# Stale holder is unresponsive — Tier-0 override:
termlink channel claim-force-release <claim-id>   # logs the override
/claim <topic> <unit-id>                          # re-acquire
/claim-transfer <new-id> claude-worker            # now hand off cleanly
```

The Tier-0 force-release is the right tool when cooperative is impossible.

## Not in scope

- **Force transfer.** No `--force` flag. If the current holder can't
  cooperate, drop to Tier-0 `claim-force-release` then re-claim. The
  two-step is intentional — it forces operator acknowledgement that
  ownership was overridden.
- **Bulk transfer.** No `/claim-transfer --topic <T> --to <X>`. Each
  hop should be a deliberate decision.
- **Transfer to self.** Pointless — and the hub may refuse it. If you
  want to refresh your own lease, the substrate may offer a renew RPC;
  this skill doesn't.

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #3
  at the daily-verb tier.
- T-2046 — substrate primitive #3 implementation (the underlying
  `channel.claim-transfer` RPC + atomicity guarantee).
- T-2019 / T-2042 — substrate primitive #1 (the claim lifecycle this
  hands off WITHIN).
- T-2093 / `/claims` — sibling READ-side skill (find your claim-id).
- T-2097 / `/claim` — sibling ACQUIRE-side skill.
- T-2098 / `/release` — sibling RELEASE-side skill.
- T-2092 / `/find-idle` — the natural pre-transfer verb (find worker,
  then transfer).
- T-1857 / `/broadcast-chat` — sender-resolution pattern reused here.
- T-1841 / `/be-reachable` — auto-establishes the identity this skill
  resolves `--by` from.
- T-2074 / `channel claims-history` — retrospective audit (when you
  want to see the chain of transfers).
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
