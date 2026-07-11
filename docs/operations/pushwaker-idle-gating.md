# Push-waker idle-gated injection (T-2402 Stage 3)

The push-waker (`scripts/be-reachable-pushwaker.sh`) rings a reachable agent by
**injecting** `/check-arc respond` into its PTY the instant a `dm.queued` /
`inbox.queued` frame lands. Before T-2402 that inject was **blind** — fired
regardless of what the REPL was doing. If the REPL was mid-turn, running a tool,
or sitting at the `claude --resume` conversation picker, the injected line was
**swallowed** (dropped, or worse, typed into a picker search box). That is the
root of the T-2400 "off=7" demo failure: the ring fired, the message was durably
delivered, but the doorbell never reached the agent's input — so it stayed silent.

Stage 3 makes the ring **idle-gated**: it injects only when the REPL is at a
READY prompt, and otherwise **defers and re-probes** on a bounded backoff until
the prompt returns to idle — never injecting blind.

## Detection mechanism — the PTY-state probe

There is no API on a running Claude Code REPL that reports "am I at a prompt?".
So we screen-scrape the terminal. The probe is `pushwaker_probe_pty` →
`pushwaker_pty_state` (pure, unit-tested):

```
termlink pty output <session> --bytes 2500 --strip-ansi
```

### Why a byte-TAIL, not `--lines`

The PTY is an **append-only stream of cursor-addressed redraws**, not a clean
screen snapshot. `--lines 200` replays the terminal's full alternate-screen
redraw history (observed: ~240 KB), which still contains a **stale**
`esc to interrupt` from the *previous* turn sitting in scrollback. A whole-blob
`contains` search misreads that as BUSY. The **most-recent** writes are at the
END of the stream, so a small byte-tail (`--bytes 2500`) reflects the CURRENT
screen state:

- A **running turn** repaints the spinner + `(esc to interrupt)` many times a
  second, so it dominates the last KB.
- An **idle prompt** repaints its status bar / footer (`? for shortcuts`,
  `new task? /clear to save …k tokens`, `Checking for update`) instead.

### The three-state classifier (fail-safe)

`pushwaker_pty_state` lowercases and strips **all** whitespace (strip-ansi mashes
cells together — `? for shortcuts` → `?forshortcuts`), then:

| Order | Match (whitespace-insensitive) | State | Action |
|---|---|---|---|
| 1 | `esctointerrupt` | **BUSY** | defer (a turn is running) |
| 2 | `esctocancel` / `resumesession` / `selectaconversation` / `loadingconversations` | **UNKNOWN** | defer (modal that would EAT the line) |
| 3 | `?forshortcuts` / `newtask?` / `checkingforupdate` / `/cleartosave` | **READY** | inject |
| 4 | *(anything else — raw shell prompt, empty read)* | **UNKNOWN** | defer |

The bias is deliberate: **only a positive idle marker yields READY.** A wrong
READY is a bad blind inject (the exact failure this stage kills), so every
ambiguity — picker, loader, raw shell, failed/empty read — resolves to UNKNOWN
and defers. Over-deferring is cheap (Stage 5 escalates a never-ready ring);
mis-injecting is not.

## The gated ring loop

`pushwaker_ring_when_ready <session> <text> <hub>`:

1. Probe the PTY state.
2. **READY** → `termlink inject … --enter`, return `0` (rung at idle).
3. **BUSY / UNKNOWN** → log a defer line, `sleep` the backoff, re-probe.
4. Repeat up to the attempt budget. If it never reaches READY, return `3` —
   **loud and un-injected** (the hand-off point for Stage 5's escalating re-ring
   / awaiting-ack registration). It does **not** fall back to a blind inject.

### Env knobs

| Var | Default | Meaning |
|---|---|---|
| `PUSHWAKER_READY_ATTEMPTS` | `30` | max probes before giving up (rc=3) |
| `PUSHWAKER_READY_BACKOFF_SECS` | `3` | sleep between probes |
| `PUSHWAKER_PTY_PROBE_BYTES` | `2500` | byte-tail size read per probe |

Defaults give ~90 s of patience per ring — enough for a normal turn to finish,
after which a genuinely stuck/absent REPL falls through to the loud rc=3 path.

## Tests

- **Unit** (`scripts/test-pushwaker-filter.sh`) — `pushwaker_pty_state` against 7
  fixtures taken from real Claude Code footer tails: idle prompt, idle status
  bar, running turn, busy-wins-over-stale-idle, resume picker, empty read, raw
  shell.
- **Integration** (`scripts/test-pushwaker-ready-loop.sh`) — hermetic fake-termlink
  shim scripts `BUSY,BUSY,BUSY,READY` across probes and records injects. Asserts
  the inject fires **exactly once and only after READY** (deferred 3× while
  busy), and that an always-busy REPL gives up (rc=3) with **zero** injects.

## Manual verification (live)

Confirm the probe matches ground truth against the running fleet:

```bash
export TERMLINK_BIN=termlink
BE_REACHABLE_PUSHWAKER_LIB=1 . scripts/be-reachable-pushwaker.sh
for s in workflow-designer aef; do echo "$s = $(pushwaker_probe_pty "$s")"; done
```

An idle agent reports `READY`; one mid-turn reports `BUSY`; one sitting at the
`claude --resume` picker reports `UNKNOWN`. Cross-check by eye with
`termlink pty output <s> --bytes 800 --strip-ansi | tail`.

To show a deferred ring landing after idle: start a long turn on a test agent,
then in another shell call
`pushwaker_ring_when_ready <session> "/check-arc respond" ""` — it logs
`not ready (state=BUSY … deferring)` until the turn ends, then
`rang … at READY prompt`.

## Related

- T-2400 — the off=7 blind-inject demo failure this stage fixes.
- T-2402 Stage 5 — escalating re-ring / awaiting-ack for the rc=3 give-up path.
- T-2316 / T-2324 — the push-waker inbox + dm rails this gates.
- `docs/operations/durable-reachable-auto-accept.md` — the auto-accept leg that
  lets the woken agent actually POST its reply.
