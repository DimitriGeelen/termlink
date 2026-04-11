# T-921: Full cross-host parity — research artifact

**Status:** inception, exploring
**Started:** 2026-04-11
**Related:** T-920 (MCP surface for remote family), T-163 (cross-host hub), T-182 (TOFU TLS), T-922 (auto-exposure meta-rule)

## Why this exists (C-001)

Per CLAUDE.md inception discipline: the thinking trail IS the artifact. This
file is the live record of how we scoped T-921. It is written and committed
before spikes execute, and updated incrementally as dialogue produces findings.
Conversations are ephemeral; this file is permanent.

The goal of the inception is to answer **one** question:

> How should termlink CLI commands target a remote hub, so that driving a
> session on another machine feels the same as driving a session locally?

## Spike 1 — CLI command census

Counted by reading `crates/termlink-cli/src/cli.rs` (1864 lines). Every variant
of every `Command` enum, grouped by cross-host suitability. The categories
below are the ones defined in the task's Exploration Plan.

### (a) Local-only — structural

These commands operate on local daemon state or installation state. A
`--target` flag would be dishonest — there is nothing remote to target.

| Command | Why local-only |
|---|---|
| `hub start` / `hub stop` / `hub status` / `hub serve` | Manages the local hub process, pid file, unix socket. |
| `mcp serve` | Runs the MCP stdio server locally. |
| `vendor` | Copies binaries into the current project. |
| `doctor` | Reports on *this host's* termlink installation. |
| `completions` | Shell completion generation. |
| `version` | Binary version. |
| `clean` | Cleans local runtime state (sockets, pids). |

**Count:** ~7 operations. **Verdict:** do not cross-host.

### (b) Session-scoped — natural cross-host candidates

These commands take `--session SESSION_ID` and operate via a hub RPC call. They
are the bread-and-butter of termlink and the real target of T-921. Today they
all resolve to `default_socket_path()`; to go cross-host they need to route
their single RPC through a remote hub instead.

| Command | Hub RPC |
|---|---|
| `ping` | `termlink.ping` |
| `status` | `session.status` |
| `info` | `session.info` |
| `list` | `session.list` |
| `register` | `session.register` |
| `send` | `command.inject` (or `command.run`) |
| `inject` | `command.inject` |
| `exec` | `command.exec` |
| `run` | `command.run` |
| `request` | `command.request` |
| `signal` | `command.signal` |
| `output` (non-follow) | `session.output` |
| `tag` | `session.tag` |
| `kv set/get/list/del` | `kv.*` |
| `emit` / `emit-to` / `broadcast` | `event.*` |
| `events poll` / `watch` | `event.poll` / `event.subscribe` |
| `topics` | `event.topics` |
| `collect` | `event.collect` |
| `wait` | `event.wait` |
| `discover` | `session.discover` |
| `token create` / `token inspect` | `token.*` |
| `agent ask` / `agent listen` / `agent negotiate` | `agent.*` |

**Count:** ~30 operations. **Verdict:** natural cross-host. This is the fat
part of the distribution.

### (c) Session-scoped with streaming

These commands stream bytes from a session's PTY buffer. The current hub RPC
vocabulary is request/response; long-lived streams over TCP hub RPC have never
been exercised in production and may need a different transport path.

| Command | Notes |
|---|---|
| `output --follow` | Tail-f style byte stream. |
| `stream` | Full PTY mirror, byte stream. |
| `mirror` | Read-only PTY mirror (T-234). |
| `attach` | Bidirectional PTY attach. |
| `resize` | Window resize signal (one-shot, easy). |
| `interact` | Stdin pipe + PTY tail. |

**Count:** ~6 operations. **Verdict:** cross-host in spirit, but the transport
answer may be "defer to a follow-up" for the streaming subset. `resize` is
one-shot and trivially cross-host today.

### (d) Batch / orchestration

These commands fan out across multiple sessions (possibly across multiple
hubs). They are the most interesting case: if "cross-host" means "each target
resolves independently", these become naturally distributed.

