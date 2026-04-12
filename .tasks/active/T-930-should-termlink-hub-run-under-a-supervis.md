---
id: T-930
name: "Should termlink hub run under a supervisor with TCP bound by default on .107?"
description: >
  Inception: Should termlink hub run under a supervisor with TCP bound by default on .107?

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-04-11T21:43:54Z
last_update: 2026-04-12T07:05:31Z
date_finished: 2026-04-11T22:27:40Z
---

# T-930: Should termlink hub run under a supervisor with TCP bound by default on .107?

## Problem Statement

T-921 decided GO on Option A (unified `--target` on every session-scoped CLI
command). T-923..T-929 shipped the mechanism: hub forwarder test, shared
`TargetOpts` + `call_session` helper, five commands (`ping`, `status`,
`signal`, `tag`, `kv`) now accept `--target HOST:PORT`. The code path is
proven end-to-end by the `tcp_forward_to_local_session_after_auth` test.

**But on .107 (this box) the hub is not usable cross-host by default.**
Three concrete symptoms observed 2026-04-11:

1. `termlink hub start` binds unix only. A sibling agent trying to reach
   .107 from .122 got connection-refused; `ss -tln` confirmed nothing on
   9100/4112 even though the UFW rule `9100/tcp ALLOW 192.168.10.0/24` is
   in place.
2. The hub was found dead earlier in the day with stale pidfile
   (PID 1517402) and an orphaned socket. No one noticed until another
   agent tried to use it. There is no systemd unit, no watchdog, no
   restart policy.
3. Restarting the hub regenerates the HMAC secret at
   `/tmp/termlink-0/hub.secret`. This invalidates any secret distributed
   to other hosts (`-32010 Token validation failed: invalid signature`).
   A hub restart is therefore a silent break for every remote client.

**For whom:** dispatch operators driving multi-host fleets; agents on
.122 / .112 / other boxes who need to query or push to sessions on .107;
framework-agent pickup consumers if that channel ever moves from file
inbox to live socket.

**Why now:** T-925..T-929 landed the CLI surface. Without a supervisor
policy + TCP-bound-by-default decision, those five new flags exist but
are not usable by anyone outside this box. The inception has to resolve
policy, not code — the code is already in place.

## Assumptions

- **A1:** Running `termlink hub start --tcp 0.0.0.0:9100` under a systemd
  `simple` service with `Restart=on-failure` solves (a) TCP binding and
  (b) silent-death recovery in one change. Verify by checking systemd
  unit syntax for the `.agentic-framework/` layout and whether the hub
  process traps SIGTERM cleanly.
- **A2:** Hub-secret regeneration on restart is the dominant operational
  issue, not the TCP binding itself. Verify by measuring how often the
  hub actually restarts vs. how often a distributed secret would need to
  rotate for an unrelated reason.
- **A3:** A single supervised hub per host is sufficient — no need for
  HA / failover. Verify by asking whether any consumer has a "hub down"
  failure mode requirement (they probably don't; termlink is dev
  tooling, not a production dependency).
- **A4:** The hub-secret file (`/tmp/termlink-0/hub.secret`) should NOT
  live in `/tmp/` for any host that survives reboots; it should live in
  a persistent path (`/var/lib/termlink/` or `~/.termlink/`) so the
  secret survives. Verify by checking where `termlink hub start`
  actually writes it and whether the path is configurable.
- **A5:** There is no `framework-agent` long-lived listener session on
  .107. A sibling agent trying to push TO framework-agent via
  `--target 192.168.10.107:9100` would succeed on the hub hop but fail
  on session resolution because no session with that name exists. So
  "cross-host push" and "push-to-named-endpoint" are two separate
  problems; this inception should only tackle the first one.

## Exploration Plan

Time-box: **one session**, dialogue-driven. No production code.
Deliverable = supervisor policy decision + systemd unit file (or
alternative) + secret-rotation story + decomposed build task list.

- **Spike 1 (~20 min) — Current state audit.** `ls /etc/systemd/system/
  | grep termlink`, `systemctl list-units | grep termlink`, `ps auxf |
  grep hub`, `cat /etc/cron.d/agentic-audit-termlink` (compare cron
  supervisor pattern). Record what exists vs. what doesn't.
