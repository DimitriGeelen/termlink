# /preflight — deploy-time substrate correctness check (T-2158 skill-layer wrap)

Wraps `scripts/substrate-preflight.sh` (T-2154). The "before I trust this
substrate, is the environment actually set up right?" verb. Catches
**deployment-time** misconfigurations that would silently corrupt the
substrate hours later — PL-021 volatile /tmp, missing hubs.toml, dead
be-reachable listener.

Read-only, no network, no auth, no state mutation. Safe to run anytime,
anywhere.

`/preflight` is the **DEPLOY-TIME** complement to the substrate
runtime-read daily-verb set:

| Skill | Question | Timing |
|-------|----------|--------|
| `/preflight` (this skill) | Is my environment set up to host a substrate? | Deploy time, before first use |
| `/substrate` (T-2096) | Is my substrate healthy right now? | Runtime, any time |
| `/self-test` (framework) | Does fw doctor / cargo test pass? | After code changes |
| `fw doctor` | Is the framework itself healthy? | Periodic |

The four answer four distinct operational questions. Confusing them is
how a hub silently regenerates its secret every reboot for 14 days
before anyone notices (PL-021 / G-058 class).

**Invocation:**

| Form | Action |
|------|--------|
| `/preflight` | Run all five checks (human-format render) |
| `/preflight --json` | Machine-readable envelope (passes through) |

## What it checks

| # | Check | Severity | Why |
|---|-------|----------|-----|
| 1 | `TERMLINK_RUNTIME_DIR` NOT on /tmp | HIGH | PL-021: hub regenerates secret + TLS cert every reboot. Two volatile mechanisms detected: tmpfs mount AND systemd-tmpfiles D-rule wipe (the rule that looks innocent in `mount` output but still nukes /tmp on boot — T-1294). |
| 2 | `~/.termlink/hubs.toml` present + has `[hubs.*]` sections | MEDIUM | Without it, every heal path (T-1054/T-1055/T-1291) fails; fleet verbs (fleet doctor, fleet verify, fleet history) all return "no profiles". |
| 3 | `~/.termlink/be-reachable.state` PID alive | MEDIUM | If pid is dead, pickup loops, agent contact, and DM receipts all look healthy at registration time but the listener is gone. Catches the "I forgot to `/be-reachable` again after reboot" footgun. |
| 4 | `termlink --version` >= project root `VERSION` | MEDIUM | T-2181: catches stale-CLIENT footgun where catalog promises flags like `--only-stuck` (T-2076) or subcommands like `fleet governor-status` (T-2062) that an older binary refuses with `unknown flag`. WARN-only — substrate still works for primitives the binary has. Skipped silently outside the project tree (no `VERSION` file). Remediation: `cargo build --release && install -m 755 target/release/termlink ~/.cargo/bin/`. |
| 5 | local hub serves T-2139 `rate_buckets_evicted_total` field | MEDIUM | T-2184: symmetric companion to Check 4. Probes running hub via `termlink hub status --governor --json` for field presence. Absence ⇒ pre-T-2139 hub (typically: operator ran `cargo install` but never restarted hub — `/proc/<pid>/exe` shows `...(deleted)`, in-memory binary keeps serving old envelopes). The CLI loyally renders absent fields as `n/a` and the operator infers missing-feature when the actual gap is missing-restart. Skipped when hub down (Check 1 territory). Remediation: restart hub to pick up new binary; verify runtime_dir persistence per Check 1 first. Origin: PL-209 spent ~30min chasing "missing telemetry" that was a missing restart. |

Exit codes:
- `0` — all PASS, substrate-ready
- `1` — WARN (medium issue — may proceed but should fix)
- `2` — FAIL (high issue — substrate WILL silently misbehave)

## Step 1: Pre-flight (of the pre-flight)

Run:

```
bash scripts/substrate-preflight.sh --json >/dev/null 2>&1 || true
```