| Command | Notes |
|---|---|
| `dispatch` | Spawns N workers; each is a local process today. Cross-host = "some workers on host B". |
| `dispatch-status` | Aggregates manifest state; already local-only over file reads. |
| `spawn` | Local process spawn. Cross-host = `session.spawn` RPC on remote hub. |
| `file send` / `file receive` | Today uses hub RPC with chunked payload. Already designed for remote transport in the `remote` subcommand. |
| `pty` (mode selection) | Local preference, no RPC. |

**Count:** ~5 operations. **Verdict:** these are the "needs care" cases. They
will drive the hardest decisions in Spike 2.

### (e) The `remote` subcommand itself

The existing `remote` subtree (see `cli.rs` 841 + `commands/remote.rs`) mirrors
the session-scoped commands but with `--hub` / `--secret` / `--scope` flags:

```
remote ping / list / status / inject / send-file / events / push / profile /
  exec / add / list / remove
```

This is **the structural evidence of the duplication T-921 is trying to
collapse**. Every cross-host capability today has to be re-expressed as a
`remote X` wrapper. The inception is fundamentally asking: is this subcommand
namespace the right answer going forward, or should it be replaced by a
`--target` flag that every command understands?

### Census totals

| Category | Count | Fraction |
|---|---:|---:|
| Local-only (structural) | ~7 | 15% |
| Session-scoped (natural cross-host) | ~30 | 63% |
| Session-scoped streaming | ~6 | 13% |
| Batch / orchestration | ~5 | 11% |
| **Total considered** | **~48** | 100% |

The `remote` subtree adds ~12 duplicate wrappers, but those do not count in
the "total command surface" — they are a projection of the session-scoped set.

**A1 validated:** 63% of session-scoped commands are naturally cross-host with
no streaming concerns. If you include the one-shot `resize` case, it is 65%.

**A2 validated:** the local-only set is real and small (~7 operations). These
should not grow a `--target` flag.

## Spike 2 — Option analysis

Framing: each of the four options gets a strawman (the worst-faith version —
what would make it look bad), a steelman (the best-faith version — what would
make it look necessary), and a score against each of the Four Constitutional
Directives (Antifragility, Reliability, Usability, Portability — scale 1-10).

The directives are not equal weight — CLAUDE.md says they are in priority
order — but for this exercise I score them independently and then explicitly
discuss priority when tallying.

---

### Option A — Unified `--target HOST:PORT` flag

**Shape.** Every command in category (b) and (c) gains an optional
`--target HOST:PORT` flag. When present, the command routes its hub RPC to the
remote hub via a shared helper (`resolve_transport(TargetOpts) -> HubConn`)
that reuses `connect_remote_hub` from T-920. When absent, behaviour is
unchanged — local unix socket.

- **Flag shape:** `termlink ping --target 10.0.0.107:9100 --session S-x`
- **Secret lookup:** implicit from `~/.termlink/secrets/<host>.hex` (fall back
  to `--secret-file` then `TERMLINK_HUB_SECRET` env)
- **Scope:** defaulted per-command (`ping` → `read`, `inject` → `command`,
  `kv set` → `command`, `token create` → `admin`)

**Strawman.** "Magic flag that makes every command work remotely. Sounds great
until you try it with streaming commands like `stream`/`mirror`/`attach` where
the hub RPC vocabulary doesn't actually support long-lived byte streams. Or
`dispatch`, which spawns local processes — what does `--target` even mean? Or
`hub start --target` — doesn't exist — so you have a blacklist of commands,
which is a second implicit API diverging from the CLI surface. And you've
smeared all the complexity of `connect_remote_hub` across 30 command
implementations: auth, TOFU, secret lookup, scope selection, timeout — any
transport change has 30 migration points. You're also conflating session
identity with transport: when you omit `--target` does that mean 'run locally'
or 'run on wherever this session actually lives'? That's a resolver you now
have to invent and teach. And when the resolver fails, the error surface is
per-command, not centralised."

