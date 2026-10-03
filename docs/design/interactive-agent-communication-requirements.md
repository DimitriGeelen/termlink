# Interactive agent-to-agent communication: requirements and design

**Owner:** the operator · **Task:** T-3335 · **Started:** 2026-10-04
**Status:** skeleton confirmed by the operator (read-back 1–15, 2026-10-04); sections are being expanded with the design conversations and decisions.
**Sources:** `docs/reports/T-3335-design-history-termlink.md`, `docs/reports/T-3335-design-history-aef.md`, and the rounds they cite (March–October 2026).

How each section is laid out:
1. **Requirements:** numbered R-x.y, from the operator's confirmed read-back. A requirement changes only by an operator decision, recorded in the Decision log.
2. **Design:** how the requirement is met, with the component or file that does it.
3. **Conversations and decisions:** the rounds in which this was discussed (date, task, source file:line), the operator's own words, the rulings, and contradictions between rounds.

**How to read the citations.** Short names, all paths under `/opt/termlink` unless stated:

1. `HT` = `docs/reports/T-3335-design-history-termlink.md`; `HA` = `docs/reports/T-3335-design-history-aef.md`. A citation marked "(HT)" or "(HA)" was taken from that history and not re-read in the original.
2. `IAC` = `docs/design/interactive-agent-communication.md`; `RAIL` = `docs/design/arc-011-message-delivery-rail.md`; `SCAPI` = `docs/design/arc-011-sidecar-api-architecture.md`; `ARC11` = `.context/arcs/arc-011.yaml`; `DNS` = `docs/operations/deterministic-notify-sidecar.md`.
3. `T3330R` = `docs/reports/T-3330-receive-side.md`; `CONSULT` = `docs/reports/T-3335-interactive-communication-consult.md`; `T2838R`, `T2291R`, `T2303R`, `T2380R`, `T2396R`, `T243R`, `T1800R` = the `docs/reports/T-<n>-…` reports of those tasks; `T007R`, `T256R`, `T1207R`, `T099R`, `T2315R`, `T1807R` = `docs/reports/T-007-…`, `T-256-…`, `T-1207-…`, `T-099-…`, `T-2315-…`, `T-1807-…`.
4. `AEF:` = `/opt/999-Agentic-Engineering-Framework`. Hub messages are cited `topic@offset` as in HA (`IN-010` = `inbox:cacc73ea32b121dd/010-termlink`, `IN-AEF` = the same topic for `999-Agentic-Engineering-Framework`, `ARC` = `agent-chat-arc`, `PICKUP` = `framework:pickup`).
5. Re-read in the original for this document: ARC11 (all), IAC (all), CONSULT, T3330R (all), SCAPI 195–267, the T-3325 and T-3330 task decisions, T007R, T256R, T1207R, T099R, T1807R, T2315R, the head of T2396R, AEF `watcher.py:1-12`, AEF `T-3397` lines 190–199 and 264–276, and AEF `decisions.yaml` D-599, D-600, D-645, D-660, D-695..D-700.
6. Live checks on this host, 2026-10-03 23:24 UTC: the cron files in `.context/cron/`, `pgrep`, `ls ~/.termlink/notify`, and `ls .agentic-framework/lib`.

---

## 1. Goal

### Requirements
- R-1.1 Agents hold interactive, two-way conversations with each other while they are working: 1:1, many-to-many, and agents with the operator.
- R-1.2 The sender always knows where its message is.
- R-1.3 No agent has to be attached or polling by hand for delivery to proceed.

### Design
1. **R-1.1, the carrier.** Conversation turns are `channel.post` messages that carry `metadata.conversation_id`, read back with `channel.subscribe` (`docs/conventions/multi-turn-dialog.md:17-25`, HT A1b). 1:1 uses `dm:<fp>:<fp>` and `inbox:<circuit>` topics. Many-to-many uses broadcast topics such as `agent-chat-arc` (skills `/broadcast-chat`, `/recent-chat`). Built and operating on the hub.
2. **R-1.1, the live leg.** A message reaches a *running* agent only if something wakes or injects into it. That is sections 5–8. On this host it does not happen (IAC:119-126, and section 15).
3. **R-1.1, agents with the operator.** The operator stated this leg as "1:1, N:N, N:operator" (T2838R:555-562, HT A6b). I found no component built for it. The operator talks to an agent through the agent's own terminal. This is designed only.
4. **R-1.2, the sender's view.**
  4a. TermLink: `scripts/notify-ledger.sh`, one row per sent message, carrying the rung it reached (sent, delivered, read). A rung advances only when a receipt is read back (ARC11 S6). Built. It sees hub receipts only, not API calls.
  4b. TermLink: `channel post --await-ack` and `channel awaiting-ack` with `~/.termlink/awaiting_ack.sqlite` (T3330R:1d3). Built and operating.
  4c. AEF: the sender ledger `.context/sidecar/direct-ack.jsonl` with states SENT, RECEIVED, HANDED_OVER, REPLIED, UNDELIVERABLE, REJECTED, ESCALATED (`AEF:lib/sidecar/direct.py:40-58`, HA B2). Built. It reaches only agents started with `claude-fw --termlink`.
  4d. Gap: a hub receipt is a per-topic watermark, so the sender cannot see a per-message timeline (T3330R:1d4). The step-by-step answer to "where is my message" is designed (section 12) and not built.
5. **R-1.3, no one attached.** The design is the sidecar plus the 30-second tick (sections 3 and 6). The sidecars run for three agents here (`pgrep`, 2026-10-03). Nothing injects, so for agents on this host R-1.3 is **not met**. AEF meets it for agents started with `claude-fw --termlink` (HA D5).

### Conversations and decisions
6. **2026-03-23, T-256.** Operator: "the spawned agent can talk to the spawning agent" with no polling (T256R:50). The first statement of the goal.
7. **2026-04-26, T-243.** Operator, verbatim in the dialogue log: "interactive multi turn conversation between two or more agents is absolutely not working" (T243R:100); and "send and wait instead of immediate response … even consider to adapt a opensource chat protocol aka signal style" (T243R:115) (HT A1c).
8. **2026-05-25, T-1800.** Operator: "interactive conversation between two or more agents… send-and-wait instead of immediate response… we want this interactive style of conversation to take place" (T1800R:6). Also "SSE is not mandatory — the goal is push messages + automatic pickup for truly dynamic, interactive conversation" (T1800R:122) (HT A2c). GO, 2026-05-25.
9. **2026-06-27, T-2291.** Operator asked for "a permanent solution … 5 termlink research agents" after a handoff was lost (T2291R:8-9) (HT A3d).
10. **2026-08-24, T-2838.** The operator's stated goal: an interactive agent-to-agent medium, 1:1, N:N, N:operator. The agent's verdict: "Partially certainly, but that interactive communication is still flaky at best." (T2838R:19, 555-562) (HT A6b).
11. **2026-10-03, T-3335.** Operator: "I am absolutely very clear that I want to have this … after 20 times me telling you this is a really important part of this design, you don't have this quite essential part of bi-directional interactive communication." (CONSULT:13). Also "You've been telling me it works … as you don't know how you can solve it, get external consultation." (CONSULT:14).
12. **Contradictions.** None about the goal. The contradictions are between *claims of working* and reality: arc-003 "no silent loss", arc-004 "shipped", arc-011 S10 "PROVEN LIVE". They are listed in section 15.

## 2. Origins

### Requirements
- R-2.1 Keep the original capability: inject keystrokes into a terminal and read its output back (T-007).
- R-2.2 Replace fire-and-forget dispatch with conversation over the hub (T-256).

### Design
1. **R-2.1.** T-007 chose option D: PTY-backed sessions for full two-way control; `command.execute` stays for run-and-capture (T007R:92-99, 239-253). Built as `termlink register --shell`/`spawn --backend tmux`, `termlink inject`/`pty inject` (`crates/termlink-cli/src/cli.rs:360, 6539-6567`, T3330R:3g1) and `pty output`. Operating. An agent can be targeted only if it runs in a TermLink-owned PTY (T1800R:68-75, HT A2bd). The affirmative prover is `scripts/session-selftest.sh` (T-2485, CLAUDE.md).
2. **R-2.2, the convention.** T-256 chose the collect fan-in (option B): workers emit `task.completed`, the orchestrator collects in the background (T256R:67-82). `emit-to` (option A, true push) exists as the `termlink_emit_to` MCP tool.
3. **R-2.2, doorbell plus mail (T-1800).** Mail is `channel.*` and carries every turn. The doorbell is `command.inject` of a fixed wake line (`/check-arc`) into the receiver's PTY and carries no content (T1800R:50-64, HT A2ba). Built: `scripts/agent-send.sh` (T-1804), `scripts/agent-respond.sh` (T-1805). Validated with a scripted responder over 3 turns (T1807R:19-45).
4. **R-2.2, what is not proven.** The live two-real-claude soak T-1810 is still `captured` (task file). Its blockers were: root cannot use `--dangerously-skip-permissions`, and the `/check-arc` doorbell did not signal respond mode (T1807R:47-77). T-1809 (respond-mode signal) is `work-completed`.
5. **R-2.2, what replaced it.** Later rounds replaced the doorbell as the notify mechanism (the sidecar flag, DNS:20-23) and then brought PTY injection back (push-waker, then the arc-011 injector). See sections 5, 6 and 8.

### Conversations and decisions
6. **2026-03-08 (date from the skeleton hint; the report carries none), T-007.** Question: how a session captures output for `query.output` and `data.stream`. Ruling: GO, hybrid PTY for registered sessions plus pipe for execute (T007R:239-253). Key point: without output capture "TermLink is a blind remote control" (T007R:17).
7. **2026-03-18, T-099.** Found that the Claude Code `Stop` hook already fires after each response and `SessionEnd` exists (T099R:33-43). T-099 closed NO-GO as a feature request. This is where the Stop hook first appears in the record; section 7 uses it.
8. **2026-03-23, T-256.** Research: the primitives exist, the gap is `emit-to`, and waiting is 10× cheaper than polling in tokens (T256R:22-45). Operator insisted on TermLink mesh agents, not the Claude Agent tool (T256R:51). Recommended path: option B now, option A later (T256R:73-76).
9. **2026-04-26, T-243.** Operator, verbatim: "no not completely, i wanted a means to simulate keyboard input and capture console output, because this enables several use cases for agentic engineering." (T243R:96). The protocol layer shipped (HT A1d).
10. **2026-05-25, T-1800 and T-1807.** Operator: "1 YES [to the split]. A: does this work with claude or does claude-fw need to be started? B: can we wire it so it becomes a deterministic workflow (ensure the doorbell is rung after the message is sent)? 2 yes make it a deep inception, it's key functionality." (T1800R:131). Also the directive "aim not to use claude-p for expensive jobs" (`.tasks/completed/T-1800-…:135`): persistent-session doorbell plus mail is primary. The known ceiling was recorded on day one: if the receiver is mid-turn, an injected line queues or is mis-consumed (T1800R:87).
11. **2026-05-28, T-1830.** All fleet hubs passed the selftest and the active conversation count was 0 across 91 topics. Cause: coordination, not infrastructure (`docs/reports/T-1830-doorbell-mail-adoption-gap.md:13-35`) (HT A2d).
12. **Contradictions.** T-256's "no polling" (2026-03-23) versus the fallback polling in section 10: both stand, since polling is now only the fallback. T-1800 made the doorbell primary; T-2291 (2026-06) said the doorbell is to be replaced (DNS:20-23); T-2303 (2026-07) brought it back through the push-waker (HT A4f). Current: the arc-011 model (sections 5–8) injects, as the doorbell did.

