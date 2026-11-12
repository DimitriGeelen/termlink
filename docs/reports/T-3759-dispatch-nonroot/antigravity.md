Seat: antigravity; model: Gemini 4 Argon (Medium); date: 2026-11-12T23:24:00Z

## 1. Ranking A–D

**Ranking: C > D > A > B** (with **E** below superior to C when workers do not need root).

- **C (Root-owned dispatch service with strict allowlist) — 1st**:
  - *Security*: Enforces a deterministic, code-checked policy boundary before any process starts, unlike D (an LLM interpreting untrusted mail) or B (full root session).
  - *Reliability*: Automated, no human or LLM relay delay, and workers register on the canonical hub so discovery (`crates/termlink-hub/src/router.rs:185-260`) and `session.exited` supervision (`crates/termlink-hub/src/supervisor.rs:158-250`) work normally.
  - *Risks and privilege-escalation paths in C*: A non-root caller causing root to run `fw termlink dispatch` (`.agentic-framework/agents/termlink/termlink.sh:2212-2225`) creates four privilege-escalation vectors unless mitigated:
    1. **Repo/git execution as root**: `run.sh` `cd`s into `PROJECT_DIR` (`termlink.sh:1622`) and `cmd_dispatch` runs `git -C "$project_dir"` (`termlink.sh:1262,1919`). If the non-root caller can write to `project_dir/.git/config` (`core.fsmonitor`, `core.sshCommand`), `.git/hooks/`, `.claude/settings.local.json`, or `.mcp.json` (`termlink.sh:1827-1851`), root executes caller-controlled code.
    2. **Worker tools running as UID 0**: `claude`, `codex`, `opencode`, and `ollama-loop` run under the spawning UID (`termlink.sh:1898,1957,1965,2039`); only `antigravity` drops privileges via `sudo -n -u dimitri-mint-dev` (`termlink.sh:1970`). Any non-review dispatch or sandbox escape gives UID 0 shell execution.
    3. **Symlink/file clobber**: Root writes `meta.json`, `result.md`, and `.context/dispatch-results/` (`termlink.sh:1537,2101-2104`). In a user-writable tree, symlink races allow root file overwrites.
    4. **Argument/PTY injection**: `cmd_dispatch` injects `exec bash $wdir/run.sh '$name' ...` into a root PTY (`termlink.sh:2225`) and sources `env.sh` for non-review tasks (`termlink.sh:1801`). Unsanitized `--name`, `--env`, or `--tools` yields root command execution.
- **D (Relay through a root-run project session) — 2nd (short-term stopgap)**:
  - *Security*: Avoids new root daemons today, but relies on an LLM as a security boundary. Prompt injection in peer mail or reviewed diffs can trick the root session into running arbitrary commands, and running `git`/`fw` as root in a non-root repo carries repo-poisoning risk.
  - *Reliability*: Fragile; depends on the root agent being alive, idle, and acting on mail (`CLAUDE.md:101`) without context drops.
- **A (Per-user hub) — 3rd**:
  - *Security*: Strong OS isolation; everything runs under the non-root UID.
  - *Reliability*: Breaks fleet single-store invariants. Workers are invisible on the main hub, and `.agentic-framework/lib/termlink-runtime.sh:71-115` (`fw_termlink_hub_discovered_dirs`) sees both the systemd root hub and user hub, returning `ambiguous` (`termlink-runtime.sh:114`).
- **B (Run design session as root) — 4th**:
  - *Security*: Total privilege collapse. Grants an interactive agent unconditional UID 0 access to `/var/lib/termlink/hub.secret` (`crates/termlink-hub/src/server.rs:104-106`) and the host, defeating the scoped token boundary (`crates/termlink-cli/src/commands/infrastructure.rs:822-825`).
  - *Reliability*: Works immediately, at the cost of host security.

## 2. Option E: Missed Alternatives

1. **Why a group-writable `sessions/` directory fails**:
   - `Session::register_bound_in` forces `set_permissions(sessions_dir, 0o700)` (`crates/termlink-session/src/manager.rs:72-75`), failing with `EPERM` for non-owners.
   - Every session's Unix socket server rejects peers where `creds.uid != owner_uid` (`crates/termlink-session/src/server.rs:258-268`; `crates/termlink-session/src/auth.rs:55-57`). The root hub (`uid 0`) cannot connect to a non-root session's Unix socket, nor can a non-root caller connect to a root-spawned worker's socket.
