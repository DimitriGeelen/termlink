# E2E Charter-Verb Validation Runbook (T-2511)

**Purpose.** A hands-on, step-by-step proof that TermLink's four charter verbs —
**discover / exchange durable messages / claim work / control terminal sessions** —
actually work end-to-end against the **current code** (HEAD), on an **isolated
scratch hub** with zero blast radius on any shared/production hub. Each step has a
copy-pasteable command, the expected output, and an explicit PASS criterion so an
operator can validate by hand.

This is the **manual, operator-driven** complement to the automated provers:

| Automated prover | Verb(s) | This runbook's step |
|---|---|---|
| `scripts/comms-selftest.sh` (T-2482) | discover + exchange | STEP 1 + 2 |
| `scripts/substrate-smoke.sh` (T-2151) | claim work | STEP 3 |
| `scripts/session-selftest.sh` (T-2485) | control terminal | STEP 4 |

Run the automated provers for a fast PASS/FAIL; run **this** when you want to *see*
each mechanic work, teach the substrate, or validate a fresh build before trusting
it.

> **Why HEAD, not the installed binary?** If you validate a stale binary and hit a
> broken mechanic, you may "fix" something already fixed in newer commits (chasing
> ghosts). Always validate the code you actually want to trust. Confirm with
> `termlink --version` vs the repo `VERSION`; if they differ, build HEAD first.

---

## 0. Prerequisites

```bash
cd /opt/termlink

# Build HEAD. NOTE: the CLI package is named `termlink`, NOT `termlink-cli`
# (the directory is crates/termlink-cli but the package name is `termlink`).
cargo build --release -p termlink          # or: cargo build --release
BIN=/opt/termlink/target/release/termlink
$BIN --version                             # confirm it matches repo VERSION
```

**Footgun — `-p termlink-cli` fails** with `package ID specification did not match`.
Use `-p termlink` (or plain `cargo build --release`).

### Scratch hub (isolated, zero blast radius)

```bash
# CRITICAL: keep the runtime_dir path SHORT. Per-session Unix sockets live at
# <runtime_dir>/sessions/<id>.sock and MUST fit in SUN_LEN (108 bytes). A deeply
# nested path (e.g. under a scratchpad) makes the hub start fine but session
# registration die with: "I/O error: path must be shorter than SUN_LEN".
export TERMLINK_RUNTIME_DIR=/tmp/tle2e
mkdir -p "$TERMLINK_RUNTIME_DIR"

$BIN hub start --tcp 127.0.0.1:9178 --json >/tmp/tle2e/hub-start.log 2>&1 &
sleep 1
$BIN hub status                            # expect: Hub: running, runtime_dir /tmp/tle2e
```

PASS: `hub status` reports running and the socket `/tmp/tle2e/hub.sock` exists.
Every command below assumes `TERMLINK_RUNTIME_DIR=/tmp/tle2e` is exported so the
CLI talks to *this* hub, not the shared one.

---

## STEP 1 — DISCOVER (find a peer)

A peer must exist before it can be discovered, so emit one presence heartbeat, then
query the roster.

```bash
export TERMLINK_BIN=$BIN
TERMLINK_CAPABILITIES="build,test" \
  bash scripts/listener-heartbeat.sh --agent-id e2e-worker --role claude-code --once

$BIN agent find-idle --json
```

**Expected:**
```json
{ "idle": [ { "agent_id": "e2e-worker",
              "capabilities": ["build","test"],
              "last_heartbeat_ms": <recent>,
              "role": "claude-code" } ],
  "ok": true }
```

**PASS:** the emitted worker appears with its role and capabilities intact, and it
is LIVE (find-idle filters heartbeats older than 60s).

**Footgun — `termlink agent listeners` does not exist.** Use `agent find-idle`
(or `scripts/agent-listeners-fleet.sh` / MCP `termlink_agent_listeners`). Only
`agent listen` exists, which is a different (subscribe) verb.

---

## STEP 2 — EXCHANGE (durable message round-trip + delivery confirmation)

```bash
# Post two durable messages (--payload carries the body; --ensure-topic auto-creates)
$BIN channel post e2e-chat --payload "hello-from-e2e" --ensure-topic --json   # -> offset 0
$BIN channel post e2e-chat --payload "second-message" --json                  # -> offset 1

# Read them back in order
$BIN channel subscribe e2e-chat --limit 10 --json

# Delivery confirmation frontier: unread -> ack -> unread
$BIN channel unread e2e-chat --json     # unread_count=2, first_unread=0
$BIN channel ack    e2e-chat            # writes a durable receipt, advances up_to
$BIN channel unread e2e-chat --json     # unread_count=0, up_to=1
```

**PASS:**
- Posts return **monotonically increasing offsets** (0, then 1).
- Subscribe-back returns both in order; `payload_b64` base64-decodes to the exact
  strings posted (content integrity).
- Unread drops from 2 → 0 after ack, and `up_to` advances `null → 1`
  (the receipt frontier moves forward).

**Footgun — `channel post <topic> <message>` (positional body) fails.** The body is
a flag: `--payload "<text>"` (or piped on stdin). A topic must exist first; use
`--ensure-topic` on the first post.

---

## STEP 3 — CLAIM (exclusive work reservation + lease lifecycle)

This step also demonstrates the **T-2510** fix live: renewing a long lease with a
tiny delta must **not** shorten it.

