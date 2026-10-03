# T-3335 — Design history of agent-to-agent communication in TermLink

**Task:** T-3335 (inception) · **Compiled:** 2026-10-03 · **Scope:** read-only survey, this file is the only write.
**Method:** every claim carries `file:line`. Operator words are quoted only where a document records them. Where a quote is a paraphrase inside a document, it is marked "(recorded as)".

Aliases used for citations (all under `/opt/termlink`):

- IAC = `docs/design/interactive-agent-communication.md` (T-3335 consolidation, 2026-10-03)
- RAIL = `docs/design/arc-011-message-delivery-rail.md` (2026-09-22)
- SCAPI = `docs/design/arc-011-sidecar-api-architecture.md` (T-3075, 2026-09-22)
- ARC11 = `.context/arcs/arc-011.yaml`
- DNS = `docs/operations/deterministic-notify-sidecar.md`
- V6 = `docs/plans/T-2296-v6-direct-transport-first-design.md`
- T243R / T1800R / T2291R / T2303R / T2380R / T2396R / T2838R / T3330R = `docs/reports/T-243-…`, `T-1800-…`, `T-2291-…`, `T-2303-…`, `T-2380-…`, `T-2396-…`, `T-2838-…`, `T-3330-receive-side.md`
- CONSULT = `docs/reports/T-3335-interactive-communication-consult.md`

**Headline finding.** There are NINE distinct design rounds, not five (A1 to A9 below). The operator's "about five" most likely means the rounds that each ended in a *named mechanism*: doorbell+mail (A2), deterministic notify sidecar (A3), push-wake (A4), delivery-to-turn contract (A6), and the arc-011 sidecar spec (A7, restated in A9). The rounds between them (A1, A5, A8) are the ones that get lost, and each loss has a documented cause: a later round was written without reading the earlier one (see C5, C6, A8 below).

---

# A. ROUNDS (chronological)

## A1. Round 0 — 2026-04-26 — T-243 multi-turn conversation primitive (the origin of "interactive")

A1a. **Sources.** `T243R:84-130` (dialogue log), `docs/conventions/multi-turn-dialog.md:1-25`.
A1b. **Design as stated.**
  A1ba. Agent conversation is carried by existing primitives: `channel.post` with `metadata.conversation_id`, `event_type`, long-poll `channel.subscribe`, and `dialog.presence` (`docs/conventions/multi-turn-dialog.md:17-25`).
  A1bb. A three-layer model (content / activity signal / presence) with a "typing/processing signal is the killer feature" hypothesis (`T243R:116-120`).
  A1bc. Three-agent fan-out inception run through TermLink itself (`T243R:124-128`).
A1c. **Operator's words (recorded verbatim in the dialogue log).**
  A1ca. "no not completely, i wanted a means to simulate keyboard input and capture console output, because this enables several use cases for agentic engineering." (`T243R:96`)
  A1cb. "3 we keep having key rotation issues, authentication issues, and interactive multi turn conversation between two or more agents is absolutely not working" (`T243R:100`)
  A1cc. "send and wait instead of immediate response and interaction … even consider to adapt a opensource chat protocol aka signal style … please analyse how a conversation normally flows, and you know agents best." (`T243R:115`)
  A1cd. "1 yes and we should three agent incept this. 2 also her three agent incept this --> use termlink !!!" (`T243R:124`)
A1d. **Outcome.** The protocol layer shipped; the missing piece was runtime pickup (stated in A2).

## A2. Round 1 — 2026-05-25 / 05-28 — T-1800 doorbell+mail, then T-1830 adoption gap

A2a. **Sources.** `T1800R:6,46,50-88,117-136`; `.tasks/completed/T-1800-interactive-agent-conversation-runtime--.md:135`; `docs/reports/T-1830-doorbell-mail-adoption-gap.md:11-40`; `.context/project/learnings.yaml` (doorbell+mail entries near 2167, 2192).
A2b. **Design as stated.**
  A2ba. **Mail = `channel.*`** carries every turn. **Doorbell = `command.inject`** is a fixed wake nudge (`/check-arc`) typed into the receiver's PTY; it never carries content (`T1800R:50-64`).
  A2bb. Happy path: A posts a turn, A injects the doorbell into B's PTY, B runs `/check-arc`, reads the structured turn, replies with a post and a `receipt` (`T1800R:56-64`).
  A2bc. **Determinism** by an atomic send verb (post, confirm offset, then ring), a closed-loop receipt, and bounded re-ring (`T1800R:77-87`).
  A2bd. **Receiver must be a hub-registered PTY session**, started via `termlink spawn --backend tmux`; plain `claude` in a user terminal cannot be targeted (`T1800R:68-75`). `claude-fw` is orthogonal (`T1800R:70-74`).
  A2be. Known ceiling recorded on day one: if the receiver is mid-turn or at a permission prompt, injected text queues or is mis-consumed, so delivery is "eventual, not instant" (`T1800R:87`).
A2c. **Operator's words.**
  A2ca. "interactive conversation between two or more agents… send-and-wait instead of immediate response… we want this interactive style of conversation to take place." Called "key functionality." (`T1800R:6`)
  A2cb. "SSE is not mandatory — the goal is push messages + automatic pickup for truly dynamic, interactive conversation." (`T1800R:122`)
  A2cc. "what can we do with pty inject or other terminal inject — do we need to redesign the protocol?" (`T1800R:129`)
  A2cd. "1 YES [to the split]. A: does this work with claude or does claude-fw need to be started? B: can we wire it so it becomes a deterministic workflow (ensure the doorbell is rung after the message is sent)? 2 yes make it a deep inception, it's key functionality." (`T1800R:131`)
  A2ce. Operator directive "aim not to use claude-p for expensive jobs" (recorded in `.tasks/completed/T-1800-interactive-agent-conversation-runtime--.md:135`): persistent-session doorbell+mail is PRIMARY; a `claude -p` daemon is demoted.
A2d. **Rulings.** GO recorded 2026-05-25 (`.tasks/completed/T-1800-…:150`). T-1830 (2026-05-28): all fleet hubs pass the selftest yet "active conversation count = 0, across 91 topics"; diagnosis is coordination, not infrastructure (no discovery primitive, no always-on listener convention, `agent-send.sh` needs peer-fp) (`docs/reports/T-1830-doorbell-mail-adoption-gap.md:13-35`).
A2e. **Change vs A1.** Adds the PTY-injection wake and the receipt/re-ring loop. A1's protocol is untouched.

## A3. Round 2 — 2026-06-25 to 06-27 — T-2285 ack-with-retry and T-2291 "permanent cross-agent comms fix" (arc-003 reliable-comms)

A3a. **Sources.** `T2291R:1-325` (RCA, five variants, six-step operator dialogue); `docs/reports/T-2285-ack-with-retry-inception.md:60-104`; `V6:1-266`; `DNS:1-172`; `.context/arcs/reliable-comms.yaml:1-27`; `.context/project/decisions.yaml` PD-053..PD-057 (near 504-520).
A3b. **Triggering problem.** A handoff was lost: shared host fingerprint (identity), no inter-hub federation (routing), no delivery/ack confirmation (`T2291R:12-33`).
A3c. **Design as stated.**
  A3ca. Three orthogonal root causes RC1 identity, RC2 routing, RC3 delivery, plus META discoverability (`T2291R:236-243`).
  A3cb. **V3 split into RC3a notify and RC3b confirm.** The "notify" mechanism is the **deterministic sidecar listener**: a no-LLM process on the recipient host pulls the agent's mail and writes a local **flag file plus a fresh heartbeat timestamp**; the agent reads the flag cooperatively at its own yield points; a stale heartbeat means "deaf, halt" (`T2291R:251-267`; `DNS:10-29`).
  A3cc. Scripts: `notify-sidecar.sh` (the ears) and `notify-check.sh` (verdicts MAIL 10 / DEAF 3 / CLEAR 0) (`DNS:33-55`).
  A3cd. **Confirm ladder, three levels** (operator "elaborate→accepted"): L1 TCP-ack ignored; L2 the sidecar's journaled receipt ("delivered to mailbox", survives a recipient restart); L3 the read receipt at the agent's yield point (`T2291R:307-316`; `V6:200-217`). Hub receipts frontier is the fallback-path confirm only.
  A3ce. **V6 apex:** direct host-to-host transport first, hub as loud fallback, per-conversation journal OFF the hub firehose, two-tier discovery (LAN broadcast + hub registry) (`T2291R:277-316`; `V6:77-83`).
  A3cf. Ack-with-retry: client-side tracker plus `channel post --await-ack --retry`, exactly-once via client_msg_id dedupe, no hub-side delivery state (`T-2285…:60-104`).
  A3cg. Auto-confirm: the sidecar journals each unread dm topic and posts `msg-type receipt stage=delivered` with no LLM turn (`DNS:120-151`).
  A3ch. The doorbell is to be *replaced* as the notify mechanism: "The §5 fix inverts the direction … a no-LLM sidecar on the recipient host pulls" (`DNS:20-23,166-172`).
