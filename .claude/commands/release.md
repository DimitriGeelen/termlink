# /release — release (or retry) a claim (T-2098 / T-2100-fixed skill-layer wrap)

WRITE-side daily verb for substrate primitive #1 (T-2019/T-2042/T-2032).
Wraps `termlink channel release` so an operator can free a claim in one
keystroke instead of recalling the CLI's two required `--flag VALUE`
pairs. The release side of the work-stealing primitive — the natural
pair to `/claim` (T-2097).

**This skill WRITES state.** Every successful invocation either
advances the claimer's persisted cursor past the offset (work
completed — the default) OR reopens the slot for the next worker
without cursor advance (work returned for retry — opt-in via
`--retry`). Once released, the slot is available for the next worker
on their next `/find-idle` → `/claim` cycle.

The full substrate #1 daily-verb surface:

- **`/claims`** (T-2093) — READ ("what's claimed?")
- **`/claim`** (T-2097) — WRITE ACQUIRE ("reserve this offset")
- **`/release`** (this skill) — WRITE COMPLETE/RETRY ("free this claim")
- **`/claim-transfer`** (T-2099) — WRITE COOPERATIVE HANDOFF

**Invocation:**

| Form | Action |
|------|--------|
| `/release <claim-id>` | Release with `--ack` (work completed; cursor advances) |
| `/release <claim-id> --retry` | Release WITHOUT `--ack` (work returned for retry; cursor stays) |
| `/release <claim-id> --claimer <id>` | Override identity (default: resolved from `/be-reachable` state) |
| `/release <claim-id> --json` | Machine-readable envelope |

**The default-ack design.** When an operator says `/release <claim-id>`
they almost always mean "I'm done with this work". So `--ack` is the
default — the skill ALWAYS adds it unless `--retry` is passed. To
release-without-ack (the rarer "couldn't finish, please retry" case),
operators explicitly type `--retry`. This makes the common case
ergonomic while keeping the rarer case discoverable.

**Auto-resolution defaults** (mirror of T-1857 sender-resolution chain,
same as `/claim`):

- `--claimer` resolves via:
  1. `--claimer` flag (explicit)
  2. `$TERMLINK_AGENT_ID` env
  3. `~/.termlink/be-reachable.state` (set by `/be-reachable`)
  4. **Refuse** with hint if all three fail. Never invent an identity.

The `--claimer` field MUST equal the claim's current holder (the hub
enforces this via CLAIM_NOT_OWNED). Auto-resolution is right because
the operator usually IS the holder — they're releasing their own
claim.

## Step 1: Pre-flight

Run:

```
termlink channel release --help >/dev/null 2>&1
```

The skill validates against the actual CLI surface:

```
Usage: termlink channel release [OPTIONS] --claim-id <CLAIM_ID> --claimer <CLAIMER>
Options:
      --claim-id <CLAIM_ID>  Opaque claim_id returned by `channel claim`
      --claimer <CLAIMER>    Same claimer value used in `channel claim` — gates ownership
      --ack                  Acknowledge work as completed — advances cursor past the offset
      --hub <HUB>            Target hub address
      --json                 Output as JSON
```

If --help exit non-zero: **stop**. Print:

```
release: `termlink` CLI not on PATH or substrate primitive release
not available in this build. Ensure you're on a version with T-2029
shipped (run `termlink --version` to verify).
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Normalize:

- First positional → `<claim-id>`
- `--retry` flag (recognized HERE in the skill) → translates to "do NOT add --ack" downstream
- All other flag tokens (`--claimer`, `--hub`, `--json`) → pass through

Default behavior when neither `--retry` nor an explicit `--ack`-suppress flag is present:
**add `--ack`**. The skill assumes "done" unless operator says otherwise.

If `<claim-id>` is missing: **stop**. Print:

```
release: claim-id required.

