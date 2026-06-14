# /canaries — unified cron-canary visibility (T-2172)

The **"are my canaries firing AND clean?"** verb. Wraps
`scripts/canary-status.sh` — scans `.context/working/.*-canary.log` +
companion `.heartbeat` files and reports per-canary status (HEALTHY /
FIRING / STALE / NO_HEARTBEAT) with a summary footer.

Read-only, no auth side-effects, no state mutation. Safe anywhere.

This is the **cron-tier visibility companion** to the runtime-tier
substrate-arc observability surface:

| Tier | Verb | Question |
|---|---|---|
| Deploy-time correctness | `/preflight` (T-2158) | Is the substrate set up right? |
| Runtime state | `/substrate` (T-2096) | Is the substrate healthy right now? |
| Cron-tier protection | `/canaries` (T-2172) | Are my watchers firing AND clean? |

Closes the dormant-tooling antipattern surfaced by **PL-168** — canary
scripts without an operator-facing trigger are dormant. This verb is the
trigger that makes the canary surface visible.

## What it scans

Auto-discovers every `.context/working/.*-canary.log` (plus the meta-
canary `.canary-aliveness.log` which uses a different naming) and pairs
each with its `.heartbeat` companion if present.

Current canaries on this host (typical):
- `substrate-preflight-canary` — daily PL-021 / hubs.toml / be-reachable check (T-2160)
- `release-mirror-canary` — daily OneDev → GitHub mirror drift check (T-1696)
- `canary-aliveness` — meta-canary watching the mirror-canary (T-1723)
- `fleet-doorbell-mail-canary` — daily doorbell+mail health probe
- `fleet-adoption-snapshot` — daily snapshot (always-appending, not following empty=healthy convention)

**No hard-coded canary list** — new canaries appear automatically the
first time their log file is written.

## Status taxonomy

| Status | Meaning | Action |
|---|---|---|
| `HEALTHY` | Log empty (or all entries older than latest heartbeat) AND heartbeat fresh | None — cron firing, no problems |
| `FIRING` | Log has entries newer than latest heartbeat | Read the log; fix the underlying drift |
| `STALE` | Heartbeat older than threshold (default 48h) | Check cron is loaded; verify the script runs |
| `NO_HEARTBEAT` | Log present but no `.heartbeat` companion | Classified by log content only |

## Invocation

| Form | Action |
|------|--------|
| `/canaries` | Full human-readable digest |
| `/canaries --json` | Machine envelope (jq-friendly) |
| `/canaries --quiet` | Render only FIRING/STALE rows (cron-friendly) |
| `/canaries --max-age-hours 72` | Custom stale threshold |

## Step 1: Pre-flight

Run:

```
bash scripts/canary-status.sh --help >/dev/null
```

If exit non-zero: **stop**. Print:

```
canaries: wrapper not found at scripts/canary-status.sh.
Ensure you're in the TermLink project root (cd /opt/termlink).
```

## Step 2: Run

Execute via Bash, pass through `$ARGUMENTS` verbatim:

```
bash scripts/canary-status.sh $ARGUMENTS
```

The script handles its own help / flag parsing — no normalization needed
at the skill layer.

## Step 3: Surface the result

The script's stdout is operator-readable. Pass it through verbatim.

In default mode, the output looks like:

```
canary-status: 4 canary(ies) — 3 healthy, 1 firing, 0 stale (threshold 48h)

  STATUS       NAME                             LAST FIRED / LATEST ENTRY
  ------       ----                             -------------------------
  HEALTHY      canary-aliveness                 hb=2026-06-11 08:33 log=2026-06-06 11:19
  FIRING       fleet-doorbell-mail-canary       hb=2026-06-11 09:23 log=2026-06-11 09:23
               ↳   - laptop-141@192.168.10.141:9100: verdict=setup-fail
  HEALTHY      release-mirror-canary            hb=2026-06-11 07:13 log=2026-06-08 07:13
  HEALTHY      substrate-preflight-canary       hb=2026-06-11 05:23 log=--

Action needed:
  FIRING — a canary is detecting a real problem. Read the log:
    cat .context/working/.<name>-canary.log
  Then fix the underlying drift (rotation, mirror sync, etc.) per the relevant runbook.
```

The trailing "Action needed" block is printed only when at least one
canary is FIRING or STALE.

## Step 4: Surface follow-up hints

If the operator's session was substrate-arc-focused, append after the
verbatim output:

```
Related substrate-arc verbs:
  /preflight             — deploy-time correctness right now
  /substrate             — runtime state right now
  cat .context/working/.<name>-canary.log   — read the firing canary's log
```

If a STALE canary appeared, append the cron-check ladder:

```
STALE canary recovery:
  sudo cat /etc/cron.d/<canary-name>           # confirm the cron is loaded
  sudo systemctl status cron                    # confirm cron daemon is up
  bash scripts/<canary-script>.sh --quiet       # run the canary manually
```

## Note: a FIRING canary on an expected-transient host

If the FIRING canary is `fleet-doorbell-mail-canary` and the offending
host is a laptop or dev box that is *expected* to be off the network
much of the time (the canonical case is `laptop-141` showing
`verdict=setup-fail`), the remedy is **not** to chase the host — it is to
declare it expected-transient so a sleeping laptop no longer DRIFTs the
whole-fleet canary (G-019 alert-fatigue; PL-219):

```sh
# add the [hubs.NAME] profile name, one per line:
echo 'laptop-141' >> .context/cron/fleet-dm-canary-transient
```

A declared host that is unreachable is classified `transient_skipped`
(shown but non-verdict-flipping); a declared host that is *reachable*
still counts pass/fail (the skip suppresses down-ness, not brokenness).
Full recipe: `docs/operations/substrate-cron-recipes.md` §
"Expected-transient hosts". For any *other* FIRING canary, read its log
and fix the underlying drift per its runbook.


## Exit codes

- 0 — all canaries healthy
- 1 — at least one canary is FIRING or STALE (operator action required)
- 2 — tooling error (working dir missing, malformed flag)

The skill surfaces the script's exit code via the Bash tool — agents
calling /canaries programmatically can gate on `$?`.

## Rules

- **Read-only by contract.** This skill never writes state. It scans
  files, classifies, and renders. Safe to invoke at any time.
- **Do not auto-act on FIRING canaries.** A FIRING result requires
  operator judgement (read the log, decide whether it's a known issue,
  apply the runbook). The skill surfaces — it does not heal.
- **Do not use `AskUserQuestion`.** Just run and report.
- **Don't compose with `/substrate`.** They answer different questions
  (cron-tier vs runtime). Operators run them separately when they want
  one or the other view. A unified mega-digest would dilute the
  signal — keep verbs orthogonal.
- **Default cadence assumption:** daily. The 48h stale threshold is
  appropriate for daily-fire canaries. If a canary fires hourly, pass
  `--max-age-hours 2` to tighten.

## Related

- T-2172 (this skill + backing script)
- PL-168 (the dormant-tooling antipattern this closes)
- T-2096 (`/substrate` — runtime sibling)
- T-2158 (`/preflight` — deploy-time sibling)
- T-2160 (substrate-preflight cron — the substrate-arc canary)
- T-1696 (release-mirror canary)
- T-1723 (meta-canary — watches the mirror canary)
- `scripts/canary-status.sh` — the underlying scanner
