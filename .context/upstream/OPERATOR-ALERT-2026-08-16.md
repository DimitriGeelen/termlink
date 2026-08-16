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

`credential.helper` is `store` (local and global) and no longer has an entry
matching that username, so git falls through to a prompt with no tty. Something
rewrote the stored credential mid-session.

Unpushed on `worktree-charter-review-2026-0814`: `8484eb9d4`, `2996e0063`,
`699f0a9fa`, the handover commit, and the T-2767 closure.

**Work is not at risk of loss.** A linked worktree's branch ref and objects live in
the shared `/opt/termlink/.git`, so they survive deletion of the worktree directory.
They are simply not on OneDev, and therefore not mirrored to GitHub.

Fix is operator-side (stored secrets). Re-supplying the credential and re-running
`git push origin HEAD` from this worktree is sufficient.

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
