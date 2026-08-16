# PICKUP — next session (written 2026-08-16, end of S-2026-0816-1621)

Read this before `LATEST.md`. It is shorter and it says what to do.

Branch: `worktree-charter-review-2026-0814`, worktree
`/opt/termlink/.claude/worktrees/charter-review-2026-0814`.
Everything is committed. **8 commits are unpushed** — see "Do not sink time here" §2.

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