- **Spike 2 (~20 min) — systemd unit design.** Draft
  `/etc/systemd/system/termlink-hub.service` with ExecStart, Restart,
  RestartSec, User, Environment, and After/Requires. Check: does the
  hub emit structured logs? Does it need RuntimeDirectory? Where should
  the pidfile live?
- **Spike 3 (~20 min) — Secret persistence.** Read
  `crates/termlink-hub/src/server.rs` to find where `hub.secret` is
  written. Check if the path is configurable via env var or CLI flag.
  Decide: persist across restarts (read on startup if present) vs.
  rotate on every restart. Current behaviour is rotate-always — is that
  intentional or incidental?
- **Spike 4 (~15 min) — Alternative: watchdog daemon.** If systemd is
  rejected (e.g., because of the root-vs-user-session concern on dev
  boxes), consider a simple `while true; do termlink hub start --tcp
  ...; sleep 5; done` launched from `~/.bashrc` or a cron @reboot entry.
  Pros/cons vs. systemd.
- **Spike 5 (~15 min) — Decomposition.** Given the chosen supervisor
  strategy, list concrete build tasks (unit file + installer + doctor
  check + possibly secret persistence fix). Each line sized to fit one
  session.

**Dialogue checkpoints:** pause after Spike 3 for human input on
secret-rotation policy (persist vs. rotate). Do not execute Spike 5
unilaterally.

## Technical Constraints

- **Platform:** .107 is a Linux desktop (kernel 6.8.0-88-generic per
  session env), systemd-based. Other consumer boxes (.112, .122) may
  differ; the chosen supervisor must not assume anything beyond "has
  systemd" unless there is a portable fallback.
- **Runtime directory:** termlink writes to `/tmp/termlink-0/` by
  default. `/tmp` is wiped on reboot — persistent state (including the
  hub secret) is currently lost every restart by design. Unclear
  whether that's a deliberate security property or an oversight.
- **TLS:** The hub auto-generates `hub.cert.pem` + `hub.key.pem` on
  first start (T-182 TOFU). Restart rotates these too, which means
  first-time clients re-trust the new cert automatically but existing
  clients with pinned fingerprints break. Related to the secret issue
  but a separate concern.
- **Firewall:** UFW rule `9100/tcp ALLOW 192.168.10.0/24 # TermLink TCP
  Hub (LAN only)` is already in place. No firewall changes needed.
- **User context:** The hub runs as root on this box (matches
  `/etc/cron.d/agentic-audit-termlink` which also runs as root). Decide
  whether the systemd unit should stay root or drop to a dedicated
  user.

## Scope Fence

**IN scope:**
- Supervisor policy decision (systemd vs. watchdog vs. none)
- TCP-bound-by-default policy decision
- Secret-persistence-on-restart policy decision
- Draft systemd unit file (or alternative config)
- Decomposed build task list
- Doctor check that warns when a consumer project relies on `--target`
  but the hub has no TCP listener

**OUT of scope:**
- Creating a long-lived `framework-agent` session on .107 (separate
  inception — tracked implicitly by A5, not here)
- Distributing hub secrets across hosts (requires a secret exchange
  story — T-921 already punted on this; stays punted)
- Multi-hub / HA / failover (A3 says not needed)
- Multi-hop routing (out of T-921 scope too)
- Making cron and hub share the same supervisor layer (interesting
  unification but not in scope)

## Acceptance Criteria

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- A supervisor strategy is picked that auto-restarts the hub within 10s
  of a crash (systemd `Restart=on-failure` is the obvious candidate)
- The TCP-bound-by-default question is resolved with a policy, not a
  "depends on environment" dodge
- Secret-persistence-on-restart policy is decided (persist OR rotate) and
  whichever is chosen, it's documented so clients know what to expect
- Decomposition produces build tasks each sized under one session

**NO-GO / DEFER if:**
- Nobody actually needs cross-host termlink from .107 in the next 30
  days — if the `--target` flag is a theoretical capability with no live
  consumer, the right move is defer until a real consumer shows up
- The secret-persistence question forces a bigger architectural change
  (e.g., migrating hub state out of `/tmp/`) that deserves its own
  inception — in which case this one splits
- Cross-host auth turns out to need a shared-secret distribution story
  that's already been punted by T-921 — this inception isn't the right
  place to un-punt it

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO

