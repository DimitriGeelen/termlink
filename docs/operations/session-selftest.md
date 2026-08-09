# Session self-test — proving the "control terminal sessions" charter verb

`scripts/session-selftest.sh` is the affirmative on-demand prover for the fourth
charter verb: **control terminal sessions**. It is the direct sibling of
`comms-selftest.sh` (which proves discover + exchange) and `substrate-smoke.sh`
(which proves claim work) — together the three cover all four charter verbs with a
"prove it works right now" command, complementing the after-the-fact canaries.

## Why it exists

TermLink *began* as a cross-terminal session-control tool (`docs/CHARTER.md`), yet
this verb was the one with **no affirmative prover**. The gap was acknowledged
in-code: `agent-conversation-selftest.sh` states "What it does NOT validate: PTY
inject", and `comms-selftest.sh` only checks the `pty_session` presence **flag**,
never that a command actually injects and runs. So nothing proved, on demand, that
you can register a terminal session and exec into it right now — the G-069
shipped≠live class applied to the PTY verb.

Unlike doorbell-wake (which needs a live peer), `termlink exec <session> <cmd>
--json` makes this **deterministic and local**, so the prover is self-contained.

## Usage

```bash
bash scripts/session-selftest.sh            # human report, exit 0/1/2
bash scripts/session-selftest.sh --json     # {ok, proven, broken_stage, stages, session, sentinel}
bash scripts/session-selftest.sh --ttl 60   # scratch-session lifetime (default 30s)
bash scripts/session-selftest.sh --hub <addr>
```

## What it proves — staged PASS/FAIL checks

| Stage | Proves | How |
|-------|--------|-----|
| **SPAWN** | a terminal session can be registered on this host | `termlink spawn --name <s> -- sleep <ttl>` |
| **EXEC** | a command injects into and runs in that session, output+exit captured, **not truncated** | `termlink exec <s> 'echo <sentinel>' --json` → `ok` + `exit_code 0` + sentinel in `stdout` + `truncated != true` |
| **EXEC_EXITCODE** | a real non-zero exit code **propagates faithfully** (exit-code fidelity) | `termlink exec <s> 'sh -c "exit 7"' --json` → `exit_code == 7` |
| **CLEANUP** | best-effort teardown (never fatal) | `signal TERM` + `clean` |

`spawn` returns before the tmux-backed shell is guaranteed exec-ready, so the EXEC
stage retries up to ~5s (`SESSION_SELFTEST_EXEC_ATTEMPTS` × `SESSION_SELFTEST_EXEC_SLEEP`)
to absorb the occasional slow start. A ready session passes on attempt 1 at no cost;
a genuinely broken verb still FAILs after the budget is spent.

**Why EXEC_EXITCODE + the truncation check (T-2563).** The happy-path EXEC only ever
runs `echo` (always exit 0), so on its own it cannot catch two silent-success
regressions: (a) a bug that pins `exit_code:0` for *all* commands — a peer running
`exec 'deploy.sh'` would then read success on failure; and (b) a truncated capture
that, in the ~64 KiB band around the output cap, reads back as `exit_code:0` with the
sentinel still present (executor.rs's own comment admits this). The negative
EXEC_EXITCODE stage proves the real code propagates; the `truncated != true` assertion
on EXEC proves the capture was complete. Note: `termlink exec --json` sets `ok` to
whether the *command* succeeded (exit 0), so the negative stage legitimately sees
`ok=false` — the discriminator is the exact `exit_code == 7`, not `ok`.

## Exit codes

- **0** proven — SPAWN, EXEC (incl. not-truncated), and EXEC_EXITCODE all PASSed: the verb works now.
- **1** broken — a stage FAILed; the broken stage is **named** (not silent).
- **2** tooling — missing `termlink`/`jq`, or the local hub is down (**fail-closed**:
  an unprovable environment is never reported "proven").

## Test hooks (PL-213 — host-independent)

The unit suite `scripts/test-session-selftest.sh` drives every path with no live
hub via:

- `TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC=<int>` — canned spawn exit code.
- `TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON=<json>` — canned `exec --json` output.
- `SESSION_SELFTEST_SENTINEL=<token>` — pins the sentinel so a canned EXEC output can
  carry it.

## Related

- `scripts/comms-selftest.sh` — proves discover + exchange (`docs/operations/comms-selftest.md`).
- `scripts/substrate-smoke.sh` — proves claim work (T-2151).
- `docs/CHARTER.md` — the four verbs this prover set collectively covers.
