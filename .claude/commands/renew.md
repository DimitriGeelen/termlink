# /renew — extend a claim's lease (T-2101 skill-layer wrap)

WRITE-side daily verb for substrate primitive #1 lifecycle (T-2030/T-2032).
Wraps `termlink channel renew` so an operator can extend an active claim's
lease in one keystroke instead of recalling two long flags plus an awkward
`--additional-ttl-ms`.

The substrate's pressure-relief verb: claims default to a 30-second lease
because most work is short. For longer work, you renew before the lease
lapses — otherwise the slot reopens to another worker mid-task (a
substrate-correctness footgun for any non-trivial unit).

The full substrate #1 daily-verb surface — NOW COMPLETE end-to-end:

- **`/claims`** (T-2093) — READ ("what's claimed?")
- **`/claim`** (T-2097) — ACQUIRE ("reserve this offset")
- **`/renew`** (this skill) — EXTEND ("I need more time")
- **`/release`** (T-2098) — COMPLETE/RETRY ("free this claim")
- **`/claim-transfer`** (T-2099) — COOPERATIVE HANDOFF

Natural long-work pattern:

```
/claim work-queue 42                     # acquire (30s default lease)
... do work, find it'll take 5min ...
/renew <claim-id>                        # extend by +30s
... more work ...
/renew <claim-id> --by-ms 300000         # extend by +5 min
... finish ...
/release <claim-id>                      # mark completed
```

**Invocation:**

| Form | Action |
|------|--------|
| `/renew <claim-id>` | Extend lease by +30000ms (default — matches CLI default) |
| `/renew <claim-id> --by-ms 60000` | Extend by +60 seconds |
| `/renew <claim-id> --claimer <id>` | Override claimer (default: resolved from `/be-reachable` state) |
| `/renew <claim-id> --json` | Machine-readable envelope |

**`--by-ms` operator alias.** The underlying CLI flag is
`--additional-ttl-ms`, which is awkward to type. This skill exposes
`--by-ms` as the operator-facing alias and translates to
`--additional-ttl-ms` downstream. The skill never exposes the awkward
form — operators never see it.

**Auto-resolution defaults** (mirror of T-1857 sender-resolution chain,
same as `/claim`, `/release`):

- `--claimer` resolves via:
  1. `--claimer` flag (explicit)
  2. `$TERMLINK_AGENT_ID` env
  3. `~/.termlink/be-reachable.state` (set by `/be-reachable`)
  4. **Refuse** with hint if all three fail. Never invent a claimer.

The `--claimer` MUST equal the claim's current holder (the hub
enforces this via CLAIM_NOT_OWNED). Auto-resolution is right because
the operator usually IS the holder — they're renewing their own claim.

## Step 1: Pre-flight

Run:

```
termlink channel renew --help >/dev/null 2>&1
```

The skill validates against the actual CLI surface (per PL-206 — always
author from `--help`, not from CLAUDE.md row shorthand):

```
Usage: termlink channel renew [OPTIONS] --claim-id <CLAIM_ID> --claimer <CLAIMER>
Options:
      --claim-id <CLAIM_ID>                  Opaque claim_id from `channel claim`
      --claimer <CLAIMER>                    Same claimer value — gates ownership
      --additional-ttl-ms <ADDITIONAL_TTL_MS> Additional milliseconds (default 30000; clamped 1h)
      --hub <HUB>                            Target hub address
      --json                                 Output as JSON
```

If --help exit non-zero: **stop**. Print:

```
renew: `termlink` CLI not on PATH or substrate primitive renew not
available in this build. Ensure you're on a version with T-2030 shipped
(run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Normalize:

- First positional → `<claim-id>`
- `--by-ms N` (skill alias) → translates to `--additional-ttl-ms N` downstream
- `--additional-ttl-ms N` (CLI native) → pass through unchanged
- Other flag tokens (`--claimer`, `--hub`, `--json`) → pass through

If `<claim-id>` is missing: **stop**. Print:

```
renew: claim-id required.

Usage:
  /renew <claim-id>                          # extend by +30s (default)
  /renew <claim-id> --by-ms 60000            # extend by +60s
  /renew <claim-id> --claimer <id>           # explicit identity override
  /renew <claim-id> --json                   # machine-readable

If you don't know your claim-id:
  /claims <topic>                            # see active claims (find yours)
```

Claimer-resolution check (run before renew):

```
if [ -z "$EXPLICIT_CLAIMER" ] && [ -z "$TERMLINK_AGENT_ID" ] && [ ! -f ~/.termlink/be-reachable.state ]; then
  print refusal hint and stop
fi
```

Refusal hint:

```
renew: claimer identity unresolved.

The `--claimer` field identifies who is renewing — MUST equal the
claim's current holder (the hub refuses CLAIM_NOT_OWNED otherwise).
Resolve via one of:

  /be-reachable                              # auto-establish session identity
  /renew <claim-id> --claimer <id>           # explicit override
  export TERMLINK_AGENT_ID=<id>              # env override
```

## Step 3: Run the verb

Construct the command:

```
termlink channel renew --claim-id <claim-id> --claimer <resolved-claimer> [--additional-ttl-ms <ms>] [--json]
```

Translate `--by-ms` (skill-side) → `--additional-ttl-ms` (CLI-side) before
invocation. The operator never sees the awkward CLI flag name.

Execute via Bash. Capture stdout + stderr + exit code.

## Step 4: Surface refusal modes loudly

Substrate renew has a known refusal taxonomy. Recognize each and
surface an actionable next-step hint:

**CLAIM_NOT_FOUND** (claim id never existed):

```
renew refused: <claim-id> not found.