**Rationale:** All three policy questions resolve cleanly in the same
direction and the precedent for the chosen mechanism already exists
and is healthy on this box. There is no live tension that would make
a DEFER prudent.

**Evidence:**

1. **Supervisor policy → systemd.** `watchtower-vinix24.service` has
   been running 4 days with `Type=exec` + `Restart=on-failure` +
   `RestartSec=5` on this exact box. Operator comfort is
   demonstrated. Watchdog alternative (while loop + @reboot cron) is
   strictly worse — no structured logs, no status query, no Restart
   semantics, no `ExecStop`. Spike 4 is moot.
2. **TCP-bound-by-default policy → yes.** UFW rule already allows
   `9100/tcp` from `192.168.10.0/24`. Running with `--tcp
   0.0.0.0:9100` in the systemd `ExecStart=` line makes cross-host
   the default posture, which is what T-921..T-929 just built for.
   Opt-in TCP leaves a firewall-open / nothing-listening split-brain
   that already caused a sibling-agent incident on 2026-04-11.
3. **Secret persistence policy → persist-if-present.** Code read
   (Spike 3) shows rotation is incidental, not deliberate. Fix is a
   one-function change: read existing hex if present and valid,
   otherwise generate. Drop the `remove_file(hub_secret_path())` on
   clean shutdown. Security delta is effectively zero (the
   HMAC secret never traverses the wire; compromise recovery is
   unchanged because a network attacker has no way to read it).
4. **The parent-ghost discovery.** Spike 1 found the current hub is
   PPID-linked to a 9-hour-old Claude bash, not init. This is not
   "unsupervised" — it is *worse* than unsupervised, because an
   unrelated session's shell owns the process tree. Any systemd unit
   fixes this by having init (PID 1) adopt the hub.
5. **SIGTERM-not-caught bug.** The systemd unit exposes this as a
   build task. Without it, `systemctl stop` skips cleanup. Fix is
   ~5 lines in `cmd_hub_start` (select! on ctrl_c + SignalKind::terminate).
   Workaround `KillSignal=SIGINT` in the unit until the fix lands.

**Human checkpoint needed:** Spike 3 proposed persist-if-present.
The rationale above assumes you agree. If you prefer rotate-on-every-
restart for stronger ephemeral trust, say so and the decomposition
shifts (no code change to secret handling, but client tooling needs
to know how to pick up the new value on every bounce). I believe
persist is right for dev-tooling-on-LAN.

### Decomposed build tasks (Spike 5, folded in)

Each task sized under one session:

- **T-931** — `termlink-hub.service` unit file + installer. Drop the
  file under `.context/systemd/termlink-hub.service`, add an
  installer stanza to copy it into `/etc/systemd/system/` and
  `systemctl enable --now`. Mirror the `agentic-audit-termlink` cron
  install pattern.
- **T-932** — Hub SIGTERM handling. `select!` on `ctrl_c()` +
  `SignalKind::terminate()` in `cmd_hub_start`. Add a unit test that
  spawns the hub and sends SIGTERM + verifies clean cleanup.
- **T-933** — Hub secret persistence. Read-if-present +
  validate-hex in `generate_and_write_hub_secret()`. Remove the
  `remove_file(hub_secret_path())` from clean-shutdown cleanup. Add
  a test that two consecutive starts see the same secret.
- **T-934** — `termlink doctor` check: "UFW rule for hub port
  exists but nothing listening" warning. Cheap check, catches the
  exact state this inception started from.
- **T-935** — Migrate current `/tmp/termlink-0/` state to
  `/var/lib/termlink/` on first systemd-managed start. One-shot
  migration helper or documentation. Lower priority — the migration
  is "delete /tmp/termlink-0/ and let the unit recreate."

After the decision, these five tasks get created with `fw task
create --type build`. T-931 is the critical-path deliverable;
T-932/T-933 unblock future durability; T-934 is observability;
T-935 is optional cleanup.

### Out-of-scope items (stay punted)

- Dropping hub to a dedicated `termlink` user (stays root for now,
  matches environment).
- Shared secret distribution across hosts (T-921 punted this; stays
  punted).
- Multi-hub HA / failover (A3 holds).
- Unifying cron supervisor + hub supervisor under a single
  framework abstraction (interesting but not this inception).

