# Herdr adoption slice 4 — persistence, detach/reattach, liveness (2026-08-15)

READ-ONLY reconnaissance. No source modified. Scope: **how herdr survives lid-close /
network drop / SSH disconnect**, whether that architecture is structurally more robust
than TermLink's registration + heartbeat + be-reachable/pushwaker rail, and what
specifically is worth borrowing.

Settled context assumed, not relitigated: we are NOT replacing the session layer
(`herdr-evaluation-synthesis-2026-08-15.md` §3 — no exit codes, no signals). This file
asks only whether the *persistence* sub-architecture has a lesson.

**Evidence tags:** `[VERIFIED-SOURCE]` pinned/authoritative endpoint ·
`[VERIFIED-PAGE]` live project page, unpinned · `[READ]` file read in this repo at the
cited line · `[INFERRED]` reasoned, not measured.

**Tooling:** nothing blocked. Bash, Grep, Read, WebFetch all functioned. No claim below
is unverified for tooling reasons; the ones that are unverified for *other* reasons are
named in §6.

---

## 1. HOW herdr does it — the actual mechanism

### 1a. Local persistence: a background server daemon owns the PTYs

`https://herdr.dev/docs/concepts/` **[VERIFIED-PAGE]**, verbatim:

> "By default, Herdr runs as a background server plus one or more attached clients.
> The server owns panes and process state. The client is the terminal UI attached to
> that server."

`https://herdr.dev/docs/session-state/` **[VERIFIED-PAGE]**, verbatim:

> "Normal detach keeps the Herdr server running. Panes, shells, agents, servers, tests,
> and command processes keep running inside that server."

`https://herdr.dev/docs/persistence-remote/` **[VERIFIED-PAGE]**, verbatim:

> "Herdr keeps panes running in a background server. Your terminal client can detach
> and reconnect later."

**That is the whole trick, and it is the same trick tmux, screen, and TermLink use:
a detached long-lived process holds the PTY, and the thing that dies on lid-close
(the client / the ssh channel) is not that process.** Survival of lid-close and
network drop is a *corollary* of the client being disposable, not a separate mechanism.
There is no checkpoint/restore, no process migration, no CRIU.

### 1b. What survives a *server* restart: layout, not processes

This is the load-bearing limitation, and it is under-advertised by the marketing page's
"brings the layout back and resumes their sessions". `/docs/session-state/`
**[VERIFIED-PAGE]**, verbatim:

> "restores the saved session shape: workspaces, tabs, panes, cwd, layout, and focus"

…but **"original pane processes are gone"**, and panes "come back as new shells in
their saved directories". Two optional softeners:

- **Pane history replay** — "stores saved pane history in `session-history.json` next
  to `session.json`". Cosmetic scrollback restoration. **Disabled by default.**
- **Native agent resume** — "Herdr can use official integration-reported session
  references to restart supported agent panes after a Herdr server restart." Enabled by
  default; requires per-agent minimum integration versions. This restarts the *agent*,
  not the process.

So: kill the herdr server and every running command dies. Persistence is bounded by the
lifetime of one daemon process — exactly TermLink's bound.

### 1c. Live handoff — the one genuinely distinct capability

`/docs/session-state/` **[VERIFIED-PAGE]**, verbatim:

> "Handoff tries to keep the current processes alive" … `herdr update --handoff`
> "asks the old server to transfer live panes to the new server, so pane processes can
> keep running."

This is the only mechanism in herdr that TermLink has no counterpart for, and it exists
to solve a problem we have documented three times. See §4 R1.

### 1d. Remote: an SSH-transported thin client, NOT a socket tunnel

The brief asked for the detail. `/docs/persistence-remote/` **[VERIFIED-PAGE]**,
verbatim:

> `herdr --remote` creates "a thin client" that "connects over SSH, starts or attaches
> to the remote Herdr server, and streams the UI back to your local terminal."

Supported on "Linux and macOS hosts on x86_64 and aarch64". The SSH plumbing:

> remote attach "runs remote setup and the bridge through a temporary SSH config that
> includes your SSH config first, then adds fallback keepalive settings and a private
> per-attach control socket for connection reuse"

Opt-out: `[remote].manage_ssh_config = false`. Windows: "Native Windows
`herdr --remote` is not part of the Windows beta. From Windows, SSH into the server and
run `herdr` there."

**Read this precisely.** The "private per-attach control socket" is an *SSH
ControlMaster* socket (connection multiplexing on the client side), **not** a forwarded
Unix socket exposing the herdr API. The herdr socket API is never carried over the
network. What crosses the wire is a rendered UI stream. So `--remote` is a convenience
wrapper around `ssh host herdr attach` with keepalives — ergonomics, not architecture.
The keepalive settings exist because the SSH channel is fragile; the server's survival
never depended on it.