A3d. **Operator's words.**
  A3da. Request 2026-06-27: "find a permanent solution … incept, ask other agents for issues too, present me RCA, with 5 variants for structural remediation scored against framework directives … 5 termlink research agents." (`T2291R:8-9`)
  A3db. Step 4: operator "flagged the missing tier: confirmation is plumbing on a tap that never opens unless the recipient is *woken*" (recorded as, `T2291R:251-253`).
  A3dc. Step 6 pivot: "critically re-evaluate the 'comms over hub' principle" (recorded as, `T2291R:280`); "Audit (operator inversion — CORRECT): the single hub-firehose IS the obfuscation problem" (`T2291R:294`); discovery "(operator decision): TWO-TIER" (`T2291R:302`).
A3e. **Outcome.** GO on V3+V2+V1 composite (`T2291R:193-225`); arc-003 closed 2026-07-02 with headline "no silent loss", `closed_at` at `.context/arcs/reliable-comms.yaml:21`, decision at `:22`.
A3f. **Change vs A2.** Wake moves from "sender pushes a keystroke into the PTY" to "recipient-side no-LLM listener raises a local flag". The confirm ladder (L1/L2/L3) and journal first appear here.

## A4. Round 3 — 2026-07-02 to 07-04 — T-2303 push transport (arc-004), T-2322 dm rail

A4a. **Sources.** `T2303R:1-210`; `.context/arcs/push-transport.yaml:1-60`; `.tasks/completed/T-2322-inception-extend-push-wake-to-dm-rail-fo.md:36-110`; `docs/operations/push-transport-recipe.md`.
A4b. **Problem.** arc-003 was reliable but "doorbell-then-poll"; the latency floor was the 15 s sidecar interval; the PTY doorbell dropped mid-turn keystrokes (`T2303R:1-30,122-135`).
A4c. **Design as stated.**
  A4ca. WebSockets (hub to client push stream) for the live agent path; webhooks only for external consumers (`T2303R:187-200`).
  A4cb. The durable layer (queue, idempotency, confirm, journal) stays underneath; a dropped socket degrades to polling (`T2303R:176-185`).
  A4cc. **Push-waker**: a process that subscribes to `inbox.queued` and `dm.queued` push frames and rings the PTY by `termlink inject <pty_session>` (`push-transport.yaml:25-40`; `T3330R:1b6a`, cited at `T3330R:61`).
A4d. **Operator's words.** "incept the addition/replacement of webhooks and websockets for the work we just did" and "reflect, tell me you understand." (`T2303R:101-102`). The operator was away for the scoping questions; the agent proceeded on delegated initiative (`T2303R:106-119`).
A4e. **Outcome.** GO (scoped); arc-004 closed `shipped` 2026-07-02 (`push-transport.yaml:54-55`); latency benchmark 85-111 ms median full wake (`push-transport.yaml:32`).
A4f. **Change vs A3.** The sidecar's 15 s pull is superseded by hub push for waking, but the waker still injects into a PTY (the doorbell returns). PL-237 later records that push-wake is dormant unless a `pty_session` is bound (`learnings.yaml:2859-2868`).

## A5. Round 4 — 2026-07-07 to 07-11 — "our mechanism is not working": loud contract, G-083 consumption, idle-gated injection

A5a. **Sources.** `T2380R:1-190`; `T2396R:1-70`; `docs/operations/pushwaker-idle-gating.md:1-30`; `docs/operations/durable-reachable-auto-accept.md:1-18`; `.tasks/completed/T-2388-tl-claude---reachable-auto-armed-injecta.md:44-70`; `.tasks/active/T-2402-woken-but-silent-make-a-rung-yet-unanswe.md:59-60`; `.context/arcs/comms-loudness.yaml:1-23`; `learnings.yaml` PL-253 (`:3044-3060`).
A5b. **Design as stated.**
  A5ba. **Loud end-to-end delivery contract**: around every send, verify each link and return a structured per-link result (`delivered, recipient_live, recipient_agent_backed, waker_running, hub_targeted, hub_read_healthy, acked`) that fails fast on the first broken link (`T2380R:137-165`).
  A5bb. Build tasks: T-2384 per-agent fp, T-2385 reachability preflight, T-2386 reply-on-sender-hub, T-2387 waker-liveness canary (`T2380R:150-160`).
  A5bc. **tl-claude `--reachable`**: one command launches an injectable, armed agent and re-arms at reboot (`T-2388…:44-62`).
  A5bd. **G-083 / PL-253 root cause**: "a heartbeat proves the PROCESS is alive, not that the SESSION is listening"; an injected wake into a busy or manual-accept session lands UNSUBMITTED and is discarded (`T2396R:14-24`; `learnings.yaml:3044-3052`). Fix: `wake-confirm.sh`, a standalone consumption check (`T2396R` "The fix").
  A5be. **Idle-gated injection** (T-2402 Stage 3): inject only when the REPL is at a READY prompt, otherwise defer and re-probe; the probe screen-scrapes `termlink pty output` (`pushwaker-idle-gating.md:11-26`).
  A5bf. Stage 5 escalating re-ring and Stage 6 "wake-protocol obligation: a woken agent drains all unread topics and for each replies OR posts an explicit acknowledged/no-action" so silence always means a bug (`T-2402…:59-60`).
A5c. **Operator's words.**
  A5ca. "our mechanism is not working" and "we did extensive work on this, read back, was it arc 004?!" (`T2380R:182-183`, dialogue log). "THIS IS REALLY BAD; WHAT NOW?" (`T2380R:186`, recorded as a quote in the same log).
  A5cb. GO: operator "yes", 2026-07-09 (`T2380R:137`).
  A5cc. "prove-first, then build" (`T2396R:4`).
  A5cd. 2026-07-10 "focus on making this work" (`T-2388…:50`).
A5d. **Change vs A4.** First round to say the wake is *shipped but dark* (E4, `T2380R:63-79`) and that a rung is not a read. Introduces the prompt-free gate (idle classifier) and the rule that a bare inject proves nothing.

## A6. Round 5 — 2026-08-24 / 08-25 — T-2838 delivery-to-turn contract

A6a. **Sources.** `T2838R:1-569`; `.tasks/completed/T-2838-delivery-to-turn-contract--build-it-or-k.md:206-228`; `docs/reports/T-2876-…` task `.tasks/completed/T-2876-cross-session-message-prover-assert-deli.md:43`.
A6b. **Operator's stated goal.** Interactive agent<->agent medium (1:1, N:N, N:operator), plus artifact passing by reference: "the orchestrator uses TermLink to dispatch an assignment to an agent with an asserted agent profile ... and should come back by writing those files and reporting back those files have been written" (`T2838R:555-562`). Assessment: "Partially certainly, but that interactive communication is still flaky at best." (`T2838R:19`)
A6c. **Design as stated.**
  A6ca. Diagnosis: the detector layer is saturated (six detectors) but the **mechanism was never built**: a missing delivery/consumption plane (`T2838R:41-66`).
  A6cb. **Native consumer** for agents we launch: the agent acks from INSIDE its own turn loop, so "LIVE != listening becomes impossible by construction"; **PTY inject** for sessions we do not control, which "never reports success without a confirmed read-cursor advance" (`T2838R:92-102`; `:414-420`).
  A6cc. Spike S2 proved the by-construction property: "A receipt cannot exist unless a turn happened." (`T2838R:175-208`).
  A6cd. Typed envelopes `assignment.v0` and `result_manifest.v0` (landed, `T2838R` item 3); `channel post` made honest: `delivered-unconfirmed` vs `consumed` (landed, item 4); `ack-status` fixed (item 2); per-agent identity as prerequisite (item 1); hub-side enforcement last (item 5) (`T2838R:431-` build decomposition).
