# Interactive agent-to-agent communication: the design, front to end

**Owner:** the operator (design); TermLink and AEF (build) · **Task:** T-3335 · **Rebuilt:** 2026-10-04 (replaces the 2026-10-03 version)
**Method:** rebuilt from both history reports (`docs/reports/T-3335-design-history-termlink.md` = HT, nine rounds; `docs/reports/T-3335-design-history-aef.md` = HA, six rounds) plus the sources they do not cite (section F of HT, eleven further finds). Every element below names the round(s) it comes from, the latest operator position, and any contradiction. The previous version stays recoverable with `git show HEAD:docs/design/interactive-agent-communication.md`; its line numbers are cited here as IAC-v1.

## 0. How to read this document

1. **What this is.** One coherent *current* design, assembled from 22 recorded rounds (HT A1-A9, HA R1-R6, and HT section F: F1-F6 and F11 are new rounds, F7-F10 are additions to known rounds). It states the design. Section 5 states what actually operates, and the two must never be blurred: the 2026-10-03 failure was exactly that blur.
2. **Corrections to the previous version (IAC-v1).**
   2a. **§3a said the prompt-free check "reads how the terminal behaves, not what the agent says" (IAC-v1:55).** That is wrong for the current design. Readiness is REPORTED BY THE HARNESS (Stop hook sets ready, UserPromptSubmit clears it). See item 21.
   2b. **It carried no source for any element.** Each element here is cited.
   2c. **It lost elements stated once:** the native consumer, DEAF-means-halt, the reply-or-explicit-no-action obligation, the startup chain, the receptionist/persistent-session lifecycle, canary mail with deadlines, the stage-vocabulary mapping and the interrupt-consent question. They are restored in section 2 or listed in section 4.
   2d. **It said the polling ladder, the telemetry window and the open points O1-O6 without quoting both sides.** Section 6 does.
3. **Citation keys** (tables are exempt from the labelling rule):