(The script itself returns non-zero on WARN/FAIL — that's expected output, not a missing-script error.) Check whether the script EXISTS first:

```
test -x scripts/substrate-preflight.sh
```

If not executable: **stop**. Print:

```
preflight: scripts/substrate-preflight.sh missing or non-executable.
Ensure you're in the TermLink project root (cd /opt/termlink).
If the script exists but is not +x, run: chmod +x scripts/substrate-preflight.sh
(See PL-208 — shipped framework scripts can silently lose +x bit.)
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. Pass through verbatim — the script
validates and errors on malformed input.

| User typed | Command emitted |
|------------|-----------------|
| `/preflight` | `bash scripts/substrate-preflight.sh` |
| `/preflight --json` | `bash scripts/substrate-preflight.sh --json` |

## Step 3: Run the script

Execute via Bash. Capture stdout + stderr + exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the script's stdout through verbatim. The format is one
  `[PASS|WARN|FAIL]` line per check followed by a one-line summary.
- Surface the exit code at the end if non-zero (so the operator sees
  the severity without having to read every line).

For `--json` mode: pass the JSON envelope through verbatim. Callers
piping/parsing rely on the script's schema:

```json
{
  "ok": true|false,
  "exit_code": 0|1|2,
  "checks": [
    {"name": "runtime_dir", "severity": "high", "status": "pass|warn|fail", "message": "...", "remediation": "..."},
    ...
  ],
  "summary": {"pass": N, "warn": N, "fail": N}
}
```

## Step 5: Failure-mode next-step hints

If exit 1 (WARN) or exit 2 (FAIL), after the script's own output append
contextual pointers depending on which check tripped:

**Check 1 (runtime_dir) failed:** PL-021 territory. Point at:
- `docs/operations/termlink-hub-runtime-migration.md` — systemd-launched hub fix
- CLAUDE.md §"Hub Auth Rotation Protocol" → "Special case — volatile runtime_dir" — watchdog-launched hub fix
- Diagnostic: `mount | grep ' /tmp '` + `cat /usr/lib/tmpfiles.d/tmp.conf`

**Check 2 (hubs.toml) failed:** No declared profiles. Point at:
- `termlink fleet add <name> <addr>` to declare a hub
- Or copy from a peer host's `~/.termlink/hubs.toml`

**Check 3 (be-reachable) failed:** Listener dead. Point at:
- `/be-reachable start` to revive

Never silent on failure. The whole point of this verb is loud-refusal at
deploy time so the silent-failure cascade never starts.

## Step 6: When PASS, recommend the next verb

For exit 0 (all PASS), after the script's "substrate-ready" line, suggest:

```
Next: /substrate    (runtime digest — is anything happening?)
      /peers --all  (who's reachable across the fleet?)
```

This nudges the operator from "environment correct" to "runtime healthy"
— two distinct questions, both load-bearing.

## Rules

- **Read-only by contract.** Never modifies state, never touches the
  network, never authenticates. Safe in any context.
- **Loud at every severity.** Even WARN must be visible — silent
  "everything looks fine" is the failure mode this verb exists to
  prevent.
- **Do NOT compose with /substrate.** Preflight answers a different
  question (environment correctness vs runtime health). Operators run
  both separately; conflating them defeats the deploy-time/runtime
  distinction.
- **Run on every new host** before declaring the substrate ready.
- **No `AskUserQuestion`** — just run and report.

## Common patterns

**First-time host setup:**

```
/preflight                   # environment correct?
/be-reachable start          # advertise myself
/substrate                   # runtime healthy?
/peers --all                 # who else is here?
```

**After a reboot, before resuming work:**

```
/preflight                   # did /tmp get wiped?
/substrate                   # any leftover stuck claims?
```

**As part of automation (CI / cron):**

```
bash scripts/substrate-preflight.sh --json | jq -e '.ok'
```

Non-zero exit → page the operator.

## Related

- T-2154 — the underlying `substrate-preflight.sh` script.
- T-2018 — arc-parallel-substrate ADR; this skill closes the deploy-time
  observability gap that complements the runtime-read primitives.
- T-2096 / `/substrate` — runtime composition digest; the complementary
  daily verb (after /preflight passes).
- T-1841 / `/be-reachable` — the listener verb Check 3 validates.
- T-1290 / T-1294 / T-1296 — PL-021 incidents (volatile /tmp); Check 1
  prevents recurrence.
- G-058 — the 16-day silent mirror drift that motivated deploy-time
  loud-fail patterns more broadly.
- `docs/operations/substrate-getting-started.md` — the first-5-minutes
  walkthrough that runs `/preflight` first.
- `docs/operations/substrate-orchestrator-recipe.md` § "Deployment &
  identity troubleshooting" (T-2157) — the runbook for FAIL/WARN exits.
- PL-187 (verb-stack rung 6: ephemeral session skills)
- PL-208 (shipped framework scripts can lose +x bit — relevant when
  this skill's own pre-flight detects a non-executable underlying script)
