# arc-011 step 1: summary of the operator's rulings (T-3344)

One document that lists every ruling the operator gave in the arc-011 step-1 interview (2026-10-04 to 2026-10-06).
It is a summary. The record of each ruling is the interview log; when the two differ, the log wins.

| Version | Date | Change | Task |
|---|---|---|---|
| 0.1 | 2026-10-06 | First version: all rulings OD-1..OD-18, OD-17 candidates, gap review GP-0/GP-11/GP-12 | T-3344 |

## 0 Sources

1. Interview log, the record of every ruling:
   [`docs/design/interactive-agent-communication/interview-step-01.md`](../design/interactive-agent-communication/interview-step-01.md).
   Each item below cites its section there as "log § OD-n".
2. The same rulings as task decisions (Chose / Rejected / Left open):
   [`.tasks/active/T-3344-arc-011-role-chain-step-1-requirements-c.md`](../../.tasks/active/T-3344-arc-011-role-chain-step-1-requirements-c.md), section Decisions.
3. The step-1 output, v0.3, which does NOT yet contain these rulings (folding them in is the next task):
   [`docs/design/interactive-agent-communication-01-requirements.md`](../design/interactive-agent-communication-01-requirements.md)
   (R-1..R-43; section 8 gaps; section 9 open questions; section 9.20 the CAND table).
4. The role chain, 8 steps:
   [`docs/design/interactive-agent-communication-role-chain.yaml`](../design/interactive-agent-communication-role-chain.yaml).
5. External reviews (each folder holds `brief.md`, `codex.md`, `glm.md`, `comparison.md`):
   5a. [`docs/reports/T-3344-step1-review/`](T-3344-step1-review/) — review of the step-1 output itself (two rounds);
   5b. [`docs/reports/T-3344-cand3-review/`](T-3344-cand3-review/) — the closing rule, CAND-3;
   5c. [`docs/reports/T-3344-od18-review/`](T-3344-od18-review/) — several agents per project, OD-18;
   5d. [`docs/reports/T-3344-gp12-review/`](T-3344-gp12-review/) — a measurable "very simple", GP-12;
   5e. [`docs/reports/T-3335-design-review/comparison.md`](T-3335-design-review/comparison.md) and
       [`docs/reports/T-3335-routing-consult/comparison.md`](T-3335-routing-consult/comparison.md) — the design
       reviews that fed step 1.
6. Messages to AEF: `framework:pickup` offset 314 (ladder v2 and message states, now partly superseded), offset 318
   (two ladders by priority replacing D-600 for agent mail; proposal that AEF owns send-and-wait), offset 319
   (URGENT: standardise operator out-of-band channels; AEF asked to hold triage until a supplement).

## 1 Delivery chain and message states

1. **OD-4 RECEIVED and STORED — A** (log § OD-4). Two stages: RECEIVED = arrived, not yet safe, informational;
   STORED = durably saved, the only stage that releases the sender. On the hub path both may travel in one message
   with two timestamps. OD-3's "not accepted" splits into "not received" and "received, not stored".
2. **OD-5 Hub record is the truth — ruled "yes"** (log § OD-5). The receiver writes each stage to the hub record
   first; the call back to the sender is a fast notice (one or two attempts, no retry storm); a returning sender
   reads the record and never re-sends a message already STORED. States are computed from the record, so sender,
   receiver and cockpit see the same state. For circuits each turn is copied to the receiver's home hub.
3. **OD-8 Stage names — B** (log § OD-8). Chain: SENT -> RECEIVED -> STORED -> HANDED_OVER -> REPLIED |
   ACKNOWLEDGED_NO_ACTION. HANDED_OVER is proven from the receiver's transcript (by hook or by typing); a typed line
   without transcript evidence is ATTEMPTED (non-terminal).
4. **OD-3 Message states and "stuck"** (log § OD-3, point 3). Five sender-visible states: WAITING, WAITING FOR
   RECIPIENT, STUCK (next step overdue AND the responsible party is reachable: not accepted within 1 min; not
   handed over within ~1 min of the agent being ready; not answered within 1 h), UNKNOWN (never treated as dead),
   DEAD (only the home hub may declare it). Urgent: accepted in 15 s, handed over at the next tool call or turn
   end, answered in 5 min.
5. **OD-13 Section 14 items — A refined** (log § OD-13). R-40 merged into R-28; R-42 promoted now: nothing reports
   "delivered" without a recorded HANDED_OVER; R-41 (typed work messages with expiry, execution-time authorisation,
   fencing token) is a later slice; R-43 dropped as a duplicate of R-9.
