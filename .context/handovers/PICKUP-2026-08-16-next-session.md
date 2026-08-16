# PICKUP — next session (written 2026-08-16, end of S-2026-0816-1621)

Read this before `LATEST.md`. It is shorter and it says what to do.

> ## BUILD ORDER — UPDATED 2026-08-16 evening. Items 1 and 2 are DONE.
>
> The earlier version of this block said the session was "analysis-rich and build-poor".
> That is no longer true. A later session on the same day built items 1 and 2, plus one
> task that did not exist when the list was written.
>
> **SHIPPED (all committed, all with `cargo test --workspace` green):**
>
> - **T-2772** `5f862c048` — the hub now tells a uid-refused Unix peer WHY, instead of
>   dropping the stream and leaving a bare `ECONNRESET`. Found because a peer agent
>   misdiagnosed that reset three times in a row. Live-proven.
> - **T-2770** `ef8fea422` — **item 1, done.** `socket_has_listener` no longer reads
>   `EACCES` as "no listener". Proven load-bearing BOTH ways: reverted, a rival hub
>   starts and hijacks the socket (owner flips `root:root` → `dimitri-mint-dev`) before
>   crashing; restored, it refuses and explains.
> - **T-2771 IW-2 / IW-3** — **item 2, done, and the answer changes the plan.** IW-2 is
>   DISSOLVED: `SO_PEERCRED` was never a portability risk, because
>   `PeerCredentials::from_raw_fd` already branches Linux / macOS / Unsupported and the
>   hub already calls it. IW-3 is ANSWERED: a same-uid peer is granted
>   `PermissionScope::Execute` unconditionally — uid is a BOUNDARY, not an identity. So
>   **T-2771 and T-2769 do NOT merge**; design them separately.
>
> **REMAINING, in order:**
>
> 1. **T-2773** *(new)* — `termlink-session`'s accept loop fails **OPEN** when peer-credential
>    extraction errors (`server.rs:239-242`), where the hub fails **closed** (T-2448). Same
>    gate, opposite posture; the hardening was never migrated to the sibling. It also has
>    the identical silent-drop T-2772 just fixed. **Start here** — bounded, agent-ownable,
>    and the fix shape is already written in `hub/src/server.rs`.
> 2. **T-2696** — wire the two unexecuted charter-verb provers. Independent of everything.
> 3. **T-2774** *(new)* — `channel_subscribe_no_hang_under_concurrent_walks_t2258` bounds its
>    walk phase at a wall-clock 10s and fails intermittently under the parallel harness
>    (observed failing a P-011 gate, passing on retry and in two full workspace runs). A
>    verification gate that fails randomly trains agents to reach for `--skip-verification`.
> 4. **T-2769** — still blocked on its own IW-2: enforcing an authenticated `claimer` would
>    reject every live caller, which is a human cutover decision. **New evidence against
>    rushing it** — T-2771 IW-6: any co-uid peer already holds `Execute`, so it can simply
>    force-release another agent's claim. On a shared-uid host an authenticated `claimer`
>    is defeated anyway. Fix the uid model first, or accept that this only helps cross-host.
>
> **Do not open another review axis before these are built.** That instruction stands and
> was honoured: everything above came from building, or from reading code while building.

Branch: `worktree-charter-review-2026-0814`, worktree
`/opt/termlink/.claude/worktrees/charter-review-2026-0814`.
Everything is committed. **8 commits are unpushed** — see "Do not sink time here" §2.

---

## 0. HIGHEST PRIORITY — local IPC is uid-coupled, and my T-2767 fix does not cover it

Analysis from the AEF agent (`/opt/999-Agentic-Engineering-Framework`), independently
arrived at, and it is correct. TermLink has **two authorization models**:

| transport | who is allowed | basis |
|---|---|---|
| remote TCP :9100 | anyone with the fleet secret | HMAC — identity-based, user-agnostic |
| local unix socket | whoever owns the socket file | POSIX file mode — **uid-based** |

A unix domain socket carries filesystem permissions, so "can this agent talk to the hub"
is decided by whichever uid started it and its umask — never by TermLink's own auth. The
unit runs `User=root` with `StateDirectoryMode=0700`; the socket lands `root:root 0755`.

