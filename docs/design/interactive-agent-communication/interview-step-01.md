# arc-011 step 1 — operator interview log (T-3344)

The open questions of `docs/design/interactive-agent-communication-01-requirements.md` section 9, put to the
operator one at a time by the orchestrator (standing instruction 2026-10-01). Each entry: the question, the
operator's ruling as given, how it was read back, and what it authorises. The role session folds all rulings into
the output in one pass after the last question.

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-04 | Log started | T-3344 |

## OD-1 Cross-host send path (section 9.4) — ruled C

1. Asked 2026-10-04 with options A (hub carries cross-host), B (sidecar-to-sidecar for everything, charter change),
   C (hubs set up a circuit; established conversations run sidecar-to-sidecar; hub path as fallback),
   D (conversation bound on the hub path; socket circuit only after a measurement, AEF).
2. Orchestrator recommended C; the output's own recommendation (A, first build) was shown and why it was not
   followed (it leaned on round-1 positions three of four reviewers revised in round 2).
3. **Operator ruling: "c".** Read back as C, built in the order every reviewer agrees on: identity hygiene and the
   hub directory with liveness first (circuit set-up needs it); conversations bound to an instance on the hub path
   (D's binding); then the direct circuit, under the reviewers' conditions: set-up and authorisation through the
   hubs, per-circuit short-lived credentials minted by the hubs, one delivery contract and one log per
   conversation on both paths. AEF's measurement (circuit-break rate, hub-path share of turn time) belongs in the
   circuit's first build.
4. Authorises: OD-1 = C in step 1 (changes R-7, R-12, R-13); the conditions become requirements; the circuit's
   trust model goes to step 2. Leaves open: the hub-to-hub directory/relay principle and the charter rewording,
   OD-18, transport details (step 4).

## OD-2 Urgent into a busy prompt (section 9.5) — ruled B

1. Asked 2026-10-04 with options A (bypass, type only the fixed doorbell, content stored first; 055),
   B (never type into a busy prompt; urgent content through the harness's hook channel; GLM),
   C (authenticated harness interrupt, else report and escalate; Codex), D (keep SQ-4: urgent only shortens the wait).
2. Orchestrator recommended B and showed the output's own recommendation (A plus B's hook route). Fact added,
   verified in the orchestrator's own session: Claude Code delivers PostToolUse hook output into a running turn
   after each tool call ("PostToolUse:Write hook additional context" appeared mid-turn); the Stop hook can hand
   content over at turn end and continue the agent.
3. **Operator ruling: "B".** Read back as: urgent content reaches a busy agent through the harness's hook channel
   (next tool call, or end of turn); nothing is ever typed into a busy prompt; an idle agent gets the normal
   one-line inject. SQ-4 is recorded as reconciled, not superseded: urgent bypasses the wait, never the
   prompt-free check.
4. Authorises: OD-2 = B in step 1 (changes R-19, R-23, R-24); a delivery-adapter requirement for harnesses without
   hooks (055's adapter point); an acceptance test that an urgent message reaches a busy agent mid-turn with
   transcript evidence. Leaves open: interrupt consent (OD-15); other harnesses' routes (OD-7).

## OD-3 Polling ladder (section 9.6) — ruled A, modified: continuous ladder plus defined message states

1. Asked 2026-10-04 with options A (keep R-10.1), B (two mechanisms: short wait then visible "stuck"; reviewers),
   C (AEF retry ladder only), D (R-10.1 capped by retention; collector-synthesized). Orchestrator recommended B.
   Orchestrator correction to the reviewers' premise: inbox topics keep the newest 1000 messages by count, not by
   age (T-3310), so a message on a quiet inbox can survive a year; the real gap is that nothing turns "waiting"
   into a visible "stuck".
2. **Operator ruling (voice, read back and confirmed):** first rung 15 s and fourth rung 15 min are correct
   ("50" = 15). Keep the operator's ladder, but continuous, "no gaps", instead of each rung twice. Poll moments
   after send:
   2a. 15 s, 30 s, 45 s;
   2b. 1, 2, 3, 4, 5, 10, 15, 30, 45 min;
   2c. 1, 2, 3, 4, 8, 12, 16, 20, 24 h;
   2d. 2, 3, 4, 5, 6, 7 days (1 week);
   2e. 2, 3, 4 weeks;
   2f. 1, 2, 3 months (3 months = 1 quarter);
   2g. 2, 3, 4 quarters (= 1 year);
   2h. 2 years (last poll). 43 polls over two years.
   Interpretations confirmed: "15 seconds, 15 seconds, 15 seconds" = 15/30/45 s; "13 minutes" = 30 min;
   "2 weeks, 2 weeks, 4 weeks" = 2/3/4 weeks; "1 quarter" = 3 months.
3. **Operator asked to define "stuck"; definition approved ("That's good, that's really good"):**
   3a. Message steps: SENT → STORED (destination accepted) → HANDED_OVER (transcript evidence) → REPLIED or
       explicit no-action. At each poll the sender classifies the message:
   3b. WAITING — the next step is not yet due. Keep polling.
   3c. WAITING FOR RECIPIENT — the recipient is known not running or not able to receive (e.g. AEF's
       WAITING_NO_RECIPIENT). Keep polling, show it, deliver when the recipient returns.
   3d. STUCK — the next step is overdue AND the party responsible is reachable and alive. Show it to the sender and
       the cockpit, naming the stalled step: not accepted (SENT, no STORED within 1 min while the destination hub is
       reachable); not handed over (STORED, no HANDED_OVER within 2 ticks of the agent becoming ready, about 1 min,
       while alive and ready — the 2026-10-03 failure); not answered (HANDED_OVER, no reply/no-action within 1 h
       while alive; a sender may set another deadline per message).
   3e. UNKNOWN — the sender cannot get information (destination hub unreachable, liveness unknown). Keep polling,
       show "unknown since T", never treat as dead.
   3f. DEAD — the recipient's home hub declares that instance ended. Stop polling, dead letter to the sender.
   3g. Urgent: accepted within 15 s; handed over at the next tool call or turn end (OD-2 ruling); answered within
       5 min.
   3h. The ladder decides when the sender looks; the state is a label that appears and clears; polling continues
       behind it. "Stuck" means someone can fix something now; "unknown" and "waiting for recipient" mean nothing to
       fix yet.
4. Authorises: R-30 and R-31 rewritten to the continuous ladder; the five states and the stuck deadlines as
   requirements (sender-visible, cockpit-visible); urgent deadlines. AEF's retry ladder (D-600) for re-sending is
   unaffected and stays AEF's. Leaves open: where escalation of "stuck" lands (OD-14).

6. **Correction, 2026-10-06 (operator, during OD-17 CAND-12): two ladders by priority, not one.** The record above
   applied the continuous ladder to all messages; the operator's intent was:
   6a. normal messages (priority below 5): each rung twice — 1 min, 5 min, 15 min, 1 h, 4 h, 1 day, 3 days, 1 week,
       1 month, 1 quarter, 1 year (22 polls, a little over two years; rungs from R-10.1 of 2026-10-03, its 15 s rung
       dropped because the operator began normal at 1 min);
   6b. urgent messages (priority 5 or higher): the continuous ladder of point 2 (15 s … 2 years, 43 polls);
   6c. the five states and stuck deadlines of point 3 are unchanged;
   6d. a message's ladder also governs its re-sends before STORED: one schedule per priority class for re-sending
       and polling; for agent mail this replaces AEF's D-600 (the normal ladder is D-600 extended to years instead of
       dead-lettering at a month). AEF to be told (point 5's pickup offset 314 carries the superseded form).
   Read back to the operator, who answered "next" without corrections; recorded as confirmed, can be overturned.

5. Shared with AEF on 2026-10-04 at the operator's "yes": framework:pickup offset 314 (pickup_id
   010-termlink/T-3344-polling-ladder-v2-and-message-states, supersedes offset 309), plus a note on AEF's inbox.

## OD-4 RECEIVED and STORED (section 9.7) — ruled A

1. Asked 2026-10-04 with options A (two calls, defined precisely; Codex), B (one call after the durable write;
   GLM, 055, AEF's build), C (two only where a sender can act on the gap), D (merge on a timing bound;
   collector-synthesized). Orchestrator recommended A because OD-1 = C (circuits: "bytes arrived" and "saved"
   are distinct facts) and OD-3 (stuck names the stalled step) make the second call useful.
2. **Operator ruling: "Yep".** Read back as A: RECEIVED = arrived, not yet safe, informational; STORED = durably
   saved, the only call that releases the sender and step 2 of the OD-3 states. On the hub path, where the store
   is instant, both may travel in one message carrying both stages with two timestamps.
3. Authorises: R-14, R-15; OD-3's "not accepted" splits into "not received" (transport) and "received, not
   stored" (receiver). Leaves open: stage names (OD-8), calls vs records (OD-5).
4. Operator follow-up (leads into OD-5): "what if the receiver sends to the sender and the sender is not
   available? Then the retry kicks in. At the same time would we also want to store it with our hub ... register
   the status? It does, because we send telemetry to the hub." Answered in place: R-12.1 already copies every
   per-step event to the hub; proposed that the hub record is the truth and the callback a fast notice (one or
   two attempts, no storm), the sender reading the hub record on return. Put to the operator as OD-5.

## OD-5 Callback to the sender, or record and pull (section 9.8) — ruled: hub record is the truth, callback a fast notice

1. Put to the operator as the answer to his OD-4 follow-up (above), with the reviewers' evidence: GLM, the
   callback "fails exactly when the sender is down"; 055 M3, a nudger that kept nudging after a reply that came by
   the other path; Codex, push asynchronously from a durable outbox, pull repairs.
2. **Operator ruling: "yes".** Read back as: the receiver writes each step (RECEIVED, STORED, HANDED_OVER, REPLIED)
   to the hub record first (R-12.1 already copies every per-step event to the hub); the hub record is the single
   source of truth for a message's state; the call back to the sender is a fast notice only, one or two quick
   attempts, never a retry storm; a sender that was down reads the hub record for its open messages when it
   returns, and never re-sends a message whose STORED is already recorded. The OD-3 states (and "stuck") are
   computed from that record, so sender, receiver and cockpit see the same state. For circuits (OD-1 = C), each
   turn is copied to the receiver's home hub: the single log per conversation.
3. Authorises: R-14, R-15, R-26/R-27 (answer-ready pull) and R-35 aligned to record-first; nudgers and watchers
   compute from the record (fixes the AEF T-3804 class). Leaves open: stage names (OD-8), alarm surface (OD-14),
   telemetry retention (OD-16).

## OD-6 Already-running sessions (section 9.9) — ruled B

1. Asked 2026-10-04 with options A (relaunch every agent; Codex, GLM), B (no relaunch; harness-side pull at yield
   points, marked pull-only; 055), C (pull and session-start listing only), D (relaunch now, harness pull later).
2. Fact added by the orchestrator, checked in `.claude/settings.json`: every Claude Code session in this project
   already runs `fw hook checkpoint post-tool` after every tool call (PostToolUse, no matcher); the hook reads its
   script fresh each call, so a mail check in an already-registered hook reaches running sessions without a
   relaunch. Limits stated: idle sessions fire no hook; other projects need their own vendored hook (AEF 1.7.424
   ships sidecar hooks); whether Claude Code picks up newly REGISTERED hooks without a restart is unverified.
3. **Operator ruling: "b".** Read back as: B with relaunch-on-natural-restart, never forced. A mail check in the
   existing PostToolUse and Stop hooks delivers to every working session (transcript evidence); each session's
   next natural start goes through the reachable launcher, so idle sessions become pushable over time; until then
   an idle session without a typable terminal shows WAITING FOR RECIPIENT (OD-3 state).
4. Authorises: R-3 acceptance and status; a hook-delivery requirement with transcript evidence; the launcher as
   the default start path. Leaves open: readiness owner (OD-7); other harnesses (055 adapter).

## OD-7 Readiness: hooks or screen, and who owns it (section 9.10) — ruled C

1. Asked 2026-10-04 with options A (hooks first, screen classifier as labelled fallback), B (hooks only, each
   harness owner builds its own; ownership collector-synthesized), C (hooks through a harness adapter contract;
   055 and Codex), D (screen classifier as the main signal).
2. **Operator ruling: "c".** Read back as: a harness adapter contract with READY, BUSY, NOT RUNNING plus evidence
   of hand-over; the Claude Code adapter is the hooks already chosen in OD-2 and OD-6; 055's opencode adapter is
   the second; a parity test per release; the screen classifier is kept only as a diagnostic and never gates a
   hand-over. Proposed ownership (orchestrator's, no reviewer source; to be confirmed with OD-9): TermLink owns the
   contract, each harness's adapter comes from the team that runs that harness.
3. Authorises: R-21 (harness-neutral wording), R-22, R-24; the contract and its parity test as requirements.
   Leaves open: ownership of the whole receive side and packaging (OD-9).

## OD-8 Stage names, and "acknowledged, no action" (section 9.11) — ruled B

1. Asked with options A (one name INJECTED, the operator's word), B (one name HANDED_OVER, AEF's word), C (two
   names, today's split), D (keep names, add no-action only). All with transcript evidence required and ATTEMPTED
   for a typed line without evidence; reviewers unanimous on that and on a terminal no-action state.
2. Orchestrator recommended B over the output's A: after OD-2/OD-6 many hand-overs arrive through the hook channel,
   where "injected" (typing) would be inaccurate; HANDED_OVER fits both routes and is what AEF already emits.
3. **Operator ruling: "b".** Read back as: HANDED_OVER is the one name for "the agent's session received the
   content", proven from its transcript, by hook or by typing; "inject" names the action; a typed doorbell with no
   transcript evidence is ATTEMPTED (non-terminal marker); ACKNOWLEDGED_NO_ACTION is a terminal state beside
   REPLIED. Chain: SENT -> RECEIVED -> STORED -> HANDED_OVER -> REPLIED | ACKNOWLEDGED_NO_ACTION.
4. Authorises: R-24, R-28; the vocabulary proposed to AEF together with the five states (follow-up to
   framework:pickup offset 314). Leaves open: receive-side ownership (OD-9).

## OD-9 Who owns the receive side, and how sidecars ship (section 9.12) — ruled C, sequenced

1. Asked 2026-10-05 with options A (adopt AEF's receiver), B (extend TermLink's scripts), C (`termlink sidecar` in
   the binary, AEF supplying harness adapters; Codex), D (defer). Facts given: releases ship the binary only;
   TermLink's scripts are unpackaged and the injector unscheduled; AEF's `lib/sidecar` (1.7.424) runs with
   receipts; the T-3341 rehearsal showed a re-vendor would start AEF's receiver beside ours on the same inbox.
2. **Operator ruling: "c".** Read back as C, sequenced so there is exactly ONE receiver per inbox at every moment:
   2a. now: AEF's receiver is the one running receiver wherever AEF is installed (here after the T-3342
       re-vendor); TermLink's notify-sidecar scripts are retired from those inboxes;
   2b. build: `termlink sidecar` in the binary to the same contract (OD-3, 4, 5, 7, 8 rulings);
   2c. switch per host only after it passes the two-agent acceptance test with a negative control; AEF's
       receiver is then retired on that host.
   The sequencing is the orchestrator's addition (no reviewer source), accepted by the ruling.
3. Authorises: R-6, R-39; "one receiver per inbox" as a requirement; retiring the scripts in the T-3342 plan; a
   build task for `termlink sidecar` after the chain's planner step. Leaves open: the ownership split with AEF,
   to be proposed to them.

## OD-10 Session level of the address (section 9.13) — ruled C

1. Asked 2026-10-05 with options A (route by project and role; session id metadata), B (exact session only,
   fenced by incarnation; Codex), C (both, with a rule: exact-instance fails loudly, role re-resolves, never
   silently redirect; reviewer consensus), D (canonical plus runtime id as worded, no further rule).
2. **Operator ruling: "C".** Read back as: the address carries both ids (canonical plus runtime, as the operator
   worded it); a NEW request goes to project + role and the home hub resolves it to one live copy (how: OD-18);
   mail INSIDE a conversation goes to the exact copy the conversation is bound to (OD-1 circuits), and if that copy
   has ended the sender gets a dead letter (OD-3 DEAD), never a silent redirect; a conversation moves to another
   copy only by explicit hand-over with its context; a copy's runtime id is never reused (orchestrator's addition).
3. Authorises: R-32, R-33, R-34; fail-loudly and never-redirect as requirements. Leaves open: role resolution
   (OD-18), the name-to-id directory (OD-11).

## OD-11 Name-to-id directory (section 9.14) — ruled A, with the operator's location insight

1. Asked 2026-10-05 with options A (the hub keeps project identity cards), B (no directory, ids inside names),
   C (AEF keeps it), D (each project keeps its own). Orchestrator recommended A built as: the project id is minted
   once by the project's framework (AEF's pid) and never changes, names are display only; each home hub keeps a
   card per project filled only from authenticated registrations it observed (project id, roles, live copies,
   liveness, version); hubs exchange cards, never messages, and never re-announce another hub's cards; only the
   home hub may say dead, anything it cannot confirm is unknown.