6. **CAND-2 at-most-once per stage — A refined, CAND-19 merged** (log § OD-17 item 3). A stage memory per message
   id at the hub: one stored copy, one hand-over, one reply. Duplicates before STORED are stored; a known id with
   lost content triggers a resend request; later duplicates are told the stage. A duplicate never resets chasing.
7. **CAND-18 sender sequence numbers — A refined** (log § OD-17 item 13). Sender-assigned per-conversation number,
   durable, never reused, is the truth about the conversation; hub offsets are local and mapped in the hub record;
   receipts carry a cumulative `up_to`; the receiver flags gaps, duplicates and reused numbers. Circuits add
   bounded buffering, gap reports with resend, resume from the last acknowledged number.
8. **CAND-14 owed-answers list — A** (log § OD-17 item 11). New P1: per agent, the list of what it owes, from the hub
   record, shown at session start/resume and in needs-attention. Work-request obligation states go into R-41.
9. **OD-16 Telemetry retention — C refined** (log § OD-16). Journey events kept until 14 days after the message is
   final, never cut while open; digests 1 year (declared forever-class exception); IW-2 ceilings trim oldest final
   events first, loudly.

## 2 Ladders, urgency and flooding

1. **OD-3 Polling ladder, corrected 2026-10-06: two ladders by priority** (log § OD-3, point 6).
   1a. Normal (priority below 5): each rung twice — 1 min, 5 min, 15 min, 1 h, 4 h, 1 day, 3 days, 1 week, 1 month,
       1 quarter, 1 year (22 polls, a little over two years).
   1b. Urgent (priority 5 or higher): the continuous ladder 15 s … 2 years (43 polls).
   1c. A message's ladder also governs its re-sends before STORED; for agent mail this replaces AEF's D-600
       (sent to AEF at offset 318).
2. **OD-2 Urgent into a busy prompt — B** (log § OD-2). Nothing is ever typed into a busy prompt; urgent content
   reaches a busy agent through the harness's hook channel (next tool call or turn end); an idle agent gets the
   normal one-line inject. SQ-4 reconciled: urgent bypasses the wait, never the prompt-free check.
3. **CAND-4 how urgent is marked — A** (log § OD-17 item 5). Priority −9..9, default 0, clamped at the receiver;
   urgent at 5 or more by default (receiver may change its own); urgency honoured only from allowed senders
   (OD-15), others downgraded, never dropped. Follow-ups: a `--priority` send option; ask AEF to carry the field.
4. **CAND-12 flooding — A (source control) refined** (log § OD-17 item 9). P1 ladder-governed re-sends with jitter,
   early re-sends refused; P1 per-agent cap starting at 100 open messages, adaptive like a congestion window;
   P1 storm prevention (hop limit, circuit breaker, coalesced hand-overs); P2 expiry, cancellation,
   receiver-advertised window. Three message classes: protocol receipts (always flow, never answered, not
   counted); automatic content (marked, counted, hop-limited, never answered automatically); deliberate content.
   Nothing automatic answers anything automatic.
5. **CAND-10 send-and-wait — A (yield-and-wake), P1** (log § OD-17 item 8). A send may carry "awaiting reply by
   <deadline>"; the sender yields; the reply is handed over into its session; past the deadline the sender wakes
   with STUCK. Ownership (operator, 2026-10-06): the tool is AEF's; TermLink supplies primitives — the missing one
   is a cross-hub read of a message's stage and reply by id (T-3347). Proposed to AEF at offset 318.

## 3 Addressing, identity and hubs

1. **OD-1 Cross-host send path — C** (log § OD-1). Hubs set up a circuit; established conversations run
   sidecar-to-sidecar; the hub path is the fallback. Build order: identity hygiene and hub directory with liveness;
   conversations bound to an instance on the hub path; then the direct circuit with hub-minted short-lived
   per-circuit credentials and one delivery contract and one log per conversation on both paths.
2. **CAND-16 per-circuit credentials — settled by OD-1** (log § OD-17 item 12). P1 requirements of the circuit
   slice; trust model to step 2.
3. **OD-10 Session level of the address — C** (log § OD-10). The address carries canonical and runtime ids; a new
   request goes to project + role and the home hub resolves it; conversation mail goes to the bound instance, and
   if it has ended the sender gets a dead letter, never a silent redirect; runtime ids are never reused.
4. **OD-11 Name-to-id directory — A with location rules 2a-2f** (log § OD-11). Project id minted once by the
   project's framework and stored in `.framework.yaml`; path and host are observed attributes; each home hub keeps
   a card per project from authenticated registrations; hubs exchange cards, never messages; only the home hub may
   say dead. Moves, renames, copies and forks have explicit rules; the same pid seen in two live places is flagged,
   never guessed. Needs a charter rewording (operator approves separately via T-2470).