**The inversion:** a remote host on another machine can authenticate in, while a local
agent on the same box cannot. And the consequence is not a nuisance — an agent that
cannot reach the existing hub **silently starts its own**. That is why there are three
on this host (root's, dimitri-mint-dev's under `/tmp/termlink`, systemd's). Nobody
misconfigured anything: **fragmentation is the default outcome** the moment two agent
runtimes run as different users, which is now the normal case (Claude Code + Codex
side by side). Silent fragmentation is a Directive #2 violation by design, not by bug.

### My T-2767 fix is BLIND to exactly this path — fix it first

`crates/termlink-hub/src/pidfile.rs::socket_has_listener` does:

```rust
if !socket.exists() { return false; }
std::os::unix::net::UnixStream::connect(socket).is_ok()
```

A non-root agent probing a `root:root 0755` socket gets **EACCES**, so `is_ok()` is
false, so the guard reports "no listener" and **permits the second hub to start**. The
guard I shipped today to prevent split-brain does not fire in the one scenario that
actually produces it — it only catches the case where the prober could have connected
anyway. It is not wrong, it is incomplete, and the incompleteness is the important half.

**Fix:** branch on `io::ErrorKind`.
- `ConnectionRefused` → leftover socket file, nothing behind it → start (the
  unclean-shutdown case the pidfile check existed for; keep it working).
- `PermissionDenied` → **a socket exists that this uid may not probe.** That is positive
  evidence of another user's hub — REFUSE, and say so, naming the uid mismatch. Do not
  treat "I can't look" as "nothing there".
- Anything else → refuse conservatively and name the error.

Regression tests should pin the EACCES case specifically (chmod the fixture socket 0700
and probe as a different uid, or inject the error kind). The existing
`stale_socket_file_with_no_listener_still_starts` already pins the refused case.

### Then the real fix — one auth model

Options, as the AEF agent framed them, with my read:

1. `UMask=0002` + shared `termlink` group + `StateDirectoryMode=0770` in the unit.
   Smallest, and it unblocks today — but it is **deployment config, not a TermLink
   fix**: every host that does not apply it still fragments, so the class survives.
2. **Authorize local clients via `SO_PEERCRED` in the hub** so local and remote share one
   model. Structurally right and the one I would build. Portability caveat (Directive
   #4): `SO_PEERCRED` is Linux; macOS needs `LOCAL_PEERCRED`/`getpeereid`, and README
   claims macOS as first-class — so this needs the T-2693 platform-lock treatment or it
   becomes a new lock-in.
3. Loopback TCP + the same HMAC, dropping the unix socket. Genuinely one path, but it
   gives up the local fast path and opens a port.

Sequence: **(0) EACCES fix — containment, stops silent fragmentation now. (2) is the
cure.** (1) is a legitimate stopgap for this host but must not be recorded as the fix.

### Operator action (agent cannot do this, and should not)

```
sudo chgrp dimitri-mint-dev /var/lib/termlink/hub.sock && sudo chmod 0770 /var/lib/termlink/hub.sock
```

The AEF agent was blocked from running it by a permission classifier, and stopped rather
than looking for a way around it — correct call, worth preserving as the norm.

### Coordination note

I could not reach the AEF agent to work this jointly: they are not on `agent-presence`,
and `.107`'s hub is unreachable from this session. **Two agents on one machine could not
use the agent-to-agent comms tool to discuss the bug in agent-to-agent comms.** That is
the finding demonstrating itself, and it is worth citing when this is filed.

---

## Start here

### 1. Restart the MCP server onto a current binary (T-2707) — 5 minutes, unblocks 2

The MCP server serves **0.11.720** against a **0.11.1440** tree. It is stale enough
that any MCP-surface finding is unattributable until it is restarted. Do this first
because item 2 depends on it.

Note the deleted-exe symptom from PL-209: `/proc/<pid>/exe` showing `(deleted)` means
a new binary was installed but the process still serves the old one.

### 2. File (or close) the `remote_call` defect — now corroborated

`termlink_remote_call` returns `-32001 Missing 'target' in params` for **every** method
tried, including `termlink.ping`, which takes no parameters at all. Params as object and
as JSON string behave identically.

What works, and narrows it: at `observe` scope the hub returned a correct,
**method-specific** scope refusal (`'channel.read' requires 'execute' scope`). So
transport, TLS, token and auth are fine and the method name arrives intact — the failure
is in dispatch after the scope check.

**This is independently corroborated.** pen-agent (`/opt/050-email-archive`, same host)
hit it separately and registered `G-MCP-REMOTE-CALL-CHANNEL-TARGET-MISSING`, saying it
cost them two sessions. They also note the CLI form works:
`termlink channel state --hub HOST:PORT --json TOPIC`. Two independent agents, same
failure, means this is probably not a stale-binary artifact — but confirm against the
restarted server from item 1 before filing, then file it properly.

### 3. T-2696 — the last GO-derived item with no work behind it

`comms-selftest.sh` (charter verbs 1+2) and `substrate-smoke.sh` (verb 3) are **executed
by nothing** — referenced only in comments, docs and task files. No cron, no CI, no
check invokes either. Only `session-selftest.sh` is wired, via T-2557's canary. Status
`captured`; ACs are still placeholders, so the G-020 gate will block until you write real
ones.

Design note before you start: these are **not** symmetrical.

- `substrate-smoke.sh` is local and deterministic — good canary substrate, mirror T-2557's
  exit-code translation (0 proven → healthy, 1 broken → FIRE, 2 tooling → non-firing).
  But it MUTATES state: it creates topics and leaves claim debris (the
  `substrate-drain-demo` topic already carries ~90 expired claim rows from past runs).
  T-2709 fixed the stuck-claims predicate so that debris no longer latches the canary,
  which is what makes a daily run tolerable. Say so explicitly if you wire it.
- `comms-selftest.sh` needs a **live peer**. Absence of a peer is not a substrate failure,
  so it cannot honestly be a cron canary — a daily run would fire on an empty fleet. If
  the conclusion is "correctly stays on-demand", record that as the finding rather than
  fake-wiring it.

One task per deliverable — these are two.

---

## Do not sink time here

### 1. DO NOT RESTART THE `.107` HUB (T-2768, `owner: human`)

`/var/lib/termlink/` has **no `hub.secret` and no `hub.cert.pem`** while the hub (PID
3093442, `termlink-hub.service`) serves TLS from memory. **Restarting regenerates both**
— PL-021's both-rotate case — forcing a fleet-wide re-pin. The normal G-070 "restart
through the unit" advice is exactly wrong here. Root cause unestablished; T-559 blocks
inspecting `~/.termlink` for a cached copy.

There is also a second, unsupervised hub (PID 3869961) started manually by another
session, holding `hub.sock` — split brain on one host. Preflight Check 6 detects it
correctly. Stopping it belongs to that session or the operator.

### 2. The OneDev push needs a human, not another agent attempt

Root cause is settled: the shared token was **leaked and revoked server-side** today
(~15:00, ring20 T-1626). It was embedded as `https://TOKEN@host` in remote URLs —
including this repo's, now removed (a real fix that stands regardless).

Three attempts, all failing, establish the remaining gap:

| # | form | result |
|---|---|---|
| 1 | `username=admin`, multi-line approve | User unknown or credential incorrect |
| 2 | `username=oauth2`, one-line `url=` approve | same |
| 3 | `oauth2` **+ username in the remote URL** | same |

`oauth2` is the correct convention (pen-agent validated it on `.201`). The blocker is
that the rotated token in `/home/dimitri-mint-dev/.onedev-token-r20260816` is for the
**`.201`** instance, and termlink's origin resolves to **`.52`**
(`onedev.docker.ring20.geelenandcompany.com` → `192.168.10.52`). `.52` needs its own
token. Asked ring20-manager at dm offset 16.

**Do not run more auth attempts.** Each failure fires `git credential reject`, wiping the
store entry, so the failure destroys its own evidence — and repeated failures risk a
lockout.

Three traps worth keeping in any runbook: the multi-line `key=value` approve form
silently no-ops on git 2.x; failed auth wipes the entry; and stale entries shadow
selection — `~/.git-credentials` here holds `admin` and `git` lines for our host, and
because the remote URL had no username, `store` returned the first match and never
consulted the new one.

**Drop file deliberately NOT shredded.** pen-agent asked termlink to `shred` it since
their own boundary hook blocked them. Declined: a secret outside this project, owned by
another user, requested by a peer agent rather than the operator — peer requests are
proposals, not authorization. It is also still the only working `.201` credential on the
host. Human decision.

### 3. Herdr is exhausted for agents — do not re-derive this

The backlog's own status block: *"Every agent-ownable item is now closed. Only 18 and 19
remain, both `owner: human`."* Ranks 1–17 closed, 20 already-implemented, 21 declined,
22 pinned-and-declined, 23 recorded. Rank 18 additionally gained a **negative** result
today (T-2766) — the hub incident looks like an argument for changing the default
`runtime_dir` and is not one, because unit, hub process and shell all resolve the same
persistent path.

If asked to "focus on herdr", the correct answer is that there is nothing there without
a human decision on 18 or 19. Say so; do not manufacture work.

---

## Live caveat that will confuse you

**Two agents on `.107` share identity fingerprint `d1993c2c3ec44c94`**: `pen-agent`
(`/opt/050-email-archive`) and termlink (`/opt/termlink`). The DM topic is
fingerprint-scoped, so **their messages and ours are the same thread**. ring20-manager
answered "pen-agent" in a thread this session then posted into. Receipts posted by one
mark the other's messages read.

Practical: sign your project prefix explicitly on that channel, and do not assume a
message addressed to "you" is yours. pen-agent proposes the same convention pending
per-agent identity (their T-1562). Worth filing as an observability gap — it silently
breaks the addressing assumption every DM verb rests on.

---

## Shipped this session (context, not work)

- **T-2767** — `hub start` refused to notice a LIVE hub when the pidfile did not name it,
  allowing a second hub to steal `hub.sock`. Now probes the socket. 21 pidfile tests,
  federation tripwire green, workspace 3593 pass.
- **T-2703** — strict-star guard for `termlink-cli` + `termlink-mcp`. Narrowed
  deliberately: `no_spoke_mesh_tripwire.rs` already covers `termlink-session` more
  strongly. Guard layer 39 → **41/41**.
- **T-2766** — negative evidence for herdr rank 18.
- **T-2765** — closed as a duplicate created in-session.

Two process lessons from this session, both the same shape — *absence of a grep hit is
not absence of the artifact*:

1. I concluded no spoke-side guard existed because three guessed identifiers did not
   match. `no_spoke_mesh_tripwire.rs` existed. The task's own `## Verification` block
   named it, and the gate caught me.
2. I handed the operator `termlink channel read`, a verb that does not exist.

Before concluding something is missing: **list the directory, and read the task's
Verification block** — it names artifacts by path.