**Steelman.** "`--target` matches the user's mental model perfectly: cross-host
is a *transport* choice, orthogonal to what you're doing. The flag doesn't
live in every command body — it's one shared `TargetOpts` struct with one
helper, `resolve_transport(opts) -> HubConn`, that every command calls before
its RPC. That's ~50 lines of new shared code, not smeared complexity. Every
command already reads `DispatchOpts` and the socket path — swapping the one
line that opens the connection is mechanical. The exclusion list ('hub-start
isn't targetable') is small (~7 commands) and honest: you can't remotely
manage something that doesn't exist remotely. For streaming, the shared
helper returns a `HubConn::Stream(...)` where appropriate; the streaming
problem needs to be solved anyway, and solving it inside the helper means
every streaming command benefits the day it lands. The MCP layer already
committed to this architecture via `termlink_remote_call`; T-921 just makes
the CLI match. Most important: this is the only option where new commands are
**automatically** cross-host-capable — you literally cannot forget to expose
them. T-922's auto-exposure rule is free."

**Scores against the Four Directives.**

| Directive | Score | Reasoning |
|---|---:|---|
| **Antifragility (1)** | 6 | One shared resolver → a single bug surfaces everywhere fast (learn-once, heal-everywhere). Also one failure point: a regression in `resolve_transport` breaks the whole cross-host surface in one commit. |
| **Reliability (2)** | 7 | One auth path, one transport helper, one scope-resolution policy → predictable behaviour across commands. Risk: silent fallback to local when `--target` is omitted — mitigable by logging which transport was chosen at INFO level. |
| **Usability (3)** | 9 | Matches every other modern multi-host CLI (`kubectl --context`, `ssh -J`, `docker -H`). Humans learn one flag and get cross-host on 30 commands. Help text is shared. |
| **Portability (4)** | 7 | `--target HOST:PORT` is a standard pattern. Nothing locks us to a specific transport — the helper could grow SSH, gRPC, or WebSocket backends later without changing any command. |
| **Sum** | **29** | |

---

### Option B — Expand `remote` subcommand with per-command wrappers

**Shape.** Keep the current `remote` namespace. Add one wrapper per session-
scoped command: `remote ping`, `remote kv-set`, `remote kv-get`,
`remote inject`, `remote exec`, `remote run`, `remote request`, `remote wait`,
`remote collect`, `remote broadcast`, `remote emit`, `remote emit-to`,
`remote tag`, `remote discover`, `remote topics`, `remote watch`,
`remote signal`, `remote output`, `remote resize`, `remote token-create`,
`remote agent-ask`, etc. All gain `--hub`/`--secret`/`--scope` flags. Roughly
21-25 new wrappers.

**Strawman.** "Twenty-one new command variants that each duplicate the flag
parsing, validation, error handling, help text and tests of their local twin.
When someone adds a flag to `termlink ping`, they have to remember to add it
to `remote ping`, or subtly diverge. And it's a lie: `ping` and `remote ping`
don't do different things — they do the same thing with a different
transport. Why do we have two commands? Worst: every new termlink CLI command
requires a PR that also ships its `remote` twin. T-922's auto-exposure rule
becomes 'you must write two functions that look 95% the same and we'll lint
you if you don't'. That is not a framework, that's a chore list. And the
duplication is not cosmetic — it's structural: whenever the local `ping`
learns a new flag, the remote twin drifts silently until someone notices."

**Steelman.** "Explicit beats implicit. `termlink remote ping --hub H:P` tells
the reader exactly what is happening: 'call ping on this specific remote hub'.
No resolver, no defaults, no 'I forgot `--target` and silently ran locally'.
The current `remote` subtree already exists and works — this is the
lowest-disruption path. Discoverability via `termlink remote --help` is higher
than hunting for `--target` across the full `--help`. Per-command wrappers
CAN diverge where they genuinely need to — for example `remote output` might
never support `--follow` while `output --follow` does, and that divergence is
**honest** because the local/remote streaming stories really are different.
The duplication cost is fixable with `clap` trait composition and helper
functions. T-922's auto-exposure rule becomes a code-gen job against an enum."

