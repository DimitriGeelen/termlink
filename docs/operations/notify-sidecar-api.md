# notify-sidecar-api — the local control surface for this host's mailbox and prompt (T-3135)

**Arc:** arc-011 (file `.context/arcs/arc-011.yaml`), slices S1 + S12.
**Design:** `docs/design/arc-011-sidecar-api-architecture.md` §6 (the bright line), §8 (urgency).
**Rulings it is bound by:** SQ-1 (the injector is a TermLink *primitive*, local-control only),
SQ-4 (urgent shortens the WAIT, never the CHECK — nothing injects into a BUSY prompt),
SQ-8 (systemd-only respawn REJECTED; a portable fallback is REQUIRED).

## What it is

One script, `scripts/notify-sidecar-api.sh`, that answers five questions about **this host's own**
notify rail — the questions an operator could previously only answer by `cat`-ing flag files and
guessing. It invents no transport and no state: every verb wraps a seam that already exists and is
proven end to end.

| verb | answers | reads / calls | exit 0 | exit 1 |
|---|---|---|---|---|
| `status` | am I alive, when did I last cycle, is the hub reachable from here, who am I | `<agent>.heartbeat`, `<agent>.flag`, `<agent>.pid`, `.supervisor.pid/.heartbeat`, `termlink hub status`, own FQDN/IP | listener ALIVE | DEAF (heartbeat stale/missing) |
| `queue` | what is pending for this agent, in the order it will be injected | `~/.termlink/journals/journal.sqlite`, same predicate + ordering as `notify-injector.sh` | rows listed (or none) | — |
| `inject <id\|next>` | put this in front of my agent now | `notify-injector.sh` (READY-gated, SQ-4) | INJECTED + VERIFIED | deferred (BUSY/UNKNOWN), NOT-RUNNING, injected-not-verified, `<id>` is not next |
| `agent-state` | READY / BUSY / UNKNOWN / NOT-RUNNING | `scripts/lib/pty-state.sh` via `termlink output` | READY | anything else |
| `ack <offset> --evidence <kind>` | tell the sender it reached the prompt | `notify-ack-read.sh` (evidence mandatory) | posted / already acked | refused |

Exit **2** on every verb is usage or tooling — *I could not look* — and is never a verdict.
`--json` on every verb emits one object.

