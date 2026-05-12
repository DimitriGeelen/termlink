---
id: T-1633
name: "TERMLINK_RUNTIME_DIR default regression — root no longer selects /var/lib/termlink"
description: >
  Regression discovered during T-1166 .122 swap (2026-05-12). Pre-cut hub binary 0.9.1702 defaulted to /var/lib/termlink when running as root with TERMLINK_RUNTIME_DIR unset. Post-cut binary 0.9.2093 requires explicit env to use that path. ring20-management-agent caught this on .122; hub came up correctly only after setting env explicitly. Fleet rollout to other hubs without explicit env will fall back to /tmp/termlink-0 (volatile, PL-021 territory). Need: search 0.9.1702..0.9.2093 commit range for runtime_dir selection logic changes; restore root-uid-0 default OR document the new contract.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [T-1166, runtime-dir, regression, G-009]
components: []
related_tasks: [T-1166, T-1310, T-1290, T-1294]
created: 2026-05-12T21:55:22Z
last_update: 2026-05-12T22:19:06Z
date_finished: null
---

# T-1633: TERMLINK_RUNTIME_DIR default regression — root no longer selects /var/lib/termlink

## Context

**Investigation conclusion (2026-05-13): NOT a binary regression. Proximate cause was operator procedure during the T-1166 .122 swap.**

### What ring20-manager observed
After the .122 swap from 0.9.1702 → 0.9.2093, the new hub came up with runtime_dir = `/tmp/termlink-0` instead of `/var/lib/termlink`. The fix was to explicitly `export TERMLINK_RUNTIME_DIR=/var/lib/termlink` before restart.

### Code-side investigation
`crates/termlink-session/src/discovery.rs::runtime_dir()` has resolution order:
1. `$TERMLINK_RUNTIME_DIR`
2. `$XDG_RUNTIME_DIR/termlink`
3. `$TMPDIR/termlink-$UID`
4. `/tmp/termlink-$UID`

There is **no** `uid == 0 → /var/lib/termlink` branch. **There never has been one.** Verified by reading discovery.rs at the T-1310 archive commit (06df2393): identical to HEAD. The function has had only 5 commits in its entire history; none of them added or removed a uid-aware path.

### Where the `/var/lib/termlink` selection actually comes from
1. **systemd unit** (`.context/systemd/termlink-hub.service:25`) — `Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink`. This was added at T-931 and is the canonical mechanism for production hubs.
2. **`scripts/hub-binary-swap.sh`** — inherits the running hub's env from `/proc/$PID/environ` (line 121) and re-exports it on respawn (lines 187, 269). The canonical swap procedure preserves the env automatically.

### Why .122 fell back to /tmp
.122 was never migrated to systemd (T-1294 was started-work, never completed). The previous 0.9.1702 hub was launched via a watchdog/launcher that set the env. When the T-1166 swap used bare `nohup termlink hub start ...` (Option C respawn), the new process did **not** inherit the launcher's env — it started fresh with `TERMLINK_RUNTIME_DIR` unset, fell through discovery.rs's chain, and landed in `/tmp/termlink-0`.

### Why ring20-manager (correctly) called it a regression
From the operator's view it looks like a binary regression because the symptom changed across the swap. But the binary itself is identical in this resolution logic. The lost-env was the cause.

