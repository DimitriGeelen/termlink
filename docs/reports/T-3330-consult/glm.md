## 1. Choice, and the strongest argument against it

1. Option C — but as a transition, not a terminus. The layering is right: the substrate owns the shared vocabulary, the per-message record, the pull verb, and the digest; frameworks own their receivers and report into it. A is disqualified by R3 alone (ledgers on per-project disks are not pullable), and by the incident itself: the stage that failed — injection — is TermLink machinery, which AEF's ledger can structurally never see. B is the correct endgame but premature: retiring AEF's receiver breaks working direct paths and regresses latency while TermLink's own path is a 15-second poller. So: C now, with declared intent to fold AEF's receiver into the termlink binary once the vocabulary is proven.
1a. Strongest argument against C: the mirroring requirement manufactures exactly the bug class that caused the incident — an out-of-band obligation on a component that can silently skip it. Buffers grow when the hub is down, mirrors lag, vocabulary drifts across N implementations. C is only safe if TermLink ships the emitter itself (library or subcommand) under R5, so mirroring is one implementation, not a spec everyone reimplements.

## 2. Is the hub the right home?

1. Yes, with corrections to how C is worded. Telemetry events are messages; a retention-bounded telemetry topic is squarely within charter. "All events kept" is not — it quietly converts the hub into a system of record. The durable artifact should be the daily digest; the event topic is working state that may age out.
1a. The real system of record is the digest sequence, which recipients persist as they pull (R3 implies this). Cross-host later, with no federation, means pull-through-discovery: ask the peer's hub once its endpoint is discovered — charter-consistent.
1b. The alternative homes fail: per-project disks (not pullable), the operator's machine (single point, not agent-readable), a new telemetry service (that genuinely would be a second substrate).
1c. Prerequisite the framing omits: the hub's latest-per-sender summary (1d) must not apply to telemetry; events need per-message retention within the window and lookup by client_msg_id, or the pull verb has nothing to pull.

## 3. What breaks first, and detection

1a. A: standardization, and the RECEIVED→INJECTED gap. Non-AEF agents get nothing; the stage that actually failed stays invisible because it lives in TermLink.
1b. B: migration. Senders with cached direct endpoints hit connection-refused (UNDELIVERABLE storm); HANDED_OVER latency regresses from synchronous to poll cadence; one binary becomes a single point of failure for the entire receive side.
1c. C: the mirror. Silent mirror lag, unbounded buffers during hub outages, events lost on crash, vocabulary drift (HANDED_OVER vs INJECTED), and clock skew between event time and hub arrival.
1d. D: trust — the next stall is discovered a day late again.
1e. Detection, all options: canary mail. The digester sends each agent a probe with hard deadlines (INJECTED within T1, REPLIED within T2); a miss alarms the operator in minutes. For C, add mirror-lag accounting: hub saw SENT (or a direct ack exists) but no mirrored event within T. Stage-latency baselines before/after any migration make B's regressions visible.

## 4. Minimal record and digest

1. R2's "every step and the delay between steps" is over-specification; agents act on exceptions, not exhaustiveness. Record, per event: client_msg_id, conversation_id, from_project, to_project, stage, event timestamp, hub arrival timestamp, transport (hub|direct), error code on terminal failures. No payloads. The dual timestamp is non-negotiable once hosts differ.
1a. Digest, per agent per day: counts and max for SENT→RECEIVED, RECEIVED→INJECTED, INJECTED→REPLIED (percentiles are noise for agents), plus two named exception lists — messages unreplied after 24h, and stage gaps over operator-set thresholds — each entry naming client_msg_id and peer, so the reading agent can nudge rather than store. Thresholds are fleet defaults, not per-agent negotiations. Record is primary; digest is derived and small enough to outlive event retention.

## 5. What is missing from the framing

1. The framing treats a liveness failure as a telemetry problem. Root cause (1b): injection is scheduled by nothing without a registered injectable terminal. Telemetry makes the stall visible at digest time; an invariant makes it nearly impossible: mail arrives for a project with no registered session → immediate ESCALATED to the operator, plus nag/dead-letter for un-injected mail older than X. This should be R6, and it outranks the five stated requirements.
1a. R1 as written is wrong: "a timestamped API call back to the sender" couples receiver progress to sender availability and starts an infinite regress (who receipts the receipt?). Record-and-pull is the correct semantics; C already implies it — say so explicitly.
1b. Negative outcomes are absent from the framing's stage list. UNDELIVERABLE, REJECTED, ESCALATED exist only in AEF's vocabulary; a standard without terminal failure states produces digests that lie by omission.
1c. The two-transport split brain is unaddressed: the hub-fallback path currently emits no receipt, so senders reaching AEF agents via fallback are invisible today. Whichever option wins, fallback must report stages too.
1ca. Security: publishing bearer-token receiver endpoints to the hub for the cross-host slice implies rotation and revocation; nobody mentioned it.
1d. R4 ("agents reflect") has no completion criterion. Make acknowledgment of the digest a required artifact, else reflection is theater.
