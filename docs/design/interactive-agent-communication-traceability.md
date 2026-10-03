# Interactive agent-to-agent communication: requirement-to-task traceability

**Task:** T-3335 · **Built:** 2026-10-04 · **Inputs:** `docs/design/interactive-agent-communication-requirements.md` (R-1.1 … R-14.4, final), `.context/arcs/arc-011.yaml`, arcs `reliable-comms`, `push-transport`, `comms-loudness`, `arc-012`, task frontmatter in `.tasks/active` and `.tasks/completed`, and the two history reports (`docs/reports/T-3335-design-history-termlink.md` = TLH, `…-aef.md` = AEFH).

**How verdicts are used (strict).** COVERED-OPERATING means working for real agents on this host today, with evidence cited. A completed task alone is not evidence. COVERED-BUILT-NOT-OPERATING means code or a script exists and was proven once or in fixtures, but nothing runs it for real agents. COVERED-OPEN-TASK means the only cover is a task that is not finished (inception or captured). PARTIAL means part of the requirement is met, or it is met on the wrong mechanism or by only one of the two projects. GAP means no task delivers it.

**Evidence base for "operating" (read this once).**
1. `docs/design/interactive-agent-communication.md:52-57` (IAC §11, 2026-10-03): for agents on this host nothing injects into a running agent; the injector is scheduled by nothing.
2. `scripts/notify-injector.sh:4-28` (T-3207, 2026-09-29): "SCHEDULED BY NOTHING". Re-checked 2026-10-04: no `notify-injector` reference in `/etc/cron.d` or `.context/cron/`. The only open task that names it is T-3330 (inception).
3. `/etc/cron.d/termlink-notify-wake-supervisor` and `…-notify-sidecar-supervisor` are installed at `*/5`; they supervise processes and notice the flag. Flag and heartbeat files exist under `~/.termlink/notify/` (claude-termlink, framework-agent-systemd, s3g, …).
4. `termlink list` (2026-10-04) shows 2 `claude-master-*` sessions tagged `fw-project=…` (the tag AEF's injector targets) among about 20 sessions; most others are shell endpoints. `ps` shows AEF's `lib/sidecar/http_server.py` and `watcher.py` processes running on this host (roughly 60 receiver and 12 watcher lines). I could not read their state directory (T-559 boundary), so whether AEF's chain actually injects into those agents is observed-running, not verified.
5. `.agentic-framework/lib/sidecar/` does not exist in this checkout (AEFH §D item 5, TLH C10).

---

# A. Summary table

AEF task status below is as stated in AEFH (AEF checkout not readable from here). Abbreviations: OPER = COVERED-OPERATING, BNO = COVERED-BUILT-NOT-OPERATING, OPEN = COVERED-OPEN-TASK.

| R | Requirement (short) | TermLink tasks | AEF tasks (per AEFH) | Verdict |
|---|---|---|---|---|
| R-1.1 | Interactive two-way conversation while working, 1:1, N:N, with operator | T-243-era protocol, T-1800 done, T-2838 done, T-3069 done, T-3135 done, T-3335 started | T-3693 receiver (claimed built) | PARTIAL |
| R-1.2 | Sender always knows where its message is | T-3070 done, T-2300 done, T-2385 done (partial-complete), T-3330 started | receipts.py states SENT…ESCALATED (code) | PARTIAL |
| R-1.3 | No attach or manual polling needed | T-3069 done, T-3071/72/79/82 done, T-3207 done (not wired), T-2389 started | T-3684/85 watcher (code present, wrapper-started agents only) | BNO |
| R-2.1 | Inject keystrokes, read output back | T-007 done, T-2485 prover, T-2644 done | none | OPER |
| R-2.2 | Conversation over hub replaces fire-and-forget | T-256 done, T-1800 done, T-2291 done | none | OPER |
| R-3.1 | One sidecar per agent, separate process with an API | T-3135 done, T-2294 done, T-3075 done | T-3693 (HTTP API per agent) | PARTIAL |
| R-3.2 | Independent of hub | T-3135 done, T-3075 done | T-3693 (loopback HTTP) | PARTIAL |
| R-3.3 | Always respawns, portable | T-3135 done, T-3050 done, T-3068 done, T-3051 done, T-3205 done | watcher supervisor (code) | OPER |
| R-3.4 | Startup chain agent/session/project/hub | none | none | GAP |
| R-3.5 | Re-resolve FQDN or IP | T-3135 done (records, compares) | none | PARTIAL |
| R-4.1 | Sender gives message + blob to own sidecar API | T-3134 done (blob), T-3135 done (local only), T-3076 done | `fw sidecar send` (code) | PARTIAL |
| R-4.2 | Sender sidecar to receiver sidecar, push first | none (SQ-1 forbids) | direct.py same-host; T-3688 not built | PARTIAL |
| R-4.3 | Hub = fallback, discovery, blob storage | T-2291 done, T-2303 done, T-3134 done, T-3076 done | hub-topic fallback (code) | OPER |
| R-5.1 | RECEIVED call back to sender | T-2300 done | RECEIVED in HTTP reply | PARTIAL |
| R-5.2 | STORED durably + second call | T-2300 done, T-3201/T-3203 done | none separate (O3) | PARTIAL |
| R-5.3 | New-message flag raised | T-2294 done, T-3068 done, T-3203 done, T-3204 done | receiver flag (code) | OPER |
| R-6.1 | 30 s flag check | T-3068 done (`*/5`), T-3330 started | watcher tick 30 s (code) | BNO |
| R-6.2 | Read queue, highest priority first | T-3071 done | inject.deliver_pending (code) | BNO |
| R-6.3 | Urgent bypass, never silently lost | T-3072 done (built to superseded SQ-4) | urgent_bypass (code); ladder compression not built | PARTIAL |
| R-6.4 | Non-urgent: inject if free else wait | T-3069 done, T-3079 done, T-3082 done | inject.py (code) | BNO |
| R-7.1 | Readiness from harness hooks | T-3250 captured, T-1207 done (design) | T-3397 hooks (code) | PARTIAL |
| R-7.2 | Not inferred from screen | T-3079 done (screen-based, contrary) | T-3397 | PARTIAL |
| R-8.1 | One short line typed (`pty inject`) | T-3069 done | inject.py:327 | BNO |
| R-8.2 | INJECTED only with transcript evidence | T-3069 done (BUSY transition), T-2876 done (prover) | hooks.py HANDED_OVER | PARTIAL |
| R-8.3 | Flag down only when queue empty | T-3082 done, T-3068 done | none | PARTIAL |
| R-9.1 | ANSWER READY call, sender pulls | none | REPLIED state only | GAP |
| R-9.2 | Roles swap for reply | T-3135 done (partial S12), T-2386 done | REPLIED (code) | PARTIAL |
| R-9.3 | Woken agent replies or says no-action | T-2402 (active, work-completed, owner human) | none | PARTIAL |
| R-9.4 | Dead ears: agent halts | T-2294 done (`notify-check.sh` DEAF) | none | PARTIAL |
| R-10.1 | Fallback poll ladder 15 s … 1 year, x2 | none in TermLink | T-3434 (different ladder), T-3770 inception | OPEN |
| R-10.2 | Ladder is framework default | none | T-3770 inception | OPEN |
| R-11.1 | Five-level circuit | T-3325 done | T-3287, T-3433 (D-599) | PARTIAL |
| R-11.2 | Canonical name + instance identity per level | T-3325 done (partial), T-2384 done | T-3534 built, T-3751 not built | PARTIAL |
| R-11.3 | `to_circuit`, never fall back across projects | T-3325 done | IN-010@141 agreement | OPER |
| R-12.1 | Timestamped events copied to hub | T-3330 started | receiver/events.jsonl local only | OPEN |
| R-12.2 | Daily digest + reflection | T-3330 started, T-3319 done | none | OPEN |
| R-12.3 | Alarms urgent only, rest escalates | T-3330 started, T-3333 captured | none | OPEN |
| R-12.4 | Later: observability DB, steward | T-3333 captured, T-3319 done, T-2250 captured | none | OPEN |
| R-13.1 | Every sidecar ships with every deployment | T-3330 started (inception only) | none | OPEN |
| R-14.1 | Native consumer | T-2838 done (spike only) | none | GAP |
| R-14.2 | Typed assignment/result messages | T-2838 done (helpers, no verb) | none | PARTIAL |
| R-14.3 | Hub refuses "delivered" without receipt | none | none | GAP |
| R-14.4 | Startup chain | none | none | GAP |

---

# B. Per requirement detail

Task card format: `T-id | name | status | owner | horizon`. Horizon `-` means null (completed tasks). A trailing `[A]` marks a file in `.tasks/active/`, `[C]` in `.tasks/completed/`.

## B1. Goal

R-1.1 (PARTIAL)
1.1a T-1800 | Interactive agent conversation runtime, deterministic doorbell+mail auto-pickup loop | work-completed | human | - [C]. Delivered the doorbell+mail protocol and the receipt/re-ring loop (TLH A2). Superseded as the wake mechanism by later rounds.
1.1b T-2838 | Delivery-to-turn contract, build it or keep nudging | work-completed | human | - [C]. Typed helpers and honest `channel post` states landed; the native consumer stayed a spike (TLH B19).
1.1c T-3069 | The injector: queue to prompt-free check to inject to verify-working to L3 | work-completed | agent | - [C]. Proven once on a fresh topic (`arc-011.yaml:148-170`); the injector is scheduled by nothing (`scripts/notify-injector.sh:4-28`).
1.1d T-3335 | External consultation: make two-way interactive communication … | started-work | human | now [A]. This design effort.
1.1e Why PARTIAL: hub mail is stored and receipted (IAC:56), but IAC:52 states nothing injects into a running agent on this host. N:operator and N:N are not distinguished anywhere in the build.

R-1.2 (PARTIAL)
1.2a T-3070 | Sender-side ledger: record DELIVERED and INJECTED events locally | work-completed | agent | - [C]. `scripts/notify-ledger.sh`, rung advances only on a receipt read back (`arc-011.yaml:87-99`). Its first live sync showed delivered-but-never-read for both messages to the AEF agent.
1.2b T-2300 | V6-S3 sidecar journaled-receipt + stage-aware confirm | work-completed | agent | - [C]. L2 `stage=delivered` receipt is durable but posted to a hub topic (`arc-011.yaml:81-86`).
1.2c T-2385 | agent contact reachability preflight + fail-fast + structured delivery result | work-completed | human | now [A]. Loud per-link result on send; one `[REVIEW]` AC unticked, so it sits in active/ partial-complete. The MCP port is T-3227 (captured).
1.2d T-3330 | Receive side: received/injected/replied telemetry as API calls … | started-work | human | now [A]. Inception, not built.
1.2e Why PARTIAL: the sender can see sent and delivered, never injected or answered, because the rungs that need the injector never occur.

R-1.3 (COVERED-BUILT-NOT-OPERATING)
1.3a T-3069, T-3071 (priority, built), T-3079 (idle classifier rebuilt), T-3082 (watermark) all [C], work-completed, agent/claude-code. Together form the chain `notify-injector.sh`.
1.3b T-3207 | notify-injector.sh is wired to nothing while notify-wake-consumer.sh is the … | work-completed | agent | - [C]. Answer recorded as "WIRE it … but not yet" (`scripts/notify-injector.sh:4-28`). No follow-up task exists.
1.3c T-2389 | Relaunch .107 agents via T-2388 tl-claude --reachable launcher | started-work | human | now [A]. The arming precondition (PL-237); open since July.
1.3d Why BNO: code exists and was proven once; nothing schedules it and no real agent is armed.

## B2. Origins

R-2.1 (OPERATING)
2.1a T-007 | IT-004: Output capture, bidirectional communication | work-completed | agent | - [C]. `termlink inject` / `output` / `exec`.
2.1b T-2644 | PTY interactive attach silently drops keystrokes when command.inject fails | work-completed | human | now [A]. Hardening of the same verb; active/ because a human AC remains.
2.1c Evidence: the session-control canary (T-2557, CLAUDE.md) and `scripts/session-selftest.sh` (T-2485) prove spawn/exec round-trip daily; not re-run in this pass, so cited, not re-measured.

R-2.2 (OPERATING)
2.2a T-256 | Inception: True push messaging, emit-to-target RPC | work-completed | agent | - [C]. Origin of hub-mediated push.
2.2b T-2291 | Permanent cross-agent comms fix, identity + delivery | work-completed | human | - [C]; T-2303 push transport | work-completed | human | - [C]. Hub conversation is the live path (IAC:56).
2.2c Why OPERATING: hub transport and storage operate (IAC:56). The wake half is covered by R-1.3.

## B3. The sidecar

R-3.1 (PARTIAL)
3.1a T-3135 | Sidecar API: local control surface + portable respawn supervisor | work-completed | agent | - [C]. `scripts/notify-sidecar-api.sh` is a local CLI (status/queue/inject/agent-state/ack), not a process with a network API (`arc-011.yaml:28-44`; note at 33-38 says the spec wording was not built).
3.1b T-2294 | V3a: deterministic notify (sidecar wake) | work-completed | agent | - [C]. One `notify-sidecar.sh` process per declared agent (flag files present in `~/.termlink/notify/`).
3.1c T-3075 | Sidecar API, separate respawning process independent of the hub | work-completed | human | - [C]. Analysis GO, not build (`arc-011.yaml:36-44`).
3.1d AEF T-3693 receiver: per-agent HTTP API on 127.0.0.1 (AEFH B1.1). Not in this framework copy.
3.1e Why PARTIAL: a per-agent process exists for TermLink's notify sidecar but has no sidecar-to-sidecar API; AEF has the API but not for our agents.

R-3.2 (PARTIAL)
3.2a T-3135, T-3075 as above. The local store and the local API work with the hub down; receiving still depends on the hub because `journal-mirror.sh` reads hub topics.
3.2b AEF T-3693 receives over loopback HTTP independent of the hub (AEFH B1.1a) on the same host only.

R-3.3 (OPERATING)
3.3a T-3135 `notify-sidecar-supervisor.sh --loop` and `--emit-unit systemd|launchd|cron|auto` (`arc-011.yaml:28-44`, SQ-8 at 350-367).
3.3b T-3050 | Notify sidecar has no launcher, cron supervisor | work-completed | agent | - [C]; T-3068 | supervised standing service | work-completed | agent | - [C]; T-3051 | canary for dead sidecar | work-completed | agent | - [C]; T-3205 | stale-code detector | work-completed | agent | - [C].
3.3c Evidence: `/etc/cron.d/termlink-notify-sidecar-supervisor` and `…wake-supervisor` installed; supervised PIDs changed overnight with fresh heartbeats (`arc-011.yaml:171-182`). Caveat: the macOS/launchd path is emitted, not exercised.

R-3.4 (GAP)
3.4a No task. TLH B2d and D4: "No document or script describes it beyond the spec statement." Same item as R-14.4.

R-3.5 (PARTIAL)
3.5a T-3135: FQDN/IP are re-read and compared per call (`scripts/notify-sidecar-api.sh:197-212` per TLH B2c). It records change; no re-resolve-and-retarget behaviour is evidenced.

## B4. Sending

R-4.1 (PARTIAL)
4.1a T-3134 | artifact CLI verbs: termlink artifact put/get | work-completed | agent | - [C]; T-3076 | Binary blob on the notify rail via artifact.put | work-completed | human | - [C]. Blob path works through the hub (`arc-011.yaml:45-69`).
4.1b T-3135: the local API does not send; SQ-1 keeps sending on `channel.post` (`arc-011.yaml:33-38`).
4.1c T-3140 | artifact manifest.from must be identity fingerprint | captured | agent | now [A]. Latent bug in three older callers.
4.1d AEF: `fw sidecar send` (AEFH B1.6).
4.1e Why PARTIAL: sending works via the hub; "give it to your own sidecar's API" is not built on our side. Open contradiction O1 (IAC:58).

R-4.2 (PARTIAL)
4.2a No TermLink task; the charter bars a second bus (IAC:44). AEF `direct.py` does same-host receiver-to-receiver; cross-host T-3688 not built (AEFH B5.3).

R-4.3 (OPERATING)
4.3a T-2291, T-2303 [C]; T-3134, T-3076 [C]. Evidence: IAC:56 states hub transport, storage and addressing operate.

## B5. Receiving

R-5.1 (PARTIAL)
5.1a T-2300: stage=delivered receipt as a hub post. `arc-011.yaml:81-86`: "Right event, wrong mechanism." S5 is `partial`.
5.1b AEF: RECEIVED returned synchronously in the HTTP reply (AEFH B2.3a).

R-5.2 (PARTIAL)
5.2a T-2300 journal.sqlite (S3, `arc-011.yaml:70-74`). T-3201 | journal-mirror watches dm: only | work-completed | agent | - [C] and T-3203 | notify-sidecar probes dm: only | work-completed | agent | - [C] fixed `inbox:` mail missing from the store; T-3204 | make the fix live | work-completed | agent | - [C] restarted the sidecars.
5.2b There is no second call; O3 (RECEIVED vs STORED, IAC:60) is undecided.

R-5.3 (OPERATING)
5.3a T-3068 (S4, `arc-011.yaml:76-80`) plus T-2294; flag files present in `~/.termlink/notify/` (this pass). The flag is read by `notify-wake-consumer.sh`, which only logs (TLH C5c); the raising is real.

## B6. The 30-second tick

R-6.1 (BNO)
6.1a T-3068 installs `*/5` supervisors (S11, `arc-011.yaml:171-182`). They notice the flag; no 30 s loop reads the flag and injects (IAC:53d).
6.1b AEF watcher tick default 30 s (AEFH B4.1, watcher.py:69) for wrapper-started agents only.
6.1c Open task: T-3330 mentions the injector wiring. No build task exists.

R-6.2 (BNO)
6.2a T-3071 | Message queue priority field, flat FIFO for now | work-completed | agent | - [C]. Ordering `COALESCE(priority,0) DESC, ts ASC, offset ASC`, clamped to [-9,9] (`arc-011.yaml:111-132`). Runs only inside the unscheduled injector.

R-6.3 (PARTIAL)
6.3a T-3072 | Urgent flag: bypass the prompt-free wait and inject directly | work-completed | agent | - [C]. Built to SQ-4: urgent re-probes up to 120 s and never injects into a busy prompt (`arc-011.yaml:133-147`, SQ-4 at 305-316). The operator's 2026-10-03 words reverse this (TLH B8e), so the built semantics are the superseded ones.
6.3b AEF `urgent_bypass` types into a busy session and detects loss afterwards (AEFH B3.4, C2 #4). Ladder compression raises `NotImplementedError` (AEFH B3.5).
6.3c Why PARTIAL: the bypass is not built on our side and the safe route (O2) is undecided.

R-6.4 (BNO)
6.4a T-3069 (S7/S10), T-3079 | Idle classifier is blind to the current Claude Code UI | work-completed | claude-code | - [C] (466 samples, false-ready 0, `arc-011.yaml:100-110`), T-3082 | queue watermark | work-completed | claude-code | - [C]. Single-shot, unscheduled.

## B7. Readiness

R-7.1 (PARTIAL)
7.1a T-3250 | Design: who observes agent-side readiness | captured | human | next [A]. The open decision (TLH D8).
7.1b T-1207 | Stop hook design | work-completed | human | - [C]: April design, not the readiness flag.
7.1c AEF T-3397 hooks: Stop sets ready, UserPromptSubmit clears first (AEFH B1.3); code present, framework copy here lacks it.
7.1d Why PARTIAL: AEF built it; TermLink's injector uses a different source.

R-7.2 (PARTIAL)
7.2a T-3079: classifier decides on terminal repaint quiescence and composer-row emptiness, i.e. it reads the screen (`arc-011.yaml:100-110`). This is the opposite of R-7.2. Contradiction C2 #3 in AEFH. The verdict stays PARTIAL only because AEF satisfies it; see G2.

## B8. Injection

R-8.1 (BNO)
8.1a T-3069: `termlink pty inject`. AEF types one fixed line (inject.py:327). Neither runs for a real agent here (IAC:52-53).

R-8.2 (PARTIAL)
8.2a T-3069 evidence is a BUSY transition then the receipt read back (`arc-011.yaml:148-170`); the requirement says transcript. T-2876 | Cross-session message prover | work-completed | agent | - [C] reads the receiver transcript, but as a test, not as the injector's gate. AEF HANDED_OVER is gated on transcript evidence (AEFH B2.4).

R-8.3 (PARTIAL)
8.3a T-3082 watermark; `scripts/notify-injector.sh:349` guards "queue empty though flag set". No evidence of a rule that lowers the flag only on an empty queue; none cited in TLH.

## B9. Answering

R-9.1 (GAP)
9.1a No task. TLH B9b: `stage=replied` does not exist anywhere; ANSWER READY is new. AEF REPLIED is a state, not a call back.

R-9.2 (PARTIAL)
9.2a T-3135 (S12 partial, `arc-011.yaml:183-201`): receiving half built; reply travels `channel.post` (`/reply`, `agent-respond.sh`). Blocked externally: `framework-agent-systemd` lacks `termlink` in `--allowed-commands`. T-2386 | Reply-on-sender-hub routing convention | work-completed | agent | - [C].

R-9.3 (PARTIAL)
9.3a T-2402 | Woken-but-silent: make a rung-yet-unanswered agent loud + self-healing | work-completed | human | now [A]. Stage 6 wake-protocol obligation (TLH A5bf); one of four ACs unticked. TLH D3: not carried into the arc-011 chain; "acknowledged, no action" state not in the receipt vocabulary.

R-9.4 (PARTIAL)
9.4a T-2294 `notify-check.sh` verdicts MAIL 10 / DEAF 3 / CLEAR 0 (TLH A3cc). TLH D2: the cron-injector model has no halt rule; only the daily `notify-sidecar` canary notices.

## B10. Fallback polling

R-10.1, R-10.2 (OPEN)
10.1a AEF T-3434 retry ladder 2x1m … 2x1mo, about 76 days (AEFH B4.2); AEF T-3770 inception to reconcile with the operator ladder (AEFH C2 #10). No TermLink task exists; the reliable-comms ack-with-retry T-2285 | Substrate ack-with-retry | work-completed | human | - [C] is a different mechanism.
10.1b The operator's "50 read as 15" dictation question is still open (TLH D14).

## B11. Addressing and identity

R-11.1 (PARTIAL)
11.1a T-3325 | notify-sidecar: honour an addressee / reply-to agent id | work-completed | agent | - [C]. `to_circuit` live since 2026-10-02 23:17Z (IAC:56). AEF T-3287, T-3433 (D-599). Session level open (O5, IAC:62).

R-11.2 (PARTIAL)
11.2a T-2384 | Send-side per-agent fp resolution | work-completed | agent | - [C]. AEF: hub id is a rotating TLS fingerprint, project = folder basename today; minted pid (T-3534) built, T-3751 not built (AEFH B6.3-B6.4). Directory (O6) undecided.

R-11.3 (OPERATING)
11.3a T-3325 (live, IAC:56). AEF agreed "never fall back across projects" (AEFH C1 #3). Operating for per-message addressing; I did not run a cross-project negative test in this pass.

## B12. Telemetry

R-12.1 … R-12.4 (OPEN)
12.1a T-3330 | Receive side: received/injected/replied telemetry … | started-work | human | now [A]. Inception, 4 ACs unticked of 4 (`.tasks/active/T-3330-*.md:118-126`).
12.1b T-3333 | Hub steward agent | captured | human | now [A]; T-2250 | R5 telemetry plane design | captured | human | later [A]; T-3319 | Learn from message traffic | work-completed | human | - [C] (inception); T-3321 | Message compaction | captured | human | later [A].
12.1c AEF writes local ledgers only; `telemetry.jsonl` is design, not code (AEFH B7.2, D13). No hub copy.
12.1d Daily digest, urgent-only alarms and escalation have no build task; IAC:94 leaves alarm surfacing open.

## B13. Deployment

R-13.1 (OPEN)
13.1a T-3330 (title includes "ship all"). Releases ship the binary only; about 13 scripts run only from the checkout; no `sidecar` subcommand (TLH B17). No build task.

## B14. Discussed once and then lost (awaiting operator confirmation)

R-14.1 (GAP) T-2838 [C] proved the property in a spike (TLH B19a); no build task.
R-14.2 (PARTIAL) T-2838 item 3: `assignment.v0` / `result_manifest.v0` helpers landed, "no CLI verb emits or consumes these yet" (TLH B19a).
R-14.3 (GAP) No task (TLH D7).
R-14.4 (GAP) Same as R-3.4.

---

# C. Gaps (GAP and PARTIAL; G1 also covers the built-not-operating chain, because it decides everything else)

G1. **Name:** Run the injector as a supervised 30-second loop for armed agents. **Type:** build. **Scope:** wrap `notify-injector.sh` in a bash supervised loop (reuse `notify-sidecar-supervisor.sh --loop`), one per declared agent through `notify-wake-agents.conf`, with the supervisor cron at `*/5` only restarting it. Covers R-1.3, R-6.1, R-6.2, R-6.4, R-8.1. **Depends on:** G3 (needs at least one armed agent), G2.
G2. **Name:** Decide the readiness source: hook flag (AEF T-3397) versus screen classifier (T-3079). **Type:** inception. **Scope:** one decision, absorbs T-3250, sets whether TermLink consumes the Stop/UserPromptSubmit flag. Covers R-7.1, R-7.2. **Depends on:** none.
G3. **Name:** Make running agents injectable (PL-237 / O4). **Type:** build. **Scope:** arming path for sessions that exist today: relaunch through `tl-claude.sh start --reachable`, or a registered pty for an already-running claude; closes T-2389's operator action with a repeatable script in `runme.sh`. Covers R-1.1, R-1.3. **Depends on:** operator ruling on O4.
G4. **Name:** Decide the cross-host send path (O1): sidecar-to-sidecar versus hub. **Type:** inception. **Scope:** reconcile SQ-1 and the charter's no-second-bus rule with AEF T-3688. Covers R-3.1, R-3.2, R-4.1, R-4.2. **Depends on:** none; blocks G5, G11.
G5. **Name:** Receipt vocabulary and calls: RECEIVED, STORED, INJECTED, ANSWER READY. **Type:** build (after a one-page mapping decision of O3 and the TermLink/AEF stage mapping). **Scope:** one stage vocabulary, receipts that keep every stage (not latest-only, `channel.receipts` limit TLH B9d), ANSWER READY pull. Covers R-5.1, R-5.2, R-9.1, R-1.2. **Depends on:** G4, T-3330.
G6. **Name:** Urgent route that cannot silently lose a message. **Type:** build. **Scope:** replace T-3072's SQ-4 semantics with bypass plus loss detection (re-inject on missing transcript evidence, as AEF's 120 s window), and an urgent ladder. Covers R-6.3. **Depends on:** G2, G7.
G7. **Name:** Injection evidence from the transcript and flag-down-on-empty. **Type:** build. **Scope:** make INJECTED depend on the receiver transcript (reuse `session-message-selftest.sh assert` logic) and lower the flag only on an empty queue. Covers R-8.2, R-8.3. **Depends on:** G2.
G8. **Name:** Startup chain and FQDN/IP re-resolve. **Type:** inception. **Scope:** no design exists; define start order agent/session/project/hub and what re-resolution retargets. Covers R-3.4, R-3.5, R-14.4. **Depends on:** none.
G9. **Name:** Woken-agent obligation and dead-ears halt in the cron model. **Type:** build. **Scope:** an explicit "acknowledged, no action" receipt, and the DEAF verdict surfaced to the agent at session start and at hook time. Covers R-9.3, R-9.4. **Depends on:** G5.
G10. **Name:** Reply through the sidecar and unblock `framework-agent-systemd`. **Type:** build. **Scope:** reply path per G4's outcome; the allowed-commands change is another project's config (T-559) and goes via a peer message. Covers R-9.2. **Depends on:** G4.
G11. **Name:** Identity: session level, stable hub name, minted project id, directory. **Type:** inception. **Scope:** closes O5 and O6 and adopts AEF T-3751's output. Covers R-11.1, R-11.2. **Depends on:** AEF T-3751.
G12. **Name:** Native consumer decision (R-14.1). **Type:** inception. **Scope:** keep or drop per the operator; if kept, build from the T-2838 spike. **Depends on:** G2.
G13. **Name:** Typed assignment and result verbs (R-14.2). **Type:** build, only if the operator confirms. **Scope:** wire `assignment.v0` / `result_manifest.v0` helpers to a CLI verb. **Depends on:** none.
G14. **Name:** Hub-side refusal of "delivered" without a receipt (R-14.3). **Type:** inception, only if the operator confirms. **Scope:** one question; touches hub semantics and the post path. **Depends on:** G5.
G15. **Name:** The arc-011 closing test as a live prover. **Type:** build. **Scope:** two real running agents, one busy, one idle, every step timestamped back to the sender, the answer returned, plus a negative control; extend `notify-rail-e2e.sh`. Without it no row above may move to OPERATING. **Depends on:** G1, G3.
G16. **Name:** Build tasks for the OPEN rows (telemetry copy to hub, digest, sidecar packaging). **Type:** build. **Scope:** T-3330 is an inception with no build children; create them when its GO lands. Covers R-12.1 to R-13.1. **Depends on:** T-3330 decision.

---

# D. Stale or misleading task states

1. **arc-011 S7, S10, S11 marked `built`** (`arc-011.yaml:100-182`). Evidence against operation: IAC:54 says "worked once"; `scripts/notify-injector.sh:4-28` says scheduled by nothing; no cron references it (re-checked 2026-10-04); the installed cron is `*/5` and only notices the flag; `notify-wake-consumer.sh` terminates in a log line (TLH C5c). T-3069, T-3071, T-3079, T-3082, T-3068 are completed with the code real and the operation absent.
2. **S9 / T-3072 `built`.** Built to SQ-4, which the operator reversed on 2026-10-03 (TLH B8e). The task is complete for a superseded requirement.
3. **S1 / T-3135 `built`.** The arc's own note says the spec wording "sender sends via a sidecar API" is not what was built (`arc-011.yaml:33-38`). It meets R-3.3 and a local-control reading only. S5 and S12 are already `partial`.
4. **S3 `built`, "journal.sqlite, 2238 rows".** The journal covered `dm:` only until T-3201/T-3203 (TLH C11); the note was never updated.
5. **T-3207 completed** with the answer "wire it but not yet" and no follow-up task. The only open reference is T-3330. This is the single largest loss: no task exists to wire the injector.
6. **T-3325 completed.** IAC:10 records it was described as waking the right agent while nothing reached any agent.
7. **T-3200 completed.** Its premise "nine of ten slices built" (TLH C6) hid that the rail did not operate.
8. **arc-003 `closed`, "no silent loss"** (`reliable-comms.yaml:12,21-22`) while the sidecar had not run since 2026-07-01 (T-3049, TLH C1). Mitigated now by the arc-claim-drift canary (T-3288) and the notify-sidecar canary, so the claim is re-checked daily, but the prover only covers the hub path.
9. **arc-004 `closed` shipped** (`push-transport.yaml:54-55`) with zero wakers (TLH C2); T-2388 and T-2389 are the unfinished arming half. `push-transport.yaml` comments still list T-2325 as active; it is completed.
10. **T-2385 and T-2402 are `work-completed` but still in `.tasks/active/`** with a human AC open. They are partial-complete, not obsolete; arc `comms-loudness` (in-progress, anchor T-2385) is their home.
11. **Arc id mismatch.** `arc-011.yaml:1` has `id: arc-010` with slug `arc-011`; `arc-012.yaml:1` has `id: arc-011`. Anything keyed on `id` mixes the two arcs.
12. **AEF claims (AEFH §D).** "Live two-agent nonce e2e 3/3" coexists with AEF's own receiver down for a day (AEFH D4); R14/R15 "built" holds only for wrapper-started agents (D5); `sidecar-target-architecture.md` is internally stale (D1). On this host the AEF processes run, but I could not verify they inject.
13. **Active tasks that are obsolete.** None is clearly obsolete. Overlaps: T-3250 is subsumed by G2; T-2389 is the operator-owned half of G3 and should stay open until G3 exists. T-3140 and T-3227 are valid, unrelated fixes. T-3321 and T-2250 are parked (`later`).

---

# E. Arc fit

E1. **arc-011 slices to requirements.**
1. S1 (T-3135): R-3.1, R-3.2, R-3.3, R-3.5; mismatches R-4.1.
2. S2 (T-3134): R-4.1 (blob), R-4.3.
3. S3 (T-2300): R-5.2.
4. S4 (T-3068): R-5.3.
5. S5 (T-2300, partial): R-5.1.
6. S6 (T-3070): R-1.2.
7. S7 (T-3069): R-6.4, R-7.1/R-7.2 (screen-based).
8. S8 (T-3071): R-6.2.
9. S9 (T-3072): R-6.3.
10. S10 (T-3069): R-8.1, R-8.2.
11. S11 (T-3068): R-6.1, R-3.3.
12. S12 (T-3135, partial): R-9.2.

E2. **Other arcs.** `reliable-comms` / `push-transport`: R-2.2, R-4.3, R-5.x (hub path). `comms-loudness` (T-2385, T-2389, T-2402): R-1.2, R-1.3 (arming), R-9.3. `arc-012` (hub as record, T-3304) touches R-12.1 only by retention and is not mapped.

E3. **Requirements with no arc-011 slice.** R-1.1, R-1.3 (outcome, no slice), R-2.1, R-2.2, R-3.4, R-4.2, R-8.3, R-9.1, R-9.3, R-9.4, R-10.1, R-10.2, R-11.1, R-11.2, R-11.3 (T-3325 is outside the arc), R-12.1 to R-12.4 (T-3330 outside), R-13.1, R-14.1 to R-14.4. Twenty-four of the 43 have no slice, including every requirement in sections 10 to 14. Arc-011 was written against the ten-step spec of 2026-09-22; the later read-back added the 30-second tick, ANSWER READY, the polling ladder, telemetry and deployment, and no slice carries those.
