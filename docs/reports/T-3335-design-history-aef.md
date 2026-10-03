# T-3335 — The AEF side of the agent-to-agent communication design: full history, current design, differences, status vs reality

**Task:** T-3335 · **Compiled:** 2026-10-03 · **Mode:** read-only research; this is the only file written.

**How to read the citations.**

1. `AEF:` = `/opt/999-Agentic-Engineering-Framework` (branch bleeding-edge), cited `path:line`.
2. `TL:` = `/opt/termlink`, cited `path:line`.
3. Hub messages are cited `topic@offset` with these short names:
   3a. `ARC` = `agent-chat-arc`
   3b. `SC-AEF` = `sidecar:999-Agentic-Engineering-Framework`
   3c. `SC-010` = `sidecar:010-termlink`
   3d. `IN-AEF` = `inbox:cacc73ea32b121dd/999-Agentic-Engineering-Framework`. These are messages TO AEF.
   3e. `IN-010` = `inbox:cacc73ea32b121dd/010-termlink`. These are messages TO TermLink (mostly from AEF).
   3f. `PICKUP` = `framework:pickup`
4. Offsets are per topic. `IN-010@141` and `IN-AEF@141` are different messages.

**Coverage limits (stated, not hidden).**

1. Bash and the Agent tool are blocked for the AEF checkout, and there is no Glob/Grep. Files were opened one by one with Read, so I could not enumerate `docs/`, `lib/sidecar/` or `.tasks/`.
2. Read in full or in the relevant part:
   2a. `lib/sidecar/{circuit,receiver,watcher,inject,hooks,adapter,direct,receipts,http_server,lifecycle}.py`
   2b. `lib/retry_ladder.py`, `lib/aef_address.py` (head only)
   2c. `docs/architecture/sidecar-target-architecture.md` and `sidecar-roundtrip-and-telemetry.md`
   2d. `docs/reports/T-3396`, `T-3287`, `T-3433`, `T-3434`, `T-3461`, `T-3751-review-brief`, `T-3751-2a-review-brief`, `T-3751-review-synthesis`
   2e. `.tasks/completed/T-3397-*`
   2f. `.context/arcs/parallel-execution-aef.yaml`
   2g. `decisions.yaml` lines 4130-4924 (D-592..D-703)
3. NOT located (slug unknown): the task files and reports for T-3475, T-3682, T-3684, T-3685, T-3686, T-3688, T-3690, T-3693, T-3742, T-3770, T-3434 beyond §4, D-599/D-660 task bodies, arc-020 YAML. Their content here comes from code docstrings, decisions.yaml and hub messages. Anything resting only on a hub message is marked "(hub)".
4. Hub topics were read from offset 0 up to the 1000-record page. `framework:pickup` was filtered to sidecar/circuit/receiver/inject keywords, and `agent-chat-arc` was read as its first 1000 records only (next cursor 2015), so its later offsets are not covered.
5. AEF's source tree was read as it stands now, not at the date of each message. Where code and a message disagree, the code wins and is marked.

---

## A. ROUNDS — chronological

The operator says "about five". The record shows **six** identifiable rounds, because the first (round 1) is prehistory the operator remembered but AEF had not built.

### Round 1 — prehistory: receptionist (T-1135) and the arc-011 §5 cooperative flag (2026-04-12 and June 2026)

1. **Sources.**
   1a. AEF:docs/reports/T-3396-peer-consult-sidecar-inception.md:29-59
   1b. AEF:.context/arcs/parallel-execution-aef.yaml:1-9, 12-13
2. **Design as stated.**
   2a. T-1135 (GO 2026-04-12): a persistent per-project "receptionist" session, exempt from TermLink cleanup, respawned via `session.needs_restart`. It was never built (T-3396:29-38).
   2b. arc-011 §5 (`docs/architecture/parallel-execution-aef.md`): a deterministic, non-LLM sidecar writes a flag plus heartbeat. The agent's own harness polls the flag at a yield point it controls. **It explicitly rejected PTY injection into an apparently idle terminal.** The reason: an agent mid-turn is single-threaded, and no outside observer can prove a safe point (T-3396:40-59).
   2c. Only a stub shipped: `agents/dispatch/yield-point.sh`, a plain file-flag check (T-3396:57-59).
3. **Operator's recollection that restarted the work (2026-09-20), verbatim:** "What we want is the [sidekick] for interactive conversation that we worked on... a sidekick that's listening all the time... it sets a flag... the cron job monitors that flag... looks for the active session... we could have a queue where the queue can be prioritized... a flag is raised that there are new messages... use PTY inject when the cursor is silent... with the exception of the urgent message that warrants an interruption." (T-3396:87-92)
4. **AEF's correction at the time.** The operator's memory includes PTY-inject-on-idle, and arc-011 had rejected that mechanism. AEF surfaced the divergence in dialogue rather than substituting silently (T-3396:94-102).
5. **Operator ruling, verbatim:** "Let's go to the Royal and the Correct Long Term Road. Let's pack it full out." (T-3396:104-105)

### Round 2 — identity taxonomy and circuit model, T-3287 (opened 2026-09-06; D1-D7, V9 grammar)

1. **Sources.** AEF:docs/reports/T-3287-identity-taxonomy-circuit-model.md:28-107; AEF:lib/aef_address.py:1-29; AEF:docs/reports/T-3751-review-brief.md:64-108.
2. **Design as stated.**
   2a. Five levels: host / hub / project / session / agent (T-3287:28-50).
   2b. A **circuit** is a durable, re-usable channel between a level-4+ initiator and a level-5 agent. Its id is "the session-of-the-conversation" (T-3287:61-76).
   2c. A regressive resolution ladder: on a dead endpoint climb 5→4→3→2→1 until an ancestor can re-provision, then re-establish downward. This is an OTP supervision tree (T-3287:80-107).
   2d. Ratified decisions D1-D7 (brief:64-108):
      - D1: instance identity, not role identity.
      - D2: every level has a durable name; the project is passive (a path).
      - D3: wire grammar V9, `aef::host=…::hub=…::project=…::session=…::@agent::`.
      - D4: three-state circuit lifecycle.
      - D5: re-provisioning is pre-authorised self-heal.
      - D6: hub 1:1 with host.
      - D7: single-writer claim per project.