Usage:
  /release <claim-id>                    # release with --ack (work completed)
  /release <claim-id> --retry            # release WITHOUT --ack (work for next worker)
  /release <claim-id> --claimer <id>     # explicit identity override
  /release <claim-id> --json             # machine-readable

If you don't know your claim-id:
  /claims <topic>                        # see active claims (find yours)
  /claims --all                          # fleet-wide view
```

Claimer-resolution check (run before release):

```
if [ -z "$EXPLICIT_CLAIMER" ] && [ -z "$TERMLINK_AGENT_ID" ] && [ ! -f ~/.termlink/be-reachable.state ]; then
  print refusal hint and stop
fi
```

Refusal hint:

```
release: claimer identity unresolved.

The `--claimer` field identifies who is releasing and MUST equal the
claim's current holder (the hub refuses CLAIM_NOT_OWNED otherwise).
Resolve via:

  /be-reachable                                # auto-establish session identity
  /release <claim-id> --claimer <id>           # explicit override
  export TERMLINK_AGENT_ID=<id>                # env override
```

## Step 3: Run the verb

Construct the command:

```
termlink channel release --claim-id <claim-id> --claimer <resolved-claimer> [--ack] [--json]
```

`--ack` is included unless the operator passed `--retry`. The internal `--retry`
token is NOT passed to the CLI — it's a skill-level flag that controls whether
`--ack` is added.

Execute via Bash. Capture stdout + stderr + exit code.

## Step 4: Surface refusal modes loudly

Substrate release has a known refusal taxonomy. Recognize each and
surface an actionable next-step hint:

**CLAIM_NOT_FOUND** (claim id never existed or expired):

```
release refused: <claim-id> not found.

Likely cause: already released, claim expired naturally, or never existed.
Inspect: /claims <topic>                              # current state
Audit:   termlink channel claims-history --since 1    # recent events (T-2074)
```

**CLAIM_NOT_OWNED** (claim exists but `--claimer` is not the holder):

```
release refused: <claim-id> is not held by <claimer-identity>.

Inspect: /claims <topic>                              # see who actually holds it
Cooperative handoff: /claim-transfer <claim-id> <new-owner>  # T-2099 (if holder
                                                              cooperates)
Operator-bypass: termlink channel claim-force-release # Tier-0, last-resort
                                                        (overrides ownership;
                                                        logs the override)
```

**AUTH_FAIL**:

```
release refused: authentication failure.

Likely cause: hub rotated its secret (T-1052/T-1053).
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

For default human-format output (with --ack, the "completed" path):

```
claim released (completed):
  claim_id:    <claim-id>
  topic:       <topic from response>
  offset:      <offset from response>
  claimer:     <claimer>                  (from /be-reachable)  [if auto-resolved]
  ack:         true                       # cursor advanced past offset
  released_at: <RFC3339 from response>

The work is marked completed. The cursor now points past offset <N> —
this work won't be re-handed out to another worker.
```

For `--retry` (the "returned for retry" path):

```
claim released (returned for retry):
  claim_id:    <claim-id>
  topic:       <topic from response>
  offset:      <offset from response>
  claimer:     <claimer>                  (from /be-reachable)  [if auto-resolved]
  ack:         false                      # cursor NOT advanced — slot reopened
  released_at: <RFC3339 from response>

Slot reopened. The next worker that claims this offset gets the same work.
Use this when YOU couldn't finish but believe ANOTHER worker can.
```

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render.

## Step 6: Identity-resolved success-side hint

When success uses auto-resolved claimer (not explicit), prepend a
one-line confirmation:

```
(using claimer=<id> resolved from ~/.termlink/be-reachable.state)
```

This is observability — the operator confirms they released as
themselves, not some stale env var.

## Rules

- **Writes state.** Either advances a cursor (`--ack`) or reopens a
  slot (no `--ack`). Don't auto-retry on transient errors — surface
  the refusal and let the operator decide.