2. **Operator's additional insight (raised before ruling):** "directories of course can change ... I can have a
   same project which is copied to another project, a directory, even on another host, or I could rename the
   directory." Answered and folded in:
   2a. Identity is the minted pid, written into the project's own `.framework.yaml`, so it travels with the files.
       Folder path and host are OBSERVED ATTRIBUTES on the card, never the identity (cf. T-2815: a path-derived
       name is wrong inside a git worktree).
   2b. Folder renamed or moved on the same host: same pid, same project; the card updates path and display name.
   2c. Moved to another host: the same pid registers at the new host's hub, which becomes its home hub; the old
       hub sees no live copies, marks the project "moved to hub X" and stops answering for it; senders follow the
       new card; no silent redirect.
   2d. Copied and both copies running: two places carry the same pid. By default the copy is a SECOND INSTANCE of
       the same project (own instance ids, OD-10; role mail per OD-18). A copy meant as a NEW project (a fork)
       must re-mint its own pid.
   2e. The first time a hub sees a pid in a new place while the old place is still alive, it does not guess: it
       flags "same project id seen at X and Y" to the operator and the cockpit; the copy is treated as a second
       instance until declared a fork and re-minted. This stops a fork from receiving the original's mail.
   2f. Runtime ids of sessions and agents belong to the running copy and are never reused (OD-10), so moving or
       copying never makes an old address reach a new copy.
