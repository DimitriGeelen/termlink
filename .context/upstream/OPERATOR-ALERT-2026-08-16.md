# OPERATOR ALERT — .107, 2026-08-16

Two items need a human. Written as a standalone file because the OneDev push is
broken (item 3), so this branch's handover may not reach anyone who is not
standing in this worktree.

---

## 1. DO NOT RESTART THE `.107` HUB (task T-2768, `owner: human`)

`/var/lib/termlink/` contains **no `hub.secret` and no `hub.cert.pem`**, while the
hub (PID 3093442, `termlink-hub.service`) is up and serving TLS on `0.0.0.0:9100`.
It holds both in memory.

```
fleet doctor →  [FAIL] Secret file not found: /var/lib/termlink/hub.secret
ls -la /var/lib/termlink/  →  hub.pid  hub.sock  bus/  sessions/
                              route-cache.json  rpc-audit.jsonl
                              (no hub.secret, no hub.cert.pem)
```

**Restarting regenerates BOTH.** That is PL-021's "both rotate" case and forces a
fleet-wide re-pin of every client. So the standard G-070 remediation — "restart
through the systemd unit" — is **exactly wrong here** until either the secret is
recovered from an out-of-band copy or a rotation is deliberately accepted and
scheduled.

Consequence while it stands: local profiles (`workstation-107-public`,
`local-test`) fail auth, so local reads degrade. Remote hubs `.121` / `.122` are
unaffected and answer normally.

**Root cause NOT established.** The T-559 project-boundary hook blocks this session
from inspecting `~/.termlink/` for a cached copy, and I did not route around it.
First thing to check: whether `~/.termlink/secrets/` holds a usable copy, and what
removed the files from a `StateDirectory=`-managed path.

---

## 2. A SECOND, UNSUPERVISED HUB IS RUNNING ON THE SAME runtime_dir

| PID | what | since |
|---|---|---|
| 3093442 | `termlink hub start --tcp 0.0.0.0:9100 --json` — the systemd unit | 14:24 |
| 3869961 | `termlink hub start` — **started manually by another Claude session** | 16:52 |

3869961 holds `hub.pid` **and** `hub.sock`. Effect: unix-socket clients reach one
instance and TCP clients the other, so the same topic name resolves to different
state **on one host**. Both look healthy to any check that examines only one of them.

`substrate-preflight.sh` Check 6 detects this correctly and names both PIDs
(`[WARN] hub-unit ... detached ghost serving outside supervision (G-070)`).

**Not actioned deliberately:** the ghost belongs to another live session. Stopping
it is that session's call or yours, not this one's — and see item 1 before any
restart of the supervised hub.

---

## 3. PUSH TO OneDev IS BROKEN — 4+ COMMITS UNPUSHED

`git push origin HEAD` succeeded at the START of this session and now fails:

```
fatal: could not read Password for
  'https://<token>@onedev.docker.ring20.geelenandcompany.com': No such device or address
```

**ROOT CAUSE — CONFIRMED, and my first diagnosis was WRONG.** This was not a local
credential-cache expiry. ring20-manager (T-1626) reports the shared OneDev push token
(admin-scoped, "Claude-Ring20") was found **LEAKED** and was **REVOKED server-side
today ~15:00**. The leak class: the token embedded as `https://TOKEN@host` in repo
remote URLs. `/opt/termlink`'s origin carried **that same token** — identical leading
substring `Ev5yUXablprd…` — so this repo was one of the leaking ones.

**Partially fixed here.** The token is now REMOVED from the remote URL and the new one
(drop file `/home/dimitri-mint-dev/.onedev-token-r20260816`, mode 600, 40 chars) is
installed via `git credential approve` instead, which is the point of the recipe —
keeping it out of `.git/config` is what stops the leak recurring:

```
git remote set-url origin https://onedev.docker.ring20.geelenandcompany.com/termlink
git config credential.helper store
printf 'protocol=https\nhost=<host>\nusername=admin\npassword=%s\n' "$TOK" | git credential approve
```

**STILL FAILING — one open question.** `git ls-remote origin HEAD` returns
`remote: User unknown or credential incorrect`. The recipe came from pen-agent's repo,
which is on **`http://192.168.10.201:6610`**; termlink's origin is
**`https://onedev.docker.ring20.geelenandcompany.com`**. Either these are different
OneDev instances needing different tokens, or `username=admin` is wrong for this
origin. Asked ring20-manager (dm offset 12); **stopped after ONE attempt rather than
trying usernames against a live auth endpoint** — repeated failures risk a lockout.

**MEASURED — the two repos are on DIFFERENT OneDev hosts:**

```
getent hosts onedev.docker.ring20.geelenandcompany.com  ->  192.168.10.52
pen-agent's origin                                      ->  192.168.10.201:6610 (http)
```

So termlink is on the **.52** instance and pen-agent is on **.201**. The drop-file
token was minted for .201, which is the most likely reason it fails against .52 —
`username=admin` is probably NOT the fault. Asked ring20-manager whether the T-1626
rotation covers .52 and whether there is a drop for it (dm offset 13).

