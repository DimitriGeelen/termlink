# arc-011 PH-1 B10 live check: a real Claude Code session receives a peer message

Task T-3411. Test: `crates/termlink-sidecar/tests/hooks_live.rs`
(`cargo test -p termlink-sidecar --test hooks_live -- --ignored --nocapture`). Run 2026-10-12.

## Result

**Pass.** One message went from agent A to a real Claude Code session acting as agent B,
through two real `termlink-sidecar` processes and a throwaway hub. The frame reached B's
model as a harness-written `hook_additional_context` record, B's sidecar recorded
HANDED_OVER from that record alone, and A's status showed HANDED_OVER with B's times.
No hook errors.

| Stage on A's chain | A's line time (ms) | B's time (peer_ts_ms) |
|---|---|---|
| SENT | 1791759513109 | |
| POSTED | 1791759513606 | |
| RECEIVED | 1791759515231 | 1791759513746 |
| STORED | 1791759515553 | 1791759514375 |
| HANDED_OVER | 1791759519931 | 1791759519320 |

B's chain for the message: RECEIVED, STORED, CLAIMED (`attempt=1 nonce=…`), HANDED_OVER
(same attempt and nonce). Both chains intact.

## Method

1. Throwaway estate (`/tmp/tl-estate-<hex>`): own HOME, runtime dir, hub with its own id,
   per-run keys. Two sidecar processes (`termlink-sidecar start --config`), each holding
   its store under the estate store root.
2. A sends one message over its send socket; B's receive loop stores it.
3. The session id is fixed up front (`--session-id`) and B's sidecar is told the
   transcript location before Claude starts, because the estate HOME carries no Claude
   credential (see "Limits").
4. `claude -p` (Claude Code 2.1.296, `claude-haiku-5-5`), cwd = the estate's disposable
   project dir, `--setting-sources ""` (no project or user settings), the hook given by
   `--settings`: PostToolUse → `termlink-sidecar hook post-tool-use` with the socket in
   its environment. Prompt: run `echo hi` with Bash, reply `done`. It replied `done`.
5. The hook pulled the message, printed the frame as `additionalContext` and started the
   detached finalizer; the finalizer asked B's sidecar, which read the transcript.

## Transcript evidence

After the Bash call: `user` (tool_result) → `attachment/hook_success` (the hook's own
stdout echo, which also contains the header) → **`attachment/hook_additional_context`**
(the frame) → the next `assistant` records. The order matches the B1 spike. The finalizer
keys only on `hook_additional_context`; the `hook_success` echo is not evidence (FC-11)
and the unit tests pin that.

## Found and fixed by this run

`termlink-sidecar start` opened its store at the estate root instead of the configured
store root (`guard_from_env` returns the estate root). Still inside the estate, so nothing
escaped, but the layout was wrong and the scripted demo could not see it (it only talks
to sockets). Fixed in `daemon.rs`; `hooks_scripted_demo` now asserts where each store is.

## Limits

- Claude ran with the operator's own login and HOME, as in the B1 spike. Its transcript
  was written to `~/.claude/projects/<encoded estate dir>/`; the test deleted that one
  directory afterwards (checked: none left). The estate tree was deleted on teardown.
- Because of that, the transcript location was registered by the test, not found by the
  sidecar under its own HOME. Finding it there is covered by `hooks_scripted_demo`. The
  run with two real sessions as the unprivileged `testdev` user, with its own Claude
  login, is B14 (OQ-3: the operator provisions `testdev` once).
- One agent was real; A was driven through its socket. Two real agents is B14.