3. **Operator ruling: "A. That's good, but we have additional insight on A. Is that recorded then too now with
   this?"** Read back as: A, including 2a-2f as part of the ruling. Recorded here and in the task's Decisions.
4. Authorises: R-33; the directory, card-exchange and location rules (2a-2f) as requirements; proposing the charter
   rewording the exchange needs ("hubs never sync messages; hubs may exchange a directory of whom they serve"),
   which the operator approves separately via T-2470. Leaves open: relay of addressed mail between hubs as a
   fallback; OD-18 role lease; the re-mint command for a fork (AEF owns pid minting; to be proposed to AEF).

## OD-12 Address rulings D-599/D-660 and stable ids (section 9.15) — ruled D

1. Asked 2026-10-05 with options A (D-599 stands, ask AEF for amended D-660, end the `sidecar:` alias),
   B (D-660 stands), C (both indefinitely), D (D-599 plus a stable hub canonical id and the fingerprint as instance).
2. Facts given: every live inbox is in D-599 form `inbox:<hub-id>/<project>`; `sidecar:` topics still exist and are
   posted to (sidecar:999 holds 30 messages on the canonical hub); the hub id in inbox names is the first 16 hex of
   the TLS fingerprint (verified with `hub probe`), so a certificate regeneration (PL-021 class) would silently rename
   every inbox on that hub. Orchestrator recommended D because OD-10/OD-11 apply "stable minted id plus observed id"
   at the session and project levels and the hub level was the one inconsistency left.