**Unexplained and worth chasing:** the OLD token string was **identical** in both
repos — the same `Ev5yUXablprd…` value embedded in a .201 remote and in a .52 remote.
One credential value against two different hosts. Either the two addresses front the
same OneDev (making the DNS reading a red herring) or one admin token was reused
across two instances. Either way it widens the T-1626 leak blast radius beyond the
repos ring20 has already counted — flagged to them.

Note for whoever picks this up: the same leaked token was embedded in at least two
repos on this host, so check any other remote before assuming it is clean. Holding at
ONE failed auth attempt against .52 deliberately — no retries until the right
credential is confirmed, to avoid an account lockout.

Unpushed on `worktree-charter-review-2026-0814`: `8484eb9d4`, `2996e0063`,
`699f0a9fa`, the handover commit, and the T-2767 closure.

**Work is not at risk of loss.** A linked worktree's branch ref and objects live in
the shared `/opt/termlink/.git`, so they survive deletion of the worktree directory.
They are simply not on OneDev, and therefore not mirrored to GitHub.

Fix is operator-side (stored secrets). Re-supplying the credential and re-running
`git push origin HEAD` from this worktree is sufficient.

---

## 4. FOR THE NEXT SESSION — three findings, not yet filed as tasks

`fw task create` is Bash and was blocked by the budget gate when these surfaced, so
they are recorded here rather than hand-written into `.tasks/` outside the tooling.
File them properly next session.

### 4a. ring20-manager ALREADY answered the push failure — go read it

`dm:88743a9ad59fda39:d1993c2c3ec44c94` on **192.168.10.122:9100** carries a
`msg_type=note` from `88743a9ad59fda39` (ring20-concierge / host ring20-manager) at
**2026-08-16T15:21:49Z**, opening:

> `pen-agent — ring20-manager here. Root cause + fix for your push failure:\n\nROOT C…`

I could not read past the preview (see 4c). **Read that note before doing anything
else about item 3** — the answer may already be in hand.

### 4b. TWO AGENTS ON .107 SHARE ONE IDENTITY FINGERPRINT (PL-195 class)

The same topic shows a `chat` at 15:16:54 from `d1993c2c3ec44c94` reading:

> `Hi ring20-concierge — pen-agent (fp d1993c2c3ec44c94, host .107, project /opt/05…`

`d1993c2c3ec44c94` is **also this session's fingerprint** (`my_fp` returned by
`agent contact`). So "pen-agent" in `/opt/05…` and termlink in `/opt/termlink`
resolve to the SAME identity, and their DMs land in the SAME `dm:` topic.

Consequences, all real: a peer cannot address one of us without reaching the other;
`/check-arc` unread counts are shared; a receipt posted by one marks the other's
messages read; and ring20-manager answered "pen-agent" in a thread that this session
then posted into as if it were its own. The identity is host-derived, not
project-derived. Worth a task — this silently breaks the addressing assumption every
DM verb rests on.

### 4c. `termlink_remote_call` does not forward `params` (MCP surface)

Three different methods against `.122`, each with correct params, all return the
identical error:

```
channel.read      {"topic": "...", "from_offset": 5, "limit": 1}  -> -32001 Missing 'target' in params
channel.relations {"topic": "...", "target": 6}                   -> -32001 Missing 'target' in params
channel.info      {"topic": "..."}                                -> -32001 Missing 'target' in params
```

**Sharpened after further testing — it is worse than "params dropped".** A fourth
call settles it:

```
termlink.ping     (NO params exist for this method)  -> -32001 Missing 'target' in params
```

Four methods, including one that takes no parameters at all, return the byte-identical
error. Passing `params` as a JSON *string* instead of an object changes nothing. So the
failure is not parameter serialization — **every `remote_call` fails the same way
regardless of method or params.**

The one thing that DOES vary is instructive: at `observe` scope the hub returned a
correct, method-SPECIFIC refusal (`'channel.read' requires 'execute' scope ... This is
a SCOPE mismatch, not a bad secret`). So the method name reaches the auth/scope layer
intact, and transport + token + TOFU all work. Dispatch after that point fails
uniformly. That narrows it to the dispatch step rather than the wire or the auth path.

The tool describes itself as "the universal cross-host escape hatch — any hub RPC
method can be invoked remotely through this one tool". On this surface, none can.

**Not yet attributed.** The MCP server here runs an older binary than the tree
(T-2707: 0.11.720 vs VERSION 0.11.1440), so this may already be fixed in current
code. Establish that first — restart the MCP server onto a current binary and re-run
the three calls above. If it still reproduces, it is a live defect on the universal
cross-host escape hatch, which would be significant.

---

## Not an alert, but worth knowing

**The herdr adoption backlog is exhausted for agents.** Its own status block:
*"Every agent-ownable item is now closed. Only 18 and 19 remain, both `owner: human`."*
Ranks 1–17 closed, 20 already-implemented, 21 declined, 22 pinned-and-declined,
23 recorded. Nothing there is available to an agent without a human decision first.

**Rank 18 gained a NEGATIVE result today (T-2766).** The incident above superficially
looks like a perfect argument for changing the default `runtime_dir` — it is not.
The systemd unit, the running hub process and the interactive shell all resolve the
same `/var/lib/termlink`, on disk-backed btrfs. Recorded in the backlog under rank 18
so the human decision is not given false support.
