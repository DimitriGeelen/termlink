---
id: T-1641
name: "Eliminate rogue UDS-only hub on .121 ring20-dashboard"
description: >
  On 2026-05-15 forensics for the T-1632/T-1633 fleet rollout, .121 was found running TWO concurrent termlink hub processes: PID 1895580 (started May 3, bound to TCP 9100 + /var/lib/termlink/hub.sock — canonical) AND PID 2391097 (started May 15 07:08, bound to /tmp/termlink-0/hub.sock only, no --tcp arg — rogue). Local agents on .121 hit the rogue via UDS; remote agents hit the canonical via TCP. The watchdog at /root/ring20-dashboard/scripts/watchdog.sh sets TERMLINK_RUNTIME_DIR=/var/lib/termlink AND uses --tcp, so the rogue did NOT come from there — some other launch path is spawning bare 'termlink hub start' without env or --tcp. Cause unknown. Risk: drift in local-vs-remote view of hub state (capabilities, legacy usage, session list, channel state, secret rotation if /tmp gets wiped). Action: identify the spawn origin (could be a manual operator command, a script under /root/ring20-dashboard/, an @reboot cron, a systemd service, or a sibling project on the same host); document; either remove the spawn or document it as intentional with a runtime_dir + --tcp fix. Workdir: /root/ring20-dashboard on .121 (via termlink remote exec). Related: T-1633 (volatile-/tmp warning will fire on the rogue once 0.9.2127 is deployed).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug, ring20-dashboard, dual-hub, fleet-hygiene, T-1633]
components: []
related_tasks: [T-1632, T-1633, T-1640]
created: 2026-05-15T20:43:37Z
last_update: 2026-05-15T21:11:11Z
date_finished: null
---

# T-1641: Eliminate rogue UDS-only hub on .121 ring20-dashboard

## Context

Forensic discovery during T-1632/T-1633 .121 deploy (2026-05-15T20:43Z):
two concurrent `termlink hub` processes on ring20-dashboard.

| PID | Started | Runtime dir | TCP bind | Origin |
|---|---|---|---|---|
| 1895580 | 2026-05-03 | `/var/lib/termlink` | `:9100` | Watchdog (canonical) |
| **2391097** | 2026-05-15 07:08 | `/tmp/termlink-0` | none (UDS-only) | **Unknown — this task** |

Watchdog at `/root/ring20-dashboard/scripts/watchdog.sh` sets
`TERMLINK_RUNTIME_DIR=/var/lib/termlink` and uses `--tcp`, so the rogue
did NOT come from there. Candidates to check: manual operator command,
@reboot cron, systemd unit, sibling project, agent self-spawn from a
session-init hook. Local agents on .121 hit the rogue via UDS; remote
agents hit the canonical via TCP — a silent local-vs-remote split.

Risk: drift in hub state (capabilities, legacy-usage telemetry, session
list, secret rotation if /tmp gets boot-wiped) between the two views.

Rogue was already killed in this session — the open question is whether
something on .121 will respawn it. This task validates the absence of a
respawn mechanism and either documents the spawn origin or removes it.

## Acceptance Criteria

### Agent
- [x] Spawn origin identified — manual operator hub start at 2026-05-15 07:08 in an env-less shell; no automated trigger. Adjacent issue: dashboard's `termlink_events.py:ensure_self_session()` spawns `termlink register --self` via subprocess.Popen with no env override, so register-time sessions also land in /tmp when the dashboard's own env lacks `TERMLINK_RUNTIME_DIR` (see RCA)
- [x] Documented (this RCA) + cross-project memo for the env hardening recommendation (see "Prevention — proposed" in RCA)
- [x] Exactly one `termlink hub` process running on .121 — `pgrep -af '[t]ermlink hub'` returns PID 2704841 only
- [x] No live UDS at `/tmp/termlink-0/hub.sock` on .121 — `ss -lx` shows no LISTEN there; the canonical UDS is `/var/lib/termlink/hub.sock`
- [x] RCA documented below

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

# Verify state on .121 from local control plane.
test "$(termlink remote exec ring20-dashboard tl-qe2pao72 'pgrep -af "[t]ermlink hub" | wc -l' 2>/dev/null | tr -d '\r\n ')" = "1"
test -z "$(termlink remote exec ring20-dashboard tl-qe2pao72 'ss -lx 2>/dev/null | awk "/\/tmp\/termlink-0\/hub.sock/{print}"' 2>/dev/null | tr -d '\r\n ')"

## RCA

**Symptom.** On 2026-05-15, .121 ring20-dashboard was found running two
concurrent `termlink hub` processes: canonical PID 1895580 (May 3,
`/var/lib/termlink`, TCP :9100) and rogue PID 2391097 (May 15 07:08,
`/tmp/termlink-0`, UDS-only, no `--tcp`). Local agents hit the rogue,
remote agents hit the canonical — a silent split-brain in capabilities,
session lists, channel state, and legacy-usage telemetry. Surfaced during
the T-1632/T-1633 fleet rollout when probing pre-state.