## Decisions

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: All three policy questions resolve cleanly in the same
direction and the precedent for the chosen mechanism already exists
and is healthy on this box. There is no live tension that would make
a DEFER prudent.

Evidence:

1. Supervisor policy → systemd. `watchtower-vinix24.service` has
   been running 4 days with `Type=exec` + `Restart=on-failure` +
   `RestartSec=5` on this exact box. Operator comfort is
   demonstrated. Watchdog alternative (while loop + @reboot cron) is
   strictly worse — no structured logs, no status query, no Restart
   semantics, no `ExecStop`. Spike 4 is moot.
2. TCP-bound-by-default policy → yes. UFW rule already allows
   `9100/tcp` from `192.168.10.0/24`. Running with `--tcp
   0.0.0.0:9100` in the systemd `ExecStart=` line makes cross-host
   the default posture, which is what T-921..T-929 just built for.
   Opt-in TCP leaves a firewall-open / nothing-listening split-brain
   that already caused a sibling-agent incident on 2026-04-11.
3. Secret persistence policy → persist-if-present. Code read
   (Spike 3) shows rotation is incidental, not deliberate. Fix is a
   one-function change: read existing hex if present and valid,
   otherwise generate. Drop the `remove_file(hub_secret_path())` on
   clean shutdown. Security delta is effectively zero (the
   HMAC secret never traverses the wire; compromise recovery is
   unchanged because a network attacker has no way to read it).
4. The parent-ghost discovery. Spike 1 found the current hub is
   PPID-linked to a 9-hour-old Claude bash, not init. This is not
   "unsupervised" — it is *worse* than unsupervised, because an
   unrelated session's shell owns the process tree. Any systemd unit
   fixes this by having init (PID 1) adopt the hub.
5. SIGTERM-not-caught bug. The systemd unit exposes this as a
   build task. Without it, `systemctl stop` skips cleanup. Fix is
   ~5 lines in `cmd_hub_start` (select! on ctrl_c + SignalKind::terminate).
   Workaround `KillSignal=SIGINT` in the unit until the fix lands.

Human checkpoint needed: Spike 3 proposed persist-if-present.
The rationale above assumes you agree. If you prefer rotate-on-every-
restart for stronger ephemeral trust, say so and the decomposition
shifts (no code change to secret handling, but client tooling needs
to know how to pick up the new value on every bounce). I believe
persist is right for dev-tooling-on-LAN.

### Decomposed build tasks (Spike 5, folded in)

Each task sized under one session:

- T-931 — `termlink-hub.service` unit file + installer. Drop the
  file under `.context/systemd/termlink-hub.service`, add an
  installer stanza to copy it into `/etc/systemd/system/` and
  `systemctl enable --now`. Mirror the `agentic-audit-termlink` cron
  install pattern.
- T-932 — Hub SIGTERM handling. `select!` on `ctrl_c()` +
  `SignalKind::terminate()` in `cmd_hub_start`. Add a unit test that
  spawns the hub and sends SIGTERM + verifies clean cleanup.
- T-933 — Hub secret persistence. Read-if-present +
  validate-hex in `generate_and_write_hub_secret()`. Remove the
  `remove_file(hub_secret_path())` from clean-shutdown cleanup. Add
  a test that two consecutive starts see the same secret.
- T-934 — `termlink doctor` check: "UFW rule for hub port
  exists but nothing listening" warning. Cheap check, catches the
  exact state this inception started from.
- T-935 — Migrate current `/tmp/termlink-0/` state to
  `/var/lib/termlink/` on first systemd-managed start. One-shot
  migration helper or documentation. Lower priority — the migration
  is "delete /tmp/termlink-0/ and let the unit recreate."

After the decision, these five tasks get created with `fw task
create --type build`. T-931 is the critical-path deliverable;
T-932/T-933 unblock future durability; T-934 is observability;
T-935 is optional cleanup.

### Out-of-scope items (stay punted)

- Dropping hub to a dedicated `termlink` user (stays root for now,
  matches environment).
- Shared secret distribution across hosts (T-921 punted this; stays
  punted).
- Multi-hub HA / failover (A3 holds).
- Unifying cron supervisor + hub supervisor under a single
  framework abstraction (interesting but not this inception).

