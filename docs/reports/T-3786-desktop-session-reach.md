# T-3786 — Reaching the 1409 design session in Claude Desktop

Inception (exploration only). Question: can mail reach project 1409's design session, which runs in the Claude
Desktop client as the non-root user `dimitri-mint-dev`, and how?

## Trigger

- 1409's root control session, 2026-11-14 (msg fee98d39): our message about T-3781 reached the root control
  session, not the design session; only the design session can run the requested `fw termlink dispatch` test, so
  the request went to it through the operator.
- Operator, 2026-11-14: "Wherever the design session is in the desktop client, I don't know if we can facilitate
  that kind of communication, it would be good though."

## Spikes

### S1 — what runs as dimitri-mint-dev (read-only, `ps --forest`, `pwdx`, 2026-11-14 ~12:00Z)

1. The "desktop sessions" are **Claude Code**, not Claude Desktop chats: `claude-desktop` (pid 616888) runs three
   copies of the bundled Claude Code binary (`~/.config/Claude/claude-code/2.1.321/.../claude`, pids 1026309,
   1354212, 783148) with `--input-format stream-json --output-format stream-json` (no `-p`).
2. All three run in `/opt/1409-sprind` with `--setting-sources=user,project,local`, so 1409's project
   `.claude/settings.json` hooks load. One resumes session `eef10490-…` (the long design session).
3. Each has its own MCP children, including `termlink mcp serve` (pids 223043, 1355346, 784062).
4. 1409's project settings carry the same mail hooks as ours: UserPromptSubmit `fw hook sidecar-inbox` +
   `sidecar-receiver-adapter`, Stop `sidecar-receiver-ready`, SessionStart `sidecar-autostart`.
5. The vendored `sidecar-inbox.sh` skips only headless sessions (`adapter._is_headless`: `-p`/`--print` in argv);
   the desktop sessions are not headless, so the hook runs on every prompt.

=> Assumption A-1 (Desktop chat) is **false**; A-2 (MCP only) is **false**; A-3 is moot.

### S2 — does mail reach those sessions? (transcript metadata only: counts and hook diagnostics, no content)

Transcripts `~dimitri-mint-dev/.claude/projects/-opt-1409-sprind/*.jsonl`:

| session | user lines | hook lines carrying peer mail | hook lines "NOT CHECKED" |
|---|---|---|---|
| eef10490 (design) | 1265 | 3 | 42 |
| dba4134c | 507 | 4 | 96 |
| dec8cb15 | 84 | 0 | 15 |
| 95bb23ac | 57 | 0 | 13 |

The NOT CHECKED reasons (478 dated lines, latest 2026-11-14T11:55:34Z, still occurring after today's installs):

- 637× `reading inbox:cacc…/1409-sprind/design: rc=1` — the design session HAS its own agent-level inbox
  (`1409-sprind/design`), and reading it fails;
- 228× `reading inbox:cacc…/1409-sprind: rc=1` — the project inbox, same failure;
- error text: `Hub is not running (no socket at /run/user/1000/termlink/hub.sock)`.

Cause: the hook's reader runs `termlink channel subscribe <topic> --json` with no `--hub`
(`.agentic-framework/lib/sidecar/inbox.py` default_reader). As `dimitri-mint-dev`, termlink resolves the hub to
that user's own runtime dir, where no hub runs. The user does have a working route to the real hub: TCP
`127.0.0.1:9100` with its own Interact token on the `127.0.0.1:9100` profile of its hubs.toml (T-3756, runme
22c), and since T-3781 a hub link file `~/.termlink/hub-register.addr` naming that address. Nothing uses it for
reads.

=> The address exists; mail to it is stored; the session cannot read it. A reader fix makes it arrive on the
design session's next prompt (surfacing on the next turn, as for our own sessions; a desktop session has no PTY,
so there is no inject/wake — the operator's next message is the trigger).

## Options

- **A. termlink falls back to the user's TCP hub.** When a client command gets no `--hub`, finds no local hub
  socket, and the caller is non-root with a hub link, it uses the link's hub address over TCP with the user's own
  credential for that address (hubs.toml profile). One rule in termlink; every caller (fw sidecar, MCP,
  scripts) benefits; no vendored change.
- **B. AEF passes `--hub`.** `fw sidecar` reads a per-user hub setting and passes `--hub` on every termlink
  call. Upstream change in AEF's code (G-062: never patched locally), AEF's release cycle.
- **C. Do nothing.** Mail for the design session keeps going through the root control session and the operator.

## Recommendation

GO, option A (bounded: one termlink resolution rule plus tests; reversible; covers every caller). Proof for the
build task: the design session's next prompts show no NOT CHECKED lines and a test message to `1409-sprind/design`
surfaces in its transcript (sentinel check, as the T-2876 prover does).

## Dialogue Log

- 2026-11-14, operator: "Wherever the design session is in the desktop client, I don't know if we can facilitate
  that kind of communication, it would be good though." Then "go ahead with T-3786".