2. **Option E1 (Unprivileged local worker + TCP remote registration on the main hub)**:
   - The hub *already* permits `session.register_remote`, `session.heartbeat`, and `session.deregister_remote` over TCP with an `Interact` token (`crates/termlink-hub/src/server.rs:796-798`; `crates/termlink-hub/src/router.rs:774-877`), including `worker_binding` uniqueness (`router.rs:808-861`) and discovery (`router.rs:243-258`).
   - If the non-root user owns the project and verdict key (`.agentic-framework/lib/verdict_ledger.py:78-80`), run workers as that non-root user and register on the main hub over TCP via `session.register_remote`. Zero root execution risk.
3. **Option E2 (Privilege-dropping dispatch daemon over a Unix socket)**:
   - If root must sign verdicts in a root-owned project (`verdict_ledger.py:162-167`), C's daemon should validate the request as root, export the `git archive` tree (`termlink.sh:1919`), and drop privileges (`setuid`/`setgid` or `systemd-run --uid=`) for the worker process.

## 3. Requirements for C (or E2)

- **Allow list**: Caller UIDs and `identity_fingerprint`s (`crates/termlink-session/src/registration.rs:28-36`); canonical root-owned `project_dir` paths; `task_type` pinned to `["review"]`; `worker_kind` restricted to `claude|codex|opencode|antigravity` from committed `policy/review-backends.yaml` (`termlink.sh:1264-1270,1325-1329`); `TERMLINK_MAX_WORKERS` cap (`termlink.sh:572-584`) and max `timeout`.
- **Request schema**: Strict JSON: `project`, `task` (`^T-[0-9]+$`), `task_type` (`"review"`), `revision` (`^[0-9a-f]{40}$`), `worker_kind`, `seat`, `run_id`, and `brief_relpath` (read via `git show <revision>:<brief_relpath>`). Forbid `--command`, `--env`, `--tools`, `--mcp-config`, `--model`, and `--workdir`.
- **Authentication**: Dedicated Unix socket using kernel `SO_PEERCRED` (`PeerCredentials::from_tokio_stream`, `crates/termlink-session/src/auth.rs:50-80`) for unforgeable `(uid, gid, pid)`. If over TCP, bind a dedicated token scope to `identity_fingerprint` — `TokenPayload` (`auth.rs:232-245`) currently lacks caller identity.
- **Logging**: Append-only JSONL recording timestamp, caller `uid`/`pid`/`identity_fingerprint`, `dispatch_id`, `project`, `task`, `revision`, `worker_kind`, resolved `worker_bin` + `model` (`termlink.sh:1697-1719`), decision (`allow`/`deny` + reason), and `exit_code`/`close_state`.
- **Fail closed**: Failed `SO_PEERCRED` (`crates/termlink-hub/src/server.rs:953-965`), unknown fields, uncommitted `revision`/`brief` (`verdict_ledger.py:44-74`), missing committed `policy/review-backends.yaml`, worker cap exceeded, or unwritable audit log (`crates/termlink-session/src/server.rs:56-61`).

## 4. What Is Wrong in the Brief's Framing

1. **Directory permissions are not the only blocker**: Cross-UID Unix socket checks (`crates/termlink-session/src/server.rs:258-268`) reject `0 != owner_uid` in both directions, blocking root-hub forwarding to non-root sessions and non-root `termlink pty inject`/`event wait` against root workers.
2. **Detection paradox on `0700` `/var/lib/termlink`**: `running_hub_dir_for_register()` (`crates/termlink-cli/src/commands/session.rs:287-300`) checks `/var/lib/termlink/hub.pid` (`crates/termlink-doctor/src/lib.rs:539-541`). On a `0700` root dir, non-root `read_pid` gets `EACCES` (`PidfileStatus::NotRunning`, `crates/termlink-hub/src/pidfile.rs:34-35`); `register` only refuses because `termlink.sh:342-358` first exports `TERMLINK_RUNTIME_DIR` from `systemctl show` (`termlink-runtime.sh:73-76`).
3. **Workers should not run as root**: Starting review workers as UID 0 violates least privilege; C should only broker registration/ledger signing and run the worker unprivileged.

RANKING: C > D > A > B
