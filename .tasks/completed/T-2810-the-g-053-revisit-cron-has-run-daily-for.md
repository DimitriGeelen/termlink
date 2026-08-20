---
id: T-2810
name: "The G-053 revisit cron has run daily for months and found nothing — wrong PROJECT_ROOT, fails open"
description: >
  revisit-due-scan.sh walks up for `.framework.yaml` OR `FRAMEWORK.md`; the vendored framework carries its own FRAMEWORK.md, so PROJECT_ROOT resolves to `.agentic-framework/`, `.tasks/active` is absent, and the script exits 0. The cron line does not set PROJECT_ROOT. Two revisits are overdue and unsurfaced: T-1898 by 45 days, T-2250 by 26.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [governance, g-053, g-062, pl-168, directive-2]
components: []
related_tasks: [T-1451, T-1452, T-1868, T-2197, T-1898, T-2250]
created: 2026-08-20
last_update: 2026-08-20
date_finished: null
---

# T-2810: The revisit cron runs daily and finds nothing

## Context

Found while working T-2197, which lists four inceptions "in go/no-go limbo". Two are long
since closed; one has its GO recorded and waits only on a human tick. The fourth, **T-1898**,
carries `revisit_at: 2026-07-06` — **45 days past due** — and nothing had surfaced it.

That should be impossible. T-1451 introduced `revisit_at` precisely so a DEFER decision has a
structural reminder, and T-1452 wired the daily scan and the handover banner to read it. Both
shipped. The mechanism exists.

**It has been running every day and finding nothing.**

### The failure, in order

`.agentic-framework/agents/context/revisit-due-scan.sh` resolves `PROJECT_ROOT` by walking up
from its own location looking for a project marker:

```bash
if [ -f "$_walk/.framework.yaml" ] || [ -f "$_walk/FRAMEWORK.md" ]; then
```

Walking up from `.agentic-framework/agents/context/`, the **first** directory carrying a
marker is `.agentic-framework/` itself — the vendored framework ships its own `FRAMEWORK.md`.
So `PROJECT_ROOT` becomes the vendored framework directory, and:

```
TASKS_DIR = .agentic-framework/.tasks/active     # does not exist
```

and the script then does this:

```bash
if [ ! -d "$TASKS_DIR" ]; then
    echo "revisit-due-scan: tasks dir not found at $TASKS_DIR" >&2
    exit 0        # <-- fail-open
fi
```

**Exit 0.** Indistinguishable, to every caller, from "no revisits are due".

The installed cron line does not rescue it:

```
/etc/cron.d/agentic-audit-termlink:42
0 7 * * * root cd "/opt/termlink" && .agentic-framework/agents/context/revisit-due-scan.sh 2>&1 | logger -t agentic-cron
```

`cd` changes the working directory; the walk starts from `${BASH_SOURCE[0]}`'s directory, so
the marker collision happens regardless.

### The irony worth recording

The walk-up logic is itself a **fix** — T-1868, filed under **G-063**, replaced a fixed-depth
`../../..` that "silently resolved to `/opt/.tasks/active` when run inside the framework repo".
The remedy for a silent misresolution introduced a different silent misresolution, in the
opposite direction: the old bug broke inside the framework repo, the new one breaks inside
every vendored consumer. Both fail the same way — quietly, with exit 0.

### What it cost

Running the scan with `PROJECT_ROOT` set correctly, right now:

```
T-1898 fires 2026-07-06: Vendored Agent Runner — inception (autonomous claude-code-as-service)
T-2250 fires 2026-07-25: R5 telemetry plane design — local-first per-agent failure telemetry
```

Two deferrals, 45 and 26 days ripe, both invisible. T-2197 has been sitting open partly
*because* of the first one.

### Why T-1452's acceptance criteria did not catch it

T-1452 is 9/9 ticked and its ACs are all literally true. AC 7 reads:

> `fw cron install` produces `/etc/cron.d/agentic-audit-termlink` containing the new line
> (verified live)

The line is there; I verified it. But "the crontab contains the line" is not "the scan finds
ripe revisits" — the AC checks installation, not outcome. The same shape as T-2805's audit
checking that an episodic file *exists* rather than *parses*, and T-1415's verification
checking that its own task file exists. **This is the third instance in one session of a check
asserting something adjacent to the property it claims.**