3. **Operator ruling: "D".** Read back as D including A's actions:
   3a. D-599's circuit form stands; AEF is asked for the amended D-660 text so both records agree;
   3b. each hub gets a canonical id minted once and stored with its runtime state, used in inbox names; the TLS
       fingerprint becomes the hub's instance id, observed on its card;
   3c. migration: today's fingerprint-based ids become the existing hubs' canonical ids, so nothing is renamed now;
       only a future rotation stops renaming inboxes;
   3d. the `sidecar:` alias ends one release after the T-3342 re-vendor; until then mail posted there is forwarded
       and the sender is warned.
4. Authorises: R-32, R-33; a hub-identity requirement; the alias end date; the request to AEF. Leaves open: where and
   how the hub id is minted (architect, step 4).

## OD-13 Section 14 items (section 9.16) — ruled A, refined

1. Asked 2026-10-05 with options A (keep R-41, R-42 as later slices, drop R-43, R-40 conditional), B (keep all),
   C (drop all), D (keep only R-40). Orchestrator refined A from the operator's later rulings.
2. **Operator ruling: "A".** Read back as the refined A:
   2a. R-40 (native consumer) is satisfied by OD-2, OD-6 and OD-8 and merged into R-28 (reply or no-action);
   2b. R-42 is promoted to a requirement now, as part of OD-5: nothing, hub or tool, reports "delivered" without a
       recorded HANDED_OVER; until then it says "accepted" or "stored";
   2c. R-41 (typed assignment/result messages) is kept as a later slice, with Codex's fields for messages that hand
       over work: an expiry, authorisation checked at execution time, a fencing token;
   2d. R-43 is dropped as a duplicate of R-9.
3. Authorises: R-40..R-43 updated as above. Leaves open: the R-41 slice (planner, step 8).

## OD-14 Alarms, escalation and the last rung (section 9.17) — ruled B

1. Asked 2026-10-05 with options A (urgent alarm + pile-up escalation + digest, no canary), B (A plus an end-to-end
   canary), C (a hub-steward agent, T-3333), D (defer). The step-1 document recommended B and left open whether a
   canary failure is an alarm and where the last rung lands; the orchestrator proposed answers from what exists.
2. Facts given: three failures fired nothing — 3 October (mail stored, never surfaced), the stray second hub, and the
   claude-termlink-alt sidecar (healthy heartbeat, 32,205 refused confirmations, fixed in T-3346). Existing parts:
   ~20 cron canaries, `/canaries`, the session-start "needs attention" list (T-3327), and the on-demand provers
   `comms-selftest.sh` and `session-message-selftest.sh`.
3. **Operator ruling: "b".** Read back as B with the proposed answers:
   3a. a daily end-to-end message canary between two real agents, checking RECEIVED, STORED, HANDED_OVER in the
       receiver's transcript, and REPLIED, against the OD-3 stuck deadlines;
   3b. a canary failure is an escalation entry, never an alarm; the urgent-only alarm rule stands;
   3c. the last rung lands in the main agent's session-start "needs attention" list (role holder per OD-18), plus
       `/canaries`, the daily digest and the 055 cockpit view when it exists;
   3d. the build proves the landing with a negative control: stop the injector, the entry appears by the next session.