**Root cause.** Multiple env-deficient `termlink` invocations on .121 land in
the default `discovery::runtime_dir()` chain (no `TERMLINK_RUNTIME_DIR`
→ no `$XDG_RUNTIME_DIR` → no `$TMPDIR` → `/tmp/termlink-$UID`). The rogue
hub was spawned by one such invocation — most likely a manual operator
`termlink hub start` from an interactive shell at 07:08 — in a process
that did NOT export the env. There is no automated trigger. Evidence:

- No script in `/root/ring20-dashboard/` or `/etc/` invokes
  `termlink hub start` outside `scripts/watchdog.sh:16`. Watchdog exports
  `TERMLINK_RUNTIME_DIR=/var/lib/termlink` (line 7), so its invocations
  go to `/var/lib`. The bare-respawn at 07:08 was NOT watchdog.
- `/tmp/termlink-0/bus/topics/` and `/tmp/termlink-0/bus/artifacts/` show
  activity 07:08–07:29 then idle — consistent with a one-off operator
  session, not a recurring cron.
- Cron file `/etc/cron.d/agentic-audit-ring20-dashboard` sets `SHELL=/bin/bash`
  and `PATH=...` but NOT `TERMLINK_RUNTIME_DIR`. Cron-launched `fw audit`
  / `fw pickup process` / `reflection-cron.sh` therefore call termlink
  CLI without env. None of them call `hub start`, but any that do (or
  ever will) would land in /tmp.
- Dashboard PID 2268166 environ also lacks `TERMLINK_RUNTIME_DIR` (only
  HOME, PATH, CLAUDE_CODE_EXECPATH) — it was started by Claude Code
  in an interactive context, not by watchdog. The dashboard's
  `ensure_self_session()` at `/root/ring20-dashboard/termlink_events.py:97-127`
  spawns `termlink register --self ...` via `subprocess.Popen(...)` with NO
  `env=` override, so register-time session sockets land in /tmp too. The
  orphaned PID 427 (still holding `/tmp/termlink-0/sessions/tl-4augvpzt.sock`
  since May 2 boot) is from this code path.

**Why structurally allowed.** Three converging gaps:

1. **No environment-level pin.** Production hosts running `termlink` rely
   on each invoker setting `TERMLINK_RUNTIME_DIR`. Cron-installed PATH
   line is set but env is not. `/etc/environment`, profile.d, and the
   container's systemd default-env do not pin it. Any operator shell
   that doesn't manually export it falls into /tmp.

2. **Silent fallback.** Pre-T-1633 binaries silently chose `/tmp/termlink-$UID`
   when env was unset and uid=0. No log. No warning. PL-021 (volatile /tmp)
   was a known concern but produced no in-binary signal. **Closed by T-1633**:
   0.9.2127 emits a tracing::warn at hub-start with the recommendation
   to set `TERMLINK_RUNTIME_DIR=/var/lib/termlink`. Now-deployed on .121.

3. **Dashboard self-register inherits caller env.** `termlink_events.py:113`
   uses `subprocess.Popen([...register...], start_new_session=True)` with
   no `env=` argument, so it inherits whatever env the dashboard was
   started with. If the dashboard was launched from a context that lacked
   the env (Claude Code, interactive operator shell), every register-self
   call lands in /tmp. There's no defense-in-depth at the call site.

**Prevention — shipped.**
- T-1633 startup warning (0.9.2127, deployed to both prod hubs today). Any
  future bare `termlink hub start` as root with env unset will log a
  WARN pointing at `TERMLINK_RUNTIME_DIR=/var/lib/termlink`. Catches the
  next operator-footgun BEFORE persistence loss is observed.
- `scripts/hub-binary-swap.sh` already inherits and re-exports the running
  hub's env from `/proc/$PID/environ` (T-1051 / T-933 family). Operators
  using the canonical swap script automatically preserve env.

**Prevention — proposed (cross-project, ring20-dashboard scope).**
- Patch `termlink_events.py:ensure_self_session()` to pass an explicit
  `env=` dict to `subprocess.Popen()` that includes
  `TERMLINK_RUNTIME_DIR=/var/lib/termlink` (or whatever the canonical
  hub's runtime_dir is). One-line defense-in-depth at the call site.
- Optionally: set `Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink` in
  /etc/environment on .121 so all root-uid shells (including Claude Code
  sessions and ad-hoc operator commands) inherit it. Container-wide pin.

**State at close.**
- Single hub on .121: PID 2704841, TCP 0.0.0.0:9100, runtime_dir=/var/lib/termlink,
  binary 0.9.2127 (includes T-1633 warning).
- `/tmp/termlink-0/hub.sock` not bound; no live hub there.
- Orphaned register session PID 427 still alive (harmless: parent=init,
  fd=9 LISTEN on session sock, no hub backing it, idle since May 15 20:43
  when its hub died). Will be cleaned up on next reboot. Killing it now
  would not improve anything and might trigger a fresh `ensure_self_session`
  cycle from the dashboard.
- No respawn observed since 20:43.

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-15T20:43:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1641-eliminate-rogue-uds-only-hub-on-121-ring.md
- **Context:** Initial task creation

### 2026-05-15T21:11:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