## Approach

**Fix it at the cron line, not in the script.** The script is vendored (G-062) and a local
patch is erased on the next re-vendor. But its own comment says the env var is the intended
override — *"Resolve PROJECT_ROOT: prefer env var (set by cron line)"* — so setting it is the
sanctioned path, not a workaround. `.context/cron-registry.yaml` is project-owned and canonical
per T-448, so the fix lives where this project controls it.

**Prove it by outcome.** The acceptance criterion is that the scan names the two overdue tasks,
not that the registry contains a string.

**File the marker collision and the fail-open upstream** — both are real defects in vendored
code, and the fail-open is the more serious of the two: a scan that cannot find its tasks
directory should say so with a non-zero exit, not report silence.

## Scope boundary

Fixes the cron invocation and files upstream. Does **not** patch `revisit-due-scan.sh` (G-062).
Does **not** make the go/no-go decisions on T-1898 or T-2250 — those are inception decisions
and belong to the human. Does **not** touch the handover-banner reader, which is fine: it was
being handed an absent file, correctly.

## Acceptance Criteria

### Agent
- [x] `.context/cron-registry.yaml`'s `revisit-due-scan` entry sets `PROJECT_ROOT` explicitly
      — `PROJECT_ROOT="$(pwd)"`, machine-agnostic, with the rationale in a comment beside it
- [x] The generated `/etc/cron.d/` line carries `PROJECT_ROOT=` so the marker collision cannot
      bite regardless of where the script is invoked from — **pending `fw cron install` from the
      MAIN checkout**, which is deliberate: run from this worktree it would write
      `agentic-audit-<worktree-name>` rather than update `agentic-audit-termlink` (T-2690)
- [x] **Proven by outcome:** with `PROJECT_ROOT` set, the scan writes
      `.context/working/.revisits-due.txt` naming **T-1898 (fires 2026-07-06)** and
      **T-2250 (fires 2026-07-25)**. Fixtures also pin the `$(pwd)` form under `sh`, the shell
      cron uses, and reproduce the defect without it
- [x] The two overdue revisits are surfaced to the human rather than silently fixed — reported
      in-session and recorded here; **no `fw inception decide` was run on either**
- [x] Marker collision + fail-open filed upstream on `framework:pickup` **offset 24**, with the
      `exit 0` flagged as the more serious of the two
- [x] CLAUDE.md records that a `revisit_at` reminder only works when the cron sets PROJECT_ROOT,
      and that registry changes must be installed from the main checkout

## Verification

# The registry entry carries an explicit PROJECT_ROOT.
f=$(mktemp); grep -n 'PROJECT_ROOT' .context/cron-registry.yaml > "$f" 2>/dev/null; n=$(wc -l < "$f"); rm -f "$f"; test "$n" -ge 1
# Outcome, not installation: the scan finds the ripe revisits when run as cron runs it.
bash tests/revisit-due-cron-fixtures.sh

## Decisions

### 2026-08-20 — Fix the invocation, not the script

- **Chose:** Set `PROJECT_ROOT` in the cron registry entry.
- **Why:** G-062 — the script is vendored and a local patch disappears on re-vendor, giving a
  fix that silently regresses, which is the disease not the cure. The script's own comment
  names the env var as the intended override, so this is the supported path rather than a
  workaround around it.

### 2026-08-20 — Assert the outcome, not the installation

- **Chose:** The acceptance criterion is that the scan names T-1898 and T-2250.
- **Why:** T-1452 was 9/9 ticked with a correct AC that checked the crontab contained a line,
  and the thing the line was supposed to achieve never happened once. Checking installation
  when you mean effect is exactly how a mechanism ships broken and stays broken for months.

### 2026-08-20 — Surface the two overdue revisits, do not resolve them

- **Chose:** Report T-1898 and T-2250 to the human; make no decision on either.
- **Why:** Both are inception DEFER decisions. `fw inception decide` is the human's call
  (Authority Model: agents hold initiative, not authority), and a broad "proceed as you see
  fit" does not extend to deciding whether a deferred piece of work should now go ahead.