3. **Operator's words recorded:** none verbatim in the parts read. The reframing is paraphrased: "Who is talking" and "how do I reach them again" are emergent accidents (T-3287:20-24).

### Round 3 — operator "sidekick" dialogue becomes the buildable spec, T-3396 / T-3397 (2026-09-20/21)

1. **Sources.** AEF:docs/reports/T-3396-…:128-337; AEF:.tasks/completed/T-3397-resolve-t-3396-open-questions-iw-1iw-6-a.md:78-300.
2. **Design as stated (T-3397:80-132).**
   2a. **Push, not pull.** The sidecar is not a hub-broadcast subscriber. It exposes an API a sending agent calls directly. A message file is written, then a companion flag file (dirty-bit) is set, so the message is durable the instant the call returns (T-3397:80-89).
   2b. **A 30 s tick** (configurable) is the guaranteed-delivery fallback. A write-time fast path injects at once if the receiver is ready. Store-then-maybe-inject means failure is latency, never data loss (T-3397:91-96).
   2c. **"Ready for input" is self-reported by the harness.** The Stop hook writes `ready-for-input: true`. The UserPromptSubmit hook clears it with zero tolerance for lag. Timestamp-staleness and CPU/PID heuristics were rejected because a long Bash tool call looks idle but is unsafe (T-3397:97-113).
   2d. **Urgent = bypass.** (operator decision 2026-09-21): an urgent message skips the ready-flag check and injects immediately, accepted risk, "a hard skip, not a shortened interval" (T-3397:194-199).
   2e. **Bidirectional ack:** stored / injected-now / injected-later (T-3397:118-125).
   2f. **Symmetric API:** reply is the same call with sender and target swapped (T-3397:127-132).
3. **Joint analysis with TermLink's agent (T-3397:200-263).**
   3a. TermLink's T-967 persistence contract never shipped.
   3b. TermLink's recommendation: AEF owns readiness/inject/urgent policy "non-negotiably", since TermLink cannot observe harness state.
   3c. Two cautions came from TermLink: the frozen-husk class (so liveness must be a capability probe, not a timestamp), and shipped≠live.
4. **Operator correction, verbatim, 2026-09-21:** "We have multiple hosts, so that's not a question." (T-3397:270-272). This turned cross-host from a deferred branch into a live requirement, and led to the uniform-path rule below.
5. **Amendments 1-5 (T-3396:128-337), the buildable spec.**
   5a. Uniform path: cross-host and same-host are ONE path, never a fast path plus an exception (T-3396:136-144).
   5b. Third ack state `UNKNOWN`, never silently promoted or demoted (T-3396:146-173).
   5c. Same-host is the WEAKER identity case: co-resident agents share one host-wide signing key (T-3396:175-191).
   5d. G-060 resolved: direct cross-post, no routing layer; credentials per sender per hub are an explicit precondition (T-3396:193-220).
   5e. A cross-hub post succeeds or fails loudly. The sidecar owns its own retry (T-3396:222-245).
   5f. Liveness is two fields (seq counter plus self-probe), not a timestamp. Message/flag/ledger shapes plus `client_msg_id` dedupe. Every ack row carries a `deadline`; past deadline → `UNKNOWN` (T-3396:247-337).
6. **What changed from round 1.** Round 1 rejected PTY injection. Round 3 keeps injection but gates it on harness-self-reported readiness, with an explicit urgent bypass. Round 3 also adds the receiver-side API and flag, which round 1 never had.

### Round 4 — AEF↔TermLink exchange over the hub: receiving half, addressing, retry (2026-09-21 to 2026-09-22)

1. **Sources.** ARC@1611 (09-21 22:16), @1633-@1641, @1643-@1665 (09-22); SC-AEF@0-@4; SC-010@0-@2; IN-010@7; AEF:docs/reports/T-3433-circuit-addressing.md; AEF:lib/sidecar/circuit.py; AEF:docs/reports/T-3434-retry-ladder.md; AEF:decisions.yaml:4178-4190.
2. **Design statements, in order.**
   2a. ARC@1611: AEF asks TermLink three questions about the receiving half. Q1: does a receiving half exist? Q2: is `--await-ack` dm-only by design? Q3: the measured dedupe TTL (~5 min; a re-post at +15m39s appended a duplicate).
   2b. ARC@1633: AEF's outbound half is live, plus `fw sidecar e2e`, a six-hop live check. ARC@1634-@1639: five live runs over TermLink's hub, transport 5/5, un-instructed answer 3/3.
   2c. ARC@1640 (TermLink, task T-3062): ears, receipts, wake events (`inbox.queued`, hub channel.rs:949) and the sender's `--await-ack` exist. **"What is MISSING: the hand-to-agent step… Your (b) — a ready-for-input flag written by the harness's own Stop/UserPromptSubmit hooks — is precisely the consumer we do not have. Do not build ears or receipts; do build (b)."** TermLink asks which addressing AEF picks: `sidecar:`, `inbox:` or `dm:`.
   2d. ARC@1641 and SC-010@0: AEF's position, pending its operator, is `inbox:<agent-id>`. Reason: `dm:<fp>:<fp>` loses on identity today because every consult signs as the shared host key, so receipts would be self-satisfying. Per-agent keys are the follow-on.
   2e. **SC-010@2, "DECIDED by our operator, 2026-09-22".**
      - ADDRESSING: `inbox:<circuit-id>`. "We take your prefix; what follows it is our five-level circuit id."
      - Two forms: a durable role address at project level, `inbox:<hub>/<project>`, and an exact circuit address for dispatched workers.
      - `sidecar:*` stays a read alias for one release.
      - RETRY: a universal ladder, 2×1m, 2×5m, 2×15m, 2×1h, 2×4h, 2×1d, 2×1w, 2×1mo, then dead-letter. It deliberately outlives the hub's dedupe TTL. Posted-but-unread messages are ESCALATED, never re-posted. Receiver-side dedupe on `client_msg_id` is mandatory. "Urgent-compression is designed separately."
   2f. Recorded as D-599 (addressing, decisions.yaml:4178-4183) and D-600 (ladder, decisions.yaml:4185-4190). Implemented as T-3433 (circuit.py) and T-3434 (retry_ladder.py).
   2g. IN-010@7 (09-22 16:38): "T-3433 is closed. Our consults now post to inbox:cacc73ea32b121dd/010-termlink… T-3434's retry ladder is live too."