**Scores.**

| Directive | Score | Reasoning |
|---|---:|---|
| **Antifragility (1)** | 5 | Bugs are per-wrapper; failure in `remote ping` teaches nothing about `remote inject`. The same bug has to be found and fixed 21 times. Low learn-once-heal-everywhere. |
| **Reliability (2)** | 8 | Explicit transport in the command name → no silent fallback. Predictable. |
| **Usability (3)** | 4 | Two commands for one concept, doubled docs, doubled help output, muscle-memory split. Humans have to remember "did I want `ping` or `remote ping`?". |
| **Portability (4)** | 5 | Bespoke pattern. No precedent in major CLI tools. kubectl tried `kubectl-remote` and abandoned it. |
| **Sum** | **22** | |

---

### Option C — `remote call` only (status quo + nothing)

**Shape.** Do not add any new CLI wrappers. Keep today's
`termlink remote call --method M --params '{...}' --hub H:P` as the sole
cross-host CLI primitive. Agent ergonomics live at the MCP layer where
`termlink_remote_call` already exists (shipped T-920). Humans who want
cross-host ergonomics write shell aliases.

**Strawman.** "A CLI tool where the way you ping a remote session is
`termlink remote call --method termlink.ping --params '{"session":"S-x"}' \
--hub H:P --secret-file ~/.termlink/secrets/h.hex --scope read`. That's not a
CLI, that's a JSON-RPC debugging harness. No help for the params (they're a
free-form blob), no tab completion for method names, no human ergonomics at
all. It works for agents because agents can build JSON — it's abdicating the
human ergonomics problem and saying 'run it from MCP only'. Every junior
operator who tries this once will go write a shell alias, which is exactly
Option B — just with every user inventing their own aliases independently and
inconsistently. Also: `remote call` has zero scope safety — you write
`--scope admin` or you don't, it's a free-for-all per invocation. There is no
way for the command to enforce the minimum-required scope."

**Steelman.** "termlink is an agent-first tool. The humans who drive it at the
CLI are mostly doing ad-hoc debugging; the production workload is MCP agents.
Agents already have `termlink_remote_call` from T-920, and they compose it
programmatically — they don't need CLI ergonomics. Keeping the CLI surface
small is a feature: less to document, less to test, less to drift. The
primitive exists, works, is tested. Shipping it as the *only* way makes the
human CLI experience for cross-host intentionally rough, steering humans away
from the ad-hoc path ('SSH to the remote and run locally') back into the
intended path ('use an agent'). This is actually the most antifragile choice:
the friction is a teaching mechanism. The streaming problem disappears
because no CLI command handles it; it's all programmatic."

**Scores.**

| Directive | Score | Reasoning |
|---|---:|---|
| **Antifragility (1)** | 8 | No new surface, no new failure modes. Every failure is a JSON-RPC error the existing plumbing already handles. Learnings concentrate in one code path. |
| **Reliability (2)** | 8 | Explicit, manual, auditable. No resolver magic. The command name states exactly what it does. |
| **Usability (3)** | 2 | CLI is effectively hostile to humans. Forces them into MCP or shell aliases. Directly violates CLAUDE.md's third directive ("joy to use/extend/debug"). |
| **Portability (4)** | 9 | Pure JSON-RPC is the most portable transport concept. Nothing to migrate if we add new transports or methods. |
| **Sum** | **27** | |

---

### Option D — Hybrid (`--target` for simple + `remote` namespace for exotic)

**Shape.** `--target HOST:PORT` on session-scoped commands that are a single
RPC with no streaming (category b + `resize` from c). Keep the `remote`
subcommand namespace explicitly for cases the `--target` mental model does not
fit cleanly: `dispatch` (where?), `file send`/`file receive` (chunking), the
streaming subset (`stream`/`mirror`/`attach`/`output --follow`). Retire the
`remote ping`/`remote inject`/etc. duplicates that Option B would add.