| Key | File |
|---|---|
| HT / HA | `docs/reports/T-3335-design-history-termlink.md` / `…-aef.md` (HA cites the AEF checkout and hub messages; those are cited here "via HA:line", because the AEF checkout is outside this session's boundary) |
| CONSULT | `docs/reports/T-3335-interactive-communication-consult.md` |
| RAIL / SCAPI / ARC11 | `docs/design/arc-011-message-delivery-rail.md` / `…arc-011-sidecar-api-architecture.md` / `.context/arcs/arc-011.yaml` |
| T3330R | `docs/reports/T-3330-receive-side.md` |
| T243R, T1800R, T2291R, T2380R, T2396R, T2838R | the `docs/reports/T-243-…`, `T-1800-…`, `T-2291-…`, `T-2380-…`, `T-2396-…`, `T-2838-…` reports |
| DNS | `docs/operations/deterministic-notify-sidecar.md` |
| T-nnn:line | `.tasks/{completed,active}/T-nnn-*.md` or `docs/reports/T-nnn-*.md` (the report is named when ambiguous) |
| TL-A1…A9 / AEF-R1…R6 / F1…F11 | rounds in HT section A / HA section A / HT section F |

## 1. The goal, in the operator's own words

1. **The strongest statement (2026-10-03).** "I am absolutely very clear that I want to have this … after 20 times me telling you this is a really important part of this design, you don't have this quite essential part of bi-directional interactive communication." (CONSULT:13)
2. **What it is (2026-05-25, T-1800).** "interactive conversation between two or more agents… send-and-wait instead of immediate response… we want this interactive style of conversation to take place." Called "key functionality." (T1800R:6)
3. **Where it began (2026-03-23, T-256).** "the spawned agent can talk to the spawning agent" with no polling (docs/reports/T-256-interactive-multi-agent-comms.md:50-51). Earlier still, the operator wanted "a means to simulate keyboard input and capture console output" (T243R:96, dialogue dated 2026-04-26).
4. **That it never worked (2026-04-26).** "interactive multi turn conversation between two or more agents is absolutely not working" (T243R:100). Again 2026-07-07: "our mechanism is not working" and "we did extensive work on this, read back, was it arc 004?!" (T2380R:182-183), then "THIS IS REALLY BAD; WHAT NOW?" (T2380R:186, recorded as a quote).
5. **That it is critical (2026-05-31, T-1898).** "really bandaid fixing ??????!!! incept incept incept ,,, again fricking critical fucntionality" (docs/reports/T-1898-vendored-agent-runner-inception.md:29).
6. **The operator's recollection of the whole mechanism (2026-09-20, AEF).** "a sidekick that's listening all the time... it sets a flag... the cron job monitors that flag... looks for the active session... we could have a queue where the queue can be prioritized... a flag is raised that there are new messages... use PTY inject when the cursor is silent... with the exception of the urgent message that warrants an interruption." (T-3396:87-92, AEF, via HA:48)
7. **The scope (2026-08-24).** An interactive agent-to-agent medium, 1:1, N:N and N:operator, plus artifacts passed by reference: the orchestrator "should come back by writing those files and reporting back those files have been written" (T2838R:555-562); the assessment: "Partially certainly, but that interactive communication is still flaky at best." (T2838R:19)
8. **The standard of proof (2026-09-24).** "reliability and fragility, so I'm not to save a few tokens just to get a quick solution. I want a solid solution." (SQ-8, ARC11:360-361)
9. **The demand to stop forgetting (2026-10-03).** "off with the design we have envisioned, in detail, front to end, because you obviously keep forgetting that." (CONSULT:15) And: "You've been telling me it works …" (CONSULT:14)
10. **Goal statement derived from the above (not a quote).**
    10a. Two running agents hold a two-way conversation. A message sent while the receiver is working or idle reaches the receiver's context, and the answer returns the same way.
    10b. The sender always knows where its message is: every step is confirmed back with a timestamp, and a message that nobody read is a detected, surfaced event, never silence.
    10c. No LLM and no hub has to be up for delivery to proceed; the sidecar carries it.
    10d. Telemetry, addressing, ladders and alarms exist to serve this, they are not the goal.

## 2. The design, front to end

Reading rule for every element: **a** = the design; **b** = the rounds it comes from; **c** = the latest operator position; **d** = contradictions and which side is current.

### 2.1 Who talks, and the parts

11. **Identity and addressing.**
    11a. Five levels: host / hub / project / session / agent. A **circuit** is a durable channel to an agent; the message carries `metadata.to_circuit` (path form now, AEF's V9 form `aef::host=…::hub=…::project=…::session=…::@agent::` read as well). Every level has TWO identities: host = FQDN (IP is the instance, never in an address); hub = a canonical NAME plus an instance id (today's hub id is a TLS fingerprint that rotates); project = the minted `pid-<16 hex>` (the readable name is display only); session = canonical id plus runtime id; agent = a ROLE plus the running instance. Matching is level by level from level 1, a differing named level means "not for me", and it **never falls back across projects**. A message with no address is for everyone.
    11b. Rounds: TL-A3 (per-agent keys, stable agent id; T2291R:239-243), TL-A5 (T-2384 per-agent fingerprint; T2380R:150-160), TL-A6 (per-agent identity as prerequisite; T2838R item 1), AEF-R2 2026-09-06 (taxonomy D1-D7, V9 grammar; T-3287:28-107, via HA:55-67), AEF-R4 2026-09-22 (D-599 `inbox:<circuit-id>`; SC-010@2, via HA:101-106), AEF-R5 (2026-09-25 five-part V9 ruling IN-010@39; 2026-09-27 D-660), AEF-R6 2026-10-03 (minted `pid`; two-identity model, 7/7 adopt-with-changes; via HA:147-149), TL-A9 (T-3325 Q1 = C, PD-186; T-3325:282-309; IAC-v1:19-31).
    11c. Latest: 2026-10-03, canonical plus instance identity per level (relayed IN-AEF@186, via HA:149); never fall back across projects (agreed by both sides, IN-010@141, via HA:245).
    11d. Contradiction. D-599 says "`inbox:<circuit-id>`" ("We take your prefix; what follows it is our five-level circuit id." SC-010@2, HA:102); D-660 says "AEF adopts 010-termlink's addressing — dm:<fp>:<fp> / inbox:<agent-id> — and does NOT keep sidecar:<agent-id>" (decisions.yaml:4617-4622, HA:124). **Current: the circuit model.** AEF's own correction, IN-010@53: "The five-level circuit model stands … 'inbox:<agent-id>' wording was a mistake on our side and is being amended" (HA:126); the amended text is unverified (O10). Trust gap: every agent on this host signs with one host key, so a receipt cannot tell agents apart (T-3280:273-; HA:100); per-agent keys are not built (HA:228). Open: session level (O6), directory (O7).

12. **The sidecar.**
    12a. Per receiving or sending agent: (i) a **separate, very simple process with an API**; (ii) **independent of the hub, because the hub goes down**; (iii) **always respawns**; (iv) carries the **startup chain: start agent, start session, start project, start hub**; (v) **re-resolves when an FQDN or IP stops resolving**. Operator wording: "a separate process exposing an API, deliberately independent of the hub because the hub goes down. Very simple. Always respawns. It also carries the startup chain — start agent, start session, start project, start hub — and re-resolves when an FQDN or IP stops resolving." (RAIL:111-117, "Operator direction, verbatim in intent"; SCAPI:10-13)
    12b. Rounds: TL-A3 (ears without an API: notify-sidecar, flag plus heartbeat; DNS:33-36), AEF-R1 (the 2026-04-12 receptionist and arc-011 §5 flag sidecar; T-3396:29-59, HA:41-47), TL-A7 (the API; RAIL:111-117), F4 (receptionist, TermLink side), TL-A9 (sidecar-to-sidecar API; T3330R:248-256), AEF-R3 and AEF-R6 (per-agent receiver with HTTP API on 127.0.0.1, bearer token, triple files; http_server.py:13-17,135-143, via HA:162-165).
    12c. Latest: 2026-10-03, "sender agent → its own sidecar (API) → the receiver's sidecar (API)" (T3330R:248-256).
    12d. Status of each part: separate process with API **built** on both sides (TermLink: local-only `notify-sidecar-api.sh`, five verbs, remote arguments refused, `docs/operations/notify-sidecar-api.md:20-28,70-78`; AEF: receiver); independent of hub **built** in AEF's direct path, in TermLink only as local control; always respawns **built** (portable `--loop` and `--emit-unit systemd|launchd|cron`, SQ-8, ARC11:350-367; T-3050 cron launcher 2026-09-21; AEF supervisor and `sidecar-ensure-1m` cron plus `@reboot`, watcher.py:527-602,679-704, via HA:170); FQDN/IP re-resolve **partial** (re-read and compare per call, `scripts/notify-sidecar-api.sh:197-212`); startup chain **never designed beyond the spec line, not built** (HT:187, D4).
    12e. Contradiction: SQ-1 (2026-09-23) reads the sidecar API as LOCAL control only, which conflicts with the spec's "sender SENDS via a sidecar API" (ARC11:28-44; T-3135 Evolution). See O1.

### 2.2 The message, front to end

13. **Send.**
    13a. The sending agent hands the message (with an optional blob) to its own sidecar; that sidecar delivers it to the receiver's sidecar API. Cross-host and same-host are ONE path, never a fast path plus an exception (T-3396:136-144, via HA:85). Sender states: SENT, RECEIVED, HANDED_OVER, REPLIED, UNDELIVERABLE, REJECTED, ESCALATED; the effective state is the highest-ranked row (direct.py:1-26,40-58, via HA:184). No registered receiver: the hub topic is the fallback; registered but unreachable: retry budget, then UNDELIVERABLE (D-697, via HA:146).
    13b. Rounds: TL-A7 spec step 1 (RAIL:33-61), AEF-R3 (T-3397:80-132), TL-A9 (T3330R:248), F5 (T-1425 `agent contact`, a per-pair dm topic), F6 (`agent-send.sh` doorbell+mail).
    13c. Latest: 2026-10-03 (as 12c) and 2026-09-21 "We have multiple hosts, so that's not a question." (T-3397:270-272, via HA:83).
    13d. Contradiction: SQ-1 keeps sending on the hub (`channel.post`); the charter forbids a second cross-host bus (SCAPI:17-35,164-167). **Not resolved: O1.**

14. **RECEIVED.**
    14a. The receiver's sidecar calls the sender's sidecar immediately: "received", timestamped; it means the bytes are in the peer's local store and survive its restart (L2, `stage=delivered`; RAIL:71). AEF answers it synchronously in the HTTP reply (http_server.py, HA:183).
    14b. Rounds: TL-A3 (L2 receipt; T2291R:307-316, V6:200-217), TL-A7 (RAIL:65-85), F7 (T-2295: a receipt is written on READ, not on detect, because an ack on detection erases the very flag that wakes the agent; T-2295:158), AEF-R3 (T-3397:118-125), TL-A9.
    14c. Latest: 2026-10-03 "an API call that's received" (T3330R:229, dialogue 1).
    14d. Contradiction: a synchronous call back is "wrong as written" for Codex and GLM, who prefer record-and-pull (T3330R:242, via HA:267); the operator chose C (hub record). See O12. Note that an L2 receipt posted by the sidecar's `--auto-confirm` (the three sidecars running today carry it) says only "stored", never "seen".

15. **STORED.**
    15a. The receiver stores the message durably, then calls the sender's sidecar again: "stored". AEF: temp file, rename, then ready flag (D-690, via HA:141); its RECEIVED already means stored.
    15b. Rounds: TL-A9 only as a separate call (T3330R:248-256 item 3); AEF-R6 collapses it.
    15c. Latest: 2026-10-03 (separate call). d. Open: O3.

16. **The new-message flag.**
    16a. A flag is set when a message is stored; an arrival record survives the ack; the flag is a dirty-bit and the message is durable before it is set. Its pending count must not be shared with an auto-ack (HT:193).
    16b. Rounds: TL-A3 (flag plus heartbeat, `~/.termlink/notify/`, PD-053), TL-A7 (S4, ARC11:70-80), AEF-R3 (dirty-bit, T-3397:80-89), TL-A9.
    16c. Latest: 2026-10-03 (T3330R:248-256 item 4).
    16d. Shift: in TL-A3 the AGENT polled the flag at its yield points (DNS:48-55); in TL-A9 a CRON tick reads it and injects. Both are kept: see items 17 and 27.

17. **The 30-second tick.**
    17a. A cron-style job checks the flag every 30 seconds. System cron's unit is one minute, so "every 30 seconds" is a supervised loop (or two staggered entries): an implementation detail, not a design change (IAC-v1:52; AEF built it as a supervised loop, watcher.py:527-602, via HA:170). With the flag up it reads the queue (item 18).
    17b. Rounds: TL-A7 step 11 "the cron drives 7 to 10" (RAIL:60; installed as `*/5`), AEF-R3 (tick configurable, T-3397:91-96; "15 s proposed, never validated", HA:208), AEF-R6 (watcher, 2026-10-02), TL-A9.
    17c. Latest, TermLink 2026-10-03: "I'm missing that we use cronjobs to monitor if there is a flag. Every 30 seconds … when the flag is up, the message queue gets read … urgent gets injected immediately. Non-urgent, we check again if the prompt is free. If the prompt is not free, we wait again until the next 30 seconds." (CONSULT:17) And AEF's operator 2026-10-02: "a watcher … watching if the message flag is up every 30 seconds and when it's up it looks if it is an urgent message and injects directly, or if it's not urgent it looks if the prompt is free and if it's free it injects the message and informs the sender." (watcher.py:3-6, via HA:139)
    17d. No contradiction on the cadence: both sides say 30 seconds. The `*/5` driver still installed on this host is the earlier S11 and checks no prompt (IAC-v1:124).

18. **The queue, with priority.**
    18a. A queue ordered by priority then arrival; the highest priority is read first. Built as `COALESCE(priority,0) DESC, ts ASC, offset ASC`, priority clamped to [-9,9] (ARC11:111-132).
    18b. Rounds: TL-A7 S8 (RAIL:149-155), TL-A9.
    18c. Latest: "read the queue, inject the highest-priority message." and "keep it flat for now, prioritise later" (T-3071:79); the 2026-10-03 statement reads the queue by priority with urgent first (CONSULT:17).
    18d. No conflict: "flat for now" was a staging order, the priority field now exists.

19. **Urgent: injected immediately, a hard bypass.**
    19a. An urgent message skips the prompt-ready check and is injected at once, even into a busy agent. It is delivered by a route that **cannot silently lose the message**. The route (the requirement, then the mechanism recorded so far):
        19aa. The message is durable in the receiver's store BEFORE any inject, and the flag stays up until it is handed over. A discarded keystroke therefore never discards the message.
        19ab. The injected line is ONE fixed line carrying a count and ids, no peer content (inject.py:3-13,251-261,327, via HA:175). Same principle as the 2026-05-25 doorbell, which "never carries content" (T1800R:50-64).
        19ac. The prompt hook (UserPromptSubmit) surfaces the stored message on the agent's next turn whatever happened to the typed line (hooks.py:101-131, via HA:178).
        19ad. HANDED_OVER is claimed only on transcript evidence (item 23); with none after 90 s the message is eligible again and an inject with no hand-over is retried after 120 s (hooks.py:14-24; inject.py:46, via HA:190,201); after the deadline (default 900 s) it is ESCALATED (direct.py:60,291-299, via HA:189).
        19ae. An urgent message raises an immediate alarm (item 30).
    19b. Rounds: TL-A7 (SCAPI:207-208; SQ-4 ARC11:305-316; T-3072), AEF-R3 (T-3397:194-199), AEF-R6 (R5 "hard bypass"; inject.py:213-248, via HA:199), TL-A9 (CONSULT:16-17).
    19c. The four positions, in date order:
        19ca. 2026-09-21 (AEF operator): a hard skip, "a hard skip, not a shortened interval" (T-3397:194-199, AEF paraphrase of the operator decision, via HA:76).
        19cb. 2026-09-22/23 (TermLink operator): "Urgent bypasses the wait. Reliability is important at all times but can also be out of band." (SCAPI:207-208).
        19cc. 2026-09-23, **SQ-4**: "urgent NEVER injects into a BUSY prompt … Urgent shortens the WAIT; it does not bypass the prompt-free CHECK." (ARC11:305-316). Built as a bounded re-probe of up to 120 s (T-3072:87-96; the build's own note: "the task title says 'bypass', and the operator's answer says do not", T-3072:259).
        19cd. 2026-10-03 (TermLink operator): "I'm completely missing the point about that we observe whether the prompt is free or not, and inject when it's free and with urgent bypass." and "urgent gets injected immediately." (CONSULT:16-17; T3330R:256)
    19d. **Current: the bypass.** The latest operator words (2026-10-03) agree with 19ca and the 2026-10-02 watcher words (item 17c) and supersede SQ-4 (2026-09-23), which was recorded before them and read "bypass" as "shorten". The T-2396 loss mode (text typed into a busy prompt can land unsubmitted and be discarded on the next `--continue`) is a constraint on the route, not a reason to drop the bypass; the route in 19a mitigates it by **detection and durability, not prevention** (HA:264). That is the claim made here and no stronger one. Confirmation requested: O2. Still unbuilt: compressing the retry ladder for urgent (`retry_ladder.py:49-54` raises `NotImplementedError`, D-600).

20. **Non-urgent: inject when the prompt is free, else wait for the next tick.**
    20a. At the tick the sidecar checks readiness (item 21). READY: inject now. Not ready: the message stays queued, the flag stays up, and it is checked again at the next tick. The outcome set is READY, BUSY, NOT RUNNING; NOT RUNNING is reported, never treated as busy, and an unknown reading never resolves to READY. AEF records an `INJECT_BLOCKED` event once per reason (inject.py:218-226,276-283, via HA:198).
    20b. Rounds: TL-A5 (idle-gated injection, T-2402 Stage 3; `pushwaker-idle-gating.md:11-26`), TL-A7 (S7, S10; ARC11:100-110,148-170), F8 (T-2410 sender-side idle gate), AEF-R3/R6.
    20c. Latest: "I want to inject, but I check if my agent is running, I see it's not running." (T-3069:92) and 2026-10-03 (CONSULT:17).
    20d. No conflict about the behaviour; the source of the readiness signal is item 21.

21. **Readiness is reported by the harness, not inferred from the terminal.**
    21a. The harness's own hooks report it. The **Stop** hook sets `ready-for-input: true` at the end of a turn. The **UserPromptSubmit** hook clears it BEFORE the next turn begins, with zero tolerance for lag. A missing or unreadable flag reads as NOT ready ("the safe direction", adapter.py:88-111, via HA:197). Readiness is per SESSION, not per project, after two Claude sessions in one project shared one flag (T-3745, via HA:147,173).
    21b. Why not the terminal: "Timestamp-staleness and CPU/PID heuristics were rejected because a long Bash tool call looks idle but is unsafe" (T-3397:97-113, via HA:75); arc-011 §5 had already rejected injection into an apparently idle terminal because "an agent mid-turn is single-threaded, and no outside observer can prove a safe point" (T-3396:40-59, via HA:46).
    21c. Rounds and endorsement: AEF-R1 (arc-011 §5), AEF-R3 **2026-09-21 T-3397:97-113** (the decision), **TermLink endorsed it the same day, ARC@1640 (T-3062): "What is MISSING: the hand-to-agent step… Your (b) — a ready-for-input flag written by the harness's own Stop/UserPromptSubmit hooks — is precisely the consumer we do not have. Do not build ears or receipts; do build (b)."** (via HA:99), AEF-R6 (built: hooks.py, adapter.py, D-694; via HA:143,243), F2 (the Stop and UserPromptSubmit events were known since 2026-03-18, T-099 report:21-37; a Stop hook is wired in this repo, but for governance, not readiness, T-173:33).
    21d. **Correction of IAC-v1 §3a** (IAC-v1:55, "it reads how the terminal behaves … T-3079's classifier decides on screen quiescence"). The screen classifier (T-3079, 466 samples, 0 false-ready; ARC11:100-110) is TermLink's earlier mechanism (TL-A5 and TL-A7). It is not the design. It may stay as a fallback for a harness without hooks (O5). Honest limit: the operator's own words never name a mechanism. 2026-09-20 "use PTY inject when the cursor is silent" (T-3396:87-92) and 2026-10-03 "observe whether the prompt is free" are both compatible with either; AEF surfaced that divergence at the time (T-3396:94-102, via HA:49) and the operator then ruled for the long-term road: "Let's go to the Royal and the Correct Long Term Road. Let's pack it full out." (T-3396:104-105, via HA:50). Ratification of this reading: O5.

22. **Injection.**
    22a. The sidecar types ONE fixed line with `termlink pty inject <session> "<line>" --enter`. The line is a doorbell (a count and ids). The content reaches the agent through the prompt hook, framed `<<<PEER-DATA … PEER-DATA>>>` as UNTRUSTED: "a request for action becomes a task proposal… never direct execution" (D-693, hooks.py:101-131, via HA:144,178). The target is a session whose OWN record says ready, registered for this project, with a live claude process, never a headless worker; a claim naming the target is written before typing (inject.py:15-27,130-145,312-324, via HA:176-177).
    22b. Rounds: TL-A2 (doorbell+mail, `command.inject` fixed nudge; T1800R:50-64; the respond-mode signal `/check-arc respond`, T-1809), TL-A4 (push-waker; push-transport.yaml:25-40), TL-A5 (T-2396 unsubmitted text, G-083), TL-A7 (S10), AEF-R6.
    22c. Latest: 2026-10-03 "an urgent message is injected into context at once; otherwise once the prompt is free" (T3330R:256).
    22d. Precondition recorded in every round since TL-A4: a TermLink-owned PTY. PL-237: dormant without a bound `pty_session`; it "cannot be retrofitted onto a running headless claude" (`scripts/notify-injector.sh:20-24`). The operator has recorded no position on how a running session becomes injectable: O4.

23. **INJECTED / HANDED_OVER only on evidence the agent saw it.**
    23a. After injecting, the sidecar reports INJECTED only on observed evidence. The strongest recorded form: HANDED_OVER after the session **transcript** contains the hook's `additional_context` attachment carrying the message under a one-time surfacing token, waiting up to 90 s; otherwise `HANDOVER_UNCONFIRMED` (hooks.py:14-24, D-696, via HA:190). TermLink's rule: no evidence kind means "I sent it and hoped"; `notify-ack-read.sh` requires `--evidence <kind>` (T-3067 AC2; RAIL:74-84); `evidence=wake-consumer` was withdrawn as untruthful (RAIL:81-84). The prover reads the receiver's transcript, never the sender's report (T-2876:43).
    23b. Rounds: TL-A3 (L3 read receipt; V6:200-217), TL-A5 (PL-253: "a heartbeat proves the PROCESS is alive, not that the SESSION is listening", T2396R:14-24), TL-A6 (A receipt cannot exist unless a turn happened; T2838R:175-208), TL-A7 (RAIL:65-85), F9 (T-3067), AEF-R6.
    23c. Latest: 2026-10-03 "when it's injected we make an API call that's injected … those timestamps we store" (T3330R:229).
    23d. Not decided anywhere: how TermLink's INJECTED (a BUSY transition) maps onto AEF's HANDED_OVER (transcript evidence) (HT:305, D9; T3330R:222). Open: O16. The rule that stands for both: nothing is "delivered" or "injected" without observed evidence from the receiver's side.

24. **Flag down only when the queue is empty.** The new-message flag is cleared when every queued message is handed over, not when one is (T3330R:248-256 item 7). Rounds: TL-A9 only. AEF's record does not describe it (HA gap).

25. **ANSWER READY, the reply, and roles swap.**
    25a. When the answer is ready the receiver's sidecar tells the sender's sidecar "answer ready"; the sender pulls it; the answer travels the same path in reverse and steps 13-25 apply with the roles swapped. The same call serves both directions: "reply is the same call with sender and target swapped" (T-3397:127-132, via HA:78). AEF credits REPLIED when a message answering a sent one is stored (`note_reply`, direct.py:258-286, via HA:188).
    25b. Rounds: TL-A1 (reply as a turn), TL-A7 (S12; blocked by the `framework-agent-systemd` allowlist, ARC11:183-202), TL-A9 (T3330R:248-256 item 8), AEF-R3.
    25c. Latest: 2026-10-03 "once we've got an answer we send it back … also timestamps … also an API call." (T3330R:229)
    25d. Status: in TermLink the reply goes through `channel.post` (`agent-respond.sh`, `/reply`), not the sidecar API (SQ-1, ARC11:183-202). Measured: peer send to reply median 49 min, max 278 min on 2026-10-02 (T-3751-review-brief:142-143, via HA:306); the 30 s tick bounds pickup when the agent is idle, not the reply.

26. **The obligation: reply, or an explicit "no action".**
    26a. A woken or injected agent drains ALL unread topics, posts a receipt per topic, and for each either replies OR posts an explicit "acknowledged, no action needed" (or a declared "working on it" / "blocked"). "Silence is never a valid choice": so silence always means a bug, caught by escalation, never a valid unlogged outcome (T-2402:60).
    26b. Rounds: TL-A5 (T-2402 Stage 6, 2026-07-11; Stage 5 escalating re-ring, T-2402:59), F8. Not carried into arc-011, AEF or IAC-v1 (HT:299, D3).
    26c. Latest operator position: none recorded in words; the GO on T-2402 was the operator's (owner: human).
    26d. Gap to close: neither sender state set has a terminal for "acknowledged, no action" (AEF's seven states have none; HA:184). Proposed extension, for ruling: O16.

27. **DEAF means halt.**
    27a. The sidecar publishes a fresh heartbeat with the flag; when the heartbeat is stale the agent "can no longer trust 'no flag = no mail,' so it halts" its message-dependent work and says so. Verdicts MAIL (10) / DEAF (3) / CLEAR (0) (DNS:25-29,33-55; `notify-check.sh`).
    27b. Rounds: TL-A3 only (T2291R:251-267). AEF has the sensor, not the halt: `liveness.yaml` from a sequence counter plus a loopback self-probe, "not-live = seq stalled 2 ticks OR the probe failed", read by `fw doctor` and `fw audit` (watcher.py:366-407, via HA:169).
    27c. Latest operator position: none after 2026-06-27. 
    27d. Carried forward as design (the hook that surfaces messages also reads the liveness record and reports DEAF); built in neither implementation (HT:298, D2).

28. **The polling fallback ladder.**
    28a. Push is primary. Where a push cannot land, the other side polls on the standard ladder, the framework default for ANY polling, changeable per situation. Each rung is polled twice: 15 s, 1 min, 5 min, 15 min, 1 h, 4 h, 1 d, 3 d, 1 week, 1 month, 1 quarter, 1 year (T3330R:257-261, item 10). Dictation said "50" for the first and fourth rung, read as 15, pending correction (HT:310, D14).
    28b. Rounds: AEF-R4 (a retry ladder 2×1m, 2×5m, 2×15m, 2×1h, 2×4h, 2×1d, 2×1w, 2×1mo, then dead-letter; D-600, via HA:105), TL-A9 (T3330R:261).
    28c. Latest, relayed 2026-10-03: "that should be the standard fallback mechanism for the framework for any polling activities. It can be changed situationally, but that should be the standard." (IN-AEF@180, via HA:322)
    28d. Contradiction: the AEF ladder is a retry ladder of about 76 days, deliberately longer than the hub's ~5 min dedupe TTL, escalating (never re-posting) posted-but-unread messages (retry_ladder.py:16-27,39-47,62-84, via HA:206-207); the operator's is a polling ladder from 15 s to a year. AEF filed T-3770 to reconcile (via HA:152). Open: O9.

29. **Blobs.**
    29a. A message may carry a binary blob. It uses `termlink artifact put/get` (content-addressed by sha256; `--expected-sha256` mandatory; SQ-3, ARC11:45-69,247-259) and the receiver verifies the hash before the flag is set (AEF D-645, via HA:216). Artifacts travel by reference: the typed `assignment.v0` and `result_manifest.v0` envelopes exist as helpers, with "no CLI verb emits or consumes these yet" (T2838R item 3).
    29b. Rounds: TL-A6 (T2838R:68-80,555-562), TL-A7 (SCAPI:176-198), AEF-R5 (D-645), TL-A9.
    29c. Latest: 2026-09-24 SQ-3 (add the verbs). d. Not built: AEF's receiver-side blob slice (T-3686); a wired manifest verb (HT:302, D6).

30. **Telemetry, daily digest, alarms, escalation.**
    30a. Every step is a timestamped event (RECEIVED, STORED, INJECTED, ANSWER READY, plus SENT and failures) carrying message id, step, time, from, to. Each sidecar records its own and copies them to the hub through a local outbox, never in the message's path; the hub keeps them for a retention window; any agent can pull one message's journey; each agent posts a **daily digest** (counts per step, delays, stuck and unanswered messages); agents reflect on it. **Alarms: immediate only for urgent messages;** everything else accumulates and escalates "like audit warnings". Later: an observability database (the AEF agent, a hub, or an agent) where learning from traffic happens; the operator proposes a hub steward agent (T-3333).
    30b. Rounds: TL-A9 only for the design (T3330R:229-231,262-266; T-3319, T-3333); AEF-R6 negotiates it (IN-AEF@141,184; AEF's own per-hop `telemetry.jsonl` is designed, not verified built, HA:233,305); F11 (T-3304: mail retention by count, 14-day default).
    30c. Latest (2026-10-03): "as a standard, also for all vendor agents, to collect the telemetry of our communications … the different steps, how much time it takes and how much delay … you need to be able to pull that information from different agents … one times per day … And you infer or reflect on what working with that information means." (T3330R:231, dialogue 3)
    30d. Contradictions and gaps: (i) retention "for example 30 days" (IAC-v1:90) versus the 14-day default ruled at T-3304 IW-2 (T-3304:158-159): O13. (ii) The external review's R6 "liveness invariant" with canary mail and deadlines (INJECTED within T1, REPLIED within T2; T3330R:237-246) is not in the design: lost (HT:301, D5). (iii) The last escalation rung must land where someone reads it: T-3461 found AEF's OPERATOR rung wrote to an unrendered queue (T-3461:82-113, via HA:117); how alarms and escalations surface is open: O15.

31. **The native consumer.**
    31a. For agents we launch, the agent acknowledges from INSIDE its own turn loop, so "LIVE != listening" becomes impossible by construction; PTY injection stays for sessions we do not control and "never reports success without a confirmed read-cursor advance" (T2838R:92-102,414-420). Spike: "A receipt cannot exist unless a turn happened." (T2838R:175-208)
    31b. Rounds: TL-A6 only (GO 2026-08-25, T-2838:206-228); absent from arc-011 and IAC-v1 (HT:297, D1).
    31c. Latest: no operator words after GO.
    31d. *Inference, not an operator statement:* AEF's UserPromptSubmit hook plus transcript-proven HANDED_OVER (items 21, 23) is this idea realised for Claude Code, because the hook runs inside the turn loop. If the operator confirms, the native consumer is not lost but implemented on AEF's side: O11.

32. **Deployment: sidecars ship with every deployment.**
    32a. Every sidecar ships with every TermLink deployment, and starts from the deployment, not from a checkout: "are all the sidecars we develop … part of our [TermLink] package? When the deployment is done we should deploy all the sidecars with it." (T3330R:230, dialogue 2, 2026-10-03)
    32b. Rounds: TL-A9 only. Evidence of the gap: releases ship only the binary, ~13 scripts run only from `/opt/termlink`, and the CLI has no `sidecar` subcommand (T3330R:139-143, 3g2). AEF's side: `fw sidecar ensure --all` from cron plus `@reboot` (watcher.py:679-704, via HA:170).
    32c. Status: not done (section 5).

33. **Lifecycle: persistent sessions and the receptionist.**
    33a. A persistent, discoverable per-project session ("receptionist") that stays alive and is exempt from cleanup: KV `persistent=true`, tag `role:receptionist`, a health check at `fw context init`, manual respawn, **auto-respawn only by explicit opt-in** (T-967:120-125); "a lightweight receptionist that only starts full Claude when a request arrives" (T-1135 response:74,88). Persistent-session doorbell+mail is PRIMARY and a `claude -p` daemon is demoted: "aim not to use claude -p for expensive jobs" (T-1800:135, via HT:51). Inbound mail never respawns an agent without an explicit grant, budget, restart limit and authenticated sender (7/7 vendors, T-3751-review-synthesis:42-58, via HA:149).
    33b. Rounds: F3 (T-233 hybrid persistence, March 2026), F4 (T-967 GO 2026-04-12), AEF-R1 (receptionist, "never built", T-3396:29-38), TL-A2 (T-1800), F6 (T-1898 vendored agent runner, 2026-05-31: "no service holds an attached claude-code, reads dm:<self>:*, and replies"), AEF-R6 (7-vendor review).
    33c. Latest operator position: GO 2026-04-12, "User approved: persistent agent sessions with KV persistent=true, tag role:receptionist, .framework.yaml config" (T-967:132); nothing newer in words. The sidecar's "always respawns" (12a) is a statement about the sidecar, not about the agent. Open: O14.

34. **The loud contract around every send, and the sender's ledger.**
    34a. Around every send the sender verifies each link and returns a structured result (`delivered, recipient_live, recipient_agent_backed, waker_running, hub_targeted, hub_read_healthy, acked`) that fails fast on the first broken link (T2380R:137-165). The deterministic-attention loop has six stages: 1 durable obligation, 2 push wake, 3 idle-gated injection, 4 receipt-or-re-ring, 5 escalate-if-stuck, 6 wake-protocol obligation; "You cannot make LLM cognition deterministic — so this task makes the ENVELOPE around it deterministic" (T-2402:33,50). The sender keeps a ledger row per message with the step reached; a rung advances only on a receipt read back (S6, `scripts/notify-ledger.sh`, ARC11:87-99). Three confirmation signals exist (envelope, frontier, reply-turn); the conversational paths standardise on the envelope (T-2295:216-).
    34b. Rounds: TL-A3 (confirm ladder L1/L2/L3), F7 (T-2295), TL-A5, F8 (T-2400 mute reachable agents, T-2410), TL-A7 (S6).
    34c. Latest operator position: GO 2026-07-09 on T-2380 (T2380R:137). d. T-2385 (hub-read-health fail-fast) and T-2386 (reply on sender hub) are shipped but not tied to the sidecar send path (HT:307, D11).

## 3. The whole path in one picture

35. **Sequence (current design).**
    35a. Sender agent → its sidecar → receiver's sidecar API (13).
    35b. Receiver: RECEIVED to the sender (14), STORED to the sender (15), flag up (16).
    35c. Every 30 s the receiver's sidecar checks the flag (17) and reads the queue by priority (18).
    35d. Urgent: inject at once, route per 19. Not urgent: ready (reported by the harness, 21)? inject (22). Not ready: wait for the next tick (20).
    35e. Prompt hook surfaces the message; transcript evidence; INJECTED/HANDED_OVER to the sender (23). Flag down when the queue is empty (24).
    35f. Agent works; replies or declares "no action" (26); ANSWER READY (25); roles swap.
    35g. Any push that cannot land falls back to the polling ladder (28); a stale heartbeat means DEAF and the agent halts (27); every step is a telemetry event, summarised daily (30).

## 4. What is lost or never built

Each item: stated once, not carried forward, or recorded but not built. Citations point to where it was stated.

36. **Lost: the native consumer** (TL-A6). T2838R:92-102,175-208; HT:297 (D1). Realised in part by AEF hooks (inference, 31d).
37. **Lost: DEAF means halt** (TL-A3). DNS:25-29,48-55; HT:298 (D2). No agent-side halt exists in either implementation.
38. **Lost: the reply-or-explicit-no-action obligation as a state** (TL-A5). T-2402:60; HT:299 (D3). Lives only in the `/check-arc respond` skill text; no sender state for it.
39. **Never built: the startup chain** (start agent, session, project, hub) and the full FQDN/IP re-resolve (TL-A7). RAIL:111-117; HT:300 (D4).
40. **Lost: canary mail with deadlines, the R6 liveness invariant** (external review, 2026-10-03). T3330R:237-246; HT:301 (D5).
41. **Never wired: typed `assignment.v0` / `result_manifest.v0`** (TL-A6). T2838R item 3; HT:302 (D6). **Never built: hub-side enforcement of consumption** (T2838R:422-427; HT:303, D7).
42. **Unruled: per-agent readiness emitted by the agent** (T-3001, T-3250, `status: captured`). `.tasks/active/T-3250-design-who-observes-agent-side-readiness.md:6-8`; HT:304 (D8). AEF's hooks answer it in practice; no TermLink ruling.
43. **Undecided: the stage vocabulary mapping** TermLink INJECTED versus AEF HANDED_OVER (T3330R:222; HT:305, D9).
44. **Returned unanswered: interrupt consent**, who owns a session's attention. Dissolved by SQ-4, which the operator's 2026-10-03 words then reversed (T-3200R:95; HT:306, D10).
45. **Shipped but not tied to the sidecar: T-2385 hub-read-health, T-2386 reply-on-sender-hub** (HT:307, D11). **Tasks without links: T-3321 message compaction, T-3319 learning from traffic** (HT:308, D12).
46. **Absent from every later round: the same-host loopback direct path** (depends on T-2024, deferred; V6:251-253) and **Tier-1 LAN broadcast discovery** (V6:254-256) (HT:309, D13).
47. **Never built: the receptionist / persistent per-project session** (AEF-R1, T-3396:29-38, T-1135; F4). Only the `agents/dispatch/yield-point.sh` stub shipped (T-3396:57-59).
48. **Not built, AEF side:** urgent compression of the retry ladder (`retry_ladder.py:49-54`); cross-host receiver (T-3688, `lifecycle.py:20`); blobs through the receiver (T-3686); retiring `sidecar:<agent>` (T-3690, "needs your agreement"); stable hub name (`circuit.py:99-133`); `project=` from the minted id (T-3751 open); `telemetry.jsonl` per-hop record (HA:305); receipt paths other than the hub-topic one (receipts.py:14-16, HA:298). Verification of the amended D-660 text, of the OPERATOR-rung fix (T-3461) and of OBS-529 is missing (HA:299,302,303).
49. **Retired by charter pruning, not by decision about comms: the activity layer ("typing/processing signal is the killer feature", T243R:116-120).** The MCP typing tools were pruned in P4 (T-2471/T-2478, per `CLAUDE.md`). The harness-reported ready flag is the narrow replacement. *This pairing is an inference.*
50. **Stated and never built: `emit-to`** (T-256 Option A; Option B, collect-based fan-in, shipped as a convention; T-256 report:61-76) and **`agent contact --require-online`** (T-1425 Q3, deferred; T-1425:95-99,193).
51. **A hook-to-agent channel that was never reused for comms:** "Stderr from a Stop hook becomes additional context the agent sees on the next turn" (T-1207 report:50). It is the same surfacing channel item 22 now needs. *Inference.*
52. **Unverified claims left in the record:** arc-003 "no silent loss" and arc-004 "shipped", both later found dark (HT:273-280); S7, S10, S11 "built" (HT:281-285); "sidecar wakes only the right agent" (HT:288). Section 5 rates them.

## 5. What actually operates today (2026-10-04), stated plainly

53. **For agents on this host, the inject-into-a-running-session step does not work.** A message reaches the receiving sidecar and is stored and receipted as a hub receipt (stage 14-16), and then **nothing injects it into a running agent's session** (item 22 onward). The agent learns of it only if it goes and looks.
    53a. Why: the injector needs a TermLink-owned PTY, and no running agent here was started with one; a running session cannot be given one afterwards (PL-237). The injector is "SCHEDULED BY NOTHING" (`scripts/notify-injector.sh:4`).
    53b. The wake consumers only log (T3330R:47); `claude-termlink` has none (T-3206).
    53c. The classifier and the injector were proven once, on one test session and a fresh topic (T-3079, ARC11:148-170). Nobody turned that into a running service.
54. **Checked this session (read-only), evidence for the above.**
    54a. `.agentic-framework/lib/sidecar` does not exist; the vendored framework is 1.6.29 (`.agentic-framework/VERSION`).
    54b. No crontab under `.context/cron/` schedules `notify-injector`. The crontabs present are the sidecar supervisor, the wake supervisor and canaries (`ls .context/cron`).
    54c. Running sidecar processes carry `--auto-confirm` (`ps`, three agents): they post the L2 "delivered" receipt on arrival, which proves storage and never a read (see 14d).
55. **Separate "recorded as built" from "operating for real agents".**

| Element | Recorded as built | Operating for real agents on this host | Evidence |
|---|---|---|---|
| Hub transport, durable topics, receipts | yes | **yes** | HT C-section; IAC-v1:127 |
| Per-message addressing (`to_circuit`) | yes (T-3325) | **yes**, since 2026-10-02 23:17Z | IAC-v1:127 |
| Sidecar process, flag, journal, supervised respawn | S3, S4, T-3050, SQ-8 built | **partly**: processes run, mail of `inbox:` was invisible for 6 days until T-3203 | HT:286; 54c |
| Queue and priority | S8 built | **no consumer reads it** | ARC11:111-132; 53a |
| Prompt-free check (screen classifier) | S7 "built" | **no**: wired to nothing | RAIL:98; 53c |
| Injector, verify, INJECTED | S10 "PROVEN LIVE 2026-09-22" | **no**: one test session; scheduled by nothing | HT:281-285 |
| 30 s tick | designed 2026-10-03 | **no**: installed driver is `*/5` and checks no prompt | IAC-v1:124 |
| Urgent bypass | not built in TermLink (SQ-4 re-probe only, never into BUSY) | **no** | T-3072:87-96 |
| Sidecar-to-sidecar API calls (RECEIVED/STORED/INJECTED/ANSWER READY) | S1 and S5 "partial": a local API only | **no**; at best hub receipts | ARC11:34-38,81-86 |
| Per-step telemetry, daily digest, steward | not built | **no** | HT:262-266 |
| Sidecars shipped with the deployment | not built | **no** | T3330R:139-143 |
| Reply and roles swap | S12 partial | **hub `channel.post` only**; blocked for `framework-agent-systemd` | ARC11:183-202 |

56. **AEF's bleeding-edge receiver implements most of this design, and reaches only agents started with `claude-fw --termlink`.** It has the receiver API, a 30 s watcher, Stop/UserPromptSubmit readiness, urgent bypass, one-line `termlink pty inject`, transcript-proven HANDED_OVER, and receipts on the hub-topic path (HA:136-153,160-233). **Our vendored framework 1.6.29 does not contain it** (54a), and the injector's own message is "no TermLink session for this project… start the agent with claude-fw --termlink" (inject.py:222-224, via HA:297). So an agent started plainly has no target.
    56a. Evidence AEF gives: a live two-agent nonce e2e (3/3) plus a negative control (IN-010@62, via HA:296). Limits: it proves the mechanism in a controlled run; the same AEF host's receiver "was not running" for a day and its hub-topic fallback sent no receipt (IN-010@141, via HA:296); cross-host is T-3688 (not built); receipts are not on every path (receipts.py:14-16). TermLink has not verified any of it.
    56b. AEF's own document is internally stale: `sidecar-target-architecture.md:5` says "not what runs today" while its register lists R1-R11, R13-R15 `built` (HA:291-293). Its R14/R15 "every agent runs a sidecar" is built only for wrapper-launched agents (HA:297).
57. **Operator-visible proof that the rail does not carry real work.** On 2026-09-30 the agent route to pen-agent failed (her session was registered under the volatile legacy runtime dir; the shared key made any receipt unattributable). The operator pasted a self-contained prompt into her session directly and reported "Pen did a lot of stuff… She picked it up. It's all fine." (T-3280:269-270)
58. **Rule restated from arc-011.** A slice is `built` only when the specified mechanism carries it (RAIL:24-27). **Nothing in section 2 may be called working until this passes live:** two real running agents, a message sent while the receiver is busy and while it is idle, every step confirmed back to the sender with timestamps, the answer returned, plus a negative control (IAC-v1:128).

## 6. Open decisions for the operator

Taken one at a time (standing instruction, 2026-10-01). Each shows both positions in the operator's or ruling's words, then a recommendation.

59. **O1. The send path across hosts.**
    59a. Position A (operator 2026-10-03): "Send: sender agent -> its own sidecar (API) -> the receiver's sidecar (API)." (T3330R:248) and, AEF 2026-09-21: "We have multiple hosts, so that's not a question." (T-3397:270-272, via HA:83)
    59b. Position B (SQ-1 2026-09-23): the sidecar API is a LOCAL control surface and "sending to a peer stays channel.post via the hub" (T-3135 Evolution; ARC11:28-44); SCAPI:164-167 calls cross-host "explicitly out of scope" and the charter's rule against a second bus is the reason (SCAPI:17-35).
    59c. Recommendation: same-host sidecar-to-sidecar API now; cross-host rides the hub `inbox:` topic (the uniform fallback of 13a) until the operator rules whether the charter's no-second-bus rule bends.
60. **O2. How the urgent bypass is made safe.**
    60a. Position A (operator 2026-10-03): "inject when it's free and with urgent bypass" (CONSULT:16); "urgent gets injected immediately" (CONSULT:17); AEF 2026-09-21: "a hard skip, not a shortened interval" (T-3397:194-199).
    60b. Position B (SQ-4, 2026-09-23): "urgent NEVER injects into a BUSY prompt … Urgent shortens the WAIT; it does not bypass the prompt-free CHECK." (ARC11:305-316)
    60c. Recommendation: confirm A with the route of 19a (durable before inject, surfaced by the hook, evidence or re-inject, immediate alarm); record SQ-4 as superseded.
61. **O3. RECEIVED and STORED: one call or two.** Position A (operator 2026-10-03): RECEIVED "immediately", then STORED "once the message is stored" (T3330R:248-256, items 2-3). Position B (AEF): RECEIVED already means stored durably (D-690; IAC-v1:134, via HA:141). Recommendation: two events on the wire (RECEIVED at accept, STORED after fsync) only if a sender can act on the gap; otherwise one.
62. **O4. How an already-running session becomes injectable (PL-237).** Position A: no operator words; the agent's earlier stopgap, a prompt hook, drew "The operator wants the real mechanism, not a workaround." (handover S-2026-1003-2337.md:257) and T-3330 IW-4 (interim wake) stays open (:263). Position B (AEF): "no TermLink session for this project… start the agent with claude-fw --termlink" (inject.py:222-224, via HA:297). Recommendation: relaunch agents through the launcher, and until then report every unlaunched agent as not reachable instead of reporting mail as delivered.
63. **O5. Ratify harness-reported readiness, and decide the fallback.** Position A (operator 2026-09-20): "use PTY inject when the cursor is silent" (T-3396:87-92); TermLink's screen classifier T-3079 (ARC11:100-110). Position B (T-3397:97-113, endorsed by TermLink at ARC@1640): readiness self-reported by Stop/UserPromptSubmit. Recommendation: B is primary; keep the classifier only as a fallback for harnesses without hooks, labelled degraded.
64. **O6. The session level of the address.** Position A (operator): "a canonical id plus a runtime id" (IAC-v1:26). Position B (7-vendor review, 6/7): routing does not stop at a session label (T-3751-review-synthesis, via HA:149). Recommendation: route to project and agent role; keep the session id as metadata.
65. **O7. The name to id directory (AEF T-3751 IW-3).** Position A (brief, IAC-v1:30): the hub keeps project identity cards. Position B: no directory, ids ride inside names (decision 2a, 7/7 option B: "function names route, instance ids ride inside", IN-010@250, via HA:149). No operator ruling recorded.
66. **O8. Who owns the sidecar protocol and the injector.** Position A (SQ-1 2026-09-23): "the injector stays in TermLink as a primitive" (ARC11:203-213). Position B (TermLink's own advice to AEF, 2026-09-21): AEF owns readiness, inject and urgent policy "non-negotiably", since TermLink cannot observe harness state (T-3397:200-263, via HA:81). Proposed split sent at IN-AEF@184: AEF the protocol, TermLink telemetry record, pull, digest and discovery; no answer found.
67. **O9. The polling ladder against the retry ladder.** Position A (operator): the 15 s to 1 year ladder, each rung twice, "the standard … for any polling" (IN-AEF@180, via HA:322; "50" read as 15). Position B (AEF D-600): 2×1m … 2×1mo, about 76 days, then dead-letter (retry_ladder.py:16-27, via HA:206). T-3770 reconciles. Recommendation: two ladders, polling for waiting on a push, retry for re-sending a post, both declared.
68. **O10. The address ruling wording.** Position A (D-599): `inbox:<circuit-id>` (decisions.yaml:4178-4183). Position B (D-660): `inbox:<agent-id>` and "does NOT keep sidecar:" (decisions.yaml:4617-4622). AEF says the D-660 wording is being amended (IN-010@53); the amendment is unverified. Recommendation: ask AEF for the amended text.
69. **O11. The native consumer: retire or confirm as realised.** Position A (GO 2026-08-25, T-2838:206-228): ack from inside the turn loop. Position B (silence in arc-011 and IAC-v1). Recommendation: confirm AEF's hook-plus-transcript mechanism as the native consumer for Claude Code and reopen it only for other harnesses.
70. **O12. Call back to the sender, or record and pull.** Position A (operator, T3330R:229): "an API call that's received … an API call that's injected … also an API call." Position B (Codex and GLM): a synchronous call back is "wrong as written" and record-and-pull meets the need (T3330R:242, via HA:267). The operator chose C (hub record, "C amended"). AEF built the synchronous `/ack` and also recommends a collector pull for telemetry (HA:267). Recommendation: keep the operator's callbacks for the live path and add the hub record as the durable copy.
71. **O13. The telemetry retention window.** Position A (IAC-v1:90): "for example 30 days". Position B (T-3304 IW-2, operator "Alright, C then", T-3304:158-159): a 14-day default, mail by count (T-3310 D2 = B, T-3304:182-184). Recommendation: take the T-3304 ruling (14 days, count, age and size ceilings); record 30 days as the example it was.
72. **O14. Respawn and the startup chain.** Position A (operator, spec): the sidecar "always respawns" and carries "start agent, start session, start project, start hub" (RAIL:111-117). Position B (T-967 joint design, GO 2026-04-12): "auto-respawn needs explicit opt-in" (T-967:125); 7/7 vendors: no respawn from inbound mail without an explicit operator grant (via HA:149). Recommendation: scope the startup chain as supervised start of the SIDECAR and its host services; agent respawn only with a per-agent grant.
73. **O15. How alarms and escalation surface.** Position A (operator 2026-10-03): "Liveness alarm only for urgent messages. Everything else accumulates and escalates when it piles up, like audit warnings (design to be decided)." (T3330R item 12) Position B (review R6 and T-3461): a liveness invariant with canary mail, and an escalation terminus that is actually read (T3330R:237-246; T-3461:82-113). The steward agent (T-3333) is `captured`. Recommendation: urgent alarm and the daily digest first; one place the operator's escalation lands, verified by a negative control.
74. **O16. The stage vocabulary, and a state for "acknowledged, no action".** Position A (TermLink): INJECTED needs a BUSY transition (RAIL:69-84). Position B (AEF): HANDED_OVER needs transcript evidence (hooks.py:14-24). "This is not decided anywhere." (T3330R:222) The reply-or-no-action obligation (T-2402:60) has no sender state in either set. Recommendation: HANDED_OVER is the one name, defined as transcript evidence; add `ACKNOWLEDGED_NO_ACTION` as a terminal state beside REPLIED.
