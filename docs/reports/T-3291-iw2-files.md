# T-3291 IW-2 — Why /var/lib/termlink/sessions holds ~10.9k entries

Measured 2026-09-30 ~23:20 CEST on .107, read-only. No `termlink list` / `clean` / `deregister` was run.
Hub: `termlink-hub.service`, MainPID 403200, active since 2026-09-26 00:00:34,
`Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink TERMLINK_SWEEP_INTERVAL_SECS=3600`.

## TL;DR

The leak is **not registrations**. Every `.json` in the dir belongs to a live `termlink register`
process. The accumulating entries are **10,144 orphan `*.sock.data` files**: the data-plane socket
of `register --shell` sessions. The hub supervisor sweep **is** running (every 30s) and **is**
removing dead sessions, but `liveness::cleanup_stale` deletes only `<id>.sock` and `<id>.json`,
never `<id>.sock.data`. Every other cleaner (doctor `--fix`, MCP clean) finds orphans by scanning
for `*.sock` files. Once cleanup_stale has removed the `.sock`, nothing can ever find the
`.sock.data` again. **Confidence: high.** 7,667 of the 8,944 orphans created since the journal
began carry a session id that the hub itself logged as "Cleaned stale session registration".

## 1. Classification of /var/lib/termlink/sessions (10,888 entries)

| Extension | Count |
|---|---|
| `.sock.data` (unix socket inode) | 10,386 |
| `.json` | 251 |
| `.sock` | 251 |
| other | 0 |

Grouped by session id:

| Set of files present | Ids |
|---|---|
| `.sock.data` only (**orphan**) | **10,144** |
| `.json` + `.sock` + `.sock.data` (live shell session) | 242 |
| `.json` + `.sock` (live non-shell session) | 9 |
| json without sock / sock without json | 0 |

The `.json` files (251):
- 251 parse (0 unparseable).
- 251 have a pid that is alive **and** whose `/proc/<pid>/cmdline` contains `termlink` (0 dead, 0 pid-reuse).
- All 251 have mtime < 1h, because the T-2230 heartbeat touches them every 30s.
- Name families are mixed orchestration workers: `tl-tN-rN` 15, `tN-reviewN` 10, `tN-roundN` 9,
  `pfN-rN` 7, and so on. None are dead.

Age of the orphan `.sock.data` files (by mtime):

| <1h | 1h–1d | 1–3d | 3–7d | 7–30d | >30d |
|---|---|---|---|---|---|
| 5 | 1,318 | 2,066 | 4,763 | 2,039 | 195 |

Orphans created per day: about 10–30/day from 2026-08-16 to 2026-09-21. Then a step change on
**2026-09-22**: 889, 1039, 450, 2217, 411, 1612, 1168, 857, 1321 per day through 09-30.

Process side: there are **513** live `termlink register` processes (504 with `--shell`).
- 251 have `TERMLINK_RUNTIME_DIR=/var/lib/termlink`. These map 1:1 to the 251 JSONs.
- 262 have neither `TERMLINK_RUNTIME_DIR` nor `XDG_RUNTIME_DIR`. They live in `/tmp/termlink-0` (§4).

## 2. What the sweep removes and what it skips (code)

- `crates/termlink-hub/src/server.rs:368-371` spawns `supervisor::run(supervisor::DEFAULT_INTERVAL, …)`.
  `DEFAULT_INTERVAL` is **30s** (`supervisor.rs:18`) and is not configurable.
- `supervisor.rs:36-45` sweeps `discovery::all_sessions_dirs()`. With `TERMLINK_RUNTIME_DIR` set,
  `all_runtime_dirs()` returns **only** that dir (`discovery.rs:44-46`, "exclusive override").
  The hub therefore sweeps `/var/lib/termlink/sessions` and nothing else.
- `supervisor.rs:92-98` → `manager::list_sessions_in(dir, true)` (`manager.rs:228-269`):
  - It iterates **only `*.json`** (`manager.rs:253`).
  - An unparseable JSON is warned about and skipped (`manager.rs:257-262`). It is never removed.
  - `.sock` and `.sock.data` files are never examined on their own.
- A session is dead when `liveness::is_alive` is false (`liveness.rs:13-26`): the pid does not
  exist (`kill(pid,0)`, where EPERM counts as alive), **or** the unix `.sock` path is missing.
  There is no cmdline check, so a reused pid keeps a registration "alive".
- Dead → emit `session.exited` to the live sessions → `liveness::cleanup_stale`
  (`supervisor.rs:159-161`).
- **`liveness.rs:44-55` `cleanup_stale` removes `reg.addr` (the `.sock`) and `<id>.json`. It never
  removes `<id>.sock.data`**, even though the path is recorded in the registration as
  `metadata.data_socket` (set at `crates/termlink-cli/src/commands/session.rs:328-331`).

Other cleanup paths, and why none of them catch the orphans:

