# T-940 RCA Report: Persistent Agent Sessions on .107

**Date**: 2026-04-12
**Session**: claude-on-local (dimitri-mint-dev)
**Related tasks**: T-921, T-931, T-933, T-936, T-940

---

## Executive Summary

Deploying persistent `framework-agent` and `termlink-agent` sessions on .107 uncovered a **runtime directory split-brain** that blocks cross-host session discovery. The hub and agent sessions now work, but with a fragile two-pool architecture. Five enhancement proposals (E1–E5) and three fix proposals (F1–F3) are documented below.

---

## Findings

### F1 — Runtime dir split-brain (CRITICAL — blocks cross-host)

| Component | Runtime dir | Pool |
|---|---|---|
| Hub (systemd, TCP 9100) | `/var/lib/termlink` | hub-visible → remote callers |
| Agent sessions (systemd) | `/var/lib/termlink` (manually set) | hub-visible → remote callers |
| CLI (`termlink list`) | `/tmp/termlink-0` (default) | local-only |
| Ephemeral task workers | `/tmp/termlink-0` (default) | local-only |

**Root cause**: T-931 migrated the hub to a persistent dir for T-933 (survive reboots) but didn't migrate anything else. The CLI has no way to discover sessions in `/var/lib/termlink` without `TERMLINK_RUNTIME_DIR` env var.

**Impact**: Remote callers reported "No sessions on .107:9100" even with 5+ sessions registered locally. Sibling agent on .109 was blocked for an entire session.

**Current workaround**: Agent service files explicitly set `TERMLINK_RUNTIME_DIR=/var/lib/termlink`. Local CLI inspection requires `TERMLINK_RUNTIME_DIR=/var/lib/termlink termlink list`.

### F2 — install.sh doesn't detect changed unit files

When a service file changes and `systemctl start` is called (already active), the old process keeps running with stale config. Required manual `systemctl restart`. This caused a false-green where services showed `active (running)` but sessions were invisible.

**Fix**: Compare checksums before/after copy, trigger `restart` instead of `start` when the file changed.

### F3 — `termlink hub status` reports "not running" with systemd hub

The CLI binary (v0.9.450 in PATH) checks `/tmp/termlink-0/hub.pid` for the hub. The systemd-managed hub writes to `/var/lib/termlink/hub.pid`. Same split-brain causes false negatives.

---

## Enhancement Proposals

### E1 — Unify runtime dir system-wide (fixes F1 + F3)

**What**: Set `TERMLINK_RUNTIME_DIR=/var/lib/termlink` in `/etc/profile.d/termlink.sh` (interactive shells) and `/etc/environment` (non-interactive processes, cron, systemd). All termlink processes join one pool.

**Why**: Eliminates the two-pool architecture. `termlink list` shows everything. `termlink hub status` finds the pidfile. Remote callers see all sessions.

**Risk**: Low — the env var is only meaningful to the termlink binary. No other software uses it. Existing `/tmp/termlink-0` contents become orphaned (cleaned by tmpfs on reboot anyway).

**Cost**: ~5 min. One line in two files.

**Verdict**: Recommended as immediate fix. Defers the deeper question of whether termlink should have a config file.

### E2 — install.sh restart-on-change (fixes F2)

**What**: After copying a service file, compare checksums. If changed, use `systemctl restart` instead of `systemctl start`.

**Why**: Prevents stale-config false-green. Caught us during this deploy — agents appeared active but weren't discoverable because they were running the previous unit file.

**Cost**: ~10 lines of bash in install.sh.

**Verdict**: Recommended. Low risk, prevents a real operational footgun.

### E3 — Agent session service files as framework scaffold

**What**: When `fw deploy scaffold` or `fw init` creates a project with termlink integration, include template service files for framework-agent and termlink-agent alongside the hub service.

**Why**: Every consumer project that has a hub will eventually need named persistent sessions. Without templates, each project reinvents the same unit files.

**Scope**: Framework upstream change (pickup to agentic-engineering-framework).

**Verdict**: Recommend as upstream feature proposal.

### E4 — Hub bridges multiple session dirs (fixes F1 permanently)

**What**: Modify the hub's session scanner to accept multiple `--sessions-dir` paths or auto-scan both `/var/lib/termlink/sessions` and `/tmp/termlink-0/sessions`.

**Why**: Eliminates the need for unified runtime dir entirely. Ephemeral sessions stay in tmpfs, persistent sessions in `/var/lib`. Hub sees both.

**Scope**: Rust code change in `termlink-hub` crate.

**Verdict**: Best long-term fix, but higher cost. Recommend as follow-up if E1 is insufficient.

### E5 — Persistent session health check endpoint

**What**: Add a `termlink agent ask framework-agent --action health` built-in that returns session uptime, fw version, PROJECT_ROOT, and pickup inbox status without custom handler code.

**Why**: Remote callers need a lightweight way to verify a session is alive and functional, beyond just `termlink ping` (which only checks the socket). Currently requires `termlink exec` + running a command.

**Scope**: Rust code change in `termlink-session` crate or agent protocol.

**Verdict**: Nice-to-have. Deferred.

---

## Recommendation

**Immediate** (do now):
- **E1**: Set `TERMLINK_RUNTIME_DIR=/var/lib/termlink` system-wide → unifies both pools
- **E2**: Patch install.sh restart-on-change → prevents stale-config deploys

**Next horizon** (pickup to framework agent):
- **E3**: Agent session templates in framework scaffold

**Later** (requires Rust changes):
- **E4**: Hub multi-dir scanning
- **E5**: Health check endpoint

---

## Value / Use Cases Served

| Use case | Before | After |
|---|---|---|
| Remote agent queries framework state | Blocked (0 sessions visible) | `termlink remote exec .107:9100 framework-agent 'fw pickup status'` |
| Remote agent sends pickup envelope | Blocked | `termlink remote push .107:9100 framework-agent envelope.yaml` |
| Local operator lists all sessions | Fragmented (two `termlink list` calls with different env) | Single `termlink list` |
| Agent session crashes | Gone until manually restarted | systemd restarts within 5s, re-registers |
| Hub restarts after reboot | Secret regenerated, remote clients broken | Secret persists in `/var/lib/termlink` |
| New consumer project deployed | Must hand-write agent service files | Scaffold includes templates (E3) |

---

## Source

Session on dimitri-mint-dev, 2026-04-12. Discovered during P-009 pickup-processor rollout and persistent agent session deployment. Five iterations of runtime-dir debugging before arriving at the two-pool workaround.