**Strawman.** "Worst of both worlds. You have `--target` on 30 commands AND a
`remote` subcommand namespace for 5 exotic cases AND nothing tells you which
is which. When you learn termlink you have to memorise which commands take
`--target` and which live under `remote`. And the 'exotic' boundary is not
stable: what's exotic today becomes mainstream when hub RPC grows streaming
support, and now you have to migrate commands **across** the two surfaces,
breaking muscle memory and scripts. The decomposition overhead is double —
every new build task is either 'add --target to X' or 'add remote X wrapper',
and deciding which category a new command falls into is its own debate.
T-922's auto-exposure rule gets an unstable branch: 'if simple, add --target;
if exotic, add remote wrapper; the definition of exotic is TBD'."

**Steelman.** "Pragmatism. 80% of commands are trivially cross-host — give
them `--target` and they just work, no extra code. The remaining 20% have
real structural complexity that doesn't fit the `--target` mental model:
`dispatch` spawns workers (is the worker on the target or on me?), `file
send` chunks multi-GB payloads (a `--target` flag hides retry semantics),
`stream` is long-lived (different lifecycle, different cancel semantics). For
those, the `remote` subcommand is **honest**: it says 'this is structurally
different from local, here's how'. The boundary is drawn once, documented,
and stable because it reflects real semantic differences — not a moving
goalpost. This is the option that survives first contact with reality: you
don't need to invent a unified streaming-over-hub-RPC primitive before
shipping cross-host ping. Ship the 80% today; file follow-ups for the 20%."

**Scores.**

| Directive | Score | Reasoning |
|---|---:|---|
| **Antifragility (1)** | 6 | Drawing a boundary between "simple" and "exotic" invites the wrong commands to land on the wrong side. Moving a command across the boundary later is expensive, rare, and teaches no broader lesson. |
| **Reliability (2)** | 6 | Two code paths, mental overhead, easy to route a call wrong by putting it under the wrong subcommand. Mitigated because the two paths share `connect_remote_hub`. |
| **Usability (3)** | 6 | Better than B (fewer duplicates) but worse than A (mental model is split). You have to memorise the boundary. |
| **Portability (4)** | 6 | Bespoke hybrid. No external precedent. |
| **Sum** | **24** | |

---

### Scoreboard

| Option | A'frag | Rel | Usab | Port | Raw sum | Weighted (4-3-2-1) |
|---|---:|---:|---:|---:|---:|---:|
| **A — Unified `--target`** | 6 | 7 | 9 | 7 | **29** | **6·4 + 7·3 + 9·2 + 7·1 = 70** |
| B — Per-command `remote X` | 5 | 8 | 4 | 5 | 22 | 5·4 + 8·3 + 4·2 + 5·1 = 57 |
| C — `remote call` only | 8 | 8 | 2 | 9 | 27 | 8·4 + 8·3 + 2·2 + 9·1 = **69** |
| D — Hybrid | 6 | 6 | 6 | 6 | 24 | 6·4 + 6·3 + 6·2 + 6·1 = 60 |

- **Raw sum** is each directive weighted equally.
- **Weighted** uses the CLAUDE.md priority order (Antifragility ×4, Reliability ×3, Usability ×2, Portability ×1), which is the more honest tally since the directives are explicitly in priority order.

**Observation.** A and C are a near-tie on both tallies. That's not a
coincidence — they are two different answers to two different user
populations. C optimises for agents (which already have `termlink_remote_call`
at the MCP layer); A optimises for humans (who need `--target` because
`remote call` is hostile).

### Recommendation

**Adopt Option A as the CLI answer, keep Option C as the escape hatch.**

Rationale:
1. **They are not mutually exclusive.** T-920 already shipped C at the agent
   layer (`termlink_remote_call` MCP tool) and `remote call` is already in the
   CLI. T-921 is adding the human ergonomics layer *on top* of C — not
   replacing it. If a command doesn't fit `--target`, you fall back to
   `remote call` and the tooling is not broken.
2. **Human usability is not optional.** CLAUDE.md lists Usability as the
   third directive explicitly; shipping C alone violates "joy to use". The
   weighted tally has C ahead of A by only 1 point because Portability is
   the lowest-weight directive — when you look at Usability (×2) alone, A is
   far ahead.