4. Authorises: R-37 changed accordingly; the canary task; the landing place with its negative-control test.
   Leaves open: the channel by which the urgent alarm reaches the operator; the hub steward (T-3333) stays parked.

## OD-15 Interrupt consent, respawn and the startup chain (section 9.18) — ruled A, refined in dialogue

1. Asked 2026-10-05 with options A (interrupts only from allowed authenticated senders; respawn only under an operator
   grant with budget and restart limits), B (no consent layer), C (operator approves each interrupt), D (never
   interrupt). Facts given: OD-2 B already makes urgent arrive via the hook, never typed; OD-6 B forbids forced
   relaunch; senders are verified by signing key (T-1427); per-agent keys exist (runme action 8, T-3346); R-9 unbuilt.
2. Operator dialogue (two turns):
   2a. agrees with A; starting an agent that is not running needs operator approval, with pre-approved situations
       recorded (Tier 2 = one-off approval, Tier 3 = pre-approved category); the agent may propose pre-approval when
       a situation recurs;
   2b. distinguish addressing something not running (role, project, dead session: may be started) from an exact
       agent that existed and is gone (not a valid start);
   2c. but an agent that died mid-conversation should be started again to reread its context and pick up.
   Orchestrator: agreed; the line is "resume the same conversation, never substitute a fresh copy" — in Claude Code
   the transcript is the conversation state and `--resume <session-id>` reloads it.
3. **Operator ruling: "yes"** to A refined:
   3a. mid-turn urgent delivery only from senders the receiver allows by verified key (default: own project plus the
       operator); others are downgraded to normal, never dropped;
   3b. mail may start an agent only for role or project addressing with nothing live, under a Tier-2 approval or a
       Tier-3 grant (budget, restart limit, allowed senders);
   3c. an exact instance that died mid-conversation is resumed (same transcript) only if the home hub says DEAD
       (never UNKNOWN), the conversation is open, the transcript is resumable, and a grant covers it; a resume
       counts against the restart limit, and a second death on the same message flags it to the operator while the
       sender sees STUCK;
   3d. an instance that ended cleanly or cannot be resumed: dead letter, the sender sees DEAD, and re-addressing to
       the role is the sender's choice; a fresh copy never answers conversation mail;
   3e. the agent proposes a Tier-3 grant after 3 recurring Tier-2 approvals; the operator approves;
   3f. R-9 stays as confirmed; its "start the agent" step follows 3b to 3d.
4. Authorises: R-9, R-19 and the consent rules beside R-8; an allow-list requirement; a resume requirement; the
   grant format. Leaves open: grant fields and storage (architect, step 4); the IAC-72 narrowing of R-9; how urgent
   is marked (CAND-4, OD-17).

## OD-16 Telemetry retention window (section 9.19) — ruled C, refined

1. Asked 2026-10-05 with options A (14 days with ceilings), B (30 days), C (per class: delivery events short, digests
   long), D (keep until the digest consumed them).
2. Correction given first: the step-1 document called 14 days an existing operator ruling; T-3304 IW-2 records it as
   the agent's recommendation for channel topics, assumed accepted. Conflict found by the orchestrator: the OD-3
   ladder polls a waiting message for up to 2 years, so a flat window deletes the journey of a still-open message
   before it completes.
3. **Operator ruling: "C".** Read back as the refined C:
   3a. journey events are kept until 14 days after the message reaches a final state (REPLIED,
       ACKNOWLEDGED_NO_ACTION or DEAD), never cut while the message is open;
   3b. digests are kept 1 year, an explicit forever-class exception with owner and reason (IW-2 rule);
   3c. IW-2 ceilings apply: past a count or size ceiling the oldest final events are trimmed first, loudly; open
       messages only as a last resort, with a needs-attention entry.
   The numbers (14 days, 1 year) are the orchestrator's, assumed accepted; they can be overturned.
4. Authorises: R-35's retention clause; the digest retention exception; correcting OD-16.d (done, OD-16.d.1).
   Leaves open: ceiling values (architect, step 4).

## OD-17 Proposed requirement changes (section 9.20) — walked one candidate at a time

1. The table has 19 candidates. Settled by earlier rulings, recorded without re-asking: CAND-5 (OD-14), CAND-7 (OD-7),
   CAND-9 (OD-10), CAND-11 (OD-15), CAND-15 (OD-11, OD-3). CAND-16 is live because OD-1 chose circuits. 13 walked.
2. **CAND-1 peer content is untrusted — ruled A** (2026-10-05).
   2a. Accepted as a P1 security invariant: peer text is delivered framed as untrusted data; a request from a peer is
       a task proposal, never direct execution (AEF D-695; extends the pickup rule G-020/T-469 to all agent mail).
   2b. A reply to something the receiver itself asked for is still data but may be acted on within the receiver's
       own task.
   2c. Operator queried the usability score (−1); orchestrator corrected it to 0 (the core rule already requires a
       task; own-request replies exempt; delegation goes through claims). A +30 → +35, order unchanged. The operator
       confirmed the correction and then ruled "A".
   2d. Leaves open: framing markers (adapter, OD-7); threat coverage (step 2).
3. **CAND-2 at-most-once acceptance — ruled A, refined in dialogue; CAND-19 merged in** (2026-10-05).
   3a. Facts given: hub dedupe holds (sender, id) for 5 minutes only (`dedupe.rs:41`, T-2049); nothing receiver-side
       looks at `client_msg_id`; AEF D-600 retries up to monthly and OD-3 keeps a message alive up to 2 years.
   3b. Operator: it must be stored right; if lost before storing, ask for retransmit; if it still has to be
       injected it must still be injected; without an answer we still chase it — "not that straightforward, cut and
       dry, one-dimensional".
   3c. Orchestrator agreed: idempotence per stage, not per message. Ruled as: a stage memory per message id (the
       settlement record, at the hub per OD-5) with one stored copy, one hand-over, one reply; a duplicate before
       STORED is stored (retransmit works); a known id with lost content triggers a resend request; STORED but not
       handed over: not stored twice, hand-over continues, sender told the stage; HANDED_OVER without reply: never
       handed over twice, sender told when; REPLIED: the reply is returned again. Chasing stays with OD-3 and OD-14;
       a duplicate never resets it. Retention per OD-16. CAND-19 merged, not asked again.
   3d. Leaves open: same id with different content (step 2).
