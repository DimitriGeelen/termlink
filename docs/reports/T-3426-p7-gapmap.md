# T-3426 / P7 gap map: real-agent wiring of arc-011 (task T-3438)

Read-only survey, written 2026-10-13. Nothing was started, stopped, committed or changed. Citations are `path:line` against the working tree. Items marked UNVERIFIED were not confirmed by a read or a run.

## 1. Per-agent signing keys (PR-1a, SI-11)

- **How a sidecar gets its key.** `daemon.rs:31` `Config.key_file` names the file. `daemon.rs:97` loads it with `Identity::load_or_create_from_file`, so a missing file is created silently.
- **Key resolution.** `termlink_session::agent_identity` resolves the key in this order (`agent_identity.rs:206`): an explicit path, then `TERMLINK_AGENT_ID` mapped to `$HOME/.termlink/identities/<id>.key` (`agent_identity.rs:195`), then the host key `~/.termlink/identity.key`. `wire.rs:708` says the shared host key is honest but not rostered, so it cannot be an agent's signing key.
- **Real keys on this host.** `~/.termlink/identities/` holds 129 `.key` files, including `claude-termlink.key`, `claude-termlink-alt.key` and `claude-termlink-verify.key`. `.claude/settings.local.json:560` sets `TERMLINK_AGENT_ID=claude-termlink`. Key files are root-only (mode 600).
- **PR-1a** (`threat-model.md:239`): a new key is recorded on the agent's card at registration, and replacing a key needs the old key's signature or operator approval. Its project root-key rule is at `threat-model.md:383`.
- **What exists.** No card or registration code was found. Grep for agent card and key registration in `crates/termlink-hub/src` and `crates/termlink-session/src` returned nothing. Hub agent presence (`agent-presence`) carries a `cv_key` and capabilities, not a public key (UNVERIFIED for the full presence payload).
- **Gap.** Nothing registers a key with the hub. PR-1a has no implementation.

## 2. Roster

- `Config.roster` (`daemon.rs:34`) is a list of 64-hex public keys. `daemon.rs:80-89` converts it to a `Roster`. `policy.rs:15` says it is fixed at start and changes only through the admin socket.
- The sidecar also admits operator-admitted keys (`pull.rs:553`, `policy::admitted_keys`).
- **Which agents are "this project's".** `.framework.yaml` has no roster or agent list. Agent ids seen on the host are `claude-termlink`, `claude-termlink-alt`, `claude-termlink-verify`, `framework-agent-systemd`, `pen-agent-systemd`, `email-archive` and `penelope` (`~/.termlink/identities/`, `~/.termlink/notify/*.flag`). Which two form the W1 pair is not in any file read here.
- **Gap.** The roster has no source. There is no operator-signed roster file, and no config generator.

## 3. Hooks (R-48) and inbox topic (R-49)

- **Arc-011 hook entry points.** `cli.rs:9` and `cli.rs:57` define `termlink-sidecar hook post-tool-use|session-start|stop|session-end`. `hook.rs:4-16` gives the table. `post-tool-use` pulls and prints `additionalContext`. `session-start` records `ready`. `stop` records `ready` only.
- **Real settings today.** Neither `.claude/settings.json` nor `.claude/settings.local.json` contains any `termlink-sidecar hook` entry (grep empty). Mail delivery is therefore not wired in this project.
- **AEF hooks already in place** (`.claude/settings.json`):
  - SessionStart (`startup`, `resume`): `fw hook sidecar-autostart`
  - UserPromptSubmit: `fw hook sidecar-inbox` and `fw hook sidecar-receiver-adapter`
  - Stop: `fw hook sidecar-receiver-ready`
  - PostToolUse: no mail hook. Only `checkpoint post-tool`, `error-watchdog`, `check-dispatch`, `loop-detect`, `check-fabric-new-file`, `commit-cadence`, `check-settings-edit` and `audit-task-tools` (`.claude/settings.json` PostToolUse block).
  - AEF hook scripts live in `.agentic-framework/lib/sidecar/{hooks,receipts,watcher}.py`.