Common flags: `--agent-id NAME` (required), `--session NAME` (required by `inject` and
`agent-state`: the agent's registered PTY), `--notify-dir`, `--journal`, `--json`, `--quiet`.

### `queue` is hub-independent, and says so

`queue` bounds its rows by the **local** delivered marker only
(`$TERMLINK_NOTIFY_DIR/.<agent>.<topic>.delivered-offset`) and prints `watermark_source=local`. The
injector additionally maxes that marker with the hub's L3 `stage=read` watermark. So on a topic
with history and no local marker yet — the first live run measured **91 pending above watermark -1**
on a topic whose hub L3 stood far higher — `queue` OVERSTATES what the injector would hand over.
That is the price of being answerable while the hub is down (design §3), and it is labelled rather
than hidden. Run `inject next` (or let the injector's cron run) once and the local marker exists.

### `inject <id>` is deliberately narrow

The queue has a declared priority policy (T-3071/T-3072): priority band first, FIFO inside the
band. `inject <id>` therefore accepts only the id the policy says is **next**; asking for any other
id exits 1 and names the actual head. Jumping the queue would be a second ordering policy living in
a second place — the drift shape this repo has documented three times. To move a message forward,
raise its priority at the source; do not bypass the order here.

## The bright line (§6) — enforced, not remembered

The API may read and act on **this host's own state**. The moment it moves a message *between*
hosts it has become a second bus, which the charter refuses (non-goal #1, G-060). So:

- `--hub`, `--peer`, `--to`, or any remote address argument → **exit 2** with a message naming the line.
- `tests/notify-sidecar-api-fixtures.sh` statically asserts the script never invokes `channel post`,
  `agent contact`, `agent send`, `remote `, `artifact put`, or `broadcast`. This mirrors
  `crates/termlink-hub/tests/no_federation_tripwire.rs`: a future edit that adds one fails the suite.

## S12 — roles swap: the agent replies as sender

**Not carried by this API, on purpose.** A reply is a message to a *peer*, and per the bright line
that is `channel.post` through the hub — the path already proven end to end by T-3069/T-3079 and
wrapped by `scripts/agent-respond.sh` / `/reply`. The API's contribution to S12 is `ack`: the
receiving side reports L3 `stage=read` truthfully, which is what lets the sender's ledger
(`notify-ledger.sh`, S6) advance and the roles swap on a real signal rather than a guess.

**External blocker, recorded not resolved:** `framework-agent-systemd`'s systemd unit does not list
`termlink` in `--allowed-commands`, so that agent cannot run the reply path. That is another
project's configuration (T-559 boundary) and cannot be fixed from this repository.

## Portable respawn (SQ-8) — `notify-sidecar-supervisor.sh --loop` / `--emit-unit`

The supervisor (T-3050) was cron-driven. Cron is not systemd, but "who respawns the respawner" still
had exactly one answer per host. Two additions make the answer portable:

- `--loop [--loop-interval S]` runs the sweep forever in-process with its own pidfile and per-cycle
  heartbeat (`$TERMLINK_NOTIFY_DIR/.supervisor.pid` / `.supervisor.heartbeat`). It needs only bash.
  `--ensure-loop` starts a loop if none is alive (idempotent; pid-recycle guarded via `/proc`).
- `--emit-unit systemd|launchd|cron|auto` prints the host-native declaration that keeps the loop
  alive: a systemd unit with `Restart=always`, a launchd plist with `KeepAlive`, or cron `@reboot` +
  a `*/5` `--ensure-loop` re-check. `auto` detects what the host has and **says which it chose** on
  stderr; a host with none of the three exits 1 rather than printing nothing.

Installing the emitted unit is the operator's step. `status` reports the supervisor's own liveness,
so a dead respawner is a readable fact instead of something inferred from sidecars decaying.

```bash
bash scripts/notify-sidecar-api.sh status --agent-id claude-termlink
bash scripts/notify-sidecar-api.sh queue --agent-id claude-termlink --json
bash scripts/notify-sidecar-api.sh agent-state --agent-id claude-termlink --session claude-termlink
bash scripts/notify-sidecar-api.sh inject next --agent-id claude-termlink --session claude-termlink
bash scripts/notify-sidecar-api.sh ack 145 --agent-id claude-termlink --topic dm:abc:def --evidence observed-turn
bash scripts/notify-sidecar-supervisor.sh --emit-unit auto
```

Fixtures: `bash tests/notify-sidecar-api-fixtures.sh` (hermetic, PL-213 seams:
`SIDECAR_API_TEST_FQDN`, `SIDECAR_API_TEST_IP`, `SIDECAR_API_TEST_HUB_RC`, `SIDECAR_API_TEST_PTY_STATE`,
`SIDECAR_API_TEST_SESSION_STATE`, `SIDECAR_API_INJECTOR`, `SIDECAR_API_ACKER`).

## Addressing — who a message is for (T-3325, five-level circuit)

Co-resident agents can share one TermLink identity (T-1448), and with it the same `dm:`
topic and receipt watermark. Before T-3325 the sidecar woke its agent on any unread
message, including mail meant for a neighbour (framework:pickup 257 item 4). It now reads
an address on each unread message and wakes only for its own mail.

**The key is `metadata.to_circuit`**, the counterpart of the `from_circuit` peers already
send. The address is the five-level circuit agreed with AEF (D-599 / T-3433): host / hub /
project / session / agent. The operator ruled grammar option C on 2026-10-03: **read both
grammars, write the path form, switch writing to V9 when AEF cuts over.**

| Form | Example |
|---|---|
| path, from host | `//dimitrimintdev/cacc73ea32b121dd/010-termlink/@claude-termlink` |
| path, from hub | `cacc73ea32b121dd/010-termlink` |
| path, with session | `//host/hub/project/<session>/@agent` |
| AEF V9 | `aef::host=dimitrimintdev::hub=cacc73ea::project=010-termlink::@claude-termlink::` |

`@` marks the agent in both grammars, so a sender can leave out a level it cannot know.
For example, `agent-send.sh` writes `//<host>/<hub>/@<agent>` because presence carries no
project. A bare `metadata.to_project` (set by `termlink agent contact name:project`) is read
as a project-level address.

**The match is level by level, from level 1** (`scripts/lib/circuit.py`):

| Message says | Verdict | Wakes? |
|---|---|---|
| no address (or unparseable) | unaddressed | yes, as before T-3325 |
| host / hub / project different from ours | foreign | no |
| agent = us | mine | yes |
| agent ≠ us, and that agent is LIVE (fresh sidecar heartbeat here, or LIVE on agent-presence) | foreign | no |
| agent ≠ us, not live | fallback | yes: the ladder falls back to the deepest level that resolves, so nothing is dropped |
| stops above agent, every named level matches | mine | yes |

The session level is read but never decides the verdict: the sidecar serves an agent and has
no session identity to compare. Host comparison accepts a short name against an FQDN. Hub
comparison accepts a prefix of at least 8 hex characters (`sha256:` is stripped).

**Acks stop at foreign mail.** The receipt is a watermark on a shared identity, so acking past
a neighbour's message would mark it read for them. Auto-confirm acks only up to the message
before the first foreign one, and acks nothing when that falls below `first_unread`.

**The arrival record advances only on new mail for us.** `last_mail_ts` changes when
`last_mail_sig` (the per-topic newest offset that wakes us) changes. Mail of ours waiting
behind an unacked foreign message therefore does not re-wake the agent every cycle.

**Fail direction is always "wake".** If the classifier is missing, a fetch comes back short,
or a topic's `first_unread` is unknown, the sidecar uses the raw unread count and the old
advance-every-cycle record (a `?` in `last_mail_sig`). A broken filter causes a spurious
wake. It never causes a silent miss.

Overrides: `FW_SIDECAR_SELF_HOST` (default `hostname -f`), `FW_SIDECAR_SELF_HUB` (default the
local hub's TLS fingerprint, 16 hex), `FW_SIDECAR_SELF_PROJECT` (default `010-termlink`),
`FW_SIDECAR_LIVE_WINDOW_MS` (default 120000), `FW_SIDECAR_PRESENCE_AGENTS` (test seam that
replaces the presence lookup). Fixtures: `bash tests/notify-sidecar-circuit-fixtures.sh`.

**Open, outside this change:**
1. The project level is the folder name until AEF mints the project UUID (requested at AEF
   inbox @129).
2. Per-project signing identity, for trust rather than routing (AEF @53, proposal 3).
3. The fallback rule as written here is the operator's description; AEF has been asked to
   confirm it.