4. **CAND-3 the closing rule — ruled A revised, with staging** (2026-10-05, after external review).
   4a. Dialogue: first proposal "two real agents for every delivery requirement"; operator: too heavy; tiered
       draft (fixture, live hub, two agents); operator: a one-agent tier is missing; four tiers; operator asked for
       external review. Codex (16 findings) and GLM (13) agreed with each other: select per failure mode, tier 1
       overclaims, check binding not identity, add topology/harness/crash/installed-system dimensions, kill-checked
       controls, continuous proof. GLM: the draft "would have passed all three cited failures".
   4b. Operator corrected the scoring: usability is value to the user, not test effort; scores re-done (A +63,
       C +21, B +2, D −21); effort belongs to the cost axis and is handled by staging.
   4c. Ruled: per-failure-mode selection; four tiers (fixture; live hub; one real agent + scripted peer through the
       installed scheduler and hooks, per harness; real agents for round trips incl. three-agent many-to-many,
       no-action and resume-refused); dimensions two hubs, crash/reboot, installed system; kill-checked negative
       controls with the three real failures as standing controls; continuous canary from the installed artifact,
       rotated, staleness detected, re-run after rotation, re-vendor and reboot; mutants per push only for
       invariants. Staging: each dimension required once its subject exists; all before arc-011 closes.
   4d. Flagged for the write-up: R-29.e "fresh heartbeat and no flag = CLEAR" contradicts the 32,205-refusal failure.
   4e. Leaves open: test estate and agent pairs (step 5).
5. **CAND-4 how urgent is marked — ruled A** (2026-10-05).
   5a. Facts given (code check): sender sets metadata `priority`, default 0; receiver clamps to −9..9
       (`journal-mirror.sh:164`); queue ordered by priority then time (`notify-sidecar-api.sh:142`); urgent threshold
       5, per-receiver override (`notify-injector.sh:98`); no `--priority` send option; AEF does not use priority.
   5b. Ruled: priority −9..9, default 0, clamped at the receiver; urgent at 5 or more by default (threshold
       confirmed), receiver may change its own; urgency honoured only from allowed senders per OD-15, others
       downgraded, never dropped. Follow-ups: `--priority` in the send tooling; ask AEF to carry the field.
   5c. Leaves open: revisit the threshold after the canary has run; attention budgets (CAND-12).
6. **CAND-6 reachability as a visible per-agent state — ruled A with 16e** (2026-10-06, "proceed as suggested").
   6a. Facts given: every existing signal (sidecar, presence and waker heartbeats, waker canary) proves a process is
       alive, not that mail reaches the agent; the claude-termlink-alt sidecar had a seconds-old heartbeat while
       refusing 32,205 confirmations; R-29.e accepts "fresh heartbeat and no flag" as CLEAR.
   6b. Operator asked whether this is the same back channel as the sidecar telemetry, given that established
       conversations run directly. Confirmed from OD-1 C and OD-5: the hub sets up the circuit, conversations run
       sidecar to sidecar, step events and a copy of each turn go to the home hub outside the message path (R-35.e);
       the reachability fields are computed from that same back channel.
   6c. Ruled: four fields on the home-hub card (receiver up; right hub as a binding check; adapter present; last
       surface time = latest real HANDED_OVER or confirmation, never a heartbeat); readable by peers under OD-15 and
       by the operator, needs-attention when a field goes bad; R-29 CLEAR needs recent surfacing progress (R-29.e
       corrected); computed from the existing back channel, no new channel; a silent back channel shows UNKNOWN,
       never DEAD.
   6d. Leaves open: the no-surfacing threshold for an idle agent (step 4).
7. **CAND-8 explicit mail-hub setting — ruled A with 14e–14h, P1** (2026-10-06).
   7a. Facts given: TERMLINK_RUNTIME_DIR does two jobs (local files and, implicitly, which hub carries mail); the
       stray hub of 2026-10-04 split 32 sessions this way; T-3340 fixed the local case by better guessing; T-3345
       lets a client read a hub's id over an authenticated call. Orchestrator recommended P1 against the document's
       P2 because CAND-6's "right hub" needs a declared answer.
   7b. Operator asked: hub crash, a new hub, several instances, an agent on another host. Answered from OD-11/OD-12
       and the R-34 principle (a wrong delivery is worse than a visible failure).
   7c. Ruled: projects declare their mail hub by canonical id plus address; clients verify via `hub_id` and refuse on
       mismatch; runtime dir for local files only. 14e restart or cert rotation keeps the id; 14f a new hub has a new
       id, clients refuse, recovery is a restore or an operator-approved re-home via runme with mail rescue; 14g the
       same id in two places is flagged, never guessed; 14h agents on any host reach the declared home hub over
       authenticated TLS, never a local fallback, and a project move includes re-homing ("moved to X").
   7d. Leaves open: declaration file and format (step 4).