---

## 2. Do they actually have fewer persistence problems? No. They are not looking.

The brief warned against reading "fewer canaries" as "fewer problems". Checked directly
against their issue tracker via the GitHub search API **[VERIFIED-SOURCE for
number/title/state; issue bodies NOT opened]**:

| # | Title (verbatim) | Maps to our |
|---|---|---|
| 2775 | Headless server: agent status stays stale without periodic re-detection | **frozen husk (T-2230/T-2235/T-2239)** |
| 2006 | Server drops all clients, remains alive without status socket on WSL2 | **frozen husk — process alive, rail dead** |
| 2711 | Running server keeps stale agent-detection manifest | **stale waker code (T-2405)** |
| 2407 | macOS: force-quitting hidden Chrome shuts down Herdr server | **dead waker (T-2387)** |
| 1762 | Linux: herdr server dies on GNOME/GDM restart while tmux survives | dead daemon |
| 2749 | Agents closed after detaching/reattaching from another device | reattach destroys state |
| 2737 | Fail to launch ("server did not become ready within 15s") | start-up race |
| 1914 | Compositor re-emits stale cells from dead sources; survives reboot | stale state persisted |
| 2129 | Session restore fails reliably with devenv.sh | restore |
| 1653 | Outdated claude hook: session dropped despite ack, restore fails | ack/restore |

**#2775 is our frozen husk, filed by a user.** "Agent status stays stale without
periodic re-detection" in headless mode is precisely T-2229's ring20 RCA: a live
process whose liveness signal stopped advancing. **#2711 is our stale-waker-code class**
— a long-lived process still executing state loaded before the file changed, which is
T-2405 verbatim. **#2006 is the husk in its purest form**: "remains alive without status
socket."

All three classes exist in herdr. None has a detector; they surfaced because users
filed tickets. TermLink found the same three classes and built
`check-frozen-husk-freshness.sh`, `check-waker-liveness-freshness.sh`, and
`check-stale-waker-code-freshness.sh`.

**Conclusion for the brief's core question: their architecture is NOT structurally more
robust here.** It is the same detached-daemon design with the same failure classes and
no detection layer. Our three canaries are evidence of *looking*, not of being worse.
The honest asymmetry runs the other way.

---

## 3. TermLink equivalent — verified, with the warts

### 3a. Detach: same mechanism, three launchers