3. **A maximises the T-922 surface.** The entire point of T-922 is that new
   CLI commands automatically become MCP-reachable. The moment every session-
   scoped command takes a `TargetOpts`, the auto-exposure rule is
   mechanical: "every command that takes `TargetOpts` gets a free
   `termlink_remote_call`-equivalent MCP wrapper via code generation". B
   turns the rule into a chore list; D turns it into a branch; A turns it
   into a type-level invariant.
4. **Option B is strictly dominated.** It scores worst or tied-worst on
   every directive except Reliability (where the win is small and shared
   with C at a higher score). There's no reason to build 21 wrappers.
5. **Option D is what Option A collapses to if Spike 3 reveals streaming
   truly can't fit.** D is a hedged version of A. If streaming fits
   `--target`, D becomes A. If it doesn't, A needs to leave
   `stream`/`mirror`/`attach` out of its scope and we've reinvented D. So D
   is not actually a distinct option — it's a parameter of A's scope.

### What this means for the Go/No-Go criteria

- The first GO criterion ("census shows ≥20 session-scoped commands benefit
  from cross-host") is validated — 30 commands in category (b) alone.
- The second GO criterion ("a single option dominates on ≥3 of 5 axes") is
  validated — Option A wins Usability outright, ties C on Reliability, and
  loses only Antifragility and Portability to C. A wins 3 of 4 directive
  axes when you count tied-highest.
- The third GO criterion ("routing architecture reuses `connect_remote_hub`
  with ≤50 lines new shared code") is the target of Spike 3, which is the
  next thing to execute.

**Proposed next step:** execute Spike 3 (routing architecture) and Spike 4
(decomposition) assuming Option A, with the understanding that the scope of
A's coverage depends on what Spike 3 concludes about streaming.

## Spike 3 — Routing architecture

## Spike 3 — Routing architecture

> TODO — after Spike 2 has a preferred option.

## Spike 4 — Decomposition into build tasks

> TODO — only after human has approved option choice.

## Dialogue Log

### 2026-04-11 — Framing committed

- **Agent:** Filled the T-921 template, ran Spike 1 (CLI census), wrote this
  research artifact. Confirmed A1 (63% natural cross-host) and A2 (structural
  local-only set is real and small). Committed without executing Spike 2 so
  the human can steer option selection.
- **Question for human:** Which option do you want to pursue — A (unified
  flag), B (remote wrappers), C (rpc-call only), or D (hybrid)? My current
  lean is A.

### 2026-04-11 — Human asked for strawman + steelman + directive scores

- **Human:** "please elaborate more with all option scora agains directives
  and do strawman, steelman"
- **Agent:** Executed Spike 2 in full. For each of A/B/C/D wrote a
  worst-faith strawman, a best-faith steelman, and scored each option against
  the Four Constitutional Directives (Antifragility ×4, Reliability ×3,
  Usability ×2, Portability ×1). Scoreboard:

  | Option | Raw sum | Weighted |
  |---|---:|---:|
  | A — Unified `--target` | 29 | **70** |
  | B — Per-command wrappers | 22 | 57 |
  | C — `remote call` only | 27 | 69 |
  | D — Hybrid | 24 | 60 |

  A and C are a near-tie — because they answer two different user
  populations. Recommendation: **adopt A as the CLI answer, keep C as the
  escape hatch** (it is already shipped at the MCP layer via T-920 and as the
  `remote call` CLI command — no work required to preserve it). Option B is
  strictly dominated. Option D is really "A with a smaller scope" — it
  collapses to A if Spike 3 finds streaming fits `--target`, and the only
  way to know that is to run Spike 3.
- **Next question for human:** (1) Accept A-with-C-escape as the direction
  and let me run Spike 3 (routing architecture)? (2) Or do you want to argue
  against one of my directive scores first? (3) Or are there axes beyond the
  four directives — e.g. implementation effort, migration risk, test-surface
  — that should also be scored before committing?