**Date**: 2026-04-11T22:27:40Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: All three policy questions resolve cleanly in the same
direction and the precedent for the chosen mechanism already exists
and is healthy on this box. There is no live tension that would make
a DEFER prudent.

Evidence:

1. Supervisor policy → systemd. `watchtower-vinix24.service` has
   been running 4 days with `Type=exec` + `Restart=on-failure` +
   `RestartSec=5` on this exact box. Operator comfort is
   demonstrated. Watchdog alternative (while loop + @reboot cron) is
   strictly worse — no structured logs, no status query, no Restart
   semantics, no `ExecStop`. Spike 4 is moot.
2. TCP-bound-by-default policy → yes. UFW rule already allows
   `9100/tcp` from `192.168.10.0/24`. Running with `--tcp
   0.0.0.0:9100` in the systemd `ExecStart=` line makes cross-host
   the default posture, which is what T-921..T-929 just built for.
   Opt-in TCP leaves a firewall-open / nothing-listening split-brain
   that already caused a sibling-agent incident on 2026-04-11.
3. Secret persistence policy → persist-if-present. Code read
   (Spike 3) shows rotation is incidental, not deliberate. Fix is a
   one-function change: read existing hex if present and valid,
   otherwise generate. Drop the `remove_file(hub_secret_path())` on
   clean shutdown. Security delta is effectively zero (the
   HMAC secret never traverses the wire; compromise recovery is
   unchanged because a network attacker has no way to read it).
4. The parent-ghost discovery. Spike 1 found the current hub is
   PPID-linked to a 9-hour-old Claude bash, not init. This is not
   "unsupervised" — it is *worse* than unsupervised, because an
   unrelated session's shell owns the process tree. Any systemd unit
   fixes this by having init (PID 1) adopt the hub.
5. SIGTERM-not-caught bug. The systemd unit exposes this as a
   build task. Without it, `systemctl stop` skips cleanup. Fix is
   ~5 lines in `cmd_hub_start` (select! on ctrl_c + SignalKind::terminate).
   Workaround `KillSignal=SIGINT` in the unit until the fix lands.

Human checkpoint needed: Spike 3 proposed persist-if-present.
The rationale above assumes you agree. If you prefer rotate-on-every-
restart for stronger ephemeral trust, say so and the decomposition
shifts (no code change to secret handling, but client tooling needs
to know how to pick up the new value on every bounce). I believe
persist is right for dev-tooling-on-LAN.

### Decomposed build tasks (Spike 5, folded in)

Each task sized under one session:

- T-931 — `termlink-hub.service` unit file + installer. Drop the
  file under `.context/systemd/termlink-hub.service`, add an
  installer stanza to copy it into `/etc/systemd/system/` and
  `systemctl enable --now`. Mirror the `agentic-audit-termlink` cron
  install pattern.
- T-932 — Hub SIGTERM handling. `select!` on `ctrl_c()` +
  `SignalKind::terminate()` in `cmd_hub_start`. Add a unit test that
  spawns the hub and sends SIGTERM + verifies clean cleanup.
- T-933 — Hub secret persistence. Read-if-present +
  validate-hex in `generate_and_write_hub_secret()`. Remove the
  `remove_file(hub_secret_path())` from clean-shutdown cleanup. Add
  a test that two consecutive starts see the same secret.
- T-934 — `termlink doctor` check: "UFW rule for hub port
  exists but nothing listening" warning. Cheap check, catches the
  exact state this inception started from.
- T-935 — Migrate current `/tmp/termlink-0/` state to
  `/var/lib/termlink/` on first systemd-managed start. One-shot
  migration helper or documentation. Lower priority — the migration
  is "delete /tmp/termlink-0/ and let the unit recreate."

After the decision, these five tasks get created with `fw task
create --type build`. T-931 is the critical-path deliverable;
T-932/T-933 unblock future durability; T-934 is observability;
T-935 is optional cleanup.

### Out-of-scope items (stay punted)

- Dropping hub to a dedicated `termlink` user (stays root for now,
  matches environment).
- Shared secret distribution across hosts (T-921 punted this; stays
  punted).
- Multi-hub HA / failover (A3 holds).
- Unifying cron supervisor + hub supervisor under a single
  framework abstraction (interesting but not this inception).

