# T-3335 design review — GLM-5.3 (opencode, zai-coding-plan/glm-5.3)

Source: glm.raw (re-run with inlined documents, no file reads; rc=0). ANSI and tool banners stripped; text unchanged.

1. **Verdict**

1a. As a requirements record, yes; as a design for the operator's goal, not yet. The chain is only as good as its live leg, and the live leg has no owner: on the main host the injector is scheduled by nothing, the wake rail "terminates in a log line," and AEF's receiver doesn't run here. Every receipt the sender can see today can be produced with zero agent involvement — the 2026-10-03 incident proved it.

1b. Single biggest weakness: **O3, reachability of already-running sessions, is the pivot of the whole design and has no recorded position from the operator.** R-1.3 ("no agent has to be attached") is asserted as a requirement while being structurally false until someone decides how sessions get armed (PTY, `claude-fw --termlink`, or hooks). Everything else — four receipt calls, ladders, circuits — is scaffolding around that undecided pivot.

2. **Gaps and contradictions**

1a. R-1.3 contradicts the arming precondition stated in R-2.1/O3; the requirement should state the precondition, not deny it.

1b. R-3.2 ("independent of the hub") is contradicted by the design (the sidecar polls hub topics) and by R-4.3, and is untestable as worded — SCAPI IW-3's question (what failure does it buy?) was never answered.

1c. R-5.1 "calls the sender's sidecar immediately" specifies no timeout, no sender-down semantics, and no ordering; combined with R-9.1 it is also internally inconsistent: replies are *pulled*, receipts are *pushed*, for no stated reason.

1d. R-6.3 is contradictory and currently specifies a nonexistent mechanism: "a route that cannot silently lose the message" exists in neither build, and the requirement contradicts SQ-4, which is still recorded RESOLVED. A requirement may not cite a route nobody has designed.

1e. R-7.1 is only deployable for AEF-armed sessions; there is no requirement saying who arms non-AEF agents (O8), so R-7.1 is untestable on this host.

1f. R-10.1 contradicts the charter: topics are retention-bounded, so a ladder with 1-quarter and 1-year rungs polls topics that may no longer exist. Nobody flagged this. It also collides with D-600's dead-letter at ~76 days (O11).

1g. R-11.2 names a hub "name" that does not exist; the id in use rotates. A requirement naming an unbuilt, unstable identifier is a future outage, not a requirement.