```bash
# Claim offset 0 with a 1-hour lease
$BIN channel claim e2e-chat 0 --claimer worker-A --ttl-ms 3600000 --json
#  -> capture claim_id and claimed_until

$BIN channel claims-summary e2e-chat --json          # active_count=1

# T-2510 CLAMP TEST: renew with a tiny +30s delta. claimed_until must NOT drop.
$BIN channel renew --claim-id <CID> --claimer worker-A --additional-ttl-ms 30000 --json
#  -> claimed_until unchanged (still ~1h out), NOT now+30s

# Exclusivity: a different worker cannot claim the same offset
$BIN channel claim e2e-chat 0 --claimer worker-B --ttl-ms 30000 --json
#  -> Error: offset 0 ... is already claimed by another worker

# Release, acknowledging completion (advances the cursor past the offset)
$BIN channel release --claim-id <CID> --claimer worker-A --ack --json     # ack:true
```

**PASS:**
- Claim returns a `claim_id` and a `claimed_until` ~1h in the future.
- **Renew keeps `claimed_until` ≥ its previous value** (monotonic-forward — the
  T-2510 clamp). If it drops to ~now+30s, the fix has regressed → double-dispatch
  risk.
- A second worker's claim on the same offset is **rejected**.
- Release with `--ack` returns `ack:true`.

---

## STEP 4 — CONTROL TERMINAL SESSIONS (spawn → exec → capture → cleanup)

The founding charter verb. Prove a command can be injected into a session, run, and
its output captured.

```bash
SENT="E2E_SENTINEL_$$"

# Spawn a shell session. Use --backend background on any host whose tmux SERVER
# environment may differ from your shell (see footgun below).
$BIN spawn --name e2e-sess --shell --backend background --wait --wait-timeout 15 --json
#  -> { "ok":true, "ready":true, "session_id":"tl-XXXXXXXX" }

# Inject a sentinel command and capture its output
$BIN exec tl-XXXXXXXX "echo $SENT" --json
#  -> { "ok":true, "exit_code":0, "stdout":"E2E_SENTINEL_...\n", "stderr":"" }

# Roster (NOTE: the verb is `list`, not `status` and not `list-sessions`)
$BIN list --json                                   # session shows state:ready

# Cleanup
$BIN signal tl-XXXXXXXX INT --json                 # { ok:true, signal:2 }
$BIN clean --json                                  # reaps the session
```

**PASS:** `exec` returns `ok:true`, `exit_code:0`, and the **exact sentinel** in
`stdout`. That is the inject→run→capture round-trip.

**Footguns:**
1. **Session roster verb is `termlink list`** — not `status --json` (different
   command) and not `list-sessions` (does not exist).
2. `signal <sid> TERM` may return "Session not found" if a prior signal already
   terminated the shell — send one signal and let `clean` reap the rest.

**Not a footgun (verified 2026-08-03):** `spawn --backend tmux` correctly reaches
an isolated runtime_dir even when a pre-existing tmux *server* carries a different
`TERMLINK_RUNTIME_DIR` in its global environment. `build_spawn_shell_cmd`
(execution.rs) prepends an explicit `export TERMLINK_RUNTIME_DIR=<dir>;` to the
command tmux runs, which overrides the server's global value inside the session's
shell. If `spawn` times out ("Timeout waiting for session to register"), suspect
the **SUN_LEN** path-length limit (§0) — a runtime_dir too long for a per-session
socket — not tmux env inheritance.

---

## Teardown

```bash
$BIN clean --json                                  # reap any lingering sessions
TERMLINK_RUNTIME_DIR=/tmp/tle2e $BIN hub stop      # stop the scratch hub
rm -rf /tmp/tle2e                                  # remove the scratch runtime_dir
```

---

## Findings scorecard (2026-08-03, HEAD 0.11.807)

All four charter verbs' **core mechanics are correct on HEAD**. Every issue
surfaced was an operator footgun or a CLI naming/discoverability gap — **no verb
was found broken**:

| # | Finding | Class | In this runbook |
|---|---------|-------|-----------------|
| 1 | `cargo build -p termlink-cli` fails (package is `termlink`) | build UX | §0 |
| 2 | Deep runtime_dir → session register dies on SUN_LEN | env/setup | §0 |
| 3 | `agent listeners` CLI verb does not exist (use `find-idle`) | naming | STEP 1 |
| 4 | `channel post <topic> <body>` positional fails (use `--payload`) | CLI UX | STEP 2 |
| 5 | Session roster verb is `list`, not `status`/`list-sessions` | naming | STEP 4 |

**Investigated and DISPROVEN** (not a bug): "tmux-server env inheritance
mis-routes `spawn --backend tmux`." Reproduced against a short-path scratch hub:
tmux backend spawns, registers on the isolated hub, and exec round-trips exactly.
`build_spawn_shell_cmd` already exports the runtime_dir into the tmux command. The
original apparent failure was the SUN_LEN long-path issue (#2), misattributed to
tmux. Kept here as a record so it is not "rediscovered".

**T-2510 clamp** (renew never shortens a lease) is proven live in STEP 3.
All four charter verbs' core mechanics are correct on HEAD — **no verb, and no
underlying mechanic, was found broken.** Every surviving finding is CLI naming,
build UX, or an OS/env setup constraint.