- **Inbox topics.** arc-011 reads `inbox:<hub_id>/<agent>` (`send.rs:153`, `pull.rs:220`). AEF's watcher reads `inbox:<session_id>` (`watcher.py:670`). AEF's addressing uses `inbox:<hub>/<project>` (`addressing.py:336`) and `inbox:<hub>/<project>/<name>` for sub-agents (`addressing.py:313`).
- **Collision.** The strings differ in segment count, so no literal collision is seen. If the arc-011 agent name equals this project's name (`010-termlink`), both would sit on `inbox:<hub>/010-termlink`. Whether AEF's receiver for this project holds that topic is UNVERIFIED.
- **R-49 / FP 6.2.3.f.** Exactly one receiver per inbox. R-49.a sequences it: AEF's receiver stays until the two-agent acceptance test passes, then it is retired per host. No consumer census for `010-termlink` exists yet.
- **R-48 status.** R-48.g: "not built". Whether Claude Code picks up new hooks without a restart is UNVERIFIED. Idle sessions fire no hook (R-48.g).

## 4. Addressing (SI-20, SI-21, R-62)

- **Present.** `send.rs:317-322` requires `to` (agent) and `to_instance`, and rejects an empty value. `wire.rs:40-51` `Msg` carries `from`, `from_agent`, `from_instance`, `to_agent`, `to_instance`, `conv`, `msg_id`, `seq`, `priority`. `wire.rs:485-489` `Circuit` parses `host|hub|project|session|agent`. `wire.rs:502` `Circuit::parse` exists.
- **Verify on delivery.** `wire.rs:315` `verify` checks `to_instance` against the receiver's own instance (`my_instance`). This is the exact-instance check that R-62 wants, but on the receiving side only.
- **Missing.**
  - No role field and no project-plus-role resolver (R-67, which the R-62 text defers to).
  - No default of "own project only" (SI-21 fixed default).
  - No operator pin for main (SI-20 pin).
  - No instance source from the real SessionStart (the `session-start` hook records `ready` but nothing ties the instance to the Claude session id; UNVERIFIED how `instance` is minted).
  - No dead-letter state for a bound copy that has ended (R-62.a).

## 5. Production marker and settings

- **Marker.** `estate.rs:69-71` `PRODUCTION_MARKER = .termlink-sidecar-root`, first line `termlink-sidecar-production v1`. `estate.rs:118-128` switches to production mode when a store sits under one. The K-1 refusals (`arc-011-ph2-architecture.md:179`) are: at `/`, not a regular file, owned by another user, group- or world-writable. No marker exists yet (`/var/lib/termlink-sidecar` absent).
- **Store.** K-1 names `/var/lib/termlink-sidecar/<agent>/` (from the brief; the directory does not exist). Each sidecar writes `<store>/sidecar/<agent>/heartbeat.json` (`arc-011-ph2-architecture.md:226`).
- **P3 env file.** `systemd-templates/termlink-sidecar@.service` reads `/etc/termlink-sidecar/%i.env` with `PROJECT_DIR`, `AGENT`, `SIDECAR_CONFIG`, `SIDECAR_BIN`, `TERMLINK_SIDECAR_STORE`, `TERMLINK_RUNTIME_DIR` and `HOME`. `/etc/termlink-sidecar` does not exist.
- **Sidecar JSON** (`daemon.rs:24-43`): `agent`, `instance`, `hub`, `hub_id`, `key_file`, `roster`, `queue_path`, plus defaults `poll_ms` 500, `timeout_ms` 10000 and optional `claude_projects`.
- **Project setting.** `SIDECAR_KEEP_RUNNING` is absent from `/opt/termlink/.framework.yaml`, so the default (off) applies.

## 6. Launcher and supervision (OQ-f D+)

- **Launchers present.** `scripts/tl-claude.sh` (`--reachable` arms be-reachable against the PTY, T-2388) and `scripts/be-reachable.sh start --agent-id X --pty-session Y`. Claude Code's own launcher `claude-fw` is referenced in CLAUDE.md; its location is UNVERIFIED.
- **Supervisor.** `scripts/sidecar-supervise.sh` already has `start | stop | check | status | may-run` (header lines 1-20). `may-run` tests `SIDECAR_KEEP_RUNNING` and the stop marker. The unit's `ExecCondition` calls it.
- **Hook-in points.**
  - Project start: `tl-claude.sh` (after the PTY exists) would call `sidecar-supervise.sh start <project>`.
  - Explicit stop: a stop script or cockpit calls `sidecar-supervise.sh stop <project>`.
  - Agent session end must NOT stop the project (6.1 item 4). Nothing in `tl-claude.sh` or `be-reachable.sh` ties session end to the sidecar yet (UNVERIFIED for claude-fw).
