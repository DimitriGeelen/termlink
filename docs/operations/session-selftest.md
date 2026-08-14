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
| **PTY_SPAWN** (T-2695) | a session with a **real PTY** can be registered | `termlink spawn --name <s>-pty --shell --backend tmux --wait` |
| **OUTPUT** (T-2695) | PTY content **streams back** — the charter's "stream output" claim | `termlink output <s>-pty --strip-ansi --json` → `ok` + non-empty `output` |
| **INJECT** (T-2695) | injected keystrokes are **interpreted by the shell** — the charter's "inject keystrokes" claim | `termlink inject <s>-pty "echo INJ'-'<nonce>" --enter` → the **unquoted** result appears in `output` |
| **CLEANUP** | best-effort teardown of BOTH sessions (never fatal) | `signal TERM` ×2 + `clean` |

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

**Why a SECOND session for the PTY stages (T-2695).** STAGE 1 spawns
`-- sleep <ttl>`, which registers a session with **no PTY** (`termlink status` reports
`pty: null`). `termlink output` correctly refuses it with `-32007 No PTY session`, and
`inject` cannot reach a terminal through it. Reuse is therefore structurally
impossible, so the PTY stages spawn their own `--shell` session. The sleep-backed
session is left exactly as it was, so the pre-existing stages — and the T-2557 canary
that runs them daily — carry zero regression risk. Both sessions are reaped on every
exit path, including PTY-stage failure.

**Why the injected sentinel is oddly quoted (T-2695).** Injecting `echo FOO` makes
`FOO` appear **twice** in the PTY: once as the terminal's echo of the keystrokes, and
once as the command's output. A naive `grep FOO` would therefore pass on the echo
alone — proving the bytes *arrived* but not that the shell *interpreted* them, which
is exactly the weakness of the existing `command_inject_resolves_keys_no_pty` unit
tests (they resolve key names with no PTY attached). So the injected text embeds shell
quoting that the shell strips: the typed line reads `echo INJECT-PROVEN'-'<nonce>`
while the output line reads `INJECT-PROVEN-<nonce>`. Matching the **unquoted** string
can only match the interpreted result.

**Why OUTPUT runs before INJECT.** If the observation channel is broken, inject's
effect is unobservable — reporting `broken_stage: INJECT` would then be a wrong
diagnosis. OUTPUT is proven first so a failure is attributed to the verb that actually
broke.

**What building these stages found.** The INJECT stage was written for the T-2694
review, which decomposed the charter's terminal-endpoints noun into the four
capabilities it actually lists (stream output, inject keystrokes, exec, doorbell-wake)
and found this prover exercised only `exec` — 1 of 4 — while its own header quoted the
acknowledged inject gap without closing it. Building the stage immediately surfaced
**T-2697**: `termlink inject` returned `{"ok":true,"bytes_injected":18}` for a complete
no-op against a session with no PTY, because the CLI discarded the handler's own
`status:"resolved"` and its remedy `note`. That defect had been invisible precisely
because no prover exercised the verb.

## Exit codes

- **0** proven — SPAWN, EXEC (incl. not-truncated), EXEC_EXITCODE, and the PTY stages (PTY_SPAWN, OUTPUT, INJECT) all PASSed: the verb works now. A PTY stage that was *skipped* (test-hook mode without its seam set) does not block `proven`; one that FAILed does.
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
- `TERMLINK_SESSION_SELFTEST_TEST_OUTPUT_STATUS=<PASS|FAIL>` (T-2695) — canned OUTPUT verdict.
- `TERMLINK_SESSION_SELFTEST_TEST_INJECT_STATUS=<PASS|FAIL>` (T-2695) — canned INJECT verdict.
  Leaving both unset makes the PTY stages report `skipped`, which never blocks
  `proven` — so pre-existing SPAWN/EXEC-focused harness cases stay green untouched.
- `SESSION_SELFTEST_PTY_ATTEMPTS` / `SESSION_SELFTEST_PTY_SLEEP` (T-2695) — PTY
  readiness retry budget, mirroring the EXEC pair.

## Related

- `scripts/comms-selftest.sh` — proves discover + exchange (`docs/operations/comms-selftest.md`).
- `scripts/substrate-smoke.sh` — proves claim work (T-2151).
- `docs/CHARTER.md` — the four verbs this prover set collectively covers.