## 3. The sidecar

### Requirements
- R-3.1 One sidecar per agent: a separate, very simple process with an API.
- R-3.2 Independent of the hub, "because the hub goes down".
- R-3.3 Always respawns, portably, not only under systemd (SQ-8).
- R-3.4 Carries the startup chain: start agent, session, project, hub.
- R-3.5 Re-resolves when an FQDN or IP stops resolving.

### Design
1. **R-3.1, TermLink.**
  1a. `scripts/notify-sidecar.sh` (591 lines) runs per agent, polls every 15 s and, with `--auto-confirm`, posts a `stage=delivered` receipt (T3330R:1a1). Three run on this host (`claude-termlink`, `framework-agent-systemd`, `claude-termlink-alt`; `pgrep` 2026-10-03). Built and operating.
  1b. `scripts/notify-sidecar-api.sh` (319 lines, T-3135) is a local control API: `status`, `queue`, `inject <id|next>`, `agent-state`, `ack <offset> --evidence`. Every remote-address argument is refused, exit 2 (T3330R:1b8; ARC11 S1). Built.
  1c. It is **not** what R-3.1 and R-4.1 describe. The arc file says so itself: "the spec wording 'sender SENDS via a sidecar API' is NOT what was built and NOT what SQ-1 permits" (ARC11:34-38).
  1d. "Very simple": the receive chain is about 2,460 lines of shell (T3330R:3h1), and nothing in the CLI binary (no `sidecar` subcommand, T3330R:3g2).
2. **R-3.1, AEF.** A per-agent receiver with an HTTP API on `127.0.0.1`, bearer-token auth, plus a watcher (`AEF:lib/sidecar/{receiver,http_server,watcher}.py`, HA B1). Built. It reaches only agents started with `claude-fw --termlink` (HA D5). Our vendored framework has no `lib/sidecar` (checked 2026-10-03), so it does not run here.
3. **R-3.2, independence from the hub.** TermLink's sidecar reads hub topics to learn of mail, so it is independent of any LLM, but not of the hub. AEF's receiver takes a message on loopback and stores it with no hub; across hosts it falls back to a hub topic (D-697, HA B1.6). Partial. SCAPI IW-3 asked "what failure does hub-independence actually buy, given local state is already local?" (SCAPI:260-267); no recorded answer.
4. **R-3.3, respawn.** `scripts/notify-sidecar-supervisor.sh --loop` (bash-only core) and `--emit-unit systemd|launchd|cron|auto` (ARC11:28-44). Cron runs the supervisor every 5 minutes (`.context/cron/notify-sidecar-supervisor.crontab`; T3330R:1a6a). AEF: a supervisor in `watcher.py:527-602`, cron `sidecar-ensure-1m` plus `@reboot` (HA B1.2d). Built and operating. Observation, not investigated: `pgrep` showed two extra processes with the same command line as `claude-termlink`'s sidecar (pids 3470129, 3471270). They may be transient subshells.
5. **R-3.4, the startup chain.** **Not built and not designed beyond the sentence** (RAIL:111-117; HT D4). `scripts/tl-claude.sh start --reachable` launches an injectable agent session (T-2388) and is the nearest thing. AEF's `fw sidecar ensure --all` restarts only the watcher and receiver (HA B1.2d). T-1135's "receptionist" was never built (HA round 1).
6. **R-3.5, re-resolve.** `notify-sidecar-api.sh:197-212` re-reads and compares the FQDN and IP on each call (HT B2c). Partial: it notices a change; it does not reconnect. AEF: none found.