| Path | Removes .sock.data? | Why it misses the orphans |
|---|---|---|
| `register` shutdown, `session.rs:475-490` | yes | runs **only** on `ctrl_c()` (SIGINT). SIGHUP/SIGTERM/SIGKILL (tmux kill, `exit` in the PTY, orchestrator teardown) skip it |
| `manager.rs:412` (`termlink list` / `clean`, L-019) | no | same `cleanup_stale` |
| `manager.rs:78` (register id collision) | no | same `cleanup_stale` |
| doctor `--fix`, `infrastructure.rs:396-420` | yes, but only alongside an orphan `.sock` | selects `ext == "sock"`. The extension of `x.sock.data` is `data`, and its `.sock` is already gone |
| MCP clean, `tools.rs:13700-13718` | same | same `ext == "sock"` selector |

The result is a one-way ratchet: once the `.sock` has been removed, no code path ever enumerates
the `.sock.data` again.

## 3. Is the sweep running in the live hub? Yes

- In `journalctl -u termlink-hub` (journal starts 2026-09-23 00:14) there are 845
  "Supervisor sweep complete" lines, 96–128 per day. The latest is 2026-09-30 21:59:40:
  `cleaned=1 total=243`.
- It logged "Cleaned stale session registration" for 7,686 distinct ids.
- **Correlation:** 10,191 orphan `.sock.data` files existed at scan time. 7,668 of their ids appear
  in the hub's cleaned log. Only 18 cleaned ids have no leftover `.sock.data`, and those are
  presumably non-shell sessions. For orphans created after the journal start the figure is
  7,667 of 8,944 (86%). The remaining ~1.3k were most likely cleaned by `termlink list`/`clean`
  (the same `cleanup_stale`, logged in the CLI process rather than the hub), by the 09-26 restart
  window, or lost to journald rate-limiting.
- `TERMLINK_SWEEP_INTERVAL_SECS=3600` is unrelated. It gates the **bus retention sweeper**
  (`retention_sweeper.rs:1-32`, `server.rs:373-380`), not the session supervisor.
- The running binary has the code path: its journal emits the `termlink_hub::supervisor` and
  `termlink_session::liveness` log lines above.

Dead-session names from the journal's "detected dead session" lines:

| Name family | Count |
|---|---|
| **`claude-master-<N>`** | **7,654** |
| `procasfit-rN-N` | 13 |
| `tNw-N-N` | 11 |
| other | ~10 |

`claude-master-<N>` is `.agentic-framework/bin/claude-fw:60` (`TERMINAL_SESSION="claude-master-$$"`).
With `--termlink`, it does `termlink spawn … --shell`. Its teardown (`claude-fw:81-87`) injects
`exit` into the PTY and runs `termlink clean`, and never sends SIGINT to the register process.
33 `claude-fw` processes are live, including cross-project fleet tmux launches such as
`/005-Yellowtwig/…/claude-fw --termlink --no-restart`. Examples from the journal:
`name="claude-master-3719045" pid=3719102`, a short-lived session.

## 4. Other runtime dirs

| Dir | Entries | .json | .sock | .sock.data |
|---|---|---|---|---|
| `/var/lib/termlink/sessions` | 10,888 | 251 | 251 | 10,386 |
| `/tmp/termlink-0/sessions` | 1,172 | 262 | 262 | **648** (≥386 orphans) |
| `/run/user/*/termlink/sessions` | none exist | | | |

`/tmp/termlink-0` holds the 262 register processes that have no `TERMLINK_RUNTIME_DIR`. **The
systemd hub never sweeps it** (`discovery.rs:44-46`). Its dead registrations are removed only by
ad-hoc CLI `list`/`clean`, and its `.sock.data` files leak the same way.

## 5. Root cause

1. **Primary gap (high confidence):** `liveness::cleanup_stale` (`liveness.rs:44-55`) does not
   remove `<id>.sock.data`. Every `--shell` session that dies by anything other than SIGINT leaves
   one behind after a *successful* sweep.
2. **No secondary reaper:** all orphan-socket scanners key on `*.sock`
   (`infrastructure.rs:403`, `tools.rs:13709`). A `.sock.data` whose `.sock` is gone is invisible
   to every code path.
3. **The SIGINT-only teardown in register** (`session.rs:475`) makes non-graceful death the
   normal case. Tmux kill sends SIGHUP, and claude-fw exits via an injected `exit`. Cleanup is
   therefore delegated to the hub, which has gap 1.
4. **Rate driver (medium confidence):** starting 2026-09-22, `claude-fw --termlink` launches
   create about 1,000 short-lived `claude-master-$$` shell sessions per day. At that churn, gap 1
   turns into roughly 1k files/day.
5. **Secondary:** the `/tmp/termlink-0` pool is outside the hub's sweep scope, because the
   explicit `TERMLINK_RUNTIME_DIR` disables multi-dir scanning.

The claim that the hub cannot keep up is **false**. The sweep keeps pace with deaths: 0 dead JSONs
remain. It simply does not delete the third file.

Suggested fix direction (not implemented here, IW-2 is read-only):
- remove `data_socket_path(sock)` in `cleanup_stale`;
- add a sweep arm that reaps a `*.sock.data` with no sibling `.json` (ideally only if a connect
  gets ECONNREFUSED);
- handle SIGTERM/SIGHUP in `register`.
