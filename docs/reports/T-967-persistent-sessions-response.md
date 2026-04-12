# T-967: Persistent Agent Sessions — TermLink Side Response

**Date:** 2026-04-12
**Coordinating with:** Framework T-1135
**TermLink task:** T-967 (inception, status: captured)
**Related tasks:** T-937 (cleanup kills dispatch workers), T-941 (service templates), T-959 (two-pool architecture)

## Answers to Framework Questions

### 1. Stale Session Cleanup — How Does It Work?

TermLink has **two independent cleanup mechanisms**, neither of which is a cron job:

**A. Hub Supervisor (in-process, runs inside `termlink hub`):**
- File: `crates/termlink-hub/src/supervisor.rs`
- Runs every **30 seconds** (DEFAULT_INTERVAL)
- Calls `liveness::is_alive(reg)` on every registered session
- `is_alive` does: (1) `kill(pid, 0)` syscall to check if PID exists, (2) verify socket file exists on disk
- Dead sessions get `session.exited` events emitted to all live sessions, then registration artifacts (`.json` + `.sock` files) are removed
- **No tag/role/metadata awareness** — kills anything where the PID is gone or socket is missing

**B. CLI `termlink clean` command (on-demand):**
- File: `crates/termlink-cli/src/commands/session.rs:512`
- Called by scripts: `tl-claude.sh`, `tl-dispatch.sh`, `sim-verify.sh`
- Same logic: scans `sessions_dir` for `.json` files, checks `liveness::is_alive()`, removes dead ones
- Supports `--dry-run` and `--json` output
- Also has **no tag awareness** for exemptions

**C. Remote Session TTL Reaper (for TCP sessions):**
- File: `crates/termlink-hub/src/remote_store.rs`
- Remote sessions expire after **5 minutes** (DEFAULT_TTL = 300s) without a heartbeat
- Reaper runs every 30 seconds (REAPER_INTERVAL)
- Again, **no persistent/protected exemption**

**There are NO external cron jobs for session cleanup** in this project. The cron-registry.yaml has audit/pickup jobs only.

### 2. Does TermLink Have Built-In Support for Persistent/Protected Sessions?

**No, but the infrastructure is 90% there.**

What already exists:
- `Registration` struct has `tags: Vec<String>`, `roles: Vec<String>`, `capabilities: Vec<String>`
- CLI supports `--tags`, `--roles`, `--cap` on `termlink register`
- Discovery supports filtering by tag: `manager::find_by_tag("persistent")`
- CLI list supports `--tag persistent` filtering
- Hub discovery supports tag-based filtering in `session.discover` RPC

What's missing:
- **Supervisor sweep does NOT check tags before cleanup** (`supervisor.rs:64-69` — partitions by `is_alive()` only)
- **`clean_stale_sessions()` does NOT check tags** (`manager.rs:323-350` — same liveness-only check)
- **Remote store `reap_expired()` does NOT check tags** (`remote_store.rs:149-153`)
- **No `persistent` tag convention** is documented or enforced

### 3. Session Naming Convention for Persistent "Receptionist" Sessions

**Proposed convention:**

```
termlink-agent         # TermLink's own receptionist
fw-agent               # Framework's receptionist  
{project-name}-agent   # Any consumer project's receptionist
```

With tags:
```
tags: ["persistent", "receptionist"]
roles: ["agent"]
capabilities: ["inject", "command", "query"]
```

The `display_name` is already used for human-readable identification and supports uniqueness checking (name conflicts are detected during registration). This convention works with existing infrastructure.

### 4. Could TermLink Add a `--persistent` Flag?

**Yes, and it's straightforward.** Two changes needed:

**Change 1: Registration side (small)**
- Add `--persistent` flag to CLI `register` command
- This just adds `persistent` to the `tags` vector — no struct changes needed
- Alternatively, it could be `--tag persistent` (already works today!)

**Change 2: Cleanup exemption (the real work)**
Three places need the tag check:

1. **`supervisor.rs` sweep** — before adding to `dead` list, check if `reg.tags.contains("persistent")`. If persistent and PID dead: emit `session.needs_restart` event instead of cleaning up.

2. **`manager.rs` clean_stale_sessions** — skip registrations tagged `persistent`

3. **`remote_store.rs` reap_expired** — if entry has `persistent` tag, skip TTL expiry (or set TTL to `Duration::MAX`)

Estimated effort: ~2 hours for a focused build task.