1h. Missing entirely: authentication of sidecar-to-sidecar calls (AEF has bearer tokens; the R-5.1/R-9.1 calls have no trust model at all); sender-side send-and-wait semantics (the operator's own T-243/T-1800 ask — visibility is not wait); and the N:operator leg of R-1.1 is "designed only" yet is the load-bearing escalation endpoint for urgent and dead-letter traffic. O16's proposed additions (untrusted peer content, exactly-once, urgent marking, canary) are all correct and overdue.

3. **The hardest parts**

1a. **Readiness.** I side with harness hooks over screen inspection (R-7.1/R-7.2) — a long silent tool call is undetectable from the screen. But I'd go further than the document: the Stop hook fires per assistant response, not per tool call, so a hook-set "ready" flag is still a guess about the next instant. The only trustworthy signal is transcript evidence under a one-time token (AEF's D-696 finalizer, T-2876's prover). Use hooks to *gate the attempt*; use transcript evidence to *prove delivery*. Keep the screen classifier as a diagnostic, never a gate.

1b. **Urgent bypass.** Here I disagree with the operator's October ruling. SQ-4 was technically right: typing into a busy composer is exactly the T-2396 silent loss. But the operator's underlying intent — urgent interrupts — is achievable safely through the harness's own sanctioned interruption channel: Stop-hook stderr becomes context on the next turn (T1207R:50). Urgent should mean: jump the queue, deliver via the hook-context channel (which cannot land unsubmitted and be discarded), and escalate to the operator past a deadline. That satisfies "bypasses the wait" without any "route that cannot silently lose" needing to be invented. Typing into a busy PTY should remain forbidden, full stop.

1c. **Already-running sessions.** Hooks-first: a Stop/UserPromptSubmit pair makes any hook-configured Claude session reachable, headless or not, with no PTY coupling. PTY inject remains for foreign sessions we truly must control. Practically: relaunch the fleet through `tl-claude.sh start --reachable` or `claude-fw --termlink`, and amend R-1.3 to say what "attached" means. Accept that a session started outside both wrappers is pull-only — stated, not papered over.

1d. **Cross-host vs the charter.** I side with SCAPI's bright line and the tripwire, against the read-back. Cross-host sidecar-to-sidecar push *is* a second bus: it needs its own discovery, retries, dedupe, and security — all of which the hub already has. The honest design: the hub carries cross-host mail (it already does, durably, with receipts and an offline queue); "push" means the already-built `inbox.queued` frame (31–111 ms) plus local inject. Direct host-to-host is a charter change to be decided explicitly, owned by TermLink — not something AEF accretes under T-3688. The T-3330 ruling handing cross-host to AEF should be reopened.

4. **Open decisions**

1a. O1: hub carries cross-host; charter unchanged (3d above).

1b. O2: merge RECEIVED and STORED. A "received but not stored" ack gives the sender nothing actionable; answer once, after fsync-and-rename.

1c. O4: record SQ-4's supersession explicitly, then implement urgent as hook-context delivery, never composer typing.

1d. O9: option C. R-13.1 plus the `${TERMLINK_SCRIPTS_DIR}` coupling makes scripts-in-a-checkout untenable; ~2.5k shell lines belongs in the binary or an installed, versioned package. That is the "solid solution" SQ-8 demands.

1e. O10: the reviewers were right. Record-and-pull (per-message events on the hub) is the source of truth; the synchronous callback may exist as an optional fast path but nothing may depend on it — it fails exactly when the sender is down.

1f. O11: two different ladders with two purposes: bounded retry with dead-letter for sends; pull-fallback polling capped at hub retention (~1 week), not one year.

1g. O13: one vocabulary — SENT, ACCEPTED, HANDED_OVER (transcript-evidenced), REPLIED. An inject without transcript evidence is ATTEMPTED and must never be named INJECTED.

5. **Build order**

1a. Arm two real agents (relaunch through one wrapper). 1b. Give the injector a scheduler — one owner, either TermLink's supervisor or AEF's watcher, not both. 1c. Emit per-message events (ACCEPTED, HANDED_OVER, REPLIED) into the sender ledger with timestamps, mirrored to a hub topic. 1d. Add `in_reply_to` to replies and a REPLIED state. 1e. Acceptance test per IAC:128: agent A messages B while B is mid-turn and while idle; B's HANDED_OVER requires the transcript token; A's ledger shows every timestamp; B's reply produces REPLIED. Negative control: point a message at an unarmed agent and at a busy prompt — the sender must see UNDELIVERABLE/ESCALATED, never a delivered receipt (that is precisely the 2026-10-03 failure).

6. **Risks and failure modes**

1a. First to break: the scheduler chain dies silently while sidecars keep posting watermark receipts — "delivered" with no consumer, again. Precedents: arc-003, the sidecar dead since July. Detection: only a canary with deadlines (GLM's R6) plus the freshness check; adopt the canary despite the "urgent-only alarms" rule — it is not an alarm, it is a heartbeat.

1b. Urgent-bypass loss into a busy prompt (T-2396 class): detected after 120 s by AEF only; TermLink has nothing.

1c. Receipt overwrite (latest-per-sender, free-form stage) collapses the per-message timeline; detected by ledger divergence — which is why the ledger, not hub receipts, must be authoritative.

1d. Trust: one host fingerprint for all agents (the D6 finding) means any project can impersonate any other on `dm:` topics; cross-host sidecar APIs would make the injection surface remotely reachable. Undetected by design today.

1e. Clock skew across hosts silently corrupts the timestamped-timeline promise.

7. **Missing from the framing**

1a. A security and consent model for injection: urgent bypass means a peer can force-interrupt a working session; sender authentication and interrupt consent (O15) are requirements, not footnotes.

1b. Sender-side wait semantics: the operator asked for send-and-wait in April; the four-call ladder gives visibility but no yield/wait primitive for the sending agent. Missing.

1c. An option nobody stated: the harness's own hook-context channel as the *universal* delivery route — it works headless, eliminates the busy-prompt loss class entirely, and both teams have already built its pieces. PTY typing is being treated as the primary route when it is the riskiest one.
