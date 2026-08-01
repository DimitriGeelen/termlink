# The "shipped == live" gate (T-2480, P3a, G-069 prevention)

## The problem

Capabilities were repeatedly recorded **closed=shipped** while dark in the field
for weeks. arc-004's push-transport was marked shipped while "0 wakers
fleet-wide" sat dark; .107 ran a binary 26 hub-side commits stale; .122 served a
pre-arc feature set for ~13 days. "Shipped" meant *code merged*, not *capability
live on the hubs that need it* — the **G-069 shipped≠live class**.

Detection exists on a daily cron (T-2359 binary-floor, T-2387 waker-liveness,
T-2415 capability canaries), but detection is retrospective. There was **no gate
at closure time**: nothing forced "shipped" to mean "confirmed live" at the
moment an arc was closed.

## The gate

`scripts/arc-live-probe.sh` is a synchronous single-hub probe that confirms a
capability/version is **actually being served right now**. It reuses the exact
probe invocations of the T-2359 / T-2415 canaries:

- **version** — `fleet doctor --json` → the hub's `.hub_version`, dotted-numeric
  compared against `--min-version`.
- **capability `cv-keys`** — `channel cv-keys agent-presence --hub <addr> --json`,
  the doorbell discovery prerequisite (capable iff a numeric `.count` comes back).
- **capability `field:<name>`** — a named non-null field on the hub's fleet-doctor
  entry (e.g. `field:hub_version`), for asserting a specific served capability.

Exit codes are contract-stable and **fail-closed**:

| Exit | Meaning |
|------|---------|
| 0 | live-confirmed — every asserted check passed against the live hub |
| 1 | shipped-but-not-live — hub reachable but version < floor OR capability rejected/omitted (the G-069 firing class) |
| 2 | tooling-error — hub unreachable / JSON unparseable / jq missing. **Never a false "live"** — a gate that failed open would re-admit the exact blindness it closes |

## How it becomes a closure gate (no new machinery)

The existing **P-011 verification gate** already runs every command in an
arc-closing task's `## Verification` block before allowing
`fw task update --status work-completed`, and blocks on any non-zero exit. So the
gate is just a convention: **an arc-closing (anchor) task adds one probe line to
its `## Verification`.**

```bash
# In the arc anchor task's ## Verification block:
bash scripts/arc-live-probe.sh --hub 192.168.10.107:9100 --min-version "$(cat VERSION)" --capability cv-keys
```

Now closure is mechanically blocked until the primary hub genuinely serves the
shipped version + capability. No AEF `arc.sh` change, reversible, no user-facing
surface removed.

### Advisory-by-convention now; mandatory-block later (human decision)

This ships the **advisory-by-convention** form (the arc author opts in per arc).
The stronger form — making the live-probe *mandatory* inside AEF
`.agentic-framework/lib/arc.sh arc_close` so no arc can close without it — is
T-2477 IW-1, a **human governance decision** (it changes closure behavior for
every consumer project and is cross-repo per G-062). It is intentionally left as
a follow-up toggle rather than shipped autonomously.

## Ad-hoc use

```bash
# Is this hub actually serving the version I think I shipped?
bash scripts/arc-live-probe.sh --hub <addr> --min-version 0.11.714

# Is this hub doorbell-capable (cv_index / push-wake discovery)?
bash scripts/arc-live-probe.sh --hub <addr> --capability cv-keys

# Machine-readable:
bash scripts/arc-live-probe.sh --hub <addr> --min-version "$(cat VERSION)" --capability cv-keys --json
# -> {ok, live, hub, checks:[{check,ok,detail}], reason}
```

Host-independent tests: `bash scripts/test-arc-live-probe.sh` (PL-213 fixtures,
no live hub needed).

## Related

- T-2477 (P3 inception, GO'd) — this is part (a); part (b) is the make-it-live
  fleet primitive (T-2481, new subsystem, needs its own GO).
- T-2359 / T-2387 / T-2415 — the daily-cron detection canaries this gate reuses.
- G-070 — restart THROUGH systemd when rolling a capability live (the remediation
  the firing path points at).
- P-011 — the verification gate that does the actual enforcing.