### Conversations and decisions
7. **2026-06-25 to 06-27, T-2291 (arc-003).** The "ears": a no-LLM listener on the recipient host pulls mail and writes a flag plus a heartbeat; the agent reads the flag at its own yield points (T2291R:251-267; DNS:10-29). No API.
8. **2026-09-20, the operator's "sidekick" recollection (AEF T-3396).** Verbatim: "What we want is the [sidekick] for interactive conversation that we worked on... a sidekick that's listening all the time... it sets a flag... the cron job monitors that flag... looks for the active session... we could have a queue where the queue can be prioritized... a flag is raised that there are new messages... use PTY inject when the cursor is silent... with the exception of the urgent message that warrants an interruption." (`AEF:docs/reports/T-3396-peer-consult-sidecar-inception.md:87-92`) (HA round 1). AEF noted that its own arc-011 §5 had *rejected* PTY injection into an apparently idle terminal; the operator ruled: "Let's go to the Royal and the Correct Long Term Road. Let's pack it full out." (T-3396:104-105).
9. **2026-09-21, AEF T-3397.** Push, not pull: the sidecar exposes an API a sending agent calls; a message file is written, then a flag (T-3397:80-89) (HA round 3).
10. **2026-09-22, arc-011 spec.** Written as "Operator direction, verbatim in intent": "a **separate process** exposing an API, deliberately **independent of the hub because the hub goes down**. Very simple. **Always respawns.** It also carries the startup chain — start agent, start session, start project, start hub — and re-resolves when an **FQDN or IP stops resolving**." (RAIL:111-117; SCAPI:10-13). Not a recorded verbatim quote.
11. **2026-09-23, SQ-1 (RESOLVED).** "the injector belongs to TERMLINK … a PRIMITIVE built over TermLink's own session control, not an orchestration engine" (ARC11:203-213). The sidecar API was read as a LOCAL control surface. SCAPI set the bright line: the API may read and act on this host's own state, never move a message between hosts (SCAPI:17-35, 153-172) (HT A7e).
12. **2026-09-24, SQ-8 (RESOLVED).** Operator verbatim principle: "reliability and fragility, so I'm not to save a few tokens just to get a quick solution. I want a solid solution." (ARC11:360-361). The derived disposition (flagged as derived, not the operator's words): systemd-only respawn is rejected and a portable fallback is required.
13. **2026-09-28, T-3200.** Operator: "did we not design this in the sidecar?" (`.tasks/completed/T-3200-…:81`). The agent had opened an inception over ground already designed. The sidecar itself had last written a heartbeat on 2026-07-01 and not run since (T-3049, `.tasks/completed/T-3049-…:3-20`) (HT A8).
14. **2026-10-02, AEF T-3682.** AEF reported that "the whole receive half had never been built" before T-3693 (IN-010@62, hub, HA round 6).
15. **Contradictions.** (a) 2026-06 listener with no API versus 2026-09 sidecar with an API: current is the API (operator read-back 2026-10-03, T3330R:252). (b) The 2026-09-22 spec has the sender call a sidecar API; SQ-1 (2026-09-23) ruled the API local only. The arc file keeps both. No later ruling reverses SQ-1, so the conflict is open: **O1**. (c) AEF's round 1 rejected injection; the operator overruled it on 2026-09-20.

## 4. Sending

### Requirements
- R-4.1 The sender gives the message, optionally with a blob, to its own sidecar's API.
- R-4.2 The sender's sidecar delivers it to the receiver's sidecar API. Push first.
- R-4.3 The hub is the fallback, plus discovery and blob storage.

### Design
1. **R-4.1, TermLink.** There is no send API on the sidecar. A sender uses `scripts/agent-send.sh`, which does `channel post` to the hub, with a random 128-bit `client_msg_id` for exactly-once (T-2049; CLAUDE.md) and a durable offline queue `~/.termlink/outbound.sqlite` for hub blips (T-2051). Built and operating. Blobs: `termlink artifact put <path> --to <peer>` and `artifact get <sha256> --expected-sha256 …` (T-3134, ARC11 S2). Built.
2. **R-4.1/R-4.2, AEF.** `fw sidecar send` posts to the peer's registered receiver with HTTP, and to the peer's hub topic `inbox:<circuit>` if no receiver is registered (`direct.py:1-26`; D-697). The receiver address is host-local only: loopback, a host registry file (`lifecycle.py:15-20`). Built for the same host.
3. **R-4.2, across hosts.** Not built on either side. AEF's T-3688 would publish each receiver endpoint to the hub for discovery; `lifecycle.py:20` reads "Cross-host addressing is T-3688" (HA B5). TermLink has the tripwire `crates/termlink-hub/tests/no_federation_tripwire.rs` against a second bus (T3330R:4c1). "Push first" in the sense of hub-to-client push is built: the arc-004 `inbox.queued` frame, 31–111 ms (`.context/arcs/push-transport.yaml:32`, HT A4e). Sender-sidecar to receiver-sidecar push across hosts is designed only.
4. **R-4.3.** Built and operating. Hub topics, `agent-presence` discovery with `cv_index` (CLAUDE.md T-2107), `artifact.put`, and the fallback topic `inbox:<circuit>`.

### Conversations and decisions
5. **2026-06-27, T-2291 step 6.** The operator asked to "critically re-evaluate the 'comms over hub' principle" (recorded as, T2291R:280). Operator inversion: "the single hub-firehose IS the obfuscation problem" (T2291R:294). Discovery: "(operator decision): TWO-TIER" (T2291R:302) (HT A3dc). Result: V6, direct host-to-host first, hub as loud fallback (`docs/plans/T-2296-v6-direct-transport-first-design.md:77-83`). R-4.2 and R-4.3 restate V6.
6. **2026-09-21, AEF T-3397/T-3396.** Operator: "We have multiple hosts, so that's not a question." (T-3397:270-273). Cross-host is a live requirement. Rule from the T-3396 amendments: one uniform path, same-host is the degenerate case (T-3396:136-144) (HA round 3).
7. **2026-09-22, SCAPI §6.** "sending to a peer is explicitly out of scope" for the API (SCAPI:164-167, HT B1c). Charter: a hub-mediated strict star; a cross-host API "would make TermLink a second bus" (SCAPI:17-35). The agent's own caveat: if the real need is cross-host delivery while the hub is down, "this is not a local-control API at all, it is a second bus … put it to the charter" (SCAPI §9, "What would change my mind").
8. **2026-09-23, SQ-1.** Sending to a peer stays `channel.post` on the hub (ARC11:28-44).
9. **2026-09-25, D-645.** "binary blobs ride TermLink file transfer via the sidecar API, not session-to-session" (`AEF decisions.yaml:4512-4517`). Rationale: a blob sent session-to-session inherits the "delivered-equals-hub-accepted" failure.
10. **2026-10-02, D-697.** "no receiver registered for the name … → hub topic fallback. A receiver that is registered but unreachable … → retry budget, then UNDELIVERABLE." (`decisions.yaml:4876-4881`).
11. **2026-10-03, operator read-back.** "Send: sender agent -> its own sidecar (API) -> the receiver's sidecar (API)." (T3330R:252). The T-3330 ruling then gives cross-host sidecar calls to AEF: "cross-host sidecar calls are AEF's T-3688; TermLink supplies discovery and the telemetry record" (`.tasks/active/T-3330-…:181-188`).
12. **Contradictions.** (a) SQ-1 (2026-09-23): "sending to a peer stays channel.post via the hub" versus the read-back of 2026-10-03: sidecar to sidecar by API. Quotes in **O1**. Current: the read-back is later and operator-confirmed, but nothing records that SQ-1 or the charter tripwire was lifted. (b) V6 (direct first) versus the charter no-second-bus rule (HT B18). (c) The T-3330 ruling hands the cross-host leg to AEF; the charter objection then applies to AEF's T-3688 (T3330R:209).

## 5. Receiving

### Requirements
- R-5.1 RECEIVED: the receiver's sidecar calls the sender's sidecar immediately.
- R-5.2 STORED: the message is stored durably and a second call goes back.
- R-5.3 A new-message flag is raised.

### Design
1. **R-5.1, TermLink.** `notify-sidecar.sh --auto-confirm` posts `channel post <topic> --msg-type receipt --metadata stage=delivered --metadata up_to=<offset>` (`notify-sidecar.sh:300-307`). It is a hub post, not an API call. It is a per-topic watermark, with no timestamp field and no per-message id. It is posted only when the offset advances, so the worst-case latency is one 15 s poll (T3330R:1a2-1a4). The arc file's own verdict: "Right event, wrong mechanism" (ARC11:85). Operating, for three agents only.
2. **R-5.1, a limit that matters.** A receipt of this kind means the sidecar saw the mail, not that the agent did. On 2026-10-03 the sidecar receipted mail as delivered while no wake consumer existed, and AEF waited about a day (`.tasks/completed/T-3325-…`, HT C8). The flag's pending count is shared with the sidecar's own auto-ack, so `pending=0` can mean "receipted", not "seen" (HT B3d).
3. **R-5.1, AEF.** `POST /message` stores, sets the flag, and answers `RECEIVED` with a timestamp in the HTTP reply (`http_server.py:86-103`). Synchronous. Built, same host.
4. **R-5.2.** TermLink: storage is `journal-mirror.sh` into `~/.termlink/journals/journal.sqlite` (ARC11 S3; 2,238 rows on 2026-09-22). It mirrored `dm:` only until T-3203 added `inbox:` (T-3200, HT A8d). There is no separate "stored" call. AEF: the message is written to a temp file, renamed, then flagged (D-690..D-697 in HA B2), and RECEIVED is answered after the store, so for AEF RECEIVED already means stored (HA C2 row 6). A distinct STORED call exists in neither build. Designed only: **O2**.
5. **R-5.3.** TermLink: `~/.termlink/notify/<agent>.flag` with `ts=` and `last_mail_ts=` (ARC11 S4; T3330R:1a3). The files exist for the three agents (`ls ~/.termlink/notify`, 2026-10-03). Built and operating. AEF: `.ready` and `.pending` files under `.context/sidecar/receiver/messages/` (T3330R:2f2).
6. **Structural limit.** `stage` is a free-form metadata key and `channel.receipts` keeps only the latest receipt per sender, so an earlier stage can be overwritten (T3330R:1d1-1d2).

### Conversations and decisions
7. **2026-06-27, T-2291 (arc-003).** Confirm ladder in three levels (the operator moved it from "elaborate" to "accepted", T2291R:307-316): L1 TCP ack, ignored; L2 the sidecar's journaled receipt; L3 the read receipt at the agent's yield point (V6:200-217) (HT A3cd).
8. **2026-07-10, T-2396.** Operator: "prove-first, then build" (T2396R:4). Root cause PL-253: "A heartbeat proves the **process** is alive, not that the **session** is listening" (T2396R:14-24). "A rung is not a read" (HT A5d).
9. **2026-09-22, arc-011.** Two events, not one: RECEIVED (L2) and INJECTED (L3 with `--evidence`) (RAIL:65-85). The agent's own admission at the top of RAIL: "I reported the rail 'working end to end'. It was not." (RAIL:14-18). `evidence=wake-consumer` was withdrawn as untruthful (ARC11 S6; SQ-7, 2026-09-23).
10. **2026-10-03, T-3330, the operator's first statement.** "what's going wrong there? … We store it and immediately go back to the sidecar receiver … an API call that's received. And then when it's injected we make an API call that's injected … those timestamps we store … once we've got an answer we send it back … also timestamps … also an API call." (T3330R:229). The brief at this point had **three** calls: RECEIVED, INJECTED, REPLIED (T3330R:6-11).
11. **2026-10-03, the read-back.** Four calls: RECEIVED, STORED, INJECTED, ANSWER READY (T3330R:252-259). Confirmed by the operator. The agent had "drifted to 'publish and pull'" and the operator restated "push by API between sidecars, with pull only as a fallback" (T3330R:250).
12. **2026-10-03, external review.** Codex and GLM-5.3 called the synchronous call back to the sender "wrong as written" and said record-and-pull meets the need (T3330R:242). All four reviewers chose option C. The operator's read-back kept push by API.
13. **Contradictions.** (a) Three calls (T3330R:6-11) versus four (T3330R:252-259): the four-call read-back is later and current. (b) L2/L3 (June) versus RECEIVED/INJECTED (September) versus four calls (October): HT B9b maps L2 = RECEIVED and L3 = INJECTED; STORED and ANSWER READY are new. (c) RECEIVED and STORED as one call or two: AEF has one, the read-back has two: **O2**. (d) Review panel versus read-back on the call back: **O10**.

## 6. The 30-second tick

### Requirements
- R-6.1 A cron-style job checks the new-message flag every 30 seconds.
- R-6.2 With the flag up, it reads the queue, highest priority first.
- R-6.3 Urgent: injected immediately, even when the agent is busy (hard bypass), by a route that cannot silently lose the message.
- R-6.4 Not urgent: injected if the prompt is free; otherwise it waits for the next tick.

### Design
1. **R-6.1, TermLink: not built.** (System cron's minimum is one minute, so "every 30 seconds" has to be a supervised loop, IAC:52.) The installed crons are `*/5` and supervise the sidecar and the wake consumer (`.context/cron/notify-sidecar-supervisor.crontab`, `notify-wake-supervisor.crontab`). The injector `scripts/notify-injector.sh` is in no crontab: its own header says "THIS SCRIPT IS SCHEDULED BY NOTHING, AND THAT IS NOT A DECISION TO RETIRE IT" (`notify-injector.sh:4`, T-3207). A search of `.context/cron/*.crontab` for it returned nothing. The wake consumer posts nothing: "the wake rail currently terminates in a log line" (`notify-wake-consumer.sh:28-32`, T3330R:1b1c). ARC11 S11 records "built" and means the supervisor cron (ARC11:171-182).
2. **R-6.1, AEF: built.** `watcher.py` ticks every `SIDECAR_TICK` seconds, default 30 (`watcher.py:13-15, 69`). It drains hub-topic mail into the receiver, injects, escalates expired deadlines, runs a self-probe and writes `liveness.yaml` (`watcher.py:412-441`). A supervisor restarts it (`watcher.py:527-602`). It is a loop, not literal cron, as the design note says (IAC:52). Operating only for `claude-fw --termlink` agents.
3. **R-6.2.** TermLink: the injector orders `COALESCE(priority,0) DESC, ts ASC, offset ASC`; priority is persisted by `journal-mirror.sh` and clamped to [-9,9] (ARC11 S8). Built, inside the unscheduled injector. AEF: I did not find priority ordering in the files read (HA).
4. **R-6.3, TermLink: built to the opposite rule.** S9 was built to SQ-4: urgent re-probes the prompt for up to `INJECTOR_URGENT_WAIT` (default 120 s) at band ≥ `INJECTOR_URGENT_THRESHOLD` (default 5), then defers; it never injects into a busy prompt (ARC11 S9). So the built TermLink behaviour contradicts R-6.3.
5. **R-6.3, AEF: bypass built, loss not prevented.** An urgent message goes to a ready session if any, else the most recently active one, even busy (`inject.py:213-248, 307-309`), and is recorded as `urgent_bypass` (`inject.py:338-341`). It then needs transcript evidence and is re-injected after 120 s (`inject.py:46`). The T-2396 loss is *detected*, not prevented (HA C2 row 4). Urgent compression of the retry ladder raises `NotImplementedError` (`retry_ladder.py:49-54`).
6. **R-6.3, the route.** "A route that cannot silently lose the message" is not built. The operator's own earlier idea was an out-of-band route: the PTY doorbell (SCAPI:210-234). The push-waker `be-reachable-pushwaker.sh` is that doorbell; it is dormant without a bound `pty_session` and no instance runs (T3330R:1b6; `pgrep`, 2026-10-03). **O4.**
7. **R-6.4, TermLink.** The injector checks, in order, whether the agent is running (its own exit 3) and whether the prompt is free, using `scripts/lib/pty-state.sh` (rebuilt in T-3079: two byte-identical reads plus an empty composer row; 466 samples, 0 false "free"). Not free: exit 4, the next cron tick. Built, proven once on a fresh topic (ARC11 S10), not scheduled.
8. **R-6.4, AEF.** Injects only into a session whose own record says ready; if none, the message waits and `INJECT_BLOCKED` is logged once per reason (`inject.py:218-226, 276-283`). Built.

### Conversations and decisions
9. **2026-06, T-2303.** The arc-003 sidecar pulled every 15 s; that was "the latency floor" arc-004 set out to remove (T2303R:1-30, HT A4b).
10. **2026-09-20, the sidekick recollection.** "the cron job monitors that flag … use PTY inject when the cursor is silent … with the exception of the urgent message that warrants an interruption" (T-3396:87-92).
11. **2026-09-21, AEF, operator decision as recorded by AEF.** "urgent means bypass — an urgent-tagged message skips the ready-flag check entirely and injects immediately, accepted risk, regardless of busy/idle state … bypass is a hard skip, not a shortened interval." (T-3397:194-199). A paraphrase by AEF, not the operator's words.
12. **2026-09-22/23, SCAPI Q3.** Operator verbatim: "Urgent bypasses the wait. Reliability is important at all times but can also be out of band." (SCAPI:207-208). The agent's reading: bypass the WAIT, possibly by another route, with the L3 confirmation still required, and "the risk to decide (not mine)": a busy-prompt inject can be silently lost (SCAPI:210-234). T-3072 had said urgent was "not yet defined, note for later" (HT A7bd).
13. **2026-09-23, SQ-4 (RESOLVED), operator.** "urgent NEVER injects into a BUSY prompt … Urgent shortens the WAIT; it does not bypass the prompt-free CHECK. Injecting into a busy prompt is the T-2396 failure where text lands unsubmitted and is discarded, so a bypass would silently lose exactly the messages most likely to be marked urgent." (ARC11:305-316).
14. **2026-10-02, operator, AEF watcher.** "a watcher … watching if the message flag is up every 30 seconds and when it's up it looks if it is an urgent message and injects directly, or if it's not urgent it looks if the prompt is free and if it's free it injects the message and informs the sender." (`AEF:lib/sidecar/watcher.py:3-6`).
15. **2026-10-03, operator, to TermLink.** "I'm completely missing the point about that we observe whether the prompt is free or not, and inject when it's free and with urgent bypass." (CONSULT:16). Then: "I'm missing that we use cronjobs to monitor if there is a flag. Every 30 seconds … when the flag is up, the message queue gets read … urgent gets injected immediately. Non-urgent, we check again if the prompt is free. If the prompt is not free, we wait again until the next 30 seconds." (CONSULT:17). Confirmed in the read-back (T3330R:256).
16. **Contradictions.**
  16a. **Urgent.** 2026-09-21 bypass (AEF) → 2026-09-22/23 "bypasses the wait … out of band" (SCAPI:207) → 2026-09-23 SQ-4 "NEVER injects into a BUSY prompt" (ARC11:305) → 2026-10-02 and 2026-10-03 bypass. **Current: bypass** (latest, and R-6.3 is confirmed). SQ-4 still reads RESOLVED in `arc-011.yaml` with no recorded supersession, and S9 is built to SQ-4. A supersession should be recorded: **O4**.
  16b. **Interval.** `*/5` built (TermLink); 15 s proposed and "never validated" in AEF's architecture doc (`sidecar-target-architecture.md:111-116`, HA B4.4); 30 s operator (2026-10-02, 2026-10-03). Current: 30 s.
  16c. **Where the check runs.** TermLink's design has the cron job check the prompt itself; AEF has the harness report it (section 7).

## 7. Readiness: is the prompt free?

### Requirements
- R-7.1 Readiness is reported by the agent's own harness: the Stop hook marks ready at the end of a turn; the prompt-submit hook clears it before the next turn.
- R-7.2 It is not inferred from the screen: a long tool call looks idle but is not safe to type into.

### Design
1. **R-7.1, AEF: built.** The `Stop` hook sets this session's ready flag; `UserPromptSubmit` clears it first (`AEF:lib/sidecar/hooks.py:3-6, 80-87`). Per-session records `.context/sidecar/sessions/<claude session_id>.json` (`adapter.py:20-35`, T-3745). A missing or unreadable flag reads as not ready (`adapter.py:88-111`). Operating only for `claude-fw --termlink` sessions: the injector says "start the agent with claude-fw --termlink" otherwise (`inject.py:222-224`).
2. **R-7.1, TermLink: not built.** TermLink has no hook-based readiness. It asked AEF to build it: "Your (b) — a ready-for-input flag written by the harness's own Stop/UserPromptSubmit hooks — is precisely the consumer we do not have. Do not build ears or receipts; do build (b)." (ARC@1640, HA round 4). T-3250 "design who observes agent-side readiness" is `captured` with no ruling. Our vendored framework has no `lib/sidecar`, so those hooks are absent here.
3. **R-7.2, TermLink's built classifier infers from the screen.** `scripts/lib/pty-state.sh` (T-3079) decides on screen quiescence and composer-row emptiness (ARC11 S7); the idle-gated push-waker screen-scraped `termlink pty output` (`docs/operations/pushwaker-idle-gating.md:11-26`). This is the method R-7.2 rules out. The classifier is proven on one live session and 466 samples; whether it can tell a long silent tool call from idle was not tested in the sources I read.
4. **R-7.2, AEF's reason.** Timestamp staleness and CPU/PID heuristics were rejected "because a long Bash tool call looks idle but is unsafe" (T-3397:97-113).
5. **The Stop hook as a mechanism.** The Stop hook "fires after every assistant response … It is NOT a session-end event" (T1207R:15-16); its stderr becomes context on the next turn (T1207R:50). Known: it fires between responses, not between tool calls inside one response (T099R:59-61). That is why "ready at end of turn" works for R-7.1.

### Conversations and decisions
6. **2026-03-18, T-099.** The Stop hook is identified as Claude Code's per-response event (T099R:33-37). Decision: no feature request needed, wire the existing hooks.
7. **2026-04-24, T-1207.** The operator, on a Stop-hook governance nudge: "No block. Use a y/n user-question pattern. Agent should pick up the nudge autonomously and ensure conversation is captured." Then "Option B — live from day 1." (T1207R:68-72). Not about readiness, but it is the first use of the Stop hook in this project.
8. **2026-05-25, T-1807.** The doorbell text `/check-arc` did not signal respond mode (T1807R:66-77); T-1809 added the signal.
9. **2026-07-07 to 07-11, T-2380, T-2396, T-2402.** "a heartbeat proves the PROCESS is alive, not that the SESSION is listening" (T2396R:14-24); idle-gated injection added at Stage 3 of T-2402, screen-scraping `termlink pty output` (HT A5be).
10. **2026-09-21/22.** AEF T-3397:97-113 chose harness self-report. TermLink (T-3062, ARC@1640) said do build (b). On 2026-09-22 SQ-2 found the blocker was the idle classifier and rebuilt it (T-3079) (ARC11:214-246). Both approaches were built, by different teams, in the same week.
11. **2026-09-25 to 10-03.** D-694 (Stop sets ready only at turn end; UserPromptSubmit clears before the new turn) and T-3745 (readiness per session, after two Claude sessions in one project shared one flag) (HA round 6).
12. **Contradictions.**
  12a. **Screen versus harness.** The operator said on 2026-09-20: "use PTY inject when the cursor is silent" (T-3396:87-92), which is a screen inference. R-7.2, confirmed 2026-10-04, says not from the screen. IAC §3a 18ca still says "independent of any LLM: it reads how the terminal behaves" (IAC:55). Current: R-7. IAC is stale on this point. **O8.**
  12b. **Project flag versus session flag.** The project-wide flag is now display only (HA B1.3b).

## 8. Injection

### Requirements
- R-8.1 One short line is typed into the agent's session (`termlink pty inject`).
- R-8.2 INJECTED (AEF: HANDED_OVER) is reported only with evidence the agent saw the message, from its transcript.
- R-8.3 The flag comes down only when the queue is empty.

### Design
1. **R-8.1.** AEF types one fixed line that carries a count and ids and no peer content: `termlink pty inject <session> "<line>" --enter` (`inject.py:3-13, 251-261, 327`). TermLink: `termlink inject <session> <text> --enter` (`notify-injector.sh:372`); the `pty inject` verb exists (`cli.rs:6539-6567`). A claim naming the target is written before typing, so the right session is credited (`inject.py:312-324`). Built, both. Operating for AEF-armed sessions only.
2. **R-8.2, AEF.** HANDED_OVER is recorded only when the prompt hook has surfaced the message: a detached finalizer waits up to 90 s for the transcript to carry the `hook_additional_context` attachment under a one-time token; otherwise `HANDOVER_UNCONFIRMED` and the message becomes eligible again (`hooks.py:14-24`; D-696). Built.
3. **R-8.2, TermLink: weaker than the requirement.** The injector verifies a BUSY transition (`notify-injector.sh:383-398`) and posts L3 `stage=read` with `evidence=idle-gated-inject`, which "asserts prompt reach, not transcript evidence" (T3330R:1b10c). Exit 5 means not verified, and then no L3 is posted. T-2876's prover does read the receiver's transcript (`scripts/session-message-selftest.sh`). So R-8.2 is met by AEF and by the prover, not by the injector.
4. **R-8.2, the name.** Mapping TermLink INJECTED to AEF HANDED_OVER is "not decided anywhere" (T3330R:5f/223; HT D9): **O13**.
5. **R-8.3.** Designed (IAC:74). Not verified as built on either side. ARC11 S10 records an open gap: "the queue has no watermark tied to the L3 rung, so on a topic with history it re-serves the oldest offset" (ARC11:163-170). AEF clears per-message `.pending` files on hand-over (T3330R:2f2); I did not find a "flag down only when empty" rule in the code I read.
6. **Why the line is typed unsubmitted-safe.** An inject into a busy prompt can land unsubmitted and be discarded (T-2396; T2396R:14-24). That is the failure R-8.2's evidence rule detects.

### Conversations and decisions
7. **2026-05-25, T-1800.** Known ceiling: injected text queues or is mis-consumed mid-turn, so delivery is "eventual, not instant" (T1800R:87).
8. **2026-07-10, T-2396.** Operator: "prove-first, then build" (T2396R:4). Fix `wake-confirm.sh`, a standalone consumption check.
9. **2026-09-01, T-2876.** Operator: "test everything end to end, and also build that as a testing harness for any future development work." (`.tasks/completed/T-2876-…:43`). Result: assert on the receiver's transcript, never the sender's `success:true` (CLAUDE.md).
10. **2026-09-22, arc-011.** Operator: "I want to inject, but I check if my agent is running, I see it's not running." (`.tasks/completed/T-3069-…:92`). So NOT RUNNING is its own outcome (HT B5b). S10 "PROVEN LIVE 2026-09-22": one run, one test session, fresh topic `proof:t3079-…` (ARC11:148-170). `evidence=wake-consumer` withdrawn (ARC11 S6).
11. **2026-10-02, AEF D-696.** "AC4 now reads 'HANDED_OVER is recorded only when the prompt hook has actually surfaced the message — never on the queue write or the inject alone'. The previous worker had written 'on success [of the inject], records HANDED_OVER'." (`decisions.yaml:4869-4874`). A decision-log correction, not a quoted operator ruling.
12. **2026-10-03, operator read-back.** "INJECTED: called back to the sender's sidecar once the message is in the prompt. … The flag is cleared only when the queue is empty." (T3330R:257-258).
13. **Contradictions.** (a) Evidence: TermLink accepts a BUSY transition (IAC:58, "for example a FREE→BUSY transition or the message appearing in the agent's transcript"); R-8.2 requires the transcript. Current: R-8.2. (b) The AEF previous worker's "on success of the inject" was corrected by D-696.

## 9. Answering

### Requirements
- R-9.1 ANSWER READY: the receiver's sidecar tells the sender's sidecar; the sender pulls the answer.
- R-9.2 Roles swap for the reply.
- R-9.3 A woken agent always replies or explicitly says "no action"; never silence.
- R-9.4 If an agent's ears are dead, it halts instead of assuming there is no mail.

### Design
1. **R-9.1, TermLink: not built.** A reply is `agent-respond.sh --reply`, a `turn` post with `conversation_id` (`agent-respond.sh:129-131`). No `stage=replied` receipt exists anywhere, and the reply carries no `in_reply_to` (T3330R:1c1-1c2).
2. **R-9.1, AEF: partly.** REPLIED is recorded by the sender's receiver when a message answering the original is stored (`direct.py:258-286`). There is no separate ANSWER READY call: the reply is a new send (HA C2 row 6).
3. **R-9.2.** TermLink S12 is `partial`: the receiving half is built (`notify-sidecar-api.sh ack`); the reply travels `channel.post` by the SQ-1 bright line; the blocker is that the `framework-agent-systemd` unit lacks `termlink` in `--allowed-commands`, a different project's configuration (ARC11:183-201). AEF: the API is symmetric, a reply is the same call with sender and target swapped (T-3397:127-132).
4. **R-9.3.** The wake-protocol obligation is built as skill text: a woken agent drains all unread topics, posts a receipt per topic and for each replies or posts "an explicit acknowledged/no-action" (`.tasks/active/T-2402-…:59-60`; Stage 6, `work-completed`). It rests on the agent following its skill; no hook enforces it. It was not carried into the arc-011 chain (HT D3).
5. **R-9.4.** `scripts/notify-check.sh` returns MAIL 10, DEAF 3, CLEAR 0; a stale heartbeat means "deaf, halt" (DNS:33-55; T2291R:251-267). Built as a script. I found no hook or yield point that runs it. The 30-second-tick model has no equivalent (HT D2). The nearest live thing is AEF's `ESCALATED` state after a 900 s hand-over deadline (`direct.py:60, 291-299`), and the `check-notify-sidecar-freshness.sh` canary.

### Conversations and decisions
6. **2026-05-25, T-1807.** A woken Claude could not tell it was rung by a peer, so it would read the turn and never post a receipt (T1807R:66-77). T-1809 fixed it.
7. **2026-06-26, T-2291.** DEAF means halt: "the agent can no longer trust 'no flag = no mail,' so it halts" (DNS:25-29, 48-55).
8. **2026-07-07, T-2380, operator.** "our mechanism is not working", "we did extensive work on this, read back, was it arc 004?!", "THIS IS REALLY BAD; WHAT NOW?" (T2380R:182-186). GO "yes", 2026-07-09 (T2380R:137). Result: T-2402 Stage 6, "silence always means a bug" (HT A5bf).
9. **2026-09-21, AEF T-3397.** Symmetric API (T-3397:127-132).
10. **2026-09-22, arc-011 S12.** "roles swap — the agent replies as sender" (ARC11 S12).
11. **2026-10-03, operator read-back.** "ANSWER READY: the receiver's sidecar tells the sender's sidecar an answer is ready; the sender pulls it." (T3330R:259).
12. **Contradictions.** (a) R-9.1 says the sender pulls; AEF delivers the reply as a pushed new message. (b) June: DEAF means halt (agent self-check at yield points). October: a cron tick injects and nobody halts. The halt rule is stated once and not restated in arc-011 or IAC (HT D2). R-9.4 is confirmed, so it stands; it has no built owner. (c) R-9.3 versus the read-back: the read-back has no "no action" state; it is in R-9.3 only.

## 10. Fallback polling

### Requirements
- R-10.1 Where a push cannot land, the other side polls: 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 day, 3 days, 1 week, 1 month, 1 quarter, 1 year, each rung twice.
- R-10.2 This ladder is the framework default for all polling, changeable per situation.

### Design
1. **R-10.1: designed only.** No code implements the 12-rung ladder. What exists: (a) AEF's retry ladder 2×1 min, 2×5 min, 2×15 min, 2×1 h, 2×4 h, 2×1 d, 2×1 w, 2×1 mo, then dead-letter; about 76 days and 16 attempts; driven by the `sidecar-sweep-5m` cron; nudge from the 15-minute rung, operator surfacing from the 1-day rung (`AEF:lib/retry_ladder.py:16-27, 62-84`; `T-3434-retry-ladder.md:11-35`). (b) TermLink's sidecar polls a fixed 15 s (`notify-sidecar.sh:121`). (c) `channel post --await-ack --retry` (T-2285) and the offline queue flush every 5 s (T-2051).
2. **R-10.2: designed only.** No framework config key carries a default ladder. It was sent to AEF at `PICKUP@309` and `IN-AEF@180`; AEF filed T-3770 to reconcile it with T-3434 and make it the declared default (IAC:81; HA round 6.8b). T-3770 was not located in this repo.

### Conversations and decisions
3. **2026-03-23, T-256.** The earliest statement: no polling should be necessary (T256R:50). Polling returns now only as a fallback.
4. **2026-09-22, AEF D-600, operator ruling.** "Universal message retry ladder: every framework message kind … retries on one shared schedule — 2x1min, 2x5min, 2x15min, 2x1h, 2x4h, 2x1d, 2x1w, 2x1mo — then dead-letters … a post that never reached the hub is RE-POSTED on the rung; a post the hub holds but nobody read is ESCALATED … URGENT compresses the ladder — designed in a separate conversation, not here." (`decisions.yaml:4185-4190`). It deliberately outlives the hub's 5-minute dedupe window, so receiver-side dedupe on `client_msg_id` is mandatory.
5. **2026-10-03, operator, TermLink.** The ladder 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 d, 3 d, 1 week, 1 month, 1 quarter, 1 year, each rung twice (T3330R:261). Dictation said "50 seconds" and "50 minutes" for the first and fourth rungs, read as 15, "pending operator correction" (T3330R:261). The operator, relayed to AEF: "that should be the standard fallback mechanism for the framework for any polling activities. It can be changed situationally, but that should be the standard." (IN-AEF@180; HA E10).
6. **2026-10-03, AEF.** Filed T-3770 (IN-010@225, @228).
7. **Contradictions.** (a) D-600 (2026-09-22: 1 min … 1 month, dead-letter at about 76 days) versus the 2026-10-03 ladder (15 s … 1 year). Both are operator rulings. They differ in purpose (D-600 is a re-post-or-escalate schedule for a sent message; R-10 is a polling cadence), so they may coexist, but the ruling that merges them is T-3770, not decided: **O11**. (b) The "50" dictation: R-10.1 says 15 and the operator confirmed the read-back, but no explicit correction is recorded: **O11**.

## 11. Addressing and identity

### Requirements
- R-11.1 The five-level circuit: host / hub / project / session / agent.
- R-11.2 Each level has a canonical name and an instance identity: FQDN for the host, a hub name, the `pid` for the project, a role for the agent.
- R-11.3 Messages are addressed with `to_circuit` and never fall back across projects.

### Design
1. **R-11.1, AEF.** `circuit.py` emits `inbox:<hub>/<project>[/<session>/]<agent>`; the host-qualified id rides in `metadata.from_circuit` (`circuit.py:10-31`). A read-side sparse V9 form `aef::host=…::hub=…::project=…::session=…::@agent::` exists (`circuit.py:247-286`). Built.
2. **R-11.1, TermLink.** T-3325 added the `to_circuit` classifier, the sidecar filter and `agent-send.sh` stamping (commit `308ee29a7`, live since 2026-10-02 23:17Z; IAC:127). It reads path form and V9, writes path form, matches level by level from level 1. Built; it only decides which sidecar *wakes*, and nothing wakes an agent here.
3. **R-11.2, status per level.**
  3a. Host: FQDN recorded; the IP never appears in an address. Built.
  3b. Hub: the id used today is the TLS fingerprint (`cacc73ea32b121dd`), which rotates on certificate rotation (PL-021). It is an instance id, and a hub **name** is not built. AEF caches the first 16 hex of the fingerprint (`circuit.py:99-133`).
  3c. Project: AEF minted `pid-<16 hex>` in `.framework.yaml` (T-3534 built, back-fill T-3750). The sidecar still uses the folder basename (`circuit.py:136-153`); moving it is T-3751 (open).
  3d. Session: canonical id plus runtime id; open.
  3e. Agent: `FW_SIDECAR_AGENT_ID` else project, deliberately not the machine-wide TermLink fingerprint (`circuit.py:164-170`). Role plus instance is designed (7/7 review).
4. **R-11.3.** Written in path form now; V9 at AEF's write-side cutover (T-3325 Q1). "Never fall back across projects": agreed by both (IN-AEF@129; IN-010@141).
5. **Trust gap.** All agents on a host sign as one TermLink identity (the "D6" finding), so `dm:<fp>:<fp>` is shared by every project on the host. Per-agent keys are not built; AEF proposed named identities for trust, not routing (IN-010@53; HA B6.9).

### Conversations and decisions
6. **2026-06-27, T-2291.** RC1 identity: a shared host fingerprint lost a handoff (T2291R:12-33, 239-243).
7. **2026-09-06 onward, AEF T-3287.** Five levels, circuits, decisions D1–D7, grammar V9 (HA round 2).
8. **2026-09-22, AEF D-599 (operator ruling).** "topics move from sidecar:<agent-id> to inbox:<circuit-id> … what follows the prefix is the framework's five-level circuit id … DURABLE ROLE ADDRESS at project level (inbox:<hub>/<project>) … EXACT CIRCUIT ADDRESS down to the agent for a dispatched worker … sidecar:* kept as a read-only transition alias for one release." (`decisions.yaml:4178-4183`).
9. **2026-09-25, operator.** Five-part circuit with V9; "dual-read both grammars… current address as a read-only alias for one release" (IN-010@39, HA round 5.4).
10. **2026-09-27, AEF D-660 (operator ruling, T-3518).** "AEF adopts 010-termlink's addressing — dm:<fp>:<fp> / inbox:<agent-id> — and does NOT keep sidecar:<agent-id>. … Option 3 (support both) was explicitly refused" (`decisions.yaml:4617-4622`). On 2026-09-29 AEF wrote that "its 'inbox:<agent-id>' wording was a mistake on our side and is being amended" (IN-010@53). The amended text was not found.
11. **2026-10-03, T-3325 Q1 = C.** Operator words: "Cozzyte is suggested and you see fit", read as "C, as suggested, proceed as you see fit" (voice transcription; the reading is stated back and is overturnable) (`.tasks/completed/T-3325-…:283`). Earlier that day the operator directed the five-level circuit with a fallback ladder towards level 1 (T-3325:308). The key is `to_circuit`; both grammars read; path form written; match from level 1; deepest level that resolves; a message with no address wakes (the old behaviour). Score C +73, A +44, B +27, D −11.
12. **2026-10-03, the naming model.** TermLink's operator: every level carries two identities (IN-AEF@186). AEF replied that it is "the same model our operator proposed on 2026-10-03" and ran a 7-vendor review: 7/7 adopt with changes; decision 2a, 7/7 option B, function names route and instance ids ride inside (IN-010@250; `AEF:docs/reports/T-3751-review-synthesis.md:19-150`). Respawn from inbound mail (2b) needs an explicit operator grant. No operator ruling on 2a was found. T-3751 IW-1 = C (2026-10-03): the project slot carries the minted id (`T-3751-review-brief.md:105-108`).
13. **Contradictions.**
  13a. **D-599 versus D-660.** D-599: `inbox:<circuit-id>`. D-660: `inbox:<agent-id>`, and "does NOT keep sidecar:". AEF says D-660's wording was a mistake (IN-010@53). Current: D-599 (the five-level circuit), but the correction is unverified in `decisions.yaml`: **O12**.
  13b. **The fallback ladder.** The operator's rule: fall back to the deepest level that resolves, never across projects (T-3325:308; IAC:29). AEF T-3287: on a dead endpoint climb 5→4→3→2→1 and re-provision (T-3287:80-107). AEF's brief: "the sidecar never climbs at send time; its lowest address is `<hub>/<project>`" (`T-3751-review-brief.md:101-103`). T-3325:309 says "no written copy was found" of the operator's description. **O5/O6**.
  13c. **Hub id.** R-11.2 says "a hub name"; the id in use rotates (IAC:24). Not built.

## 12. Telemetry

### Requirements
- R-12.1 Every step is a timestamped event, copied to the hub, pullable from any agent.
- R-12.2 Each agent posts a daily digest and reflects on it.
- R-12.3 Alarms only for urgent messages; everything else escalates when it piles up, like audit warnings.
- R-12.4 Later: an observability database and the learning from it; possibly a hub-steward agent.

### Design
1. **R-12.1, TermLink: designed only.** The T-3330 ruling (IW-1 = C amended): each sidecar records its own events and copies them to the hub through a local outbox, never in the message path; the hub keeps them for a retention window; any agent can pull (`.tasks/active/T-3330-…:181-188`; IAC §7). IW-2 (the event record, the pull verb, the digest) is open and unbuilt. Today the data exists in pieces: hub receipts, flag `ts=`, the journal, the sender ledger.
2. **R-12.1, AEF: local only.** Per-direction ledgers `direct-ack.jsonl`, `receipts.jsonl`, `receipts-sent.jsonl`, `receiver/events.jsonl`, `watcher/ticks.jsonl`, `liveness.yaml`; `fw sidecar latency` reads them (`direct.py:19`, `receipts.py:28-36`, HA B7). No hub copy. A per-hop `telemetry.jsonl` is a design (`sidecar-roundtrip-and-telemetry.md:184-247`) and I found no code writing it.
3. **R-12.2.** The digest is not built (IAC:126). Reflection is linked to T-3319 "learn from message traffic" (`work-completed`; I did not read its result).
4. **R-12.3.** Not built as specified. Existing infrastructure canaries fire on their own checks (waker liveness T-2387, unconfirmed delivery T-2295, dead-letter T-2558, `check-notify-sidecar-freshness.sh`). The urgent-only immediate alarm, and the "piles up, then escalates" path, are designed only. A precedent for escalation exists: the guard layer's WARN tier escalates after 14 days (T-3260, CLAUDE.md).
5. **R-12.4.** The hub-steward agent is T-3333, `captured`, no recommendation. The observability database is its own later inception.

### Conversations and decisions
6. **2026-10-02, T-3319.** Operator asked to learn from message traffic (`.tasks/completed/T-3319-…:6`) (HT B15a).
7. **2026-10-03, operator, standing requirement.** "as a standard, also for all vendor agents, to collect the telemetry of our communications … especially with the sidecar … the different steps, how much time it takes and how much delay … also for other inbox things. And you need to be able to pull that information from different agents, additionally they also should be able to offer it on a regular basis, let's say one times per day … So you can work with that information. And you infer or reflect on what working with that information means." (T3330R:231).
8. **2026-10-03, external review of IW-1.** Codex, GLM-5.3, qwen3 and gemma4 chose C 4/4. Objections: two histories from async mirroring (Codex); the mirror is "an out-of-band duty a component can skip, the same bug class as the incident" (GLM); missing: a liveness invariant, "canary mail with deadlines (INJECTED within T1, REPLIED within T2)" (GLM, "R6"); gemma4 dissented on the hub as home (T3330R:233-246).
9. **2026-10-03, the ruling.** IW-1 = C amended, in four parts: protocol, telemetry, alarms, observability database later (`T-3330-…:181-188`). Operator words recorded in the read-back: "Liveness alarm only for urgent messages. Everything else accumulates and escalates when it piles up, like audit warnings (design to be decided)." and "The hub keeps the telemetry for now (the reproducible entity). An observability database comes later (the AEF agent, a specific hub or a specific agent, possibly bundled with every hub) … linked to T-3319." (T3330R:265-266). The operator also proposed the hub steward agent (T-3333:119).
10. **Left open by the ruling.** How alarms and escalations surface; IW-2, IW-3, IW-4; sending the design to AEF for overlap feedback (`T-3330-…:181-188`).
11. **AEF's view.** Cross-agent collection option (b), a collector that pulls each agent's file, because option (a) "fails exactly when the rail fails" (`sidecar-roundtrip-and-telemetry.md:235-247`). AEF answered "RECEIVED" to the overlap question with the comparison "to follow" (IN-010@251).
12. **Contradictions.** (a) IW-1 = C: each sidecar *pushes* events to the hub. AEF recommends a collector that *pulls*. (b) GLM's liveness invariant (alarm when mail for a project has no injectable session) versus the operator's "alarms only for urgent": the operator's rule stands; GLM's R6 is not in the requirements (see Open decisions, O16). (c) The panel's "record-and-pull" versus the operator's push-by-API (**O10**).

## 13. Deployment

### Requirements
- R-13.1 Every sidecar ships with every TermLink deployment.

### Design
1. **Today: not met.** `release.yml` publishes five binaries plus `checksums.txt` and no scripts (`release.yml:257-263`). `install.sh`, the Homebrew formula and `scripts/fleet-deploy-binary.sh` move a binary only (T3330R:3a-3d).
2. **The coupling.** About 13 receive-side scripts (about 2,460 shell lines for the core chain, about 3,900 with the launchers) run only from the `/opt/termlink` checkout. MCP tools resolve scripts at `${TERMLINK_SCRIPTS_DIR:-/opt/termlink/scripts}` (`crates/termlink-mcp/src/tools.rs:30274-30285`) and return "script not found" elsewhere (T3330R:3e-3f). There is no `sidecar` subcommand in the CLI (T3330R:3g2).
3. **How it is installed here.** `runme.sh` is the host's install path (CLAUDE.md, T-3272); its action 10 restarted the three sidecars on 2026-10-02.
4. **The options.** T-3330 listed four (T3330R:4a-4d): (A) adopt AEF's receiver; (B) extend TermLink's scripts; (C) move the receive side into the binary as `termlink sidecar …`, which ships through every existing channel; (D) defer. No ruling. IW-3 of T-3330, "how sidecars ship; whether TermLink's own receive scripts are replaced by AEF's sidecar", is open: **O9**.
5. **Portability.** The shell core needs `bash`, `jq`, `sqlite3`, `python3`, `tmux`, cron; the supervisor can emit systemd, launchd or cron units (ARC11 S1). README promises macOS in five places, and the platform-lock check (T-2693) governs new Linux-only reads.

### Conversations and decisions
6. **2026-09-24, SQ-8.** "I want a solid solution." Portable respawn is required (ARC11:350-367).
7. **2026-10-03, operator.** "are all the sidecars we develop … part of our [TermLink] package? When the deployment is done we should deploy all the sidecars with it." (T3330R:230). The agent checked and found releases ship only the binary (T3330R:§3). IAC:42 restates it as a requirement.
8. **Contradictions.** None recorded. The ruling that would settle how is missing (**O9**).

## 14. Discussed once and then lost

### Requirements (to be confirmed or dropped by the operator)
- R-14.1 Native consumer: the agent confirms from inside its own turn (August, T-2838).
- R-14.2 Typed assignment and result messages (August).
- R-14.3 The hub refuses to call a message delivered without a receipt (August).
- R-14.4 The startup chain (September; never built).

### Design
1. **R-14.1.** Spike S2 proved "A receipt cannot exist unless a turn happened." (T2838R:175-208). The native consumer was a spike and never a build. Built from T-2838: the per-agent identity prerequisite, `ack-status` fixed, `channel post` made honest (`delivered-unconfirmed` versus `consumed`) (HT A6cd). AEF's UserPromptSubmit hook plus transcript finalizer gives the same by-construction property for AEF-armed sessions (`hooks.py:14-24`), so R-14.1 is partly met by another route.
2. **R-14.2.** Typed envelopes `assignment.v0` and `result_manifest.v0` landed as helpers with "no CLI verb emits or consumes these yet" (T2838R item 3, "Not claimed"). Designed, helpers only.
3. **R-14.3.** Hub-side enforcement was the last item of the T-2838 build order and was never built (T2838R:422-427). Not built.
4. **R-14.4.** The same sentence as R-3.4 (RAIL:111-117). It duplicates R-3.4. Not built.

### Conversations and decisions
5. **2026-08-24/25, T-2838.** Diagnosis: "the detector layer is saturated (six detectors) but the mechanism was never built" (T2838R:41-66). Native consumer for agents we launch, PTY inject for sessions we do not (T2838R:92-102; 414-420). The operator's goal: "the orchestrator uses TermLink to dispatch an assignment to an agent with an asserted agent profile ... and should come back by writing those files and reporting back those files have been written" (T2838R:555-562). GO 2026-08-25 (`.tasks/completed/T-2838-…:206-228`).
6. **Why lost.** A6 is never mentioned in arc-011, T-3200 or IAC (HT D1, E3). The loss mechanism is recorded: "starting work without reading the record that already contains the answer" (`.tasks/completed/T-3200-…:76-84`).
7. **Contradictions.** None recorded. Keep-or-drop is **O7**.

## 15. Status

### What operates today
- 15.1 AEF has built most of sections 4–9 in its bleeding-edge sidecar, reaching only agents started with `claude-fw --termlink`.
- 15.2 On this host nothing injects into a running agent: our framework copy does not contain AEF's sidecar, and no agent here was started injectable.

### Detail
1. **How to read the table.** "Recorded" is what a register or a task says. "Operating" is what I could confirm on 2026-10-03, 23:24 UTC, or what the source states. A recorded "built" that is not operating is the failure this document exists to stop (IAC:125).

| Item | Recorded as | Operating | Evidence |
|---|---|---|---|
| Hub transport, topics, receipts, offline queue | built | yes | CLAUDE.md T-2049/T-2051; IAC:127 |
| Notify sidecars (3 agents) | built | yes, receipts only | `pgrep`; T3330R:1a6c |
| Sidecar supervisor and wake-supervisor crons | built, "VERIFIED FIRING" (ARC11 S11) | yes, `*/5`, they supervise processes | `.context/cron/*.crontab` |
| Wake consumers (2 agents) | built | yes, log a line only | `notify-wake-consumer.sh:28-32` |
| Injector `notify-injector.sh` | S7, S9, S10 "built", "PROVEN LIVE" | **no**: scheduled by nothing | header `:4`; no crontab names it |
| Push-waker `be-reachable-pushwaker.sh` | arc-004 "shipped" 85–111 ms | **no**: no instance, no bound `pty_session` | `pgrep`; T3330R:1b6; HT C2 |
| 30-second tick (TermLink) | designed | no (`*/5`, checks no prompt) | IAC:124 |
| AEF receiver, watcher, hooks, injector | built, live two-agent e2e 3/3 (IN-010@62) | only for `claude-fw --termlink` agents; not here | HA D5; `ls .agentic-framework/lib/sidecar` fails |
| AEF cross-host receiver (T-3688) | planned | no | `lifecycle.py:20` |
| Local sidecar API | S1 "built" | yes, local control only, not a sender API | ARC11:34-38 |
| STORED, ANSWER READY calls; telemetry; digest | designed | no | IAC:126 |
| Per-message addressing `to_circuit` | built | yes since 2026-10-02 23:17Z, decides who wakes | IAC:127 |
| Sidecars shipped with every deployment | requirement | no | T3330R:§3 |

2. **Recorded-as-built, not operating, with the cause.**
  2a. arc-003 "no silent loss" (closed 2026-07-02): the sidecar it rests on had not run since 2026-07-01 (T-3049, HT C1).
  2b. arc-004 push-wake "shipped" (2026-07-02): zero wakers on 2026-07-07 and again on 2026-09-28 (T2380R:63-79; `notify-injector.sh:14-18`, HT C2).
  2c. "Wake delivered" (July): a rung is not a read (PL-253, HT C3).
  2d. First end-to-end proof of the notify rail (2026-09-22): flag raise and auto-confirm only, never an agent's prompt (DNS:204-212, HT C4).
  2e. arc-011 S10 "PROVEN LIVE": one run on one fresh topic (ARC11:148-170, HT C5).
  2f. T-3200 premise "nine of ten slices built": inbox mail was invisible to the rail for six days (HT C6).
  2g. T-3325 "the sidecar wakes only the right agent": said while nothing reaches the agent (IAC:10, HT C8).
  2h. AEF "R14, R15 built": true for wrapper-started agents only (HA D5).
3. **The closing rule (IAC:128).** Nothing in sections 4–9 may be called working until a live test passes with two real running agents: a message injected while the receiver is busy and while it is idle, every step confirmed to the sender with timestamps, the answer returned, and a negative control.
4. **Not verified by me.** AEF's e2e claim; whether T-3770, T-3688 and T-3751 moved since 2026-10-03; the amended D-660 text.

---

## Open decisions

Each is one unresolved question. "Position" quotes both sides.

**O1. The send path against the charter's no-second-bus rule.**
1. Position A, sidecar to sidecar by API: "Send: sender agent -> its own sidecar (API) -> the receiver's sidecar (API)." (operator read-back, T3330R:252); "The sender's sidecar delivers it to the receiver's sidecar through the receiver's API." (R-4.2).
2. Position B, sending stays on the hub: SQ-1, "sending to a peer stays channel.post via the hub" (ARC11:28-44); the API "may read and act on this host's own state, never move a message between hosts" (SCAPI:17-35, 153-172); "if the real requirement is cross-host delivery while the hub is down, then this is not a local-control API at all, it is a second bus, and the honest move is to say so and put it to the charter" (SCAPI, "What would change my mind").
3. Context: AEF already does receiver to receiver on one host and plans cross-host as T-3688 (`direct.py:1-26`; `lifecycle.py:20`). The T-3330 ruling gives cross-host sidecar calls to AEF (`T-3330-…:181-188`). The operator said "We have multiple hosts, so that's not a question." (T-3397:270-273).
4. To decide: does the charter change, does AEF own cross-host, or does the hub carry it?

**O2. RECEIVED and STORED: one call or two.**
1. Two: "RECEIVED: the receiver's sidecar immediately calls the sender's sidecar. STORED: called again once stored." (T3330R:253-254; R-5.1, R-5.2).
2. One: AEF answers RECEIVED after the durable store, so RECEIVED already means stored (`http_server.py:86-103`; HA C2 row 6; IAC:134).
3. To decide: keep two calls (a fast "I have it" before the disk write) or merge.

**O3. How an already-running session becomes reachable.**
1. Position: none recorded from the operator (HT B13c; IAC:135).
2. Facts: injection needs a terminal the injector can type into; "cannot be retrofitted onto a running headless claude" (`notify-injector.sh:20-24`, PL-237); "Who arms agents with `pty_session` (an operator relaunch via tl-claude.sh)?" (T3330R:225); the T-3330 IW-4 "interim wake path" is open.
3. R-1.3 ("no agent has to be attached") conflicts with this today.
4. Options to put to the operator: relaunch every agent through `tl-claude.sh start --reachable` or `claude-fw --termlink`; accept that running sessions are reached only by the pull and the session-start listing (T-3327); or build a harness-side channel (hooks only, no PTY).

**O4. Urgent into a busy prompt.**
1. Operator, 2026-09-23 (SQ-4): "urgent NEVER injects into a BUSY prompt … Urgent shortens the WAIT; it does not bypass the prompt-free CHECK." (ARC11:305-316).
2. Operator, 2026-10-03: "inject when it's free and with urgent bypass"; "urgent gets injected immediately" (CONSULT:16-17); R-6.3.
3. R-6.3 adds "by a route that cannot silently lose the message". No such route exists: AEF detects loss after 120 s (`inject.py:46`); the operator's earlier "out of band" idea was the PTY doorbell (SCAPI:207-234).
4. To decide: record that the 2026-10-03 rule supersedes SQ-4; choose the safe route; decide interrupt consent (T-3200 IW-1, "should an inbound peer message interrupt a working session", HT D10).

**O5. Who keeps the name-to-id directory.**
1. Agent recommendation: the hub keeps project identity cards (IAC:30).
2. Operator: no ruling (T-3751 IW-3 pending, IAC:137; synthesis:117-150).
3. To decide: hub, AEF, or each project.

**O6. The session level of the address.**
1. Operator: "a canonical id plus a runtime id" for session (IAC:26).
2. Review: "routing does not stop at a session label" (6/7) (IAC:26).
3. AEF: per-session readiness records exist (T-3745) but the address session slot is optional (`circuit.py:10-31`).
4. Also open: the fallback rule itself has no written source (T-3325:309).

**O7. Section 14 items: keep or drop.**
1. R-14.1 native consumer: partly met by AEF's prompt-hook plus transcript evidence (`hooks.py:14-24`); T-2838 built only part (T2838R item 3). Suggest: keep only if TermLink wants a non-AEF consumer.
2. R-14.2 typed messages: helpers landed, no verb uses them. Keep, as a later slice.
3. R-14.3 hub refuses to call a message delivered without a receipt: never built. Keep or drop.
4. R-14.4 startup chain: duplicates R-3.4. Suggest: drop R-14.4 and keep R-3.4.

**O8. Readiness: harness hooks or screen.**
1. R-7.1/R-7.2: harness-reported, not inferred from the screen.
2. The built TermLink classifier infers from the screen (ARC11 S7; IAC:55), and the operator said "use PTY inject when the cursor is silent" (T-3396:87-92).
3. TermLink has no hook readiness; it told AEF "do build (b)" (ARC@1640); T-3250 is `captured`.
4. To decide: who owns the readiness hook for non-AEF agents, and whether the screen classifier stays as a second opinion or is retired.

**O9. How sidecars ship and who owns the receive side (R-13.1; T-3330 IW-3).**
1. Options A–D (T3330R:4a-4d). A: AEF's receiver, only AEF-governed projects. B: TermLink scripts, shipped by no package. C: `termlink sidecar` in the binary, about 2.5–3.9k shell lines to port. D: defer.
2. Operator: ships "with it" (T3330R:230). Which implementation: open.

**O10. Synchronous call back, or record and pull.**
1. Operator: push by API between sidecars, pull only as the fallback (T3330R:250-261).
2. Codex and GLM: the call back is "wrong as written" (it couples to the sender's availability; "who receipts the receipt"); record-and-pull meets the need (T3330R:242). AEF built the synchronous `/ack` and recommends a collector pull for telemetry (HA C2 rows 7–8).
3. To decide: confirm the operator's design against the review, and say what happens when the sender's sidecar is down (the polling ladder, R-10.1).

**O11. The polling ladder.**
1. R-10.1: 15 s … 1 year, each rung twice; "50" read as 15, no recorded correction.
2. D-600: 2×1 min … 2×1 month, then dead-letter; urgent compression not designed.
3. To decide: confirm the first rung; merge or separate the two ladders; make it a framework config default (R-10.2); how urgent compresses it.

**O12. Address rulings D-599 and D-660, and the stable ids.**
1. D-599: `inbox:<circuit-id>`. D-660: `inbox:<agent-id>`, no `sidecar:` (`decisions.yaml:4178-4183, 4617-4622`). AEF says D-660 is being amended (IN-010@53), not verified.
2. R-11.2 wants a hub name and the `pid`; the code uses the TLS fingerprint and the folder name (`circuit.py:99-153`). T-3751 and the dual-read migration are open; decision 2a (7/7 option B) has no operator ruling.
3. The `sidecar:` alias "for one release" has no end date (T-3461:128-145; S7 T-3690 needs peer agreement).

**O13. Stage names.**
1. TermLink INJECTED needs a BUSY transition; AEF HANDED_OVER needs transcript evidence; "This is not decided anywhere." (T3330R:223).
2. To decide: one vocabulary, and whether the weaker TermLink evidence may ever be called INJECTED (R-8.2 says transcript).

**O14. Alarms, escalation, and the hub steward.**
1. "How alarms and escalations surface is still to be designed" (IAC:94); the steward is `captured` (T-3333).
2. Operator rule: alarms only for urgent; the rest escalates like audit warnings (T3330R:265).

**O15. Interrupt consent and respawn from inbound mail.**
1. Interrupt: raised in T-3200 and dissolved by SQ-4, and with SQ-4 reversed it returns (HT D10).
2. Respawn: the 7-vendor review says no agent is respawned by inbound mail without an explicit operator grant, budget, restart limits and an authenticated sender (IAC:27; synthesis:42-58).

**O16. Proposed changes to the requirements (for the operator; none made).**
1. **R-3.2/R-4.2/R-1.3 conflict with each other and with section 15.** Not a wording change, a question: while the receiver's sidecar depends on an injectable session, R-1.3 is false for running agents (O3).
2. **Missing: peer content is untrusted.** AEF frames peer text `<<<PEER-DATA … PEER-DATA>>>` and says "a request for action becomes a task proposal … never direct execution" (D-695). Proposed: add as a requirement in section 8 or 9.
3. **Missing: exactly-once.** Receiver-side dedupe on `client_msg_id` is mandatory because the ladder outlives the hub's dedupe window (D-600; T-2049). Proposed: add to section 5.
4. **Missing: how urgent is marked.** The queue uses `priority` clamped to [-9,9] and urgent at ≥5 by default (ARC11 S8, S9). Proposed: add to section 6.
5. **Missing: a liveness invariant.** GLM's "canary mail with deadlines, INJECTED within T1, REPLIED within T2" (T3330R:237-246; HT D5). R-12.3 says alarms only for urgent; this would be a counter-proposal. Operator to say yes or no.
6. **Missing: the closing rule.** "Nothing is working until two real running agents pass a live test with a negative control" (IAC:128). Proposed: add to section 15 or the Goal.
7. **R-5.2/R-9.1 wording.** If O2 or O10 changes, R-5.2 and R-9.1 change with it.
8. **R-11.2.** "a hub name" is not what runs; the hub id is the fingerprint (O12).
9. **R-14.4** duplicates R-3.4 (O7).

## Decision log

Operator words are quoted only where a document records them. "Agent-recorded" means the document states a decision and does not quote the operator.

| Date | ID | Decision | Operator's words or record | Source |
|---|---|---|---|---|
| 2026-03-08 | T-007 | GO: PTY-backed sessions plus pipe for execute (hybrid) | Date from the skeleton hint | T007R:239-253 |
| 2026-03-18 | T-099 | NO-GO as a feature request: `Stop` and `SessionEnd` already exist | agent-recorded | T099R:101-109 |
| 2026-03-23 | T-256 | Convention now (collect fan-in), `emit-to` later | "the spawned agent can talk to the spawning agent" | T256R:50, 73-76 |
| 2026-04-24 | T-1207 | Stop hook: no block, ask the user; live from day 1 | "Option B — live from day 1." | T1207R:68-72 |
| 2026-04-26 | T-243 | GO on the multi-turn dialog protocol | "use termlink !!!" (T243R:124) | T243R:96-124 |
| 2026-05-25 | T-1800 | GO: doorbell plus mail; persistent session primary | "aim not to use claude-p for expensive jobs"; "make it a deep inception, it's key functionality" | T1800R:131; `.tasks/completed/T-1800-…:135,150` |
| 2026-06-27 | T-2291 | GO on the V3+V2+V1 composite; discovery two-tier; direct first, hub fallback | "(operator decision): TWO-TIER" | T2291R:193-225, 302 |
| 2026-07-02 | T-2303 | GO (scoped): WebSocket push; arc-004 closed shipped | "incept the addition/replacement of webhooks and websockets" | T2303R:101-102; `push-transport.yaml:54-55` |
| 2026-07-02 | T-2315 | Advisory GO: a push-waker rings the PTY; final decision human | not verified in this pass | T2315R:90-92 |
| 2026-07-09 | T-2380 | GO: loud end-to-end delivery contract | "yes" | T2380R:137 |
| 2026-07-10 | T-2396 | Prove first, then build | "prove-first, then build" | T2396R:4 |
| 2026-08-25 | T-2838 | GO: delivery-to-turn contract | agent-recorded | `.tasks/completed/T-2838-…:206-228` |
| 2026-09-01 | T-2876 | Message prover asserts on the receiver | "test everything end to end, and also build that as a testing harness" | `.tasks/completed/T-2876-…:43` |
| 2026-09-20 | AEF T-3396 | Build the long-term road, with PTY inject on idle | "Let's go to the Royal and the Correct Long Term Road. Let's pack it full out." | T-3396:104-105 |
| 2026-09-21 | AEF T-3397 | Urgent is a hard bypass; cross-host is live | "We have multiple hosts, so that's not a question." | T-3397:194-199, 270-273 |
| 2026-09-21 | CONTEXT_WINDOW | 800000 (earlier setting, superseded by 950000) | "Operator instruction (2026-09-21) is to set the cap to 800000" | `.tasks/completed/T-3047-…` |
| 2026-09-22 | SQ-2 | Premise disproved; the blocker was the idle classifier | agent-recorded, on operator approval | ARC11:214-246 |
| 2026-09-22 | SQ-5 | Install the wake-supervisor cron | "operator answered INSTALL" | ARC11:317-349 |
| 2026-09-22 | D-599 | Addressing `inbox:<circuit-id>`; five levels; durable role plus exact circuit | "Operator ruling 2026-09-22" | `AEF decisions.yaml:4178-4183` |
| 2026-09-22 | D-600 | Universal retry ladder 2×1 min … 2×1 month | "Operator ruling 2026-09-22" | `decisions.yaml:4185-4190` |
| 2026-09-23 | SQ-1 | The injector stays in TermLink; the sidecar API is local control | "operator decided: the injector belongs to TERMLINK" | ARC11:203-213 |
| 2026-09-23 | SQ-3 | Add `termlink artifact put/get` | "operator decided: YES" | ARC11:247-259 |
| 2026-09-23 | SQ-4 | Urgent never injects into a busy prompt | "operator decided: urgent NEVER injects into a BUSY prompt" — **contradicted 2026-10-03, not recorded as superseded** | ARC11:305-316 |
| 2026-09-23 | SQ-6 | Backfill once, recorded as a backfill | "operator decided: backfill ONCE, recorded AS a backfill" | ARC11:260-280 |
| 2026-09-23 | SQ-7 | The sixth T-3068 AC is obsolete | "operator decided: the sixth AC is OBSOLETE" | ARC11:281-304 |
| 2026-09-24 | SQ-8 | Solid over quick; portable respawn required | "reliability and fragility, so I'm not to save a few tokens just to get a quick solution. I want a solid solution." | ARC11:350-367 |
| 2026-09-25 | D-645 | Build toward the T-3397 design; blobs through the sidecar API | "Operator ruling 2026-09-25" | `decisions.yaml:4512-4517` |
| 2026-09-25 | circuit | Five-part circuit, V9 grammar, dual read | "dual-read both grammars… current address as a read-only alias for one release" | IN-010@39 (HA) |
| 2026-09-27 | D-660 | AEF adopts `dm:`/`inbox:` addressing, not `sidecar:` — wording conflicts with D-599, being amended | "Option 3 (support both) was explicitly refused" | `decisions.yaml:4617-4622`; IN-010@53 |
| 2026-10-02 | D-696 | HANDED_OVER only when the prompt hook surfaced the message | agent-recorded correction, not an operator quote | `decisions.yaml:4869-4874` |
| 2026-10-02 | D-700 | AEF keeps regular contact with 010-termlink and 055 | "Standing directive (operator 2026-10-02)" | `decisions.yaml:4897-4902` |
| 2026-10-02 | watcher | 30-second watcher with urgent bypass | "a watcher … watching if the message flag is up every 30 seconds …" | `watcher.py:3-6` |
| 2026-10-03 | T-3325 Q1 | C: `to_circuit`, read both grammars, write path form | "Cozzyte is suggested and you see fit", read as "C, as suggested, proceed as you see fit" (voice, overturnable) | `.tasks/completed/T-3325-…:283` |
| 2026-10-03 | T-3330 IW-1 | C amended: sidecar API calls, events copied to the hub, daily digest, urgent-only alarms, observability database later | read-back confirmed by the operator | `T-3330-…:181-188`; T3330R:248-266 |
| 2026-10-03 | T-3330 | Telemetry as a standard for all vendor agents | "as a standard, also for all vendor agents, to collect the telemetry of our communications …" | T3330R:231 |
| 2026-10-03 | T-3330 | Ship every sidecar with the deployment | "When the deployment is done we should deploy all the sidecars with it." | T3330R:230 |
| 2026-10-03 | tick | 30-second flag tick; urgent immediately | "Every 30 seconds … urgent gets injected immediately." | CONSULT:17 |
| 2026-10-03 | ladder | Standard polling ladder, 15 s … 1 year, each rung twice; "50" read as 15 | "that should be the standard fallback mechanism for the framework for any polling activities. It can be changed situationally, but that should be the standard." | T3330R:261; IN-AEF@180 (HA E10) |
| 2026-10-03 | naming | Two identities per level; project canonical id is the minted `pid`; project slot carries the minted id (T-3751 IW-1 = C) | operator proposal relayed (IN-AEF@186); 7-vendor review 7/7; **no operator ruling on 2a** | IN-010@250; `T-3751-review-brief.md:105-108` |
| 2026-10-03 | CONTEXT_WINDOW | 950000: 900k is the working limit; `TOKEN_CRITICAL` about 902k | "900k tokens is the session limit" (T-3332 description) | `.tasks/completed/T-3332-…` |
| 2026-10-04 | T-3335 | Skeleton of this document (15 sections, R-x.y) confirmed by read-back | confirmed by the operator | this file, header |

Two notes on the table:

1. **Naming model.** The 7-vendor result is a review, not a ruling. The "ruling" entry is limited to what the operator said (two identities per level) and to T-3751 IW-1 = C.
2. **CONTEXT_WINDOW** is not part of the communication design. It is listed because the budget thresholds govern how long an agent session can run, and the operator asked that stop conditions be stated by threshold name (T-3192, CLAUDE.md).