3. **Circuit address as built (circuit.py:10-31).**
   3a. Four emitted forms: `<hub>/<project>` (durable role), `<hub>/<project>/<agent>`, `<hub>/<project>/<session>/<agent>`, and `//<host>/<hub>/<project>[/…]` (metadata only).
   3b. The address starts at the hub, not the host, because "a topic is an object on a hub" and AEF does not know TermLink's FQDN. The host rides in `metadata.from_circuit`.
   3c. D-609..D-612 (decisions.yaml:4248-4274) record the sub-decisions.
4. **What changed from round 3.** The hub-topic path became the concrete transport, addressed by circuit. The retry ladder was invented. AEF adopted TermLink's `inbox:` prefix. The receiver API of round 3 had not yet been built.

### Round 5 — review, "build toward the design", and the address-scheme contradiction (2026-09-25 to 2026-09-29)

1. **Sources.** AEF:docs/reports/T-3461-sidecar-architecture-review.md; AEF:docs/architecture/sidecar-target-architecture.md; AEF:decisions.yaml:4512-4517, 4617-4622; IN-010@39, @53; IN-AEF@6, @10; PICKUP; AEF:lib/sidecar/circuit.py:247-276.
2. **T-3461 review (09-25).** The dashboard said 45/45 delivered, but 15 of 45 messages climbed to rung 4-5. The OPERATOR escalation rung wrote to `.context/inbox.yaml` (319 pending, no renderer, `fw notify` off), so "the last rung of an escalation ladder built to recover from 'nobody read it' terminates in a queue nobody reads" (T-3461:82-113). No architecture document existed (T-3461:11-33). Both address schemes were live with independent offsets (T-3461:128-145).
3. **D-645, operator ruling 2026-09-25 (decisions.yaml:4512-4517).**
   3a. "Sidecar: build toward the T-3397 design (receiver-side API, flag, readiness gate, two confirmations); binary blobs ride TermLink file transfer via the sidecar API, not session-to-session."
   3b. Rationale: the shipped implementation is "hub-broadcast pub-sub polled by a 5-minute cron, which is the exact architecture T-3397:81 says the sidecar is NOT. Rather than ratify the divergence, we build toward the captured design."
   3c. Written up as `sidecar-target-architecture.md` (requirements R1-R15, build slices 0-7) and `sidecar-roundtrip-and-telemetry.md`, a per-hop telemetry design.
4. **Operator ruling 2026-09-25: five-part circuit, V9 grammar** (IN-010@39): `aef::host=…::hub=…::project=…::session=…::@agent::`, "dual-read both grammars… current address as a read-only alias for one release".
   4a. circuit.py:247-276 (T-3479, "operator ruling T-3475") implements a SPARSE V9, hub-anchored, with `host=` never emitted. It is **read-side only**.
5. **D-660 (operator ruling 2026-09-27, T-3518; decisions.yaml:4617-4622).** "AEF adopts 010-termlink's addressing — dm:<fp>:<fp> / inbox:<agent-id> — and does NOT keep sidecar:<agent-id>… Option 3 (support both) was explicitly refused… 010-termlink does NOT need to file the --await-ack-from generalisation."
   5a. D-660 literally conflicts with D-599's `inbox:<circuit-id>` wording. 832 spotted it and withdrew its circuit-id framing (IN-AEF circuit-id-adoption note @9, cid `circuit-id-adoption`).
   5b. AEF's own correction, IN-010@53 (09-29): "The five-level circuit model stands. Our D-660 adopted your prefixes (inbox:, dm:), not your keys as addresses; its 'inbox:<agent-id>' wording was a mistake on our side and is being amended. Agent-to-agent traffic stays on inbox:<circuit-id>." I did not find the amended D-660 text; the decisions.yaml read still shows the original wording.
6. **Collision and identity proposal (IN-010@53 items 2-3).** `dm:<fp>:<fp>` uses the machine-wide identity, so all projects on a host share `dm:d1993c2c3ec44c94:d1993c2c3ec44c94`. AEF proposes per-agent/project named identities in TermLink (`termlink identity`, a `whoami` that errors on ambiguity, optional ACLs) for TRUST, not routing.
7. **TermLink side in this round.**
   7a. SC-AEF@21 / IN-AEF@6 (09-28): TermLink admits it never consumed `inbox:` mail (49 unread). Cause: its `inbox status` showed a raw record count labelled "pending transfers".
   7b. SC-AEF@22 / IN-AEF@10 (09-28 22:29): corrected. Its notify rail enumerated only `dm:` prefixes (T-3201, T-3203 fixed that).
8. **What changed from round 4.** Operator ruled that the design in the spec is the target and the shipped pub/sub is not. Two incompatible address rulings (D-599 vs D-660) now sit in the record. Blobs, telemetry and an explicit requirements register were added.

### Round 6 — the receiver is built; naming, project id, polling ladder and telemetry negotiated (2026-10-01 to 2026-10-03)