8. **CAND-10 sender-side send-and-wait — ruled A (yield-and-wake), P1** (2026-10-06).
   8a. Facts given: the operator asked for it on 2026-04-26 (T-243, "send and wait instead of immediate response")
       and 2026-05-25 (T-1800); `termlink agent ask` blocks up to 30 s; `--await-ack` waits for delivery, not an
       answer; the delivery half (reply handed over into the sender's session) is already required.
   8b. Ruled: a send may carry "awaiting reply by <deadline>"; the sender yields; the reply is handed over into its
       session linked to the conversation; past the deadline the sender is woken with STUCK; an unwakeable idle
       sender shows WAITING FOR RECIPIENT; a short blocking wait remains for quick exchanges.
   8c. Bias noted by the orchestrator: second P1-above-document recommendation in a row.
   8d. Leaves open: default deadline (OD-3's 1 h natural); late replies after STUCK still wake, marked late.
   8e. Ownership (operator, 2026-10-06, during GP-11): the tool belongs to AEF, where messages are formed and agents
       receive instructions; TermLink supplies primitives (the missing cross-hub stage/reply read is T-3347). It polls
       on the message's own ladder when push cannot land (R-30/R-31), never a fixed cadence. Proposed to AEF at
       framework:pickup offset 318 together with the ladder correction of offset 314.
9. **CAND-12 attention budgets — ruled A (source control), refined in dialogue** (2026-10-06).
   9a. Operator reframed the risk as flooding (re-sending too often, piling up) and asked for ladder adherence, a
       cumulative per-agent cap derived from system capacity, and back-off against storms borrowed from networking
       (CSMA/CD, spanning tree). Along the way OD-3 was corrected to two ladders by priority (see OD-3, point 6).
   9b. Cap derivation given: the hub is not the bottleneck (1,000 open messages at the 15 s rung ≈ 67/s, within the
       governor's 1,000/s); receiver attention is: cap ≈ answer rate × answer deadline × peers ≈ 20 × 1 h × 5 ≈ 100,
       a guess until telemetry; then adaptive like TCP's congestion window.
   9c. Ruled: P1 ladder-governed re-sends with jitter, early re-sends refused; P1 cumulative per-agent cap starting
       at 100, adaptive, overflow waits at the sender and shows in needs-attention; P1 storm prevention (hop limit,
       circuit breaker, coalesced hand-overs, and the three message classes below); P2 expiry, cancellation,
       receiver-advertised window.
   9d. Operator's definitional correction: stage confirmations are automatic replies too and must keep flowing.
       Three classes, confirmed "yes": (1) protocol receipts always flow, are never answered, do not count against
       the cap; (2) automatic content carries the marker, gets receipts, never triggers an automatic content reply,
       counts against the cap, hop-limited, may be answered deliberately; (3) deliberate content, normal rules.
       Nothing automatic answers anything automatic; receipts answer nothing; only an agent's decision creates
       content.
   9e. Leaves open: jitter width, adaptive-cap parameters, hop-limit value (step 4); telling AEF.
10. **CAND-13 clock skew and version skew — ruled A** (2026-10-06).
    10a. Facts given: urgent accept deadline 15 s and cross-host telemetry make skew matter; .107 NTP-synchronised,
         nothing checks other hosts; hubs report version and protocol version (T-3345), protocol 1 fleet-wide;
         the protocol-too-old error is unwired (T-2700, operator's own decision, not taken here).
    10b. Ruled: P1 every deadline measured on one clock (the observer's own), never across hosts; P2 NTP required
         and checked, telemetry tagged with the stamping host and a skew estimate, cross-host delays flagged past a
         bound; P2 versions on the agent card, "old" distinct from "deaf", incompatible protocol refused loudly.
    10c. Leaves open: skew bound (step 4); T-2700; 055's "one version estate-wide".
11. **CAND-14 obligation contract and visible unanswered state — ruled A** (2026-10-06).
    11a. Facts given: ring20-manager's 8 unanswered requests (RV2 items 50, 52; cause not in the record); the sender
         side is already covered by R-28/OD-8, OD-3 STUCK, OD-14 escalation and CAND-10; the receiver has no list of
         what it owes.
    11b. Ruled: sender-side unanswered state recorded as covered; new P1 owed-answers list per agent from the hub
         record, shown at session start/resume and in needs-attention; work-request obligation states (accepted/
         declined, progress deadline, completed/failed) into R-41's later slice with the CAND-1 task proposal as
         "accepted".
    11c. Leaves open: escalation to another role holder (OD-18).
12. **CAND-16 per-circuit credentials, one delivery contract — ruled A: settled by OD-1** (2026-10-06).
    12a. OD-1 point 3 already made these conditions of the circuit; point 4 made them requirements and sent the
         trust model to step 2. Recorded as P1 requirements of the circuit slice, after identity, directory and
         binding. Persist-before-ack is OD-5, dedupe is CAND-2, sequence numbers are CAND-18.
    12b. Leaves open: credential lifetime and binding (step 2).
13. **CAND-18 ordering by sequence number — ruled A, refined in dialogue** (2026-10-06).
    13a. Operator checked his understanding (message reconstruction, pick-up in a conversation, checking and
         identifying sequences: confirmed with a worked example), then set the principle: hubs may keep different
         ledgers, the source of change is the sender, sequencing as in TCP. Orchestrator explained TCP (sender-
         assigned SEQ, cumulative ACK, SACK, duplicates by number, random ISN), SCTP (TSN plus per-stream SSN, no
         head-of-line blocking across streams) and Kafka's idempotent producer (producer id, sequence, epoch);
         operator found the SCTP/Kafka mapping "very good".
    13b. Operator asked whether sender numbering and hub numbering in parallel, with the truth read from the sender's,
         is logical. Confirmed: the sender's number is the truth about the conversation; the hub's offset is the
         truth about that hub's storage and arrival; the hub record maps one to the other; the receiver still checks
         the sender's numbers (gaps, duplicates, reused numbers), since a number is a claim like any peer content.
    13c. Ruled: P1 sender-assigned per-conversation number, durable, never reused; P1 hub offsets local only, mapped
         in the hub record; P1 cumulative `up_to` per conversation in receipts, receiver flags gaps, duplicates and
         reused numbers; with the circuit slice, bounded buffering, gap reports with resend, resume from the last
         acknowledged number. Order per conversation only; non-conversation messages by hub arrival.
    13d. Leaves open: gap-wait bound (step 4); same number with different content (step 2).
14. **OD-17 complete** (2026-10-06): 12 candidates walked and ruled; CAND-5, -7, -9, -11, -15 settled by earlier rulings;
    CAND-19 merged into CAND-2.

## OD-18 Several agents per project: who answers, who coordinates (section 9.21) — ruled A revised, after external review

1. Asked 2026-10-06 with options A (deterministic rule at the home hub over a fenced lease held by code; introduce,
   then step aside), B (a central coordinator agent carrying the traffic), C (no exclusivity, first claim wins),
   D (the sender names the exact instance). Background: the operator's two options of 2026-10-04 (one central agent,
   or a coordinator function that another agent takes over), merged by the round-3 reviewers (RV2 items 43-51);
   ring20-manager's 8 unanswered requests; hub claims with leases exist (T-2019, T-2046) without a generation.
2. Operator asked for review by the subscription reviewers. Codex and GLM (16 findings each,
   `docs/reports/T-3344-od18-review/`) agreed: a sidecar-renewed lease proves the process is alive, not that the agent
   can take a turn (the 32,205-refusal sidecar would hold "main" forever); busy is not dead; fence at role state
   changes, not per message; generation never reused; accepted work is not moved blindly; operator pin;
   vacancy goes to the operator; OD-14's last rung is circular when main is the failure; ship "main" first.
3. **Operator ruling: "A"** (revised): only "main" now; readiness-gated lease (adapter ready plus real hand-over or an
   idle readiness check); busy keeps main, takeover only after lapse plus quiet period plus cooldown, no automatic
   take-back; durable generation checked at role state changes, old holder told and releases on clean shutdown,
   its conversation replies stay valid, unfenceable external effects reconciled; unaccepted role requests move,
   accepted ones stay, a dead holder's obligations classified with uncertain ones to the operator; selection by
   operator pin, else healthy incumbent, else priority then reachability then stable id, eligibility attested on
   the card, first holder appointed at setup; vacancy "unassigned" to the operator, starts only under an OD-15
   grant; two live copies give "authority unknown"; remote senders resolve via the verified home hub; visible "who
   holds main", audited override; two standing tests (lapse during an open obligation; stuck holder with a live
   sidecar) must end in a visible takeover.
4. **OD-14 amended by this ruling:** the last escalation rung lands in main's list and also goes to the operator or
   cockpit whenever main is unassigned, failing or being taken over.
5. Authorises: R-32, R-34 changes; new requirements for "main"; the OD-14 amendment; the two standing tests.
   Leaves open: lease duration, quiet period, cooldown, idle readiness check, priority configuration format (step 4).

## Gap review (section 8) — walked one item at a time

1. **GP-0 adversary list — ruled A** (2026-10-06). ADV-1..ADV-7 (section 2.3) confirmed as written. Added: ADV-8 a
   flooding or looping peer (CAND-12); ADV-9 a stale or partitioned authority holder (OD-18); ADV-10 estate drift —
   skewed clocks, mixed versions, a re-vendor that deletes local fixes (CAND-13, G-062). ADV-6 widened to a misfiled
   signing key (T-3346) and two live copies of one project (OD-11). ADV-8..10 are the collector's proposal from this
   interview's rulings. Step 2 may add more.
2. **GP-11 operator as a party — open** (2026-10-06). Operator corrected the first brief: out-of-band channels already
   exist (ntfy, Signal, Mattermost, Watchtower, runme). ring20-manager's live inventory received (six channels, three
   two-way; Mattermost desk = where decisions should go). Questions out to AEF, Penelope (050) and ring20-dashboard on
   conversation operator-out-of-band-channels. At the operator's request an URGENT standardisation request went to
   AEF (framework:pickup offset 319) before he asked to wait for all four answers; AEF was told to hold triage until a
   supplement with the full inventory (inbox offset 561).
3. **GP-12 a measurable "very simple" — ruled A revised, after external review** (2026-10-06).
   3a. First recommendation: five structural proxies (one process per agent, one install command, one status call,
       ~10 API calls, a 1,000-line cap). Operator asked for external review: Codex and GLM (16 findings each,
       `docs/reports/T-3344-gp12-review/`) agreed the proxies measure packaging — the misfiled key, the waker
       exclusion and the runtime wrong hub pass all five — and that counts are gameable.
   3b. Ruled: acceptance by verified behaviour — truthful status call with separately verified, freshness-stamped
       facts and an end-to-end probe; the five real failures as standing fault injections with a specific diagnosis
       in a declared time; automatic recovery with zero manual steps; drilled diagnosis time; clean install with
       upgrade, rollback and restart. Inventories of components, stores, identities/keys, API operations and message
       states, additions justified. Size and counts are growth tripwires only. One sidecar per agent stays.
   3c. Leaves open: diagnosis-time bound and state cap (step 3); applying it to AEF's Python sidecar.


### 2026-10-06 — progressive insight: the chase loop is sidecar code, not an LLM turn (operator)

Operator (voice, restated and accepted): the sender-side chase of an unanswered message follows the two
ladders (OD-3 correction: normal = each rung twice from 1 min; urgent = continuous from 15 s), and it must be
**an API call on the sidecar, never an LLM turn** — calling the model to poll is very wasteful and slow. The loop
is **vendor-neutral**: a cron job, or a one-shot script that on each run computes the next rung and reschedules
itself (self-rescheduling, not a resident model loop). When a reply arrives it reaches the agent by the
normal path, prompt injection. Ownership: AEF, together with send-and-wait (pickup 319); goes into the 319
supplement. The interim `scratchpad/ladder-watch.sh` used in this session is the right shape but runs inside
the agent's harness; the requirement is that it runs without one.