A6d. **Outcome.** GO 2026-08-25 (`T-2838…:206-228`).
A6e. **Change vs A5.** Replaces "detect when the wake fails" with "make success conditional on a read-side receipt". Introduces the native consumer and typed manifests.
A6f. **Follow-on (2026-09-01).** T-2876 prover: assert on the RECEIVER's transcript, never the sender; operator: "test everything end to end, and also build that as a testing harness for any future development work." (`T-2876…:43`; CLAUDE.md "Session-to-session message prover").

## A7. Round 6 — 2026-09-22 to 09-25 — arc-011 "mailbox to prompt": the operator's ten-step sidecar spec

A7a. **Sources.** `RAIL:1-231`; `SCAPI:1-267`; `ARC11:1-367`; `.tasks/completed/T-3075-sidecar-api---separate-respawning-proces.md:66-140`; `docs/reports/T-3075-sidecar-api-sovereign-question-status.md:1-53`; `docs/operations/notify-sidecar-api.md:1-60`; `.tasks/completed/T-3069-…:92`; `.tasks/completed/T-3071-…:78-80`.
A7b. **Design as stated (the spec).**
  A7ba. Sequence: sender sends via a **sidecar API** (with optional blob) to the receiver; the receiver stores on local disk, sets a flag, calls back RECEIVED; the sender records its own ledger; an **independent cron** checks the prompt is free (no LLM), reads the queue by priority, injects, **verifies the agent is working**, reports INJECTED; roles swap on reply (`RAIL:33-61`; `ARC11:3-9`).
  A7bb. Sidecar: "a **separate process** exposing an API, deliberately **independent of the hub because the hub goes down**. Very simple. **Always respawns.** It also carries the startup chain — start agent, start session, start project, start hub — and re-resolves when an **FQDN or IP stops resolving**." (`RAIL:111-117`, written as "Operator direction, verbatim in intent"; the same wording at `SCAPI:10-13`).
  A7bc. **Two events, not one**: RECEIVED (L2 `stage=delivered`) and INJECTED (L3 `stage=read` with `--evidence`), with "evidence=wake-consumer" withdrawn as untruthful (`RAIL:65-85`).
  A7bd. Priority: "flat FIFO for now, per operator" (`RAIL:149-155`; operator "keep it flat for now, prioritise later", `T-3071…:78-80`). Urgent: initially "not yet defined, note for later" (`T-3072…:5`).
  A7be. Blob via `artifact.put`, mandatory `--expected-sha256` (`SCAPI:176-198`; `ARC11:45-69`, SQ-3).
A7c. **Operator's words (recorded).**
  A7ca. "I want to inject, but I check if my agent is running, I see it's not running." (`.tasks/completed/T-3069-…:92`)
  A7cb. "Urgent bypasses the wait. Reliability is important at all times but can also be out of band." (`SCAPI:207-208`)
  A7cc. "read the queue, inject the highest-priority message." (`T-3071…:78-79`)
  A7cd. SQ-8 (2026-09-24): "reliability and fragility, so I'm not to save a few tokens just to get a quick solution. I want a solid solution." (`ARC11:360-361`)
A7d. **Rulings.**
  A7da. SQ-1 (2026-09-23): the injector stays in TermLink as a primitive; the sidecar API is a LOCAL control surface (`ARC11:203-213`).
  A7db. SQ-2 (09-22): premise disproved; real blocker was the idle classifier (`ARC11:214-246`).
  A7dc. SQ-3: add `artifact put/get` verbs (`ARC11:247-259`). SQ-5: install the cron (`ARC11:317-349`). SQ-6: backfill once, recorded as a backfill (`ARC11:260-280`). SQ-7: sixth AC obsolete (`ARC11:281-304`).
  A7dd. **SQ-4 (2026-09-23): "urgent NEVER injects into a BUSY prompt"**; urgent shortens the WAIT, not the CHECK (`ARC11:305-316`; `T-3072…:259-265`). This contradicts A9 (see B-urgent).
  A7de. SQ-8: systemd-only respawn REJECTED, portable fallback REQUIRED (`ARC11:350-367`).
A7e. **Analysis (T-3075).** The sidecar API collides with the charter's "hub-mediated strict star" unless it is local control; "bright line": the API may read and act on this host's own state, never move a message between hosts (`SCAPI:17-35,153-172`). Open IW-1..IW-4 (`SCAPI:260-267`). Also: the orchestration half (queue, policy, inject, verify) is arguably AEF's (`SCAPI:103-149`), settled the other way by SQ-1.
A7f. **Change vs A3 and A6.** Re-adds the *injection* leg that A3 had replaced, now gated by a prompt-free check, and adds the queue/priority/ledger/API. It never refers to A6's native consumer.

## A8. Round 7 — 2026-09-28 — T-3200 inbox consumer: the round that re-opened settled ground, and the sidecar revival

A8a. **Sources.** `docs/reports/T-3200-inbox-consumer-inception.md:1-125`; `.tasks/completed/T-3200-consumer-for-the-project-inbox-rail---ho.md:44-90,106-160,228`; `.tasks/completed/T-3203-…`, `T-3204-…:120-148`, `T-3205-…`, `T-3206-…`, `T-3207-notify-injectorsh-is-wired-to-nothing-wh.md:116`.
A8b. **What happened.** The agent opened an inception asking "how should a durable mailbox reach a session prompt", proposing options A (cron canary), B (handover), C (wake on `inbox.queued`), D (`/inbox` skill) (`T-3200R:37-75`), and presented "should an inbound peer message interrupt a working session" as an open sovereign question (`T-3200R:95`).
A8c. **Operator's words.** The operator asked: "did we not design this in the sidecar?" (`.tasks/completed/T-3200-…:81`). The task then records: "arc-011 is the operator's own ten-step sidecar spec and NINE of its ten slices are [built]" (`:228`) and "I opened an inception over ground the operator had already designed and ruled on … I read it only after the operator asked" (`:76-84`).
A8d. **Findings.** The real cause was a bug: `journal-mirror.sh` enumerated `dm:` only, so AEF's `inbox:` mail (49 messages, T-3434 ladder at rung 4) was invisible to the rail (`T-3200…:50-68`). Fixed by T-3201/T-3203; restart by T-3204.
A8e. **Revival findings.** T-3049 (2026-09-22): the sidecar "last wrote a heartbeat 2026-07-01 … and has not run since" while arc-003 claimed "no silent loss" (`.tasks/completed/T-3049-…:3-20`). T-3207 (2026-09-28): `notify-injector.sh` is "SCHEDULED BY NOTHING, AND THAT IS NOT A DECISION TO RETIRE IT"; answer "WIRE it … but not yet" (`scripts/notify-injector.sh:4-28`; `T-3207…:116`). T-3206: `claude-termlink` has no wake consumer.
A8f. **Change.** No new design; this round is evidence of the loss mechanism, not a design. It is listed because its dialogue contains the operator's reaction to losing the sidecar design.

## A9. Round 8 — 2026-10-02 to 10-03 — T-3325 addressing, T-3330 receive-side telemetry, T-3333 steward, T-3335 front-to-end restatement