`termlink register` owns the PTY (`crates/termlink-session/src/pty.rs:103–120`, per the
companion doc's full-file read) and is started detached by one of three launchers, all
in `crates/termlink-cli/src/commands/execution.rs` **[READ]**:

- `spawn_via_terminal` — `osascript` → Terminal.app, a separate application (:516)
- `spawn_via_tmux` — `tmux new-session -d` (:531)
- `spawn_via_background` — `setsid sh -c` (:542)

Each severs the process from the invoking shell, so SSH disconnect and lid-close do not
reach it. **Structurally identical to herdr's guarantee.**

### 3b. Reattach from any terminal: registry + per-session socket

`Registration::json_path` / `default_socket_path`
(`crates/termlink-session/src/registration.rs:311–318`) **[READ]** — a JSON sidecar plus
a Unix socket per session under `sessions_dir`. Any unrelated `termlink` invocation
finds and drives an existing session. This is herdr's client-attach model, already
present.

### 3c. Heartbeat: in-process, and that is the husk's structural cause

Two heartbeat loops, both `tokio` tasks **in the same process that owns the PTY**:

- PTY path — `crates/termlink-cli/src/commands/session.rs:388–407` **[READ]**, T-2230.
  The comment states the original defect verbatim: `touch_heartbeat` "had zero
  production callers", so `heartbeat_at` "stayed frozen at creation time forever".
- `--self` event-only path — `crates/termlink-session/src/endpoint.rs:175–192`
  **[READ]**, T-2235, a never-completing branch of a `tokio::select!`.

Both default to 30s via `TERMLINK_HEARTBEAT_INTERVAL_SECS`. Regression pinned by
`heartbeat_strictly_advances_over_time` (`registration.rs:539–564`) **[READ]** — which
explicitly documents that the *pre-existing* test tolerated an unchanged timestamp and
therefore could not catch the freeze.

**The structural point:** the liveness signal shares a process, a runtime, and a
`RwLock` with the thing whose liveness it reports. A wedged runtime or a writer holding
`shared.write()` freezes the heartbeat while the process stays alive — the husk. herdr's
#2775/#2006 show the same coupling produces the same bug there.

### 3d. The presence rail is already process-separated — we are ahead here

`scripts/be-reachable.sh` **[READ]** spawns the heartbeat (`:258–261`) and the push-waker
(`:300–305`) as two independent `nohup setsid` processes, and
`scripts/fleet-rearm-wakers.sh` header **[READ]** states the design intent verbatim:

> "The push-waker is spawned by be-reachable as a `setsid` process-GROUP leader,
> SEPARATE from the heartbeat/listener pid. So we can reap ONLY the waker pgroup and
> respawn the current-code waker with identical args — the heartbeat/presence process
> is NEVER touched."

That is a zero-outage code-roll on a live rail. **herdr's `update --handoff` is the same
idea applied to the PTY-owning server, which is the layer where we do not have it.**

---

## 4. Recommendations

All three are **category (a) — adopt the IDEA**. No herdr code is proposed for copying,
so no attribution or NOTICE handling is triggered. Flagged explicitly per the licence
rule.

### R1 — Close the "upgrade kills every session" gap. **[IDEA — category (a)]**

**What to adopt.** herdr's insight that a session daemon must have a story for its own
binary being replaced while it holds live processes (`herdr update --handoff`, §1c).

**Evidence in herdr.** `/docs/session-state/` **[VERIFIED-PAGE]**: handoff "asks the old
server to transfer live panes to the new server, so pane processes can keep running."
That they built it is evidence the problem bites in production.

**Does TermLink have it? NO — verified absence.** `grep -n 'handoff|hand-off|live.migrat|
re-exec|reexec'` across all `*.rs`, `*.sh`, `*.md` returned 30 files, and **every
code hit is `claim-transfer`** (`crates/termlink-cli/src/commands/channel.rs`,
`crates/termlink-session/src/claim_client.rs`, `crates/termlink-mcp/src/tools.rs`) —
work-claim ownership transfer, an unrelated concept. No session or hub handoff, no
re-exec, no fd passing. The only live code-roll we own is
`scripts/fleet-rearm-wakers.sh`, whose own header scopes itself to the waker and states
the heartbeat process "is NEVER touched" — i.e. it deliberately does not cover the
PTY-owning process. Upgrading `termlink` today means every registered session must die.

**This is the same root as T-2405** (alive process on old code), one layer down. We built
detection *and* remediation for the waker; for the session we have neither.

**Effort.** Full fd-passing handoff: **weeks**, and it re-opens the data-plane and
registration invariants — not proposed. The honest, proportionate subset:
**~0.5–1 day** — surface stale-binary sessions. `Registration.metadata.termlink_version`
already exists and is populated at registration
(`registration.rs:220`, `:286` — `env!("CARGO_PKG_VERSION")`) **[READ]**, so a
`termlink sessions --stale-binary` comparison against the installed binary is a read
over data we already write. This mirrors T-2359's version-floor pattern at the session
tier instead of the hub tier.

**Directives.** D1 Antifragility (the upgrade path is currently a scheduled outage),
D2 Reliability (a session running pre-fix code is invisible today).

### R2 — Fix the `setsid` fallback: it is a real persistence hole off Linux. **[IDEA — category (a)]**

Not borrowed from herdr; found while verifying §3a against their guarantee. Reporting it
because it directly contradicts the survival property this slice is assessing.

**The defect.** `spawn_via_background`
(`crates/termlink-cli/src/commands/execution.rs:541–556`) **[READ]** runs
`setsid sh -c <cmd>` and, on spawn error, falls back to a **bare `sh -c` with no
`setsid` and no `nohup`** (`:548–555`). macOS ships no `setsid` — it is util-linux —
which `.context/checks/platform-lock-allowlist:26` states explicitly **[READ]**. So on a
**headless macOS host with no tmux**, `Auto` resolves to `Background`
(no WindowServer → no tmux → Background, per the companion doc's read of
`resolve_spawn_backend`), the fallback fires, and the session process is left in the
invoking shell's session with no `nohup`.

**Our own shell rail already does better.** `scripts/be-reachable.sh:258–261` **[READ]**
uses the identical setsid-present/absent branch but wraps **both** arms in `nohup`. The
Rust path does not. Same decision, two answers, and the weaker one is on the charter-verb
path.

**The allowlist reason is incomplete against its own rule.** Entries at lines 30–31 read
"process starts, no session leader" and the header calls it "Degraded but functional".
Accurate as far as it goes — but T-2693's stated rule (allowlist:11–13) is that the reason
"must state **how the non-Linux path behaves**", and *functional for spawning* is not
*functional for surviving disconnect*, which is the property `background` exists to
provide. The README inaccuracy is already noted there (`:28–29`); the persistence
consequence is not.

**Caveat, stated rather than buried.** Whether SIGHUP actually reaches this child is
conditional: the kernel HUPs the session leader and the controlling terminal's foreground
process group, and this child is stdio-null'd and backgrounded. Whether it dies depends
on the parent shell's job-control behaviour. **I did not test this empirically** — it is a
credible hazard, not a demonstrated failure. It is also exactly why `setsid`/`nohup`
exist, and why we use them everywhere else.

**Fix.** Replace the external-binary dependency with `libc::setsid()` in
`std::os::unix::process::CommandExt::pre_exec` — works identically on macOS and Linux,
removes the fallback entirely, and retires **three** platform-lock allowlist entries
(`execution.rs`, `dispatch.rs`, `tools.rs`). **Effort ~0.5 day** including the allowlist
edit and a test. Note `pre_exec` is `unsafe` and must stay async-signal-safe; a bare
`setsid()` qualifies.

**Directives.** D4 Portability (this *is* the Directive-4 defect class T-2693 was built
for), D2 Reliability.

### R3 — `herdr --remote`: **do NOT adopt.** **[explicit no]**

An SSH-bridged thin client (§1d) solves "render a remote TUI locally". TermLink's remote
story is RPC to the hub over TCP, and its clients are agents, not humans at a TUI. Adding
an SSH UI bridge would be breadth accretion with no charter verb behind it — the exact
finding T-2468/P4 corrected, and the charter-drift canary (T-2483) exists to prevent.
Recorded here so the decision is visible rather than silently omitted.

The one transferable detail is cheap and unglamorous: herdr adds **SSH keepalives** by
default because it learned the channel is the fragile part. If any TermLink workflow
depends on a long-lived `ssh` (the `ssh:` `bootstrap_from` anchor in the reauth path is
the candidate), the same hardening applies. **Not investigated in this slice** — flagged,
not claimed.

---

## 5. Summary table

| Property | herdr | TermLink | Verdict |
|---|---|---|---|
| Survive lid-close / net drop / SSH disconnect | detached background server owns PTYs | detached `register` owns PTY, 3 launchers (`execution.rs:516/531/542`) | **parity** |
| Reattach from any terminal | client attaches to server socket | JSON registry + per-session Unix socket (`registration.rs:311–318`) | **parity** |
| Survive server/daemon restart | **NO** — "original pane processes are gone" | NO | parity |
| Scrollback restored after restart | optional `session-history.json`, off by default | no | herdr, marginally |
| **Live handoff across binary upgrade** | **YES** (`update --handoff`) | **NO** (waker only, `fleet-rearm-wakers.sh`) | **herdr — see R1** |
| Remote access | SSH thin client + ControlMaster + keepalive | TCP RPC to hub | different models, not comparable |
| Liveness signal decoupled from PTY owner | no (#2775, #2006) | no for sessions; **yes** for the presence rail | parity on sessions |
| **Detection of the husk / stale-code classes** | **none** — found via user issues | 3 canaries (T-2239, T-2387, T-2405) | **TermLink, decisively** |

---

## 6. What I did NOT establish

Stated rather than dropped, per the methodology this repo has been correcting toward.

1. **Herdr docs pages are unpinned `[VERIFIED-PAGE]` and were read through WebFetch's
   summarising model.** Direct quotes are reliable; a page's *silence* on a topic is not
   proof of absence. Notably, `/docs/concepts/` and `/docs/install/` did not state the
   socket path, and I never established it.
2. **Herdr issue bodies were not opened.** Numbers, titles and states come from the
   GitHub search API; the §2 mapping onto our failure classes is my reading of the
   titles **[INFERRED]**. #2775 and #2006 are strong on the title alone; the others are
   weaker.
3. **The SIGHUP hazard in R2 is reasoned, not reproduced.** See the caveat in R2. A 5-minute
   empirical test on a macOS host without tmux would settle it, and should precede the fix
   being described as a bug fix rather than a hardening.
4. **I did not read `crates/termlink-cli/src/commands/pty.rs` or `pty.rs:103–120` myself.**
   Those citations are carried from `herdr-internal-tmux-surface.md`, which read the former
   in full. Second-hand, though from a line-cited source.
5. **S14 — "survives SSH disconnect" is still not proven by any test in this repo.** The
   companion doc flagged this and I did not close it. §3a argues it from the *mechanism*
   (`setsid`/`tmux -d`/Terminal.app), which is sound but is not a test. Given R2 identifies
   a platform where the mechanism silently degrades, **the absent test is now more
   interesting than it was** — it is the guard that would have caught R2. Worth a task
   independent of whether R1 or R2 proceeds.
6. **Herdr's handoff implementation is undescribed.** The docs say what it does, not how
   (fd passing? re-exec? SCM_RIGHTS?). If R1 were ever scoped beyond the read-only
   detector, that would need reading their source — and *that* is where the category-(b)
   licence question would arise. It does not arise for anything recommended here.