### Three-prong remediation
1. **Already-correct tooling:** `scripts/hub-binary-swap.sh` is the canonical swap procedure and does the right thing. The T-1166 .122 deploy did NOT use this script (used Option C manual respawn instead because no watchdog/systemd unit). Document this preference.
2. **Defensive startup warning (this task's code deliverable):** When the hub binary starts as root AND `TERMLINK_RUNTIME_DIR` is unset AND falls through to `/tmp/termlink-$UID`, emit a `tracing::warn!` pointing operators at `TERMLINK_RUNTIME_DIR=/var/lib/termlink` and the volatile-runtime_dir warning (PL-021). This catches future bare-respawn footguns at the next restart.
3. **Architectural fix already captured:** Install systemd unit on .122 (T-1294, started-work). Once systemd-launched, the env is in the unit file and survives binary swaps trivially without bespoke env-inheritance logic.

## Acceptance Criteria

### Agent
- [x] Investigation finding documented in this task's Context section — no binary regression, lost env via bare-respawn
- [x] Hub startup emits a clear warning when running as root with default runtime_dir = `/tmp/termlink-$UID` (no `TERMLINK_RUNTIME_DIR` set) — `warn_if_volatile_default_runtime_dir` in server.rs, invoked at the top of `run_with_tcp`
- [x] Warning points operators at `TERMLINK_RUNTIME_DIR=/var/lib/termlink` and references PL-021 (volatile /tmp) — text includes both
- [x] Warning is one-shot at startup (not repeated per-request) — called from `run_with_tcp` before pidfile acquire
- [x] `cargo check --workspace` passes — clean (1 pre-existing unrelated warning in termlink-mcp)
- [x] Test asserts the warning condition fires (when uid=0 + no env + /tmp path) — `runtime_dir_warn::warns_when_root_and_tmp_and_env_unset`
- [x] No false positive: warning does NOT fire when `TERMLINK_RUNTIME_DIR` is explicitly set (even to /tmp), or when uid != 0 — three negative tests cover env-set, non-root, and root+non-tmp (4 cases total, all pass)

### Human
- [ ] [REVIEW] On next .122 deploy (post-bake), the warning is visible in hub stderr/journal if the operator forgets to set the env.
  **Steps:**
  1. After deploy + restart, check hub logs: `journalctl -u termlink-hub -n 50 2>/dev/null || tail -50 /var/log/termlink-hub.log`
  2. Look for the warning line containing `TERMLINK_RUNTIME_DIR` and `/var/lib/termlink`
  **Expected:** If the operator forgot the env, warning appears. If env is set correctly, NO warning appears.
  **If not:** Capture log output, attach to task.

## Verification

cargo check --workspace
cargo test -p termlink-hub --lib runtime_dir_warn
grep -q "TERMLINK_RUNTIME_DIR" crates/termlink-hub/src/server.rs

## RCA

**Symptom:** After T-1166 .122 swap, hub came up with runtime_dir=`/tmp/termlink-0` instead of `/var/lib/termlink`. Looked like a binary regression.

**Root cause:** Not a code regression. The .122 hub was launched via a watchdog/launcher that set `TERMLINK_RUNTIME_DIR` in its environment. The T-1166 swap used a bare `nohup termlink hub start ...` respawn (Option C, due to no systemd unit on .122) which did NOT inherit the launcher's env. New process started with `TERMLINK_RUNTIME_DIR` unset → fell through `discovery::runtime_dir()`'s default chain → `/tmp/termlink-0`. The binary's resolution logic is unchanged and has never had a uid-aware default.

**Why structurally allowed:** Three structural gaps converged:
1. **No systemd unit on .122.** T-1294 (started-work, never completed) was the canonical fix — systemd's `Environment=` line makes the env survive any binary swap. .122 was always one step behind the fleet architecture.
2. **No defensive startup warning.** Hub silently selects `/tmp/termlink-0` when run as root without env, even though PL-021 has flagged volatile-/tmp for months as a known footgun. No log message at startup says "you probably didn't mean this."
3. **Canonical swap script (`scripts/hub-binary-swap.sh`) was not used.** That script inherits the running hub's env from `/proc/$PID/environ` and re-exports it on respawn. The Option C improvised path bypassed it.

**Prevention:**
- **Code (this task):** Hub startup warning when uid=0 + env unset + path → /tmp. Catches the next bare-respawn footgun at the next restart, not after operators notice persistence loss.
- **Architectural (T-1294, separate task):** Install systemd unit on .122. Once systemd-launched, env survives swaps trivially.
- **Procedural (already exists):** `scripts/hub-binary-swap.sh` is the canonical swap procedure. Document that bare-respawn is a last-resort path and must include `TERMLINK_RUNTIME_DIR=...` prefix.

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

## Updates

### 2026-05-12T21:55:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1633-termlinkruntimedir-default-regression--r.md
- **Context:** Initial task creation

### 2026-05-12T22:19:06Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
