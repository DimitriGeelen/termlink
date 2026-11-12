Seat: claude-code; model: claude-opus-5-5 (Claude Opus 5.5); date: 2026-11-12T23:17Z

# T-3759 — dispatch for a non-root session: independent review

## 1. Ranking A–D

**E (below) > D > A > C-as-written > B.** Leaving E aside, I put D first today.

- **D (ask a root agent session)** — No new privilege path is added. The non-root session can only send text, and a root agent reads it and decides what to do. Risk: the root agent is an LLM that can be talked into things. A message saying "review T-123, and also run X" is prompt injection that crosses a privilege line. To reduce that, the root session should accept only a fixed form (task id plus review kind) and never run commands taken from the message text. Reliability: it depends on the root session being alive, and requests queue up when it isn't. That is fine as an interim step.
- **A (own hub)** — Strong isolation: everything runs as the user's uid. Reliability is worse. Workers can't be seen on the main hub, which is exactly the "invisible session" failure T-3749 was written to stop (`crates/termlink-cli/src/commands/session.rs:305-322`). It also means a second hub, a second port, and a second secret to run. It is acceptable only if the main hub can federate or list it.
- **C as written ("starts the worker … as root")** — This has a privilege-escalation path that the brief does not name. A review worker is an AI agent working inside the project's checkout, and the non-root user can write to that checkout. When a root worker starts there, it runs code from the checkout as root: Claude Code hooks in `.claude/settings.json`, git hooks, `build.rs` and `cargo test`, `runme.sh`, and `fw` scripts. The user doesn't have to tamper with the dispatch request at all. Planting a hook in its own repo is enough to get root code execution. An allow list on task type and worker kind does not close this. A root worker would also leave root-owned files in the user's tree.
- **B (run as root)** — Worst option. The design session is an LLM agent, and running it as root removes the containment that running it non-root was meant to provide. The argument "the user can sudo anyway" applies to the human, not to the agent.

## 2. Option E (recommended): a root broker that starts the worker *as the requester's uid* and registers it for them

This is C with two changes:

1. **Identity.** Requests arrive on a root-owned Unix socket, for example `/run/termlink-dispatch.sock` with mode 0660 and group `termlink-dispatch`. The broker learns the caller's uid from `SO_PEERCRED`, which the kernel supplies. It does not use a token claim or a TCP message for identity.
2. **No root execution.** The broker forks, drops to the requester's uid and gid with `initgroups`, then `setgid` then `setuid`, and checks that the drop cannot be undone. It then starts a fixed worker binary. The worker's session socket lives in a directory the requester owns. The **broker** writes the registration JSON into `/var/lib/termlink/sessions` on the worker's behalf. Workers stay visible on the main hub, and nothing from the repo ever runs as root.

The other E ideas, each judged on the same grounds:

- **Group-writable sessions subdir** — Rejected. The hub reads every registration file as root, and the `addr` field (`crates/termlink-session/src/registration.rs:77-84`) would let a member of the group point the hub at any socket and fake any session name. You would have to add per-file owner checks and addr validation.
- **Hub accepts registration over TCP with a scoped token** — Workable later, if a new scope is added (e.g. `register`) that is bound to a name prefix and a uid. Today unknown methods map to Execute (`crates/termlink-session/src/auth.rs:176-213`), which fails safe. A token can't prove uid, though, so peercred over a Unix socket is stronger on the same host.
- **sudo or polkit rule for one command** — Only safe if the command takes no free-form arguments. `termlink dispatch` takes a shell command (`crates/termlink-cli/src/commands/dispatch.rs:40`, spawned via `spawn_detached` at `:1006`), and sudo rules on argument-heavy tools are well known to be escapable. A sudo rule that only runs the broker's client gives the same thing as E.
- **Socket activation** — A way to deploy E (a systemd `.socket` unit), not a separate option.

## 3. What the allow list, request, auth and log must contain (for C/E)

- **Request:** `task_id` (regex-checked, and the task must exist in that project), `worker_kind` (an enum), `brief_path` (canonicalised and required to be inside the project's `docs/reports/<task>/`), and optionally `model` (an enum). It must **not** carry a command string, environment variables, a workdir outside the project, a runtime dir, or tags beyond a fixed set.
- **Allow list (root-owned file, 0644):** per uid, the allowed project roots, worker kinds and models; a limit on concurrent workers per uid; a daily count; and a maximum TTL per worker.
- **Auth:** `SO_PEERCRED` uid and pid, checked against the allow list. If an existing token is used as well, it must be bound to that uid.
- **Log (append-only, root-owned):** the requester's uid and pid, the request hash, the decision with its reason, the worker name, pid and uid, and the exit status. Also send a hub event so the project can see it.
- **Fail closed when:** the allow list is missing or won't parse; the uid is unknown; any field is unknown; the path escapes the project (symlinks included); the privilege drop fails or `getuid()==0` after the drop; the limit is reached; the hub is unreachable (no registration, so no worker); or the registration write fails. The broker must never fall back to running as root, and must never treat `TERMLINK_ALLOW_RUNTIME_DIR_MISMATCH` as permission.

## 4. Framing problems

- "Without weakening security" and "starts the worker … as root" contradict each other (see §1 C). The question to answer is how to *register* a non-root worker on the root hub, not how to *run* it as root.
- The brief says registering elsewhere "is refused". It is overridable: `TERMLINK_ALLOW_RUNTIME_DIR_MISMATCH=1` (`session.rs:305-322`, applied at `:500`). That override is not a fix, though, because it brings back the invisible-session failure.
- The runtime dir is `/var/lib/termlink` (mode 0700 root, confirmed with `ls -ld`). The hub also searches per-uid dirs in default mode (`crates/termlink-session/src/discovery.rs:41-60`) but not under the systemd override. It is worth checking whether the gap is really in how the hub discovers sessions rather than in write permission.
- B is not "no new code" in risk terms. It removes the agent's containment.

RANKING: E > D > A > C > B