Likely cause: claim never existed (typo in the id), or was already released.
Inspect: /claims <topic>                              # current state
Audit:   termlink channel claims-history --since 1    # recent events (T-2074)
```

**CLAIM_NOT_OWNED** (claim exists but `--claimer` is not the holder):

```
renew refused: <claim-id> is not held by <claimer-identity>.

Inspect: /claims <topic>                              # see who actually holds it

If you were the original claimer but lost identity (session restart):
  Check ~/.termlink/be-reachable.state for current identity.
  Pass --claimer <original-id> to renew under the right identity.
```

**CLAIM_LAPSED** (lease already expired — the case renew exists to PREVENT):

```
renew refused: <claim-id> already lapsed.

The lease expired before this renew arrived — the slot has reopened
and another worker may have re-claimed the offset.

This is the failure mode renew exists to prevent. Next time, renew
SOONER (well before claimed_until in the original claim envelope).

To continue working on this offset, re-claim it:
  /claims <topic>                            # confirm offset is free
  /claim <topic> <offset>                    # re-acquire (gets new claim_id)
```

**AUTH_FAIL**:

```
renew refused: authentication failure.

Likely cause: hub rotated its secret (T-1052/T-1053).
Diagnose: termlink fleet doctor
Heal: termlink fleet reauth <profile> --bootstrap-from auto
```

**RATE_LIMITED** (-32008):

```
renew refused: rate-limited by hub governor.

Inspect: /governor --only-pressured
Wait `retry_after_ms` (see error payload) and retry — but BEWARE: while
you wait, the lease keeps ticking down. Consider re-claiming if it
lapses before rate-limit clears.
```

**Other / unknown errors:** surface stderr verbatim with no editorialization.

## Step 5: Render success

For default human-format output, render:

```
lease renewed:
  claim_id:        <claim-id>
  topic:           <topic from response>
  offset:          <offset from response>
  claimer:         <claimer>                 (from /be-reachable)  [if auto-resolved]
  added_ms:        <ms>
  new_claimed_until: <RFC3339 from response>

The lease now extends to <new_claimed_until>. Plan your next renew BEFORE that.

Next steps:
- Continue work.
- Renew again if needed: /renew <claim-id> --by-ms <ms>
- Release when done:     /release <claim-id>
```

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render.

## Step 6: Claimer-resolved success-side hint

When success uses auto-resolved claimer (not explicit), prepend a
one-line confirmation:

```
(using claimer=<id> resolved from ~/.termlink/be-reachable.state)
```

This is observability — the operator confirms they renewed as
themselves, not some stale env var.

## Rules

- **Writes state.** The lease extension is persisted at the hub. Don't
  auto-retry on transient errors — surface the refusal and let the
  operator decide.
- **`--by-ms` alias is operator UX, not CLI surface.** The skill
  translates `--by-ms` to `--additional-ttl-ms` before invocation. The
  operator never sees the awkward CLI name.
- **Never invent a claimer.** If claimer-resolution fails, refuse with
  a hint. The hub will refuse silently-defaulted identities via
  CLAIM_NOT_OWNED anyway — better to refuse loudly client-side.
- **Loud refusals.** Each known error class gets an actionable
  next-step. The CLAIM_LAPSED case especially — operators need to know
  the slot may have been re-claimed already.
- **No `AskUserQuestion`** — just run and report.
- **No auto-renew loop.** This skill renews ONCE per invocation. For
  long-running work that needs continuous renewal, the operator wires
  a watch loop themselves (or eventually the substrate ships a
  daemon-mode renewer — out of scope here).

## Common patterns

**Single renewal for known-extended work:**

```
/claim work-queue 42                # 30s default lease
... realize this will take 2min ...
/renew <claim-id> --by-ms 120000    # +2 minutes
... finish ...
/release <claim-id>
```

**Defensive renewal mid-work (when you've already burned ~80% of lease):**

```
... 25 seconds into a 30s lease, work isn't done yet ...
/renew <claim-id>                   # +30s default — buys breathing room
```

The default (+30s) matches the CLI default and is the right "buy me
breathing room" amount for most work. Pass `--by-ms` only when you
need a specific extension.

**Long-haul renewal loop (driven by a shell wrapper, not this skill):**

```
# In a background shell:
while ! work_done; do
  /renew <claim-id> --by-ms 30000
  sleep 25
done
```

This skill is the verb; the loop is the operator's responsibility.

## Not in scope

- **Auto-renew daemon.** No background mode. One renew per invocation.
- **Renew to a specific timestamp.** The CLI takes `--additional-ttl-ms`
  (relative); not `--claimed-until` (absolute). The skill mirrors that.
- **Bulk renew.** No `/renew --topic <T>` or `/renew --all`. Each
  claim is renewed individually.

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes the
  renew verb at the daily-verb tier.
- T-2030 — substrate `channel.renew` JSON-RPC verb (the underlying
  implementation).
- T-2032 — substrate primitive #1 CLI surface (claim/renew/release
  wrappers; the parent task that shipped these verbs).
- T-2093 / `/claims` — sibling READ-side skill.
- T-2097 / `/claim` — natural predecessor (renew extends the claim
  acquired here).
- T-2098 / `/release` — natural successor (release after the renewed
  lease's work completes).
- T-2099 / `/claim-transfer` — alternative successor (hand off
  before lease expires).
- T-1857 / `/broadcast-chat` — sender-resolution pattern reused here.
- T-1841 / `/be-reachable` — auto-establishes the identity this skill
  resolves `--claimer` from.
- T-2074 / `channel claims-history` — retrospective audit.
- T-2100 / PL-206 — fix-up + learning that established "always author
  from `--help`" rule. This skill applied that rule from inception —
  CLI flags grepped from `termlink channel renew --help` output.
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
