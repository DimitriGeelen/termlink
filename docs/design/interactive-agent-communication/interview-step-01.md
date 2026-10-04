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