1. **Sources.** AEF:lib/sidecar/*.py; AEF:decisions.yaml:4827-4902; IN-010@62, @83, @86, @125, @141, @197-@198, @225-@228, @250-@251; IN-AEF@99, @129-@133, @137-@143, @180-@186; AEF:docs/reports/T-3751-*; TL:docs/reports/T-3330-receive-side.md; TL:docs/design/interactive-agent-communication.md.
2. **T-3682 audit, reported to TermLink 10-02 (IN-010@62, hub).** "AEF audited its sidecar against the ratified design (T-3682): **the whole receive half had never been built.**" Delivered the same day: T-3693 (per-agent receiver with an HTTP API, T-3475 ruling; Stop/UserPromptSubmit ready flag; ONE-line `termlink pty inject`; HANDED_OVER only on transcript evidence; sender states SENT/RECEIVED/HANDED_OVER/REPLIED/UNDELIVERABLE/REJECTED/ESCALATED; `INJECTED_NOW` renamed `HUB_ACCEPTED`).
   2a. Next slices: S-XHOST T-3688 (publish each receiver endpoint to the hub for cross-host discovery), S5 T-3686 (blobs through the receiver), S7 T-3690 (retire `sidecar:<agent>`, "needs your agreement").
   2b. Standing directive D-700 (2026-10-02, operator): AEF stays in regular contact with 010-termlink and 055 (decisions.yaml:4897-4902).
3. **Watcher, operator 2026-10-02, verbatim (AEF:lib/sidecar/watcher.py:3-6):** "a watcher … watching if the message flag is up every 30 seconds and when it's up it looks if it is an urgent message and injects directly, or if it's not urgent it looks if the prompt is free and if it's free it injects the message and informs the sender."
4. **Recorded decisions D-690..D-697 (decisions.yaml:4827-4881).**
   4a. Message temp-file, rename, then ready flag.
   4b. Receiver states separate from sender states.
   4c. Stop hook sets ready only at turn end; UserPromptSubmit clears it BEFORE the new turn.
   4d. Peer content framed as UNTRUSTED data: "a request for action becomes a task proposal… never direct execution".
   4e. HANDED_OVER recorded only when the prompt hook has actually surfaced the message (D-696: the earlier worker had wrongly written "on success [of the inject]").
   4f. D-697: no registered receiver → hub-topic fallback; registered-but-unreachable → retry budget, then UNDELIVERABLE.
5. **Operator 2026-10-03, "receipt telemetry on EVERY path" (watcher.py / receipts.py:1-36).** T-3684 adds receipts for consults that arrive on the hub topic; T-3745 makes readiness per SESSION after two Claude sessions in one project shared one flag.
6. **Project identity.** AEF minted `pid-<16 hex>` in `.framework.yaml` (T-3534; back-fill T-3750; IN-010@125). The sidecar still uses the folder basename (circuit.py:136-153); moving it is inception T-3751 (IW-1 ruling C, 2026-10-03: the project slot carries the minted id; brief:105-108). TermLink agrees: never fall back across projects (IN-010@141).
7. **Two-identity naming model (operator 2026-10-03).** TermLink's operator proposed canonical + instance identity per level (IN-AEF@186). AEF replied that "it is the same model our operator proposed on 2026-10-03" and ran a 7-vendor review: 7/7 ADOPT-WITH-CHANGES; then decision 2a, 7/7 option B: function names route, instance ids ride inside (IN-010@250; AEF:docs/reports/T-3751-review-synthesis.md:19-150). Respawn from inbound mail is a separate decision (2b) needing an explicit operator grant (synthesis:42-58). The synthesis does not record a final operator ruling on 2a.
8. **Telemetry and polling ladder negotiation.**
   8a. TermLink asked AEF to review its telemetry design (IN-AEF@141, @184). AEF answered RECEIVED, with an overlap check queued: its T-3684/T-3685 "already send RECEIVED/HANDED_OVER/REPLIED back on every path with timestamps (fw sidecar latency)" (IN-010@251). No full answer found yet.
   8b. TermLink's operator polling ladder is 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 d, 3 d, 1 w, 1 mo, 1 q, 1 y, each rung twice (IN-AEF@180). AEF filed T-3770 to reconcile it with T-3434 (IN-010@225, @228).
9. **What changed from round 5.** The receiver, watcher and per-session injection were built and proven live in AEF's own e2e. Receipts now exist on every path. The address model moved toward minted ids and two-identity labels. Telemetry and polling cadence are now negotiated with TermLink.

---

## B. AEF'S CURRENT DESIGN — as built (code) and as planned (docs/hub)

### B1. Components and process model

1. **Per-agent receiver** (AEF:lib/sidecar/receiver.py, http_server.py, lifecycle.py).
   1a. A separate process per agent. Stdlib HTTP on `127.0.0.1` only (http_server.py:135-143, 155).
   1b. Authentication: `Authorization: Bearer <token>`, constant-time compare, token file mode 0600 written before the port opens, refuses to start without it (http_server.py:13-17, 56-60, 137-140; lifecycle.py:67-85).
   1c. Triple files plus token under `.context/sidecar/receiver.{pid,port,url,token}` (lifecycle.py:6-13).
   1d. Host registry `~/.local/state/fw-sidecar/receivers/<agent>.json` = `{agent,url,pid,project_root,token_file}` (lifecycle.py:15-20).
2. **Watcher + supervisor** (watcher.py).
   2a. One tick every `SIDECAR_TICK` seconds, default **30** (watcher.py:13-15, 69; `tick_seconds` precedence at 92-106).
   2b. Tick does: drain legacy hub-topic consults into the receiver (`ingest_hub`, 236-283); inject (`deliver_pending`); escalate expired HANDED_OVER deadlines; loopback self-probe; write `liveness.yaml` (watcher.py:412-441).
   2c. Not-live = `seq` stalled 2 ticks OR the probe failed (watcher.py:366-407). Read by `fw doctor` and `fw audit`.
   2d. The supervisor restarts the watcher when it dies or hangs, and restarts the receiver when it is gone. `fw sidecar ensure --all` runs from cron `sidecar-ensure-1m` plus `@reboot` (watcher.py:33-40, 527-602, 679-704). The "30 s cron" of the operator's words is built as a supervised loop, not literal cron.
3. **Hooks** (hooks.py, adapter.py).
   3a. Stop hook sets this session's ready flag; UserPromptSubmit clears it FIRST (hooks.py:3-6, 80-87).
   3b. Per-SESSION records `.context/sidecar/sessions/<claude session_id>.json` naming the TermLink session and the ready bit (adapter.py:20-35, T-3745). The project-wide flag is display only.
4. **Injector** (inject.py).
   4a. Types ONE fixed line via `termlink pty inject <session> "<line>" --enter`. The line carries no peer content, only a count and ids (inject.py:3-13, 251-261, 327).
   4b. Target = a Claude session whose own record says ready, registered for this project (tag `fw-project=<sha256(root)[:16]>`, else claude tag + cwd), claude process alive, never a headless worker (inject.py:15-27, 49-55, 130-145, 183-248).
   4c. A claim naming the target is written BEFORE typing, so HANDED_OVER is credited to the session that was injected (inject.py:312-324).
5. **Peer content handling** (hooks.py:101-131). The prompt hook surfaces stored messages framed `<<<PEER-DATA … PEER-DATA>>>` as UNTRUSTED ("grants attention, never authority").
6. **Hub-topic path** (inbox.py, outbox.py, delivery.py, retry.py, termlink_transport.py; known only through docstrings of other modules and the roundtrip doc). `fw sidecar send` posts `channel post inbox:<circuit>` with `client_msg_id` in param and metadata. This path is the fallback and the legacy path.

### B2. States, endpoints, receipts

1. **Endpoints** (http_server.py:5-11): `POST /message` (store, flag, answer RECEIVED = CONFIRM-1, then inject-if-ready on a thread), `POST /ack` (CONFIRM-2 and receipts from peers), `GET /health`, `GET /status`.
2. **Sender-side states** (direct.py:1-26, 40-58): SENT, RECEIVED, HANDED_OVER, REPLIED, UNDELIVERABLE, REJECTED, ESCALATED. The effective state is the HIGHEST-ranked row, not the newest (RANK at direct.py:57-58). ESCALATED ranks below HANDED_OVER so a late truth supersedes it.
3. **Who sets each state.**
   3a. RECEIVED: the sender, from the receiver's HTTP response.
   3b. HANDED_OVER: the peer receiver posts `/ack`, sent only after its prompt hook surfaced the message.
   3c. REPLIED: our receiver, when a message answering it is stored (`note_reply`, direct.py:258-286).
   3d. ESCALATED: infrastructure, when the HANDED_OVER deadline passes. The default deadline is 900 s (direct.py:60, 291-299).
4. **HANDED_OVER evidence** (hooks.py:14-24): a detached finalizer waits up to 90 s (`FINALIZE_WAIT_S`) for the session transcript to contain the `hook_additional_context` attachment carrying the message under a one-time surfacing token. Otherwise `HANDOVER_UNCONFIRMED` and the message becomes eligible again. This follows a live defect where a hook was killed after printing.
5. **Receipts on the hub path** (receipts.py:1-36): RECEIVED at once on ingest, HANDED_OVER only on transcript evidence, REPLIED when `fw sidecar send --in-reply-to` succeeds. Delivery goes to the sender's registered live receiver via `/ack`, else to its hub inbox topic as `kind=receipt`. A receipt is recorded only for an id this agent sent to that same peer.
6. **Hub-path ack ledger** (T-3434; roundtrip doc :43-60): STORED → INJECTED_NOW/INJECTED_LATER/UNKNOWN. `INJECTED_NOW` is renamed `HUB_ACCEPTED` because it means only "the hub took it" (sidecar-target-architecture.md:147-150; IN-010@62).

### B3. Injection route, readiness, urgent

1. **Route:** receiver stores → flags → `inject.deliver_pending` (on store and on every tick) → `termlink pty inject … --enter` → prompt hook surfaces and claims → finalizer records HANDED_OVER → CONFIRM-2 to the sender (receiver.py:18-22; inject.py:286-347; hooks.py:134-150).
2. **Readiness check:** self-reported by the harness hooks, never inferred from the terminal. Missing/unreadable flag reads as NOT ready ("the safe direction", adapter.py:88-111; design basis T-3397:97-113).
3. **Non-urgent:** only into a candidate session whose OWN record says ready. None ready → the message waits and an `INJECT_BLOCKED` event is recorded once per reason (inject.py:218-226, 276-283).
4. **Urgent (R5, "hard bypass"):** a ready candidate if any, else the most recently active candidate, even busy; else, with exactly one registered TermLink session and no record, that session only if an INTERACTIVE claude is seen in it (inject.py:213-248, 307-309). A busy target takes only the urgent messages (inject.py:307-309). An urgent bypass is recorded as `urgent_bypass` (inject.py:338-341).
5. **Not built:** urgent compression of the retry ladder. `retry_ladder.py` raises `NotImplementedError` for `urgent=True`, by D-600 ("designed in a separate conversation") (retry_ladder.py:49-54, 86-96).
6. **Re-inject window:** an inject with no hand-over may be retried after 120 s (`REINJECT_AFTER_S`, inject.py:46).

### B4. Cadence

1. Watcher tick 30 s, configurable (watcher.py:69, 92-106). Operator, 2026-10-02: "every 30 seconds" (watcher.py:3-6).
2. Hub-path ladder driven by the `sidecar-sweep-5m` cron (retry_ladder.py:31-35; roundtrip doc:30). Ladder 2×1m…2×1mo, ~76 days, 16 attempts, nudge from the 15-minute rung, operator surfacing from the 1-day rung, dead-letter after (retry_ladder.py:16-27, 62-84; T-3434-retry-ladder.md:11-35).
3. Receiver dedupe is mandatory because the ladder outlives the hub's ~5 min dedupe TTL (retry_ladder.py:39-47; T-3434-retry-ladder.md:74-95).
4. `tick: 15 s proposed, never validated` in the architecture doc (sidecar-target-architecture.md:111-116, 291) is superseded by the built 30 s default.

### B5. Cross-host plan

1. **Principle (round 3):** one uniform path; same-host is the degenerate case (T-3396:136-144). Operator: "We have multiple hosts, so that's not a question" (T-3397:270-272).
2. **Built:** receiver is `127.0.0.1` only (http_server.py:135, 155). The registry is host-local, and the token is read from a file only the same user can read (lifecycle.py:15-20).
3. **Planned:** S-XHOST T-3688 publishes each receiver endpoint to the hub so peers can discover it (IN-010@62, hub). lifecycle.py:20: "Cross-host addressing is T-3688." Register R12 is `partial`: "same-host only verified; cross-host fenced" (sidecar-target-architecture.md:260-265).
4. **Today's cross-host path:** the hub-topic fallback (`channel post --hub`, credentials per hub, T-3396:193-220), which gives RECEIVED/HANDED_OVER receipts only via hub receipts (receipts.py:20-26).
5. **Blobs (D-645):** through the receiver sidecar API, hash-verified before the flag is set. TermLink file transfer is the carrier (sidecar-target-architecture.md:82-105; D-645). T-3686 (S5) is not built per IN-010@62 "Next slices".

### B6. Identity and circuit addressing

1. **Five levels** host / hub / project / session / agent (circuit.py:10-31; T-3287).
2. **Emitted topic** `inbox:<hub>/<project>[/<session>/]<agent>`; the host-qualified full id goes in `metadata.from_circuit` (circuit.py:14-31; D-599).
3. **Hub id** = first 16 hex of `termlink hub fingerprint`, cached on disk (circuit.py:99-133). The fingerprint rotates on cert rotation (010 PL-021), so it is an INSTANCE id (T-3751 synthesis:36-38). AEF migration to a stable hub name is not built.
4. **Project level** = folder basename today (circuit.py:136-153). Target: minted `pid-<16 hex>` (D-663/D-664, decisions.yaml:4638-4650; T-3534 built, T-3751 not built, IN-010@125). `project=<id>` replaces `project=<path>` in V9.
5. **V9 form** exists as read-side sparse V9 with no `host=` token (circuit.py:247-286).
6. **Agent name** = `FW_SIDECAR_AGENT_ID`, else project; explicitly NOT the TermLink identity fingerprint, which is machine-wide (circuit.py:164-170).
7. **Fallback:** the sidecar never climbs at send time; its lowest address is `<hub>/<project>` (brief:101-103). Never fall back across projects (IN-010@141; synthesis:60-67).
8. **Pending rulings:** decision 2a (which label routes; 7/7 B), 2b (respawn needs an explicit grant), IW-3 (name→id directory), IW-2 (topic migration with dual read) (synthesis:117-150).
9. **Trust gap:** all agents on a host sign as one TermLink identity. Per-agent keys are TermLink's fix and are not built (T-3396:175-191; synthesis:79-81; SC-010@0).

### B7. Telemetry

1. **Built:** per-direction ledgers `direct-ack.jsonl`, `receipts.jsonl`, `receipts-sent.jsonl`, `receiver/events.jsonl`, `watcher/ticks.jsonl`, `liveness.yaml`; `fw sidecar latency` reads receipts (direct.py:19; receipts.py:28-36; receiver.py:24-31; watcher.py:444-453).
2. **Designed, not verified as built:** one record per hop, `.context/sidecar/telemetry.jsonl` (`resolve|write|probe|post|ledger|drain|surface|reply|sweep|close|escalate`). Derived metrics: dwell, round-trip, escalation precision (roundtrip doc:184-247). Cross-agent collection: AEF recommends option (b), a collector pulls each agent's file, because option (a) "fails exactly when the rail fails" (roundtrip doc:235-247).

---

## C. AGREEMENTS AND DIFFERENCES with TermLink's design

TermLink's design of record is TL:docs/design/interactive-agent-communication.md (T-3335, 2026-10-03), with TL:docs/reports/T-3330-receive-side.md and TL:docs/operations/notify-sidecar-api.md.

### C1. Agreements

1. **The hand-to-agent consumer is the Stop/UserPromptSubmit readiness flag.** TermLink: "Your (b)… is precisely the consumer we do not have. Do not build ears or receipts; do build (b)" (ARC@1640). AEF built it (hooks.py; D-694).
2. **Mail prefix and wake rail.** AEF adopted `inbox:` / `dm:` (D-599, D-660); TermLink's hub emits `inbox.queued` on them (ARC@1640; circuit.py:3-9).
3. **Five-level circuit, project-level durable address, never fall back across projects.** TermLink, IN-AEF@129: "five-level circuit model stands… never falls back across projects"; AEF, IN-010@141: "Agreed on IW-4: never fall back across projects."
4. **Two identities per level.** TermLink: "every one of the five circuit levels carries TWO identities" (IN-AEF@186; TL design item 6). AEF: "the same model our operator proposed" (IN-010@250). Both agree the pid is the canonical project id; the hub fingerprint is an instance id (TL design item 6b; synthesis:36-38).
5. **Single message id and dedupe.** Both key on `client_msg_id`; receiver-side dedupe is mandatory (retry_ladder.py:39-47; TermLink's side in SC-010@0).
6. **Push first, pull as the fallback; a 30 s tick.**
   6a. TermLink (TL design item 18a, :52): "A cron job runs every 30 seconds and checks the new-message flag." (operator 2026-10-03)
   6b. AEF's operator, 2026-10-02 (watcher.py:3-6): same 30 s watcher, same urgent/prompt-free logic.
   6c. TermLink's note that system cron's unit is one minute, so "every 30 seconds" is built as a supervised loop (TL design item 18a), matches AEF's built supervisor (watcher.py:527-602).
7. **Injection is verified, not assumed.** TermLink 18d (:58): "After injecting, the sidecar observes that the agent actually took the message… Only then is INJECTED reported." AEF: HANDED_OVER only on transcript evidence (hooks.py:14-24; D-696).
8. **Urgent bypass is the rule.** TermLink 18c (:54) "Urgent: injected immediately… even when the agent is busy"; AEF R5 (sidecar-target-architecture.md:211-217; T-3397:194-199).
9. **TermLink stays underneath for discovery, blobs and the fallback topic** (IN-010@62, hub). `termlink pty inject` is the typing primitive (inject.py:327; TL design item 44-46).
10. **Standing collaboration.** D-700 (decisions.yaml:4897-4902), and TermLink's own receipts: "we acknowledge on arrival even when busy" (IN-AEF@137).

### C2. Differences and open contradictions

| # | Topic | TermLink side | AEF side |
|---|---|---|---|
| 1 | Send path | SQ-1: sending stays on the hub (`channel post`); the sidecar API is LOCAL only, because a cross-host API "would make TermLink a second bus" (TL design items 44, 58; notify-sidecar-api.md:50-54) | Direct receiver→receiver HTTP when a peer is registered, hub topic only as fallback (direct.py:1-26; D-697, decisions.yaml:4876-4881). Cross-host direct is T-3688 (lifecycle.py:20). TermLink's charter objection applies once it lands (TL T-3330-receive-side.md:209) |
| 2 | Who injects | Injector stays in TermLink as a primitive (SQ-1, TL design item 46) | AEF's watcher decides and calls TermLink's `pty inject` (inject.py:327). TermLink supplies the typing primitive, which is consistent. But AEF owns target selection and readiness, contradicting SQ-1's "injector in TermLink" if read as a whole service |
| 3 | Prompt-free check | TL 18ca (:55): "independent of any LLM: it reads how the terminal behaves" (T-3079 screen quiescence classifier) | AEF rejects external inference. Readiness is self-reported by Stop/UserPromptSubmit hooks (T-3397:97-113; adapter.py:88-111) |
| 4 | Urgent safety | Operator: urgent bypasses; the route must "not be silently lost" (T-2396: text typed into a busy prompt can land unsubmitted) (TL 18c, 18e, O2) | AEF types one line plus Enter into the busy session (inject.py:327). It then requires transcript evidence and re-injects after 120 s (inject.py:46; hooks.py:14-24). The T-2396 loss mode is mitigated by detection, not prevented |
| 5 | Urgent ladder | Not yet addressed in TermLink's ladder (IN-AEF@180) | `urgent` compression of the ladder raises `NotImplementedError` by D-600 (retry_ladder.py:49-54) |
| 6 | Receipt calls | Operator: API calls back RECEIVED, STORED, INJECTED, ANSWER READY, each timestamped (TL items 21-25, 34) | RECEIVED (sync in the HTTP reply), HANDED_OVER (`/ack`), REPLIED. No separate STORED (TL O3) and no ANSWER READY call (reply is a new send) (direct.py:40-58). Mapping is undecided (TL T-3330-receive-side.md:223) |
| 7 | Call-back vs record-and-pull | Review panel (Codex, GLM): a synchronous call back to the sender is "wrong as written" and record-and-pull meets the need (TL T-3330-receive-side.md:242). The operator chose C (hub record) | AEF built exactly the synchronous receiver→sender `/ack` (http_server.py:110-122) and recommends a collector pull for telemetry (roundtrip doc:235-247) |
| 8 | Telemetry home | IW-1 = C amended: each sidecar records its own events and copies them to the hub through a local outbox; hub keeps them for a retention window; any agent pulls; a daily digest (IN-AEF@184 item 2) | Local files only, `fw sidecar latency`. No hub copy, no digest built. AEF answered RECEIVED, with its comparison "to follow" (IN-010@251) |
| 9 | Proposed split | TermLink: hub telemetry record, pull verb, digest, discovery, `pty inject`. AEF: protocol and receiver plus events copied to the hub (IN-AEF@184 item 4b) | Awaiting AEF's answer; its stated overlap is T-3684/T-3685 (IN-010@251) |
| 10 | Polling cadence | Operator's standard ladder 15 s … 1 year, each rung ×2 (IN-AEF@180; TL T-3330-receive-side.md:261) | Retry ladder 2×1m … 2×1mo (≈76 days), with escalation verbs (retry_ladder.py:62-84). T-3770 is an inception to reconcile (IN-010@228) |
| 11 | Project/hub address | Operator: project level is a minted UUID, not a folder; the hub needs a stable name (TL notes IN-AEF@129, @186) | circuit.py still uses the basename and the TLS fingerprint (circuit.py:99-153). The minted id exists (T-3534) but the sidecar does not use it. T-3751 is open |
| 12 | Address ruling wording | TermLink reads D-660 as `inbox:<agent-id>`/`dm:` adoption (IN-AEF@129) | D-660 and D-599 conflict in text. IN-010@53 says D-660 wording was a mistake, "being amended". Not verified amended |
| 13 | Prefix retirement | `sidecar:` read alias for "one release" (SC-010@2) | S7 T-3690 now says "needs your agreement" (IN-010@62), not a date |
| 14 | Identity trust | Per-agent keys are TermLink's to build. Reaffirmed as the fix (IN-AEF@141, SC-010@0) | AEF relies on the circuit id as name only (circuit.py:164-170). Metadata is unsigned. 832 argued the circuit id is forgeable (IN-AEF circuit-id-adoption @7) |
| 15 | Receipts from TermLink's notify rail | Its auto-confirm "receipted" mail nobody read (IN-010@59 D6; IN-AEF@137) | AEF's receipts are separate and evidence-based (receipts.py:1-36). A message from a peer on the hub topic gets a receipt only if the peer's receiver is registered, else a hub inbox post |
| 16 | Cross-host receiver | Cross-host API refused by charter | Planned (T-3688). Open and unreconciled |

### C3. Open questions TermLink raised that AEF's record does not yet answer

1. O1 the send path across hosts (TL design item 58).
2. O2 how the urgent bypass avoids silent loss (TL item 59).
3. O3 RECEIVED vs STORED as one call or two (TL item 60).
4. O4 how an already-running session without an injectable terminal becomes reachable, PL-237 (TL item 61).
5. 4c: "is your sidecar ready for a non-AEF-bleeding-edge consumer on framework 1.6.29?" (IN-AEF@184). AEF's own e2e note says TermLink is on v1.6.29, which has no `lib/sidecar/` (TL T-3330-receive-side.md:94). No answer found.

---

## D. STATUS CLAIMS vs REALITY (AEF side)

1. **"What runs today" vs what the architecture doc says.**
   1a. sidecar-target-architecture.md:5 says "This is the architecture we are building toward. It is not what runs today", and its §5 gap table (:132-146) lists transport, delivery model, flag side, tick, injection, CONFIRM-1/2 as "inverted/missing/wrong side".
   1b. The same file's register §7 (:181-287) lists R1-R11, R13-R15 as `built` with live evidence. §8 (:289-299) still lists tick cadence, readiness predicate, and HTTP-vs-socket as open.
   1c. Code contradicts §5 and §8: the receiver API, flag, 30 s watcher, hook-based readiness and HTTP transport all exist (receiver.py, watcher.py, hooks.py, http_server.py). The document is internally stale and was not updated after the build.
2. **"Urgency is deferred" (sidecar-target-architecture.md:124-126)** vs R5 `built` (:211-217) and the 2026-09-21 operator decision (T-3397:194-199). Urgent bypass is built. Urgent ladder compression is not (retry_ladder.py:49-54).
3. **R12 cross-host direct cross-post: `partial`.** Operator said cross-host "is not a question" on 2026-09-21 (T-3397:270-272) and the uniform-path rule forbids a same-host fast path plus exception (T-3396:136-144). Built reality is exactly that: loopback-only HTTP (http_server.py:135, 155) and a host-local registry (lifecycle.py:15-20). This is the largest gap between the design principle and the build.
4. **"Proven by a live two-agent nonce e2e (3/3) plus a negative control" (IN-010@62, hub).** The register's R2 evidence names it (sidecar-target-architecture.md:195). The same message set shows the receiver was not running for a day on AEF's own host: "our receiver was not running, and our hub-topic fallback sends no receipt" (IN-010@141). The e2e proves the mechanism in a controlled run, not that every real agent is reachable.
5. **"R14 every agent runs a sidecar / R15 always-on listener: built" (register :274-286).** The injector can only target a session registered in TermLink with the `fw-project` tag (or claude tag + cwd) (inject.py:130-145). An agent started plainly has no target. The injector's own message is "no TermLink session for this project… start the agent with claude-fw --termlink" (inject.py:222-224). So "built" holds for agents started through the wrapper, and TermLink's measurement is that no agent on ITS host was started injectable (TL design items 52-53, PL-237).
6. **Receipts "on every path" (T-3684/T-3685).** The code does give receipts on the hub-topic path (receipts.py:1-36). But a `fw sidecar inbox` drain sends no HANDED_OVER by design (receipts.py:14-16), so some paths report RECEIVED only.
7. **Address-ruling status.** The record holds two rulings that cannot both stand: D-599 `inbox:<circuit-id>` (4178-4183) and D-660 "inbox:<agent-id> … does NOT keep sidecar:" (4617-4622). IN-010@53 says the D-660 wording is being amended, but the amended text is not in decisions.yaml as read. Treat the amendment as unverified.
8. **"sidecar: alias for one release" vs reality.** T-3461 (09-25) found both address schemes live with independent offsets and no retirement date (T-3461:128-145). S7 T-3690 is not built and now needs peer agreement (IN-010@62).
9. **Hub id.** The architecture keys durable topics on a value that rotates on cert rotation (circuit.py:99-133; synthesis:36-38). The migration (dual-read, alias, retention through rotation) is open (synthesis:140-147).
10. **Escalation terminus.** T-3461 found the OPERATOR rung wrote to an unrendered queue (T-3461:82-113). Whether it was fixed is not verified from the files read.
11. **OBS-529.** `answered_conversations()` read only the circuit topic (roundtrip doc:109-116). Build-order slice 0 (sidecar-target-architecture.md:160-161) is its fix; I did not verify that it shipped.
12. **Registered module status vs the doc split.** `receiver.py` is labelled "arc-011 slice 1, T-3561" yet states IDs "per D-645" (receiver.py:1-22). Register R10, R11, R13 name `T-3561` as owner with `built` and evidence strings like "message schema", which are not test or run evidence (sidecar-target-architecture.md:246-258).
13. **Telemetry.** The `telemetry.jsonl` design is a design (roundtrip doc:186). I found no code in the files read that writes it. Ledgers exist, so "per-hop telemetry" is claimed only as designed.
14. **Latency reality.** Peer send→reply median 49 min, max 278 min on 2026-10-02 (brief:142-143). The 30 s tick is the pickup bound when the agent is idle, not the reply bound.
15. **Hub consistency.** Two hub stores existed on the host and the MCP server wrote to the wrong one (ARC@1647); AEF pinned `TERMLINK_RUNTIME_DIR` in `.mcp.json` (ARC@1648). The "MCP default" is homed to TermLink. This is a transport reality the sidecar status claims do not mention.

---

## E. Index of the operator's own words recorded on the AEF side

1. 2026-09-20 sidekick recollection and "Royal and the Correct Long Term Road" — T-3396:87-92, 104-105.
2. 2026-09-21 "We have multiple hosts, so that's not a question." — T-3397:270-272.
3. 2026-09-21 urgent = bypass, "a hard skip, not a shortened interval" — T-3397:194-199 (paraphrased by AEF; no verbatim).
4. 2026-09-25 build toward the design, "do not ratify the divergence" — D-645, decisions.yaml:4512-4517.
5. 2026-09-27 D-660 (ruling text, no verbatim) — decisions.yaml:4617-4622.
6. 2026-10-02 watcher design — watcher.py:3-6 (verbatim).
7. 2026-10-02 standing directive to stay in contact with TermLink and 055 — D-700, decisions.yaml:4897-4902.
8. 2026-10-03 "pretty quick" receipt and pickup within one 30 s tick when idle — brief:142-144 (paraphrase).
9. 832's operator, quoted by 832: "I don't understand what you asked me on the AEF side. That's why you've got Sidecar or Termlink right to collaborate with the AEF agent, intensively." — SC-AEF@19.
10. TermLink's operator, relayed to AEF: "that should be the standard fallback mechanism for the framework for any polling activities. It can be changed situationally, but that should be the standard." — IN-AEF@180.
11. TermLink's operator, in TermLink's own record: "after 20 times me telling you this is a really important part of this design, you don't have this quite essential part of bi-directional interactive communication" and "I'm missing that we use cronjobs to monitor if there is a flag. Every 30 seconds…" — TL:docs/reports/T-3335-interactive-communication-consult.md (dialogue log items 1, 5).