5. **OD-12 Hub canonical id — D** (log § OD-12). Each hub gets a canonical id minted once; the TLS fingerprint
   becomes its instance id; existing fingerprint ids become the canonical ids (nothing renamed now); the
   `sidecar:` alias ends one release after the T-3342 re-vendor, forwarded with a warning until then.
   (T-3345, done 2026-10-06, exposes `hub_id` in `hub.version`.)
6. **CAND-8 explicit mail-hub declaration — A, P1** (log § OD-17 item 7). Projects declare their mail hub by
   canonical id plus address; clients verify `hub_id` and refuse on mismatch; the runtime dir is for local files
   only; a new hub id means refuse and re-home via runme with mail rescue; no local fallback across hosts.
7. **CAND-13 clocks and versions — A** (log § OD-17 item 10). P1 every deadline on the observer's own clock; P2 NTP
   required and checked, telemetry tagged with host and skew estimate; P2 versions on the agent card, "old"
   distinct from "deaf", incompatible protocol refused loudly (T-2700 remains the operator's own decision).
8. **CAND-6 reachability as a visible state — A** (log § OD-17 item 6). Four fields on the home-hub card: receiver
   up; right hub; adapter present; last surface time (a real HANDED_OVER or confirmation, never a heartbeat).
   R-29.e "fresh heartbeat and no flag = CLEAR" is corrected: CLEAR needs recent surfacing progress.

## 4 Consent, start and resume, receive side

1. **OD-6 Already-running sessions — B** (log § OD-6). No forced relaunch; a mail check in the existing PostToolUse
   and Stop hooks delivers to working sessions; the reachable launcher on each natural restart; an idle session
   without a typable terminal shows WAITING FOR RECIPIENT.
2. **OD-7 Readiness — C** (log § OD-7). A harness adapter contract (READY, BUSY, NOT RUNNING plus hand-over
   evidence); Claude Code's adapter is the hooks; opencode (055) second; parity test per release; the screen
   classifier is diagnostic only. TermLink owns the contract; each harness team owns its adapter.
3. **OD-9 Receive side — C, sequenced** (log § OD-9). Exactly one receiver per inbox at every moment: now AEF's
   receiver where AEF is installed (TermLink's notify-sidecar scripts retired from those inboxes); build
   `termlink sidecar` in the binary to the same contract; switch per host only after the two-agent acceptance test
   with a negative control.
4. **OD-15 Interrupt consent, respawn, resume — A refined** (log § OD-15). Mid-turn urgent delivery only from
   allowed verified senders (default own project plus operator); mail may start an agent only for role/project
   addressing with nothing live, under Tier-2 approval or a Tier-3 grant; a dead exact instance is resumed (same
   transcript) only if the home hub says DEAD, the conversation is open and a grant covers it; a fresh copy never
   answers conversation mail; the agent proposes a Tier-3 grant after 3 recurring Tier-2 approvals.
5. **CAND-1 peer content is untrusted — A, P1** (log § OD-17 item 2). Peer text is framed as untrusted data; a
   peer's request is a task proposal, never direct execution; replies to the receiver's own requests may be acted
   on within its own task.

## 5 Several agents per project: "main"

1. **OD-18 — A revised after external review** (log § OD-18; review [`T-3344-od18-review/`](T-3344-od18-review/)).
   Only "main" now. A readiness-gated fenced lease at the home hub (adapter ready plus a real hand-over or an idle
   readiness check); busy keeps main; takeover only after lapse plus quiet period plus cooldown, no automatic
   take-back; durable generation checked at role state changes; unaccepted role requests move, accepted ones stay,
   uncertain obligations go to the operator; selection by operator pin, else healthy incumbent, else priority,
   reachability, stable id; vacancy "unassigned" goes to the operator; two live copies give "authority unknown";
   visible "who holds main" and an audited override.
2. **Two standing tests**: a lapse during an open obligation, and a stuck holder with a live sidecar, must both end
   in a visible takeover.
3. **OD-14 amended**: the last escalation rung also goes to the operator or cockpit whenever main is unassigned,
   failing or being taken over.

## 6 Alarms and verification

1. **OD-14 Alarms and escalation — B** (log § OD-14). A daily end-to-end canary between two real agents checking
   RECEIVED, STORED, HANDED_OVER (transcript), REPLIED against the stuck deadlines; a canary failure is an
   escalation entry, never an alarm; the last rung is main's session-start needs-attention list plus `/canaries`,
   the digest and the 055 cockpit; proven by a negative control (stop the injector, the entry appears by the next
   session). The hub steward (T-3333) stays parked.
2. **CAND-3 the closing rule — A revised, staged** (log § OD-17 item 4; review [`T-3344-cand3-review/`](T-3344-cand3-review/)).
   Tests selected per failure mode, in four tiers: fixture; live hub; one real agent plus scripted peer through
   the installed scheduler and hooks, per harness; real agents for round trips including three-agent many-to-many,
   no-action and resume-refused. Dimensions: two hubs, crash/reboot, installed system. Kill-checked negative
   controls with the three real failures (2026-10-03 unsurfaced mail, the stray hub, the 32,205-refusal sidecar) as
   standing controls. Continuous canary from the installed artifact. Each dimension required once its subject
   exists; all before arc-011 closes.

## 7 Simplicity

1. **GP-12 a measurable "very simple" — A revised after external review** (log § Gap review item 3; review
   [`T-3344-gp12-review/`](T-3344-gp12-review/)). Acceptance by verified behaviour, not counts: a truthful status call
   with separately verified, freshness-stamped facts and an end-to-end probe; the five real failures as standing
   fault injections each with a specific diagnosis in a declared time; automatic recovery with zero manual steps;
   drilled diagnosis time; clean install with upgrade, rollback and restart. Inventories of components, stores,
   identities/keys, API operations and message states, with additions justified. Size and counts are growth
   tripwires only. One sidecar per agent stays.

## 8 Operator leg (open)

1. **GP-11 operator as a party — OPEN** (log § Gap review item 2). Out-of-band channels already exist (ntfy, Signal,
   Mattermost, Watchtower, runme). ring20-manager's inventory received (six channels, three two-way; the Mattermost
   desk is where decisions should go). Answers pending from AEF, Penelope (050-email-archive) and ring20-dashboard
   on conversation `operator-out-of-band-channels`. AEF holds triage of offset 319 until a supplement carrying all
   four inventories. Then the operator rules.

## 9 Adversaries

1. **GP-0 adversary list — A** (log § Gap review item 1). ADV-1..ADV-7 (requirements section 2.3) confirmed. Added:
   ADV-8 a flooding or looping peer; ADV-9 a stale or partitioned authority holder; ADV-10 estate drift (skewed
   clocks, mixed versions, a re-vendor deleting local fixes). ADV-6 widened to a misfiled signing key and two live
   copies of one project. Step 2 may add more.

## 10 What is still open

1. GP-11 (section 8 above): three inventories outstanding, then the AEF supplement, then a ruling.
2. Gap review items not walked one by one: GP-1..GP-10 are answered by the ODs and CANDs named in their "Goes to"
   column (requirements section 8); GP-13 (durability failure scope), GP-14 (trusted evidence producers) and GP-15
   (adversary privileges) go to step 2.
3. The step-1 document has not yet absorbed any ruling. Corrections the fold must carry: R-29.e (CAND-3, CAND-6);
   ADV-8..10 and the widened ADV-6; R-6 acceptance per GP-12; OD-3 two ladders; R-40..R-43 per OD-13.
4. Operator sign-off of step 1 (owner human) after the fold.
5. Separate operator decisions referenced but not taken here: charter rewording for card exchange (T-2470), the
   protocol-too-old error (T-2700), the hub steward (T-3333).

## 11 What goes to which later step

| Step | Role | Items handed to it |
|---|---|---|
| 2 | Threat modeler | circuit trust model and credential lifetime (OD-1, CAND-16); same id with different content (CAND-2); same sequence number with different content (CAND-18); framing threats (CAND-1); GP-1, GP-6, GP-13, GP-14, GP-15; adversaries ADV-1..10 |
| 3 | Security floor and phasing | GP-12 diagnosis-time bound and state cap; ordering of the P1/P2 items; CAND-3 staging per dimension |
| 4 | Architect | hub-id minting (OD-12); grant fields and storage (OD-15); retention ceilings (OD-16); jitter, adaptive cap, hop limit (CAND-12); skew bound (CAND-13); gap-wait bound (CAND-18); mail-hub declaration format (CAND-8); idle no-surfacing threshold (CAND-6); "main" lease durations and priority format (OD-18); GP-4 offline; GP-11 design once ruled |
| 5 | Evidence specialist | the test estate and agent pairs for the CAND-3 tiers |
| 6 | Pseudo-coder | — |
| 7 | Review panel | — |
| 8 | Planner | R-41 typed work-message slice (OD-13); the `termlink sidecar` build (OD-9) and the build tasks of arc-011 |

## 12 What belongs to AEF, not TermLink

1. The send-and-wait tool (CAND-10); TermLink supplies T-3347.
2. Carrying the two priority ladders for re-sends, replacing D-600 for agent mail (offset 318).
3. Carrying the `priority` field (CAND-4) and the HANDED_OVER vocabulary and five states (OD-8, offset 314).
4. Project-id minting and a re-mint command for forks (OD-11).
5. The amended D-660 text (OD-12).
6. The operator out-of-band channel standard (GP-11, offset 319).