- **`--ack` default with `--retry` opt-out.** The common case ("I'm
  done") is one keystroke. The rarer case ("couldn't finish") requires
  the operator to type `--retry`. This intentional asymmetry encodes
  the "completion is the expected outcome" heuristic — silent retries
  are a substrate-correctness footgun.
- **Never invent a claimer.** If `--claimer` resolution fails, refuse
  with a hint. The hub will refuse silently-defaulted identities via
  CLAIM_NOT_OWNED anyway — better to refuse loudly client-side.
- **Loud refusals.** Each known error class gets an actionable
  next-step. Don't pass through raw `Err(-32017): ...` lines.
- **No `AskUserQuestion`** — just run and report.
- **No bulk-release.** This skill releases ONE claim by id. For
  fleet-wide cleanup, the operator drops to CLI. Bulk operations are
  not daily-verb-shaped.

## Common patterns

**Standard release after work completion (the >90% case):**

```
/claim work-queue 42             # acquire offset 42 (T-2097)
... do the work ...
/release <claim-id>              # release with --ack (cursor advances)
```

The `<claim-id>` is in `/claim`'s success output.

**Release for retry (work couldn't be completed):**

```
/claim work-queue 42             # acquire
... ran into an irrecoverable error ...
/release <claim-id> --retry      # release WITHOUT --ack
                                  # next worker gets offset 42 again
```

This is the "I couldn't, but maybe you can" pattern. Used when:
- The work surfaced a hard dependency the current worker lacks
- The worker is shutting down mid-task and wants to surrender cleanly
- A retry-on-different-worker would have different odds of success

**Release after a cooperative handoff (new owner releases when done):**

```
# After T-2099 cooperative transfer:
/claim-transfer <id> claude-worker
... worker does the work using its own /release call ...
# claude-worker calls: /release <id>   (with auto-resolved claimer=claude-worker)
```

**Release when you've forgotten the claim-id:**

```
/claims work-queue                # find your offset/claim_id in the active list
/release <claim-id>               # paste the id from above
```

**Release with explicit claimer (multi-session scenarios):**

```
/release <claim-id> --claimer claude-orchestrator
```

Useful when the calling session is acting on behalf of an
orchestrator identity that holds the claim.

## Not in scope

- **Bulk release.** No `/release --all` or `/release --topic <T>`.
  Daily-verb means one claim per invocation. Operators wanting fleet
  cleanup use the CLI directly.
- **Force release.** Releasing claims you don't hold is Tier-0
  (`termlink channel claim-force-release`). This skill never bypasses
  ownership — it would silently undermine claim accountability.
- **Auto-release on session end.** Out of scope; explicit release is
  intentional. A session that crashes leaves the lease to expire
  naturally (the substrate handles this — see T-2042).

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes the
  release verb at the daily-verb tier.
- T-2029 — substrate `channel.release` JSON-RPC verb (the underlying
  implementation; gates ownership via `--claimer`, exposes `--ack`).
- T-2032 — substrate primitive #1 CLI surface (claim/renew/release
  wrappers; the parent task that shipped these verbs).
- T-2046 / `/claim-transfer` — cooperative handoff alternative to
  release-then-re-claim.
- T-2093 / `/claims` — sibling READ-side skill (find your claim-id).
- T-2097 / `/claim` — sibling ACQUIRE-side skill (the natural pair).
- T-2099 / `/claim-transfer` — cooperative HANDOFF (alternative to
  release for the "I have to hand this to someone specific" case).
- T-1857 / `/broadcast-chat` — sender-resolution pattern reused here.
- T-1841 / `/be-reachable` — auto-establishes the identity this skill
  resolves `--claimer` from.
- T-2074 / `channel claims-history` — retrospective audit (when you
  want to know "did I already release this?").
- T-2100 — fix-up task that corrected this skill's flag names (origin:
  shipped initially with wrong flags from CLAUDE.md row shorthand).
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