- **Backstop.** `runme.sh:194-243` installs the unit and `sidecar-backstop.crontab`. Whether the crontab is installed is UNVERIFIED.

## 7. Admin sockets and kill switch

- **Sockets.** `<agent>.admin.sock` (`api.rs:4`). It is owner-uid only (`api.rs:304`).
- **Env.** `TERMLINK_SIDECAR_ADMIN_SOCKET` (`client.rs:37`) for one socket. `TERMLINK_SIDECAR_ADMIN_SOCKETS` (colon-separated, `client.rs:146`) for `--all`.
- **Kill switch.** `client.rs:148-170`: `admin delivery off|on --why TEXT [--socket S | --all [--list FILE]] [--json]`. Each flip is read back. `--why` is required.
- **List file.** `session-start-alerts.sh:78` reads `.context/working/sidecar-admin-sockets.list`. The file does not exist. A missing list means no alert, so it is a silent state. Its host-wide scope is not documented in code.
- **runme.** No kill-switch action found in `runme.sh` (grep for `delivery`). The P6 design says it should exist (`client.rs:25`).

## 8. Open choices for the operator

1. **Which pair is W1?** Options: A `claude-termlink` + `claude-termlink-alt` (both claude sessions on this host); B `claude-termlink` + `framework-agent-systemd` (the systemd agent, so SessionStart and PTY behaviour differ); C a fresh pair. Facts: the first two are the only human-driven claude ids on the host; `claude-termlink-verify` exists but was not checked for activity.
2. **Key provisioning.** Options: A reuse the existing `identities/*.key` files; B mint new keys for the pair; C defer PR-1a to PH-4, with W1 on the fixed default. Facts: no card code exists, so PR-1a would need a hub record that does not exist yet.
3. **Roster source.** Options: A a static `roster` list in each sidecar JSON, written by `runme.sh`; B an operator-signed roster file (SI-21), which waits for PH-4; C a hand-maintained list reviewed each time. Facts: `policy.rs:15` fixes the roster at start.
4. **Inbox collision and R-49 sequencing.** Options: A give the arc-011 agents names that cannot equal the project name; B retire AEF's receiver on this host now (R-49 step 3); C keep AEF and run the negative control first. Facts: AEF's watcher uses `inbox:<session_id>` and project-level topics; the census is UNVERIFIED.
5. **Hook wiring.** Options: A add `termlink-sidecar hook post-tool-use|stop|session-start` to `.claude/settings.json` beside the AEF hooks; B put them in `settings.local.json` for this host only; C hold until the R-48.g restart question is answered. Facts: no PostToolUse mail hook exists; AEF's UserPromptSubmit hooks already read the AEF inbox.
6. **Instance source (R-62).** Options: A the Claude session id from the SessionStart payload; B a fresh id minted by the sidecar at `session-start`; C the claude pid. Facts: `hook.rs` records `ready` but nothing yet names the instance; R-62.a says a copy's id is never reused.
7. **Production store and owner.** Options: A `/var/lib/termlink-sidecar/<agent>/` owned by root (K-1 default); B a per-user path owned by the agent's login; C another volume. Facts: the directory does not exist; K-1 refuses a marker owned by another user.
8. **SIDECAR_KEEP_RUNNING for W1.** Options: A off (the sidecar lives with the project, no restart); B on (systemd unit restarts it, including after reboot). Facts: off is the current default; on needs `/etc/termlink-sidecar/<project>.env`.
9. **Operator pin for main (SI-20).** Options: A pin `claude-termlink` as the main agent; B no pin, own-project default only; C pin per pair member. Facts: PH-2 allows only a pin with no configuration surface (`floor-and-phasing.md:111`).
10. **Kill-switch scope.** Options: A one admin socket per agent, with `--all --list` run by a runme action; B a single host-wide list file under `.context/working/` (gitignored, so it is not versioned); C a list under `/etc/termlink-sidecar/`. Facts: `session-start-alerts.sh:78` reads the `.context/working/` path; it is absent now.
11. **Launcher hook.** Options: A `tl-claude.sh` calls `sidecar-supervise.sh start` at project start; B `be-reachable.sh start` does it; C the cockpit (055) owns start. Facts: 6.1 item 2 says start always starts the sidecar; item 4 says session end never stops it.
12. **Scope of the W1 kill-switch drill.** Options: A run `admin delivery off --all` on the pair only; B also test it against one non-pair sidecar as a negative control. Facts: R-49.d asks for a negative control.

Count of open choices: **12**.
