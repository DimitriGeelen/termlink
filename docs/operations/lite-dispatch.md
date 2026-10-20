# Lite dispatch — short jobs through TermLink (T-3386)

Operator ruling RR-13 (T-3351, 2026-10-10; CR-21): all delegation goes through
TermLink and vendor sub-agents are not used. That ruling only holds if a
one-minute search through TermLink costs about what a sub-agent would. Today's
full worker does not: it is a whole project session. A lite worker is the same
`termlink dispatch` (same registration, same hub collect, same timeout), with a
stripped worker command.

## What the start costs (measured 2026-10-20, this host)

One trivial job, "Reply with exactly: OK", model `haiku`, run from `/opt/termlink`:

| worker | wall | input tokens (cache create + read) | cost |
|---|---|---|---|
| full: `claude -p` in the project | 15.6 s | 159,384 (148,253 + 11,131) | $0.1488 |
| lite: flags below | 5.0 s | 3,205 (3,205 + 0) | $0.0006 |

About **50× fewer input tokens and ~230× cheaper**, three times faster. The full
worker's cost is the project, not the job: CLAUDE.md here is ~315 KB, plus every
hook, MCP server and skill. A real lite job (Grep for which file defines
`collect_call_bound`, reply with the path) took 7.5 s, 2 turns, $0.0003, and
answered correctly.

Prices change; the token counts are the durable part. Re-measure:

```bash
cd /opt/termlink && unset CLAUDECODE
claude -p "Reply with exactly: OK" --output-format json --no-session-persistence --model haiku > /tmp/full.json
claude -p "Reply with exactly: OK" --output-format json --no-session-persistence --model haiku \
  --safe-mode --strict-mcp-config --disable-slash-commands --tools Read,Grep,Glob \
  --system-prompt "You are a short-job worker. Do the job, answer concisely." > /tmp/lite.json
# read .usage and .total_cost_usd from each
```

## The lite worker

Built by `crates/termlink-cli/src/commands/dispatch/lite.rs`:

| flag | removes |
|---|---|
| `--safe-mode` | CLAUDE.md, skills, plugins, hooks, MCP servers, custom agents (keeps normal auth; `--bare` would need an API key) |
| `--strict-mcp-config` (no `--mcp-config`) | any MCP server from anywhere |
| `--disable-slash-commands`, `--no-session-persistence` | skills; a transcript per job |
| `--system-prompt <short>` | the default system prompt (caller may supply its own) |
| `--tools Read,Grep,Glob` | everything else; read-only by default, caller may widen or pass none |
| `--model`, `--max-budget-usd` | optional |

The job prompt is one argv element, never spliced into a shell line. Prompts
over 100,000 bytes are refused before spawn (Linux rejects one argument over
128 KiB at exec time): put the material in a file and ask the worker to Read it.

**The result comes back as a file.** The worker writes into its job dir
(default `$TMPDIR/termlink-lite/<pid>-<ms>/`, never the project tree):

- `result.md` — the worker's final text
- `stderr.log`
- `exit_code`

and emits `task.completed` on its own session with
`{"result_path": "<job dir>/result.md", "exit_code": N}`, which dispatch's
collect returns like any other worker result.

## What a lite worker is not

- **Not governed by this project's hooks.** `--safe-mode` drops them, which is
  where the saving comes from. So a lite job is for *reading and answering*:
  the default tools cannot write, run commands or reach the network. A job that
  must edit the tree, commit or run tests is a full worker (`fw termlink
  dispatch`), and pays the full start.
- **One job per dispatch.** Fan out by dispatching several.
- **Not a review worker.** Review dispatch has its own pinned, signed start
  (T-3580); lite does not replace it.

## Status

The module, its tests (`cargo test -p termlink --bin termlink dispatch::lite`)
and this doc are in. The CLI flag (`termlink dispatch --lite --prompt …`) and
the vendored `fw termlink dispatch --lite` are proposed in the T-3386 task
Updates and not yet wired.