A9a. **Sources.** `IAC:1-137`; `CONSULT:1-21`; `T3330R:1-266`; `.tasks/active/T-3330-receive-side-receivedinjectedreplied-tel.md:181-195`; `.tasks/completed/T-3325-notify-sidecar-honour-an-addressee--repl.md:282-309`; `.tasks/active/T-3333-hub-steward-agent-an-always-on-aef-agent.md:119`; `.context/project/decisions.yaml` PD-186 (`:1348-1354`); `docs/operations/notify-sidecar-api.md:108,160`.
A9b. **Design as stated (the operator's protocol, read back and confirmed).**
  A9ba. Send: sender agent to its own sidecar (API) to the receiver's sidecar (API). RECEIVED: the receiver's sidecar immediately calls the sender's sidecar. STORED: called again once stored. A new-message flag is set (`T3330R:248-256`; `IAC:63-75`).
  A9bb. Inject: "an urgent message is injected into context at once; otherwise once the prompt is free. A cron-style check in the receiver's sidecar retries while the prompt is busy." (`T3330R:256`)
  A9bc. INJECTED called back; flag cleared only when the queue is empty; ANSWER READY, then the sender pulls; push primary, pull fallback on the standard polling ladder 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 d, 3 d, 1 week, 1 month, 1 quarter, 1 year, each rung twice (`T3330R:257-261`; `IAC:77-81`).
  A9bd. Telemetry: every step a timestamped event; each sidecar records its own and copies to the hub through an outbox; hub retains a window; daily digest per agent; alarms immediate only for urgent, others accumulate and escalate "like audit warnings"; observability database and learning later (T-3319); hub steward agent proposed (T-3333) (`T3330R:262-266`; `IAC:87-96`; `T-3333…:119`).
  A9be. Addressing: five-level circuit host/hub/project/session/agent, `metadata.to_circuit`, match level by level, never fall back across projects (`T-3325…:307-309`; `decisions.yaml:1348`; `IAC:19-31`).
  A9bf. Deployment: "are all the sidecars we develop … part of our [TermLink] package? When the deployment is done we should deploy all the sidecars with it." (`T3330R:230`).
  A9bg. **The 30-second flag tick** (2026-10-03): "I'm missing that we use cronjobs to monitor if there is a flag. Every 30 seconds … when the flag is up, the message queue gets read … urgent gets injected immediately. Non-urgent, we check again if the prompt is free. If the prompt is not free, we wait again until the next 30 seconds." (`CONSULT:17`; implemented as design §3a, `IAC:52-59`).
A9c. **Operator's words (verbatim where the document quotes).**
  A9ca. "I am absolutely very clear that I want to have this … after 20 times me telling you this is a really important part of this design, you don't have this quite essential part of bi-directional interactive communication." (`CONSULT:13`)
  A9cb. "You've been telling me it works … as you don't know how you can solve it, get external consultation." (`CONSULT:14`)
  A9cc. "off with the design we have envisioned, in detail, front to end, because you obviously keep forgetting that." (`CONSULT:15`)
  A9cd. "I'm completely missing the point about that we observe whether the prompt is free or not, and inject when it's free and with urgent bypass." (`CONSULT:16`)
  A9ce. "what's going wrong there? … We store it and immediately go back to the sidecar receiver … an API call that's received. And then when it's injected we make an API call that's injected … those timestamps we store … once we've got an answer we send it back … also timestamps … also an API call." (`T3330R:229`)
  A9cf. "as a standard, also for all vendor agents, to collect the telemetry of our communications … the different steps, how much time it takes and how much delay … you need to be able to pull that information from different agents … one times per day … And you infer or reflect on what working with that information means." (`T3330R:231`)
  A9cg. Q1 ruling: "Cozzyte is suggested and you see fit", read as "C, as suggested, proceed as you see fit" (voice transcription, overturnable) (`T-3325…:283`). Operator also directed the five-level circuit and its fallback ladder (`T-3325…:308`).
A9d. **Rulings.** T-3325 Q1 = C (`T-3325…:282-287`; PD-186). T-3330 IW-1 = "C amended" (`T-3330…:181-188`). External review 4/4 chose C (`T3330R:233-246`).
A9e. **Change vs A7.** (1) Sidecar-to-sidecar API calls carry RECEIVED/STORED/INJECTED/ANSWER READY (A7 had one RECEIVED/INJECTED pair via hub receipts). (2) A 30 s cron tick replaces the `*/5` driver. (3) Urgent bypasses the prompt-free check, reversing SQ-4. (4) Telemetry, ladder, steward, sidecars-in-the-package, five-level addressing are new.

---

# B. THE CUMULATIVE DESIGN (element by element)

Each element: rounds that state it, latest operator position, contradictions.

## B1. Sidecar as a separate process with an API
B1a. **Rounds:** A3 (listener, no API, `DNS:33-36`), A7 (API, `RAIL:111-117`), A9 (API calls between sidecars, `IAC:36-39,63-68`).
B1b. **Latest operator position:** sidecar-to-sidecar API, push primary (`T3330R:248-256`).
B1c. **Contradiction:** SQ-1 reads the API as local control only, "sending to a peer stays channel.post via the hub" (`ARC11:28-44`); A9 step 20 sends sidecar to sidecar (`IAC:64`, open point O1 at `IAC:132`). SCAPI §6 says sending to a peer is "explicitly out of scope" (`SCAPI:164-167`). AEF's receiver already does sidecar to sidecar on one host (`T3330R:98-103`).
B1d. **What exists:** `notify-sidecar-api.sh`, local-only, five verbs, remote args refused (`docs/operations/notify-sidecar-api.md:20-28,70-78`).

## B2. Independent of the hub, always respawns, startup chain, FQDN/IP re-resolve
B2a. **Rounds:** A7 only (`RAIL:111-117`), restated A9 (`IAC:36-39`).
B2b. **Operator position:** "separate process … independent of the hub because the hub goes down … Always respawns." (`RAIL:111-113`); SQ-8 solid over quick, portable respawn (`ARC11:350-367`).
B2c. **Built:** `notify-sidecar-supervisor.sh --loop` and `--emit-unit` (`ARC11:28-44`); FQDN/IP are re-read and compared per call (`scripts/notify-sidecar-api.sh:197-212`).
B2d. **Not built / not found:** the startup chain (start agent, session, project, hub). No document or script describes it beyond the spec statement.

## B3. Local store, flag, arrival record
B3a. **Rounds:** A3 (flag + heartbeat files under `~/.termlink/notify/`, PD-053; journal as V6 S1), A7 (S3 journal, S4 flag plus arrival record `last_mail_ts`, `ARC11:70-80`), A9 (`IAC:40,63-75`).
B3b. **Latest position:** store durably, set a flag, flag down "only when the queue is empty" (`T3330R:258`; `IAC:74`).
B3c. **Contradiction:** A3 had the AGENT poll the flag at yield points and "halt if deaf" (`DNS:48-55`); A9 has a CRON tick read the flag and inject (`IAC:52-59`). The yield-point DEAF/halt rule is not restated in A7 or A9.
B3d. **Shift in flag meaning:** the flag's pending count is shared with the sidecar's own auto-ack, so "pending=0" can mean "receipted", not "seen" (`T3330R:1a1-1a3` at `:24-26`; handover `S-2026-1003-1305.md:273`).

## B4. Queue and priority
B4a. **Rounds:** A7 (S8, `RAIL:149-155`), A9 (`IAC:41`).
B4b. **Position:** flat FIFO first, then priority (`T-3071…:78-80`); built as `COALESCE(priority,0) DESC, ts ASC, offset ASC`, priority clamped to [-9,9] (`ARC11:111-132`).

## B5. Prompt-free check (independent of any LLM)
B5a. **Rounds:** A2 (known ceiling), A5 (idle-gated waker, `pushwaker-idle-gating.md:11-26`), A7 (S7, `ARC11:100-110`), A9 (`IAC:42,55`).
B5b. **Position:** "I check if my agent is running, I see it's not running" (`T-3069…:92`) so NOT RUNNING is its own outcome; ambiguity never resolves to READY.
B5c. **Built:** `scripts/lib/pty-state.sh`, classifier rebuilt by T-3079 on quiescence plus composer-row emptiness, 466 samples, 0 false-ready (`ARC11:100-110`).
B5d. **Contradiction:** none about the check itself. The question is whether urgent skips it (B8).

## B6. Cron driver and interval
B6a. **Rounds:** A7 (S11, spec step 11 "the cron drives 7 to 10", `RAIL:60`), A9 (30 s, `IAC:52`).
B6b. **Latest operator position:** every 30 seconds (`CONSULT:17`). The design records that system cron's minimum is one minute, so 30 s needs a supervised loop or two staggered entries (`IAC:52`).
B6c. **Reality:** the installed crons are `*/5` (`.context/cron/notify-sidecar-supervisor.crontab`, `notify-wake-supervisor.crontab`), and they supervise processes; no cron runs the injector (`scripts/notify-injector.sh:4-28`; `IAC:124`).

## B7. Injector and verification
B7a. **Rounds:** A2 (doorbell), A5 (waker injects), A7 (S10 inject + verify BUSY transition, `ARC11:148-170`), A9 (verified INJECTED, `IAC:58,71`).
B7b. **Position:** INJECTED needs an observation (BUSY transition or message in the receiver's transcript), because a bare inject can land unsubmitted (T-2396) (`RAIL:69-84`; `IAC:58`).
B7c. **Stronger evidence variants:** AEF records HANDED_OVER "only on transcript evidence" from the UserPromptSubmit hook (`T3330R:2e3,2g2` at `:105,119`); T-2876 reads the receiver's transcript (`T-2876…:43`).

## B8. Urgent semantics
B8a. **Rounds:** A7 (S9/SQ-4), A9.
B8b. **Quote 1 (A7, operator, `SCAPI:207-208`):** "Urgent bypasses the wait. Reliability is important at all times but can also be out of band." The agent's reading (not the operator's wording): urgent bypasses the WAIT but not the CHECK, with an out-of-band route possible (`SCAPI:210-234`).
B8c. **Quote 2 (A7 ruling, `ARC11:305-316`):** "urgent NEVER injects into a BUSY prompt … Urgent shortens the WAIT; it does not bypass the prompt-free CHECK."
B8d. **Quote 3 (A9, operator, `CONSULT:16-17`):** "inject when it's free and with urgent bypass" and "urgent gets injected immediately".
B8e. **Contradiction:** SQ-4 versus the 2026-10-03 statement. IAC follows the operator's later words (`IAC:55,59,133`) and records the T-2396 loss risk as the constraint the urgent route must meet (open point O2).
B8f. **Alarm sense of urgent (A9):** immediate alarms only for urgent messages (`T3330R:265`; `IAC:94`).

## B9. Receipt ladder: L2/L3 versus RECEIVED/STORED/INJECTED/ANSWER READY
B9a. **Rounds:** A3 (L1/L2/L3, `V6:200-217`), A7 (two events RECEIVED and INJECTED, `RAIL:65-85`), A9 (four calls, `IAC:65-74`).
B9b. **Mapping stated by the documents:** L2 `stage=delivered` = RECEIVED (`RAIL:71`); L3 `stage=read` = INJECTED (`RAIL:72`); STORED and ANSWER READY are new in A9; `stage=replied` does not exist anywhere (`T3330R:76`).
B9c. **Open contradiction:** are RECEIVED and STORED two calls (A9) or one (AEF's RECEIVED already means durably stored)? (`IAC:134`, open point O3; `T3330R:98-103`).
B9d. **Structural limit:** `stage` is a free-form metadata key; `channel.receipts` keeps only the latest receipt per sender, so an earlier stage can be overwritten (`T3330R:81-82`).

## B10. Roles swap and reply
B10a. **Rounds:** A1 (reply as a turn), A7 (S12), A9 (`IAC:75`).
B10b. **Position:** the answer travels the same path in reverse (`IAC:75`). **Built:** reply goes through `channel.post` (`agent-respond.sh`, `/reply`), not the sidecar API, by SQ-1 bright line (`ARC11:183-202`).
B10c. **Blocker (A7):** `framework-agent-systemd` lacks `termlink` in its allowed commands (`ARC11:183-202`).

## B11. Blobs
B11a. **Rounds:** A6 (result manifest by reference, `T2838R:68-80`), A7 (S2 artifact path, `SCAPI:176-198`), A9 (`IAC:83-85`).
B11b. **Position:** `artifact.put` content-addressed, mandatory `--expected-sha256` (SQ-3, `ARC11:247-259`). **Built:** `termlink artifact put/get` (`ARC11:45-69`).

## B12. Sender ledger
B12a. **Rounds:** A7 (S6), A9 (`IAC:46`). **Built:** `scripts/notify-ledger.sh`, rung only advances on a receipt read back; withdrawn evidence ignored (`ARC11:87-99`).
B12b. **Relation to A3:** `awaiting_ack.sqlite` and `channel awaiting-ack` (`T3330R:1d3b`, `:84-85`).

## B13. Wake: doorbell, push-waker, `pty_session`, be-reachable, tl-claude
B13a. **Rounds:** A2 (doorbell), A3 (doorbell replaced by sidecar flag), A4 (push-waker), A5 (`tl-claude --reachable`), A7/A9 (injector uses the same PTY).
B13b. **Precondition recorded in every round after A4:** a termlink-owned PTY. PL-237: dormant without a bound `pty_session` (`learnings.yaml:2859-2868`); "cannot be retrofitted onto a running headless claude" (`scripts/notify-injector.sh:20-24`).
B13c. **Operator position on the open question O4 (how a running session becomes injectable):** not recorded. IAC lists it as open (`IAC:135`).

## B14. Identity and addressing
B14a. **Rounds:** A3 (per-agent keys, stable `agent_id` key, `T2291R:239-243`), A5 (T-2384 per-agent fp), A6 (per-agent identity prerequisite, `T2838R` item 1), A9 (five-level circuit, `IAC:19-31`).
B14b. **Latest position:** five levels host/hub/project/session/agent; canonical FQDN, hub name+instance, project minted `pid`, session canonical+runtime id, agent = role+instance (`IAC:21-27`). Directory of name to id is open (`IAC:30,137`).
B14c. **Contradiction:** hub "id" today is a TLS fingerprint that rotates (`IAC:24`). Session level still open (`IAC:136`).

## B15. Telemetry, digest, steward, alarms, learning
B15a. **Rounds:** A9 only (`T3330R:231,262-266`), linked to T-3319 "learn from message traffic" (operator 2026-10-02, `.tasks/completed/T-3319-…:6`).
B15b. **Position:** `IAC:87-97`. **Open:** how alarms and escalations surface (`IAC:94`); steward agent T-3333 is `captured`, no recommendation yet.

## B16. Polling ladder
B16a. **Rounds:** A9 only (`T3330R:261`). **Open:** "50" dictation read as 15, pending operator correction (`IAC:80`). Sent to AEF @309 / @180, AEF T-3770 (`IAC:81`).

## B17. Deployment: every sidecar ships with the TermLink package
B17a. **Rounds:** A9 only (`T3330R:230`). **State:** releases ship the binary only; ~13 scripts run only from the `/opt/termlink` checkout (`T3330R:139-143`); no `sidecar` subcommand in the CLI (`T3330R:3g2`).

## B18. Transport: direct versus hub
B18a. **Rounds:** A3 (V6 direct-first, `V6:221-235`), A7 (SQ-1 hub stays), A9 (O1).
B18b. **Contradiction:** A3 wanted direct host-to-host with hub fallback and the journal off the firehose; the charter tripwire forbids a second bus (`SCAPI:17-35`); AEF's cross-host receiver is its own T-3688 (`T3330R:98-103`).

## B19. Native consumer and typed manifests (A6)
B19a. **Rounds:** A6 only. **Status:** typed helpers landed with "no CLI verb emits or consumes these yet" (`T2838R` item 3 "Not claimed"); the native consumer was a spike, never a build (`T2838R:175-208`). Not mentioned in A7, A8 or A9 (see D1).

---

# C. STATUS CLAIMS vs REALITY

C1. **arc-003 "no silent loss" (closed 2026-07-02).**
  C1a. Claim: closed, headline "no silent loss", `reliable-comms.yaml:12,21-22`.
  C1b. Later evidence: the notify sidecar it rests on "last wrote a heartbeat 2026-07-01 … and has not run since"; no process, no cron, no skill (`.tasks/completed/T-3049-…:3-20`).
C2. **arc-004 push-wake "shipped" (closed 2026-07-02).**
  C2a. Claim: live-E2E-proven, 85-111 ms (`push-transport.yaml:32,54-55`).
  C2b. Later evidence (2026-07-07): zero waker processes, "all 8 live claude --resume sessions … are NOT push-reachable" (`T2380R:63-79`). Again 2026-09-28: "ZERO LIVE listeners carry pty_session" across ~90 canary entries (`scripts/notify-injector.sh:14-18`).
C3. **"Wake delivered" (A5).** Claim: a rung is a delivery. Later: "A heartbeat proves the PROCESS is alive, not that the SESSION is listening" (`T2396R:14-24`); G-083 resolution still "Not yet built — pending operator" in 2026-08 (`T2838R:39`).
C4. **First end-to-end proof of the notify rail (2026-09-22).** Claim: `pending 0 to 1 to 0` "the first time this rail was ever proven end to end" (`DNS:204-212`). Scope of the proof: flag raise and auto-confirm receipt only. It never touched an agent's prompt.
C5. **arc-011 S10 "PROVEN LIVE 2026-09-22" and S7/S10/S11 `built`.**
  C5a. Claim: `ARC11:148-170` (inject, BUSY transition, L3 read back on topic `proof:t3079-…`).
  C5b. Evidence it was one run on a fresh test topic (`IAC:125`). The injector "is SCHEDULED BY NOTHING" (`scripts/notify-injector.sh:4`); T-3207 answer "WIRE it … but not yet" (`T-3207…:116`).
  C5c. S11 "installed … VERIFIED FIRING" (`ARC11:171-182`) refers to the wake supervisor. That consumer "posts nothing … Neither agent … declares an --action today, so the wake rail currently terminates in a log line" (`T3330R:47`, quoting `notify-wake-consumer.sh`).
  C5d. Drive interval is `*/5`, not 30 s, and it checks no agent's prompt (`IAC:124`).
C6. **T-3200 premise (2026-09-28).** Claim in the task: "Nine of ten slices are built" (`T-3200…:228`, via the arc file) so the rail "already exists". Reality: inbox mail was invisible to it for 6 days (`T-3200…:50-68`), the injector was scheduled by nothing, and `claude-termlink` had no wake consumer (`T-3206`).
C7. **S1 sidecar API "built" (2026-09-29).** The arc file itself says "the spec wording 'sender SENDS via a sidecar API' is NOT what was built and NOT what SQ-1 permits" (`ARC11:34-38`). S5 is `partial` for the same reason (`ARC11:81-86`). S12 `partial` (`ARC11:183-202`).
C8. **T-3325 "sidecar wakes only the right agent" (2026-10-02/03).** The T-3335 design records that this description was made "while nothing reaches the agent at all" (`IAC:10`). The sidecar receipted mail as delivered while the agent had no wake consumer; AEF waited about a day (`T-3325…:303-304`; handover `S-2026-1003-1305.md:54`).
C9. **Operator-facing verdict (2026-10-03).** "You've been telling me it works" (`CONSULT:14`). IAC §11 states plainly: for agents on this host, steps 24 onward do not work; INJECTED, replied, telemetry, digest and shipped sidecars are not operating (`IAC:119-126`); closing rule: nothing is "working" until two real running agents pass a live test with a negative control (`IAC:128`).
C10. **AEF's receiver (T-3693).** AEF claims delivery with a live two-agent nonce e2e (`T3330R:2j1` at `:127`). The vendored copy has no `lib/sidecar/` (`T3330R:2a1`, `:90`); cross-host is T-3688 (not built); its 30 s watcher is T-3684/T-3685 (not built at the time of writing) (`T3330R:2j2`). Not verified by TermLink.
C11. **S3 "journal.sqlite, 2238 rows"** (`ARC11:70-74`) versus S6 and the T-3203 gap: the journal covered `dm:` only, not `inbox:`.

---

# D. GAPS (stated once, not carried forward) — most likely lost

D1. **Native consumer / ack from inside the agent's own turn loop** (A6). Spike proved "A receipt cannot exist unless a turn happened" (`T2838R:175-208`); intended as the fix for "LIVE != listening" (`T2838R:92-102`). Absent from arc-011 and from IAC. A9's verified-INJECTED depends on screen-scraping and a transcript check instead.
D2. **Agent self-check at yield points, DEAF means halt** (A3). "the agent can no longer trust 'no flag = no mail,' so it halts" (`DNS:25-29,48-55`). The cron-injector model (A9) has no equivalent: nobody halts when the 30 s tick is dead except the `notify-sidecar` canary.
D3. **Wake-protocol obligation: always ack or explicitly defer** (A5, T-2402 Stage 6) (`.tasks/active/T-2402-…:60`). Not carried into the arc-011 receive chain; replied/ANSWER READY has no "acknowledged, no action" state.
D4. **The startup chain and FQDN re-resolve** (A7). Only FQDN/IP recording exists (`scripts/notify-sidecar-api.sh:197-212`); "start agent, start session, start project, start hub" has no implementation or design (`RAIL:111-117`).
D5. **Canary mail with deadlines** (GLM external review): INJECTED within T1, REPLIED within T2 as a liveness invariant ("R6") (`T3330R:237-246`, table). IAC carries no liveness invariant; alarms are urgent-only.
D6. **Typed `assignment.v0` / `result_manifest.v0` with a wired verb** (A6): helpers exist, "no CLI verb emits or consumes these yet" (`T2838R` item 3). Not in IAC §6 blobs.
D7. **Hub-side enforcement of consumption** (A6 item 5): "the hub can refuse to call a delivery complete without a receipt" (`T2838R:422-427`). Not in any later design.
D8. **Per-agent readiness emitted by the agent, not inferred from presence** (T-3001, T-3250): "decide whether the agent emits its own state or consumers stop inferring dispatchability from presence" (`.tasks/active/T-3250-design-who-observes-agent-side-readiness.md:6-8`). This is the same question as O4 and as AEF's Stop/UserPromptSubmit ready flag (`T3330R:2j1`); `status: captured`, no ruling.
D9. **Stage vocabulary mapping TermLink to AEF** (HANDED_OVER requires transcript evidence; TermLink INJECTED requires a BUSY transition) (`T3330R:5f` at `:222`): "This is not decided anywhere."
D10. **"Interrupt consent" / who owns a session's attention** (T-3200 IW-1, `T-3200R:95`). It was dissolved by SQ-4. With SQ-4 now reversed by the operator (B8), the consent question returns without a recorded answer.
D11. **Reply-on-sender-hub routing (T-2386) and hub-read-health fail-fast (T-2385)** (A5): shipped as tasks (`.tasks/completed/T-2386-…`), but neither is mentioned in the arc-011 or IAC send path. The T-2380 "loud per-link result" contract is not tied to the sidecar API.
D12. **Message compaction of repeated wake notices** (T-3321 title) and **learning from traffic** (T-3319): both exist as tasks; neither is linked from IAC except T-3319 at `IAC:95`.
D13. **Same-host / loopback direct path** depends on T-2024 (deferred) (`V6:251-253`), and **Tier-1 LAN broadcast discovery** was left out of V6 (`V6:254-256`). Both are still absent from every later round.
D14. **Dictation uncertainty on the ladder** ("50" read as 15) is still open (`IAC:80`).

---

# E. Where the loss mechanism is documented (why rounds disappear)

E1. T-3200: the agent "read [the arc] only after the operator asked 'did we not design this in the sidecar?'" and notes it is "the same failure as the T-3130 miss recorded earlier in this session: starting work without reading the record that already contains the answer" (`.tasks/completed/T-3200-…:76-84`).
E2. RAIL opens with the same admission for 2026-09-22: "I reported the rail 'working end to end'. It was not." (`RAIL:14-18`), and sets the rule that a slice is `built` only when the specified mechanism carries it (`RAIL:24-27`). IAC repeats the failure on 2026-10-03 (`IAC:10`).
E3. The arc register (`ARC11`) and design docs hold the sidecar rounds (A7) and IAC consolidates A7 and A9, but nothing links A1 to A6 into one document: A6's native consumer (D1), A3's yield-point deafness rule (D2) and A5's wake obligation (D3) live only in their own inception reports.

*End of report. Read-only survey; no other file was written.*

---

# F. Earlier rounds found by recall (2026-10-04)

**Method.** The semantic index (`fw ask`) and a full read of sources neither history cites. Seven are new rounds (F1-F6, F11); four are additions to rounds already in section A (F7-F10). Same format as section A. Citations are `file:line` under `/opt/termlink`; `T-nnn` task files are `.tasks/completed/T-nnn-*.md` unless marked active. The rebuilt design that uses them is `docs/design/interactive-agent-communication.md`.

**Total rounds now known:** 9 (section A) + 6 (AEF history, `T-3335-design-history-aef.md`) + 7 new here = **22**. AEF round 1 (receptionist, T-1135) and F4 below are the two halves of the same 2026-04-12 round; they are counted once each because each side records different content.

## F1. Round — 2026-03-08 — T-007 terminal output capture, bidirectional (the origin of "control a terminal")

F1a. **Sources.** `docs/reports/T-007-output-capture-bidirectional.md:13-17,99,239-253`; `.tasks/completed/T-007-it-004-output-capture--bidirectional-com.md:14` (created 2026-03-08).
F1b. **Design as stated.**
  F1ba. "The half-duplex problem: after injecting keystrokes (`command.inject`), there's no way to read what the terminal produced in response. Without output capture, TermLink is a blind remote control." (`T-007…:17`)
  F1bb. Decision GO: a TermLink-owned PTY for registered sessions (master read/write), scrollback ring buffer, `query.output`, `data.stream`, `command.inject` wired to a PTY master write (`T-007…:239-253`).
  F1bc. Constraint recorded on day one: "User must start their shell through TermLink… Existing terminal sessions can't be 'attached to' retroactively" (`T-007…:52-53`). This is the root of PL-237 (a running session cannot be made injectable later).
F1c. **Operator's words.** None recorded in the report.
F1d. **What it adds.** The injection primitive and its ownership precondition. Nothing in later rounds restates that the precondition was known in March 2026.

## F2. Round — 2026-03-18 and 2026-04-24 — T-099 / T-1207 / T-173 the Stop hook: the harness reports a turn boundary

F2a. **Sources.** `docs/reports/T-099-postmessage-sessionend-hook-request.md:21-37,57-67`; `docs/reports/T-1207-stop-hook-inception.md:5-12,45-72`; `.tasks/completed/T-1207-stop-hook-design--conversation-governanc.md:97`; `.tasks/completed/T-173-wire-stop-hook-for-conversation-governan.md:33`.
F2b. **Design as stated.**
  F2ba. Claude Code's `Stop` hook "fires when Claude finishes responding", `UserPromptSubmit` fires "before Claude processes a prompt" (`T-099…:21-37`). Both exist; T-099 closed NO-GO on a feature request because they already existed.
  F2bb. T-1207 (the same events used for governance): Stop never blocks; "Stderr from a Stop hook becomes additional context the agent sees on the next turn", and the agent then asks the human a y/n (`T-1207 report:49-56`).
  F2bc. A Stop hook is wired in this repo for governance (`stop-guard.sh`, T-1211, installed 2026-04-25; `T-173…:33`).
F2c. **Operator's words (recorded as).** "No block. Use a y/n user-question pattern" and "Option B — live from day 1." (`T-1207 report:67-71`; `T-1207…:97`)
F2d. **What it adds.** The hook primitives that AEF later uses to REPORT readiness (Stop sets ready, UserPromptSubmit clears it) existed and were wired here five months earlier, for a different purpose. A hook-to-agent channel (Stop stderr to the next turn's context) was designed and never reused for message surfacing. No round connected the two until AEF's T-3397 (2026-09-21).

## F3. Round — 2026-03-23 — T-256 "the spawned agent can talk to the spawning agent", and T-233 persistent specialists

F3a. **Sources.** `docs/reports/T-256-interactive-multi-agent-comms.md:5-83`; `docs/reports/T-233-Q1-persistent.md:26,59`; `docs/reports/T-233-Q1-hybrid.md:1-74` (task T-233 created 2026-03-23, `.tasks/completed/T-233-*.md:14`).
F3b. **Design as stated.**
  F3ba. Problem: orchestrator spawns workers, workers write files, the orchestrator polls. Wanted: workers talk back "in real-time — no polling" (`T-256…:5`).
  F3bb. Findings: all event consumption is poll-based (250-500 ms); the gap is no `emit-to <target>`; option A `emit-to` (cleanest, not built), option B hub-side `event collect` fan-in (works today, shipped as a convention) (`T-256…:26,59-76`).
  F3bc. Claude Code constraint: a background `termlink event collect` costs about 800 tokens versus 10-18K for polling (`T-256…:39-44`).
  F3bd. T-233: a hybrid of persistent, warm-standby and on-demand specialists; "`termlink agent ask` can wake a specialist by injecting a prompt into a Claude Code session" (`T-233-Q1-persistent.md:26`).
F3c. **Operator's words.** "the spawned agent can talk to the spawning agent" (`T-256…:50`); insisted on TermLink mesh agents over the Claude Code Agent tool (`T-256…:51`).
F3d. **What it adds.** The earliest statement of the goal (no polling, two-way) and the first decision to prefer a wait over a poll. `emit-to` (push to a named target) was never built; push arrived 3.5 months later as the arc-004 hub-to-client WebSocket (A4).

## F4. Round — 2026-04-12 — T-967 / T-1135 persistent "receptionist" sessions (TermLink side)

F4a. **Sources.** `.tasks/completed/T-967-persistent-agent-sessions--mark-protect-.md:83,120-132`; `docs/reports/T-1135-persistent-sessions-response.md:35-88`; `docs/reports/T-967-persistent-sessions-findings.md`. AEF's side of the same round: AEF history round 1.
F4b. **Design as stated.**
  F4ba. Two needs conflict: the cleanup cron kills stale sessions, and persistent agent sessions "must stay alive indefinitely so other agents can discover and contact them" (`T-967…` Problem Statement).
  F4bb. Joint design with the framework agent: KV `persistent=true` (cleanup exemption), tag `role:receptionist` plus `project:<name>`, a non-blocking health check at `fw context init`, `fw doctor` reports it, respawn manual via `fw termlink respawn` with auto-respawn only by explicit opt-in, config in `.framework.yaml` (`T-967…:120-125`).
  F4bc. TermLink adds: `spawn --persistent`, cleanup warns about but does not remove dead persistent sessions, naming `{project}-agent`, and "a persistent session could be a lightweight 'receptionist' that only starts full Claude when a request arrives" (`T-1135 response:74,88`).
F4c. **Operator's words (recorded).** "User approved: persistent agent sessions with KV persistent=true, tag role:receptionist, .framework.yaml config. Joint design with framework agent completed via PTY coordination." (`T-967…:132`, 2026-04-12T10:26Z)
F4d. **Outcome.** GO. AEF's later audit says the receptionist "was never built" (AEF history round 1, `T-3396:29-38`).
F4e. **What it adds.** The persistent always-present per-project process the operator described again on 2026-09-20 ("a sidekick that's listening all the time") and the lightweight-front, start-Claude-on-demand variant. The rule "auto-respawn needs explicit opt-in" later returns as the 7/7 vendor finding on respawn-from-mail.

## F5. Round — 2026-04-30 — T-1425 agent-contact pattern RFC

F5a. **Sources.** `docs/reports/T-1425-agent-contact-pattern-rfc.md:11-208` (key lines 29-46, 87-99, 150-160, 192-193).
F5b. **Design as stated.**
  F5ba. Vendored agents improvise agent-to-agent contact; the RFC proposes a verb that resolves a per-pair `dm:<sender>:<sender>` topic and posts `msg_type=request` with `metadata.thread`, `requires_ack`, optionally awaiting an `m.receipt` (`:36-46`).
  F5bb. Solo synthesis, operator-requested fast-forward: Q2 ack semantics C (no ack by default, `--ack-required` opt-in); Q3 offline receiver C (queue by default, `--require-online` deferred); Q4 identity A (strict reject of a mismatched `metadata.from`) (`:150-160,192-193`).
F5c. **Operator's words.** "Operator asked for fast-forward synthesis 0h into the 48h soak window." (`:150-152`)
F5d. **What it adds.** The default that sends are fire-and-forget unless `--ack-required` — the origin of "delivered means queued" that A3 and A7 later had to undo — and the unbuilt `--require-online` (a send that fails fast when the peer is offline).

## F6. Round — 2026-05-25 to 2026-05-31 — T-1807 / T-1809 doorbell+mail validation, and T-1898 the vendored agent runner

F6a. **Sources.** `docs/reports/T-1807-doorbell-mail-loop-validation.md:35-84`; `.tasks/completed/T-1809-doorbell-respond-mode-signal--woken-chec.md` (ACs); `docs/reports/T-1898-vendored-agent-runner-inception.md:7-49`.
F6b. **Design as stated.**
  F6ba. T-1807: a 3-turn conversation over doorbell+mail passed with receipts per turn (offsets 0/3/6), but only with a mechanical responder. A live claude was blocked by two things: root cannot use `--dangerously-skip-permissions` (allowlist `Bash(termlink:*)` instead), and the doorbell `/check-arc` is read-only browse mode, so a woken claude reads the turn and never posts a receipt (`:47-77`).
  F6bb. T-1809: the doorbell text becomes `/check-arc respond`, a respond-mode signal.
  F6bc. T-1898 (2026-05-31): the presence half ships; "the agent half does not — no service holds an attached claude-code, reads `dm:<self>:*`, and replies"; the symptom was a LIVE agent with zero receipts on its DM topic (`T-1898 report:21`).
F6c. **Operator's words (verbatim in the dialogue log).** "really bandaid fixing ??????!!! incept incept incept ,,, again fricking critical fucntionality" (`T-1898 report:29`); "YOU ARE VIOLATING FRAMEWORK GOVERNANCE!!!!" when the agent skipped inception (`:33`); "REFLECT ON WHY AND TELL ME" (`:37`).
F6d. **What it adds.** The respond-mode signal (the doorbell must say "reply"); the first named gap "no service holds an attached claude and replies" — the native-consumer problem (A6, D1) four months before A6; and the first operator statement that this is critical functionality that keeps getting band-aided.

## F7. Addition to A3 — 2026-06-27 / 06-28 — T-2295 three ack mechanisms, and "receipt on READ, not on detect"

F7a. **Sources.** `.tasks/completed/T-2295-v3b-delivery-confirm-by-default--canary.md:158,216`.
F7b. **Design as stated.** A sidecar that auto-acks on detection erases the very flag that wakes the agent (`channel ack` advances the same `<self>` frontier the sidecar reads), so the receipt must be emitted on READ. Cross-agent comms has THREE confirmation signals: (A) the `msg_type=receipt` envelope (a post, never touches the frontier), (B) the `channel.receipts` frontier, (C) the reply turn. The conversational paths standardise on A (`:158,216`).
F7c. **Operator's words.** None recorded.
F7d. **What it adds.** The rule that makes the L2/L3 ladder necessary, and the constraint on the sidecar's `--auto-confirm` (stored, never read), which is still what runs today.

## F8. Addition to A5 — 2026-07-11 — T-2400 / T-2402 / T-2410 the deterministic-attention control loop

F8a. **Sources.** `.tasks/active/T-2402-woken-but-silent-make-a-rung-yet-unanswe.md:33,50,58-60`; `.tasks/completed/T-2400-reachable-agents-launch-mute--tl-claude-.md:37-45`; `.tasks/completed/T-2410-idle-gate-agent-send-doorbell-ring-sende.md:50,230-232`.
F8b. **Design as stated.**
  F8ba. "The ONLY non-deterministic node left is the agent's cognition… You cannot make LLM cognition deterministic — so this task makes the ENVELOPE around it deterministic" (`T-2402:33`). Six stages: 1 durable obligation, 2 push wake, 3 idle-gated injection, 4 receipt-or-re-ring, 5 escalate-if-stuck, 6 wake-protocol obligation (`T-2402:50`).
  F8bb. Stage 6: a woken agent drains ALL unread topics, a receipt per topic, and replies OR posts an explicit "acknowledged, no action needed"; "Silence is never a valid choice" (`T-2402:60`).
  F8bc. T-2400: a `--reachable` agent comes up in manual permission mode, wakes, composes a reply and STALLS at "Do you want to proceed?" — discoverable and wakeable but MUTE; fixed by default auto-accept (`T-2400:37-45`).
  F8bd. T-2410: the sender's blind inject can corrupt a busy peer's input; the sender-side ring is idle-gated; a persistently busy interactive peer is "the operator-held design fork (PL-253 / T-2396)" (`T-2410:50,232`).
F8c. **Operator's words.** GO on the arc (`owner: human`); no new quotes in these files.
F8d. **What it adds.** The six-stage control loop as one list (the history cites stages 3, 5, 6 only); the mute-agent failure; and the explicit statement that persistently busy peers are an operator-held design fork. That fork is exactly the urgent-bypass decision (open decision O2).

## F9. Addition to A7 — 2026-09-21 / 09-22 — T-3050 sidecar launcher, T-3067 L3 evidence gate

F9a. **Sources.** `.tasks/completed/T-3050-notify-sidecar-has-no-launcher---cron-su.md:5-25`; `.tasks/completed/T-3067-l3-stageread-tell-the-sender-its-message.md` (ACs 2, 6, 7).
F9b. **Design as stated.** T-3050: the notify rail had been dark for 82 days because nothing starts it; a declared-agents conf plus an idempotent cron supervisor gives autostart and self-heal in one mechanism, with identity declared (`<agent-id> <self-fp>`) because `termlink whoami` is ambiguous across 18 candidate sessions. T-3067: L3 `stage=read` is posted only with `--evidence <kind>`; a bare inject "lands UNSUBMITTED and is discarded" (T-2396), so "I injected it" is not evidence; the e2e LADDER stage reports `L2-ONLY` as non-green.
F9c. **Operator's words.** None recorded here.
F9d. **What it adds.** The origin of the always-respawns launcher (the T-3075 spec's "always respawns" has a cron answer first) and the evidence rule behind INJECTED.

## F10. Addition to A8 — 2026-09-30 — T-3280 pen-agent: the operator relays by hand because the agent route cannot reach

F10a. **Sources.** `.tasks/completed/T-3280-ask-pen-agent-to-confirm-its-relayed-ope.md:269-273`.
F10b. **What happened.** `agent-send.sh --to penelope` refused ("heartbeat does not declare pty_session — sender cannot ring the doorbell"); the fallback `agent contact` landed in the host's self-DM; an automated receipt 12 s later made the topic read, indistinguishable from pen's because all agents share one key. The operator pasted a prompt into pen's session directly.
F10c. **Operator's words (recorded).** "Pen did a lot of stuff… She picked it up. It's all fine." (`T-3280…:270`)
F10d. **What it adds.** A dated, operator-visible case of the rail failing to carry real work, and the shared-key defect that makes receipts unattributable. Not a design round; evidence for the status section.

## F11. Round — 2026-10-01 — T-3304 hub storage model: retention for mail and for telemetry

F11a. **Sources.** `docs/reports/T-3304-hub-storage-model.md:114-193` (items 6-20).
F11b. **Design as stated (rulings).**
  F11ba. IW-1 = B: the hub is authoritative for coordination within a declared retention window (item 8). IW-2 = C: a bounded default with a sweeper on every hub and a ceiling on post; 14-day default assumed (item 10). IW-3 = B: honest reads, an ordered gap signal first (item 12). IW-4 = C-prime (item 15). T-3310 D1 = D+, D2 = B (mail by count, the rest 14 days), D3 = C (ceilings on post) (items 17-20).
  F11bb. Messages are small envelopes in bounded pages; binary payloads use the chunked artifact path and never meet the 20 s limit (item 1).
  F11bc. Retention should combine count, age and size, with limits configurable from a settings page, plus a job collapsing duplicates; deriving knowledge from the message flow is its own inception → T-3319 / T-3320 / T-3321 (item 19).
F11c. **Operator's words (verbatim in the log).** "record B, with all this discussion we had" (item 8); "Alright, C then" (item 10); "take your recommendation … vote for B" (item 12); "let's go with C as suggested and recommended" (item 15); "I agree with the recommendation" (item 18); "as suggested" (item 20).
F11d. **What it adds.** The retention rulings the telemetry design must obey. The design's "for example 30 days" window (previous design §7) conflicts with the 14-day default (open decision O13).