### 5. Task ID on TermLink Side

- **T-967** — "Persistent agent sessions — mark, protect, verify, discover" (inception, captured)
- **T-937** — "Cleanup kills active dispatch workers" (related bug — dispatch workers also suffer from this)
- **T-941** — "Include persistent agent session service templates in framework deploy scaffold"
- **T-959** — "Two-pool architecture (persistent /var/lib + ephemeral /tmp)"

### 6. Cost Model for Idle Persistent Sessions

**Local (Unix socket) sessions:**
- **Memory:** ~2KB for the Registration JSON on disk + whatever the session process itself uses (shell process: ~5MB RSS, Claude Code: ~150MB RSS)
- **CPU:** Zero when idle — Unix socket listener uses epoll/kqueue, no busy loop
- **Disk:** One `.json` + one `.sock` file per session (~2KB total)
- **File descriptors:** 1 socket listener FD per session
- **Hub overhead:** Supervisor sweep reads JSON files every 30s — O(n) with number of sessions, but n is small

**Remote (TCP) sessions:**
- **Memory:** ~500 bytes per `RemoteEntry` in hub's HashMap
- **Network:** Heartbeat every <5min to prevent TTL expiry — trivial bandwidth
- **Hub overhead:** Reaper scans HashMap every 30s — O(n), negligible

**Bottom line:** The cost of persistent sessions is dominated by the **process itself** (the Claude Code instance or custom agent), not by TermLink's session tracking. An idle listening session that just holds a Unix socket open is essentially free.

The real cost question is: **what runs INSIDE the persistent session?** If it's a Claude Code instance with prompt cache warming, that's ~150MB RAM per project. For 3-5 projects, that's <1GB — easily manageable on a dev machine.

## Counter-Proposals

### A. Use Tags, Not a New Flag

Rather than a `--persistent` flag that adds a boolean to the Registration struct, I propose using the existing `tags` mechanism:

```bash
# Register a persistent session (works TODAY for registration)
termlink register --name "fw-agent" --tags persistent,receptionist --roles agent

# Discover persistent sessions (works TODAY)
termlink list --tag persistent
termlink discover --tag persistent  # via hub
```

The only build work is making the cleanup code **check for the tag**. This is cleaner than a struct-level boolean because:
- No schema migration
- Composable with other tags (`persistent,priority,debug`)
- Existing discovery/filtering infrastructure handles it

### B. `session.needs_restart` Event Instead of Silent Skip

When the supervisor finds a persistent session with a dead PID, don't just skip cleanup — emit a `session.needs_restart` event:

```json
{
  "topic": "session.needs_restart",
  "payload": {
    "session_id": "tl-abc2defg",
    "display_name": "fw-agent",
    "tags": ["persistent", "receptionist"],
    "reason": "process_died",
    "last_heartbeat": "2026-04-12T10:30:00Z"
  }
}
```

This lets the framework's `/resume` flow react:
- If persistent agent is up: great, carry on
- If persistent agent is down: it receives `needs_restart` event (or discovers it via `session.discover` returning empty)
- Framework can auto-restart via systemd or `tl-claude.sh`

### C. Grace Period, Not Permanent Exemption

Instead of permanent cleanup exemption, add a **grace period** for persistent sessions:
- Normal sessions: cleaned immediately when PID dead
- Persistent sessions: keep registration for **5 minutes** after PID death (allow restart)
- After grace period: clean up even persistent sessions (truly dead)

This prevents persistent session registrations from accumulating as garbage if the restart mechanism fails.

### D. Remote Session TTL Override

For remote persistent sessions, instead of infinite TTL:
- Set TTL to **30 minutes** (vs. default 5 minutes)
- Heartbeat interval stays at <5 minutes
- This gives 6x the tolerance for network hiccups without risking permanent zombie entries

## Recommendation

**GO** — The implementation is minimal and the infrastructure exists. Propose two build tasks:

1. **T-968 (build):** Add `persistent` tag exemption to supervisor sweep, clean_stale_sessions, and remote store reaper. Add `session.needs_restart` event topic. ~2h.

2. **T-969 (build):** Add `/health` or `session.ping` endpoint for persistent session verification. Framework calls this during `/resume` to check agent availability. ~1h.

The framework side (T-1135) can then:
- Check `termlink discover --tag persistent` during `/resume`
- React to `session.needs_restart` events
- Report persistent agent health in `fw doctor`
