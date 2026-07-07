# arc-004 push-wake — fleet activation runbook (T-2381)

**Status of the mechanism:** WORKS on the current binary (0.11.408). Both rails
verified PASS on 2026-07-07 via the hermetic E2E harness
(`scripts/demo-pushwaker-e2e.sh` inbox rail; `scripts/demo-dm-rail-pushwake.sh`
dm rail) — a DM/inbox deposit push-wakes a real PTY in ~85–111 ms with correct
addressee filtering (no false wakes).

**Status in the field:** DORMANT fleet-wide. The code is sound; nobody meets its
preconditions. This runbook is how you make it live per host.

Related: root-cause evidence in
`docs/reports/T-2380-comms-confirm-ack-field-gap-inception.md` (E4); the
confirm/ack half is the T-2380 inception.

---

## Why it's dormant (the three stacked preconditions)

1. **Injectable PTY required (the load-bearing one — PL-237).** The pushwaker
   is spawned only when `be-reachable` has a bound `pty_session`
   (`scripts/be-reachable.sh:296`). `default_pty_session()`
   (`be-reachable.sh:150-167`) returns non-empty **only** inside tmux (`$TMUX`)
   or screen (`$STY`). A plain `claude --resume` process (e.g. `TERM=linux`, no
   multiplexer) gets **no pushwaker at all** and silently falls back to the ~15 s
   poll floor. Dormancy is logged loud — `pushwake_dormancy_warn()`
   (`be-reachable.sh:55-68`) prints `push-wake DORMANT — no pty_session bound
   (poll-floor only)` — but nothing surfaces that warning to an operator.
   The waker rings by calling `termlink inject <pty_session> "/check-arc respond"
   --enter` (`be-reachable-pushwaker.sh:128`); with no injectable PTY name there
   is nothing to ring.

2. **DMs must land on the recipient's LOCAL hub.** `cmd_start` spawns the
   pushwaker **without** `--hub`, so both rails subscribe on the **local** hub
   (`be-reachable.sh` run_waker, ~178-184). A DM posted to a *different* hub than
   the one the recipient's waker is subscribed to will never push-wake them —
   it just sits durably. (This is the same hub-targeting issue T-2380 E1 raises
   for the confirm/ack half; the sender must target the recipient's home hub.)

3. **Presence must be published (PL-200).** After vendored binary swaps the
   `agent-presence` emitter (`listener-heartbeat.sh`) is not auto-restored, so
   peers can't even discover the agent to DM it. Diagnostic:
   `bash scripts/agent-listeners.sh --hub <addr>:9100 --include-offline --json`
   → `total_listeners=0` means this gap is present.

**Good news on the shared-host fp trap (PL-236/PL-166):** the dm rail resolves
the per-agent self-fp via `termlink agent identity --resolve --json`, honoring
the exported `TERMLINK_AGENT_ID` (`be-reachable.sh:290-293`, set at :234). So on
a shared host (all sessions signing as one host fp), the dm rail still addresses
the correct agent — the verifier confirmed distinct per-agent identities
(`7054902c` receiver vs `4ce1f0cb` poster) push-wake correctly. This sidesteps
the trap **on this path** (it does not fix `identity show`, which still
misreports — PL-236).

---

## Activation recipe (per agent)

The agent must run inside an injectable PTY. Two supported shapes:

**A. Agent runs inside tmux/screen (pty_session auto-binds):**
```
tmux new -s <agent-name>          # launch the agent's claude inside this
# then, from /opt/termlink, inside that tmux session:
bash scripts/be-reachable.sh start --agent-id <agent-name> --pty-session <agent-name>
```

**B. Explicit injectable PTY (no multiplexer):**
```
bash scripts/be-reachable.sh start --agent-id <agent-name> --pty-session <injectable-session-name>
```
where `<injectable-session-name>` is a session `termlink inject` can drive
(e.g. a `termlink spawn --shell --backend tmux` session, as the demos use).

## Verify it's live (not dormant)
```
bash scripts/be-reachable.sh status
# expect: push_waker: running (pid …)   — NOT "push-wake DORMANT"
```
Then confirm both rails:
- `pushwaker_pid` non-null in the be-reachable state → inbox rail armed.
- `self_fp` non-empty in the state + `watching dm.queued` in the be-reachable
  log → dm rail armed (not "dm rail disabled").

## Deliver a DM so it push-wakes
Post to the recipient's **local** hub:
```
termlink agent contact --target-fp <recipient-fp> --hub <recipient-home-hub> --message "..."
```
The recipient's local-hub waker receives the `inbox.queued`/`dm.queued` frame and
injects `/check-arc respond` into their PTY within ~100 ms.

---

## The fleet reality (why a runbook isn't enough on its own)

The actual fleet agents are plain `claude --resume` processes **not** launched
inside tmux/screen (verified on .107: `TERM=linux`, `TMUX` unset, be-reachable
not running — for all 8 sessions). So "make arc-004 work as intended across the
fleet" has a fork, and it is a **decision**, not just a rollout:

- **Operational path:** mandate that every fleet agent is launched inside tmux
  (or a `termlink spawn` PTY) and armed with `be-reachable start --pty-session
  --agent-id` at session start. Cost: change the launch convention for every
  host; retrofitting means relaunching running agents.
- **Structural path (needs a GO):** teach `be-reachable`/the agent launcher to
  auto-allocate an injectable scratch PTY for a headless agent so push-wake works
  **without** tmux. This is the true "just works as intended" fix. Folded into
  T-2380 as candidate **C7**.

Recommendation: adopt the operational path now (this runbook) for hosts we can
relaunch, and take C7 (auto-PTY) through the T-2380 go/no-go for the permanent
"headless agents just work" fix.

## Fleet activation state (2026-07-07)
| Host | Agent shape | Push-wake | Action |
|------|-------------|-----------|--------|
| .107 | 8× plain `claude --resume` + 5 bare `--shell` | DORMANT (no tmux, no be-reachable) | relaunch-in-tmux + arm, or C7 |
| .122 | ring20-management-agent | unknown — coordinate | ask ring20-manager to arm per this runbook |
| .121 | (no session yet — T-2379) | n/a | session first (T-2379), then arm |
| .141 | unreachable | n/a | — |
