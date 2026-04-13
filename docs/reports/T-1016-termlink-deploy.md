# T-1016: termlink deploy — research artifact

## Problem

Deploying a new termlink binary to remote hosts currently requires:
1. Building a musl static binary locally
2. Using `termlink send-file` to transfer it
3. Remotely stopping the hub / killing processes
4. Replacing the binary (race with running sessions)
5. Restarting the hub
6. Verifying connectivity

This process has multiple failure modes observed in T-1023 and T-1027:
- **Hub connectivity chicken-and-egg:** Killing the hub via `termlink remote exec "kill PID"` severs the very connection used to control the deployment
- **TOFU breakage:** Hub restart generates new TLS cert (pre-T-1028), invalidating all client trust
- **Secret rotation:** Hub restart could rotate secrets (pre-T-933), breaking auth
- **Binary-busy:** Can't overwrite a running binary (`Text file busy`); must use rename-then-replace
- **Partial deployment:** If any step fails, the remote host can be left in an inconsistent state
- **No SSH fallback:** When hubs are down, there's no way to reach the host at all

**Real-world incidents:**
- T-1023: .121 hub restart killed termlink connectivity permanently; agent on .121 had to self-recover
- T-1027: Both .109 and .121 unreachable — .109 hub down, .121 secret mismatch after cert regen

## Investigation

### Spike 1: Current deployment flow analysis

The manual deployment flow from T-1023:

```
# On .107 (source):
cargo build --release --target x86_64-unknown-linux-musl
termlink send-file ring20-management /tmp/deploy/termlink ./target/.../termlink

# On .109 (target, via termlink):
termlink remote exec ring20-management "mv /root/.cargo/bin/termlink /root/.cargo/bin/termlink.old"
termlink remote exec ring20-management "mv /tmp/deploy/termlink /root/.cargo/bin/termlink"
termlink remote exec ring20-management "chmod +x /root/.cargo/bin/termlink"
# Can't restart hub via termlink — kills the connection
```

**Problem:** The "restart hub" step is destructive to the deployment channel.

### Spike 2: What a `termlink deploy` command needs

Minimum viable:
1. **Transfer binary** — send-file or similar (already works)
2. **Atomic swap** — rename old, install new, chmod (needs to be a single remote exec)
3. **Hub restart** — must be self-initiated by the target, not remote-triggered
4. **Verification** — ping after restart to confirm success

The key insight: the target host must orchestrate its own restart. The deployer can only trigger it and then wait.

### Spike 3: Self-restart mechanism

Options:
- **A: fork-exec restart** — Target receives the new binary, spawns a "restarter" script that: waits for deployer to disconnect, stops hub, swaps binary, starts hub. This is what `hub restart` does internally.
- **B: systemd restart** — If hub runs under systemd, the target just needs `systemctl restart termlink-hub`. Binary swap happens while hub is stopped. Systemd auto-restarts on failure.
- **C: Two-phase deployment** — Phase 1: transfer + stage. Phase 2: human or cron triggers the actual swap. Safe but slow.

**Option B is the clear winner for supervised hubs.** The hub binary is replaced while the systemd service is stopped. The restart is clean, atomic, and the deployer just waits for the hub to come back and re-authenticates.

### Spike 4: Command design

```
termlink deploy <hub-profile> [--binary <path>] [--restart-method systemd|manual]
```

Steps:
1. Connect to target hub
2. Send binary to staging location (e.g., `/tmp/termlink-deploy/termlink`)
3. Execute atomic swap: `mv old binary.old && mv staged binary && chmod +x`
4. Trigger restart:
   - systemd: `systemctl restart termlink-hub` (via remote exec)
   - manual: print instructions, wait for user
5. Wait for hub to come back (poll with timeout)
6. Re-authenticate and verify version
7. Report success/failure

### Spike 5: Prerequisites and constraints

- **Target must have a running hub** — otherwise can't deploy (no channel)
- **Auth must be valid** — secret must match
- **Binary must be compatible** — musl static or matching libc
- **systemd unit must exist** (for automatic restart)
- **After T-1028:** TLS cert persists across restart — TOFU stays valid
- **After T-933:** Hub secret persists — auth stays valid
- **After T-1029:** Local TOFU fallback works when runtime dirs differ

With T-1028 and T-933, the restart-breaks-auth problem is solved. The main remaining challenge is the binary swap itself.

## Findings

1. **The core problem is restart coordination**, not transfer. Binary transfer via send-file works fine.
2. **systemd-managed hubs make this straightforward** — `systemctl restart` handles the lifecycle atomically
3. **T-1028 + T-933 eliminate the TOFU/auth breakage** — restarts preserve certs and secrets
4. **The command is ~100 lines** — connect, transfer, swap, restart, verify
5. **Edge case: hub not running** — deploy is impossible without a running hub. This is the bootstrap problem (first deploy still needs SSH or physical access)
6. **Edge case: hub running old binary** — old binary may not support all RPC methods. Need graceful degradation.

## Dialogue Log

- Agent analyzed deployment failures from T-1023 and T-1027
- Key insight: hub restart is the hard part, binary swap is easy
- systemd restart solves the lifecycle coordination cleanly
- T-1028/T-933 solve the auth/TOFU breakage that previously made restart destructive