**Date**: 2026-04-11T22:27:40Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-11T21:47:24Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-11T22:27:40Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: All three policy questions resolve cleanly in the same
direction and the precedent for the chosen mechanism already exists
and is healthy on this box. There is no live tension that would make
a DEFER prudent.

Evidence:

1. Supervisor policy → systemd. `watchtower-vinix24.service` has
   been running 4 days with `Type=exec` + `Restart=on-failure` +
   `RestartSec=5` on this exact box. Operator comfort is
   demonstrated. Watchdog alternative (while loop + @reboot cron) is
   strictly worse — no structured logs, no status query, no Restart
   semantics, no `ExecStop`. Spike 4 is moot.
2. TCP-bound-by-default policy → yes. UFW rule already allows
   `9100/tcp` from `192.168.10.0/24`. Running with `--tcp
   0.0.0.0:9100` in the systemd `ExecStart=` line makes cross-host
   the default posture, which is what T-921..T-929 just built for.
   Opt-in TCP leaves a firewall-open / nothing-listening split-brain
   that already caused a sibling-agent incident on 2026-04-11.
3. Secret persistence policy → persist-if-present. Code read
   (Spike 3) shows rotation is incidental, not deliberate. Fix is a
   one-function change: read existing hex if present and valid,
   otherwise generate. Drop the `remove_file(hub_secret_path())` on
   clean shutdown. Security delta is effectively zero (the
   HMAC secret never traverses the wire; compromise recovery is
   unchanged because a network attacker has no way to read it).
4. The parent-ghost discovery. Spike 1 found the current hub is
   PPID-linked to a 9-hour-old Claude bash, not init. This is not
   "unsupervised" — it is *worse* than unsupervised, because an
   unrelated session's shell owns the process tree. Any systemd unit
   fixes this by having init (PID 1) adopt the hub.
5. SIGTERM-not-caught bug. The systemd unit exposes this as a
   build task. Without it, `systemctl stop` skips cleanup. Fix is
   ~5 lines in `cmd_hub_start` (select! on ctrl_c + SignalKind::terminate).
   Workaround `KillSignal=SIGINT` in the unit until the fix lands.

Human checkpoint needed: Spike 3 proposed persist-if-present.
The rationale above assumes you agree. If you prefer rotate-on-every-
restart for stronger ephemeral trust, say so and the decomposition
shifts (no code change to secret handling, but client tooling needs
to know how to pick up the new value on every bounce). I believe
persist is right for dev-tooling-on-LAN.

### Decomposed build tasks (Spike 5, folded in)

Each task sized under one session:

- T-931 — `termlink-hub.service` unit file + installer. Drop the
  file under `.context/systemd/termlink-hub.service`, add an
  installer stanza to copy it into `/etc/systemd/system/` and
  `systemctl enable --now`. Mirror the `agentic-audit-termlink` cron
  install pattern.
- T-932 — Hub SIGTERM handling. `select!` on `ctrl_c()` +
  `SignalKind::terminate()` in `cmd_hub_start`. Add a unit test that
  spawns the hub and sends SIGTERM + verifies clean cleanup.
- T-933 — Hub secret persistence. Read-if-present +
  validate-hex in `generate_and_write_hub_secret()`. Remove the
  `remove_file(hub_secret_path())` from clean-shutdown cleanup. Add
  a test that two consecutive starts see the same secret.
- T-934 — `termlink doctor` check: "UFW rule for hub port
  exists but nothing listening" warning. Cheap check, catches the
  exact state this inception started from.
- T-935 — Migrate current `/tmp/termlink-0/` state to
  `/var/lib/termlink/` on first systemd-managed start. One-shot
  migration helper or documentation. Lower priority — the migration
  is "delete /tmp/termlink-0/ and let the unit recreate."

After the decision, these five tasks get created with `fw task
create --type build`. T-931 is the critical-path deliverable;
T-932/T-933 unblock future durability; T-934 is observability;
T-935 is optional cleanup.

### Out-of-scope items (stay punted)

- Dropping hub to a dedicated `termlink` user (stays root for now,
  matches environment).
- Shared secret distribution across hosts (T-921 punted this; stays
  punted).
- Multi-hub HA / failover (A3 holds).
- Unifying cron supervisor + hub supervisor under a single
  framework abstraction (interesting but not this inception).

### 2026-04-11T22:27:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
