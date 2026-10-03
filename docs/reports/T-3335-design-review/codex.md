## 1. Overall

**Not yet, and not reliably as specified.** The design contains the necessary components, but leaves their critical contracts unresolved. The documents report that this host stores and receipts messages without injecting them into running agents.

The single biggest weakness is **the absence of an enforced, end-to-end delivery contract owned by one component and tested against real agents**. Separate teams can truthfully complete their pieces while the conversation remains impossible. Wiring the injector is necessary; it will not by itself fix that weakness.

This assessment is based on the three documents, not an independent runtime audit.

## 2. Gaps and contradictions

2a. **R-1.2 is impossible literally during a partition.** The sender can know the last confirmed stage, its evidence and age, and that subsequent progress is unknown. Specify that contract instead of “always knows where.”

2b. **R-5.1/R-5.2 have contradictory meanings.** RECEIVED sometimes already means durable storage, yet STORED follows it. Define RECEIVED as volatile acceptance and STORED as durable commitment. Only STORED permits the sender to release its delivery obligation.

2c. **R-8.2 overstates transcript evidence.** A matching context attachment proves admission into a particular turn’s recorded context, not comprehension or action. Distinguish CONTEXT_ACCEPTED from the agent’s response. A BUSY transition proves neither.

2d. **R-6.1/R-6.3 need latency budgets.** A 30-second poll cannot provide immediate urgent handling from arrival. Specify separate deadlines for detection, context admission and response; trigger urgent work on arrival.

2e. **R-9.1/R-9.2 lack reply correlation and termination rules.** Require `message_id`, `in_reply_to`, conversation identity and explicit message types. Receipts and terminal “no action” responses must not create fresh reply obligations.

2f. **R-10.1/R-10.2 conflict operationally with bounded retention.** Year-scale polling cannot recover already-expired messages. Define expiration, payload ownership and deduplication retention. Do not promise exactly-once agent actions merely because posting is deduplicated.

2g. **R-9.3/R-9.4 need enforceable limits.** Software can detect missing responses; it cannot guarantee model cooperation. Define deadlines and escalation. Halt message-dependent work when deaf, with explicit recovery criteria.

2h. **The documents themselves drift.** Open-decision numbers differ between files; section 14 remains labelled unconfirmed; the full design proposes a screen fallback contrary to R-7.2. Use stable decision identifiers and one normative requirements version.

## 3. The hardest parts

3a. **Readiness:** use harness hooks, as R-7 requires. However, a ready flag is only an observation: user input can race the injector. Require session incarnation, readiness generation, exclusive injection ownership and invalidation on restart. Serialize submission through the harness where possible. Screen inspection may help diagnosis but must not independently authorize injection. Test the actual deployed harness’s hook behavior.

3b. **Urgent bypass:** I disagree with treating durable storage plus retry as sufficient safety. It protects the message from disappearance; it does not protect a busy terminal from unintended input, submission or interruption. Prefer an authenticated harness interrupt/control operation that bypasses ordinary scheduling. Where unsupported, report that limitation and escalate immediately. Retaining unconditional busy-PTY typing should be an explicit risk acceptance, not a reliability claim.

3c. **Already-running sessions:** inventory their actual control capabilities. Register an existing controllable terminal where supported; otherwise preserve a checkpoint and relaunch through the reachable launcher. Hooks alone cannot wake a completely idle session without an external trigger. Publish UNREACHABLE until an end-to-end challenge succeeds.

3d. **Cross-host delivery:** retain hub-mediated transport, with sidecars exposing a uniform local API. Clients can address the chosen destination hub without hub federation. During hub outages, local acceptance remains available and cross-host delivery queues. If cross-host delivery while hubs are down is mandatory, explicitly amend the charter: assigning the second transport to AEF does not remove it from the architecture.

## 4. Open decisions

4a. **Ownership and packaging:** TermLink should own the transport-neutral message lifecycle, durable queues, receipts and terminal-control primitive. AEF should supply harness readiness/context adapters and governance policy. Ship one versioned runtime with TermLink, ideally `termlink sidecar`, rather than maintaining competing delivery engines.

4b. **Callbacks:** preserve push, but make it asynchronous from a durable outbox. Receiver progress must not depend on sender availability. Pull repairs missed notifications. Persist state transitions and their outbound receipt obligations atomically.

4c. **Receipt vocabulary:** keep the confirmed RECEIVED/STORED distinction until explicitly changed; define their evidence precisely. Add CONTEXT_ACCEPTED, ANSWER_READY, NO_ACTION and expiration/failure outcomes. Keep events rather than selecting the “highest-ranked” status: escalation can precede a late answer.

4d. **Identity:** distinguish role-addressed delivery from exact-session delivery. Resolve roles through leased registrations; fence exact targets by incarnation. Never silently redirect an exact-session message to another instance.

4e. **Polling and alarms:** use bounded, jittered retries tied to conversation deadlines. Retain longer schedules for archival recovery only. A single overdue ordinary message must remain visibly overdue even without an immediate operator alarm.

## 5. Build order

5a. Build the smallest complete path: one supported launcher and harness adapter; one armed receiving session; durable inbox; supervised wake loop; transcript-correlated admission receipt; durable reply with `in_reply_to`; automatic delivery back into the sender’s session. Include sender-visible status from the start.

5b. Add crash recovery, deduplication, session fencing and installation verification before expanding transports. Do not make this slice wait for a directory redesign, steward agent or observability database.

5c. **Acceptance test:** launch two real agents from the installed package, without manual pickup. A sends B a fresh nonce requiring a computed response. Test B idle, then busy on a controlled long tool call; B answers and A automatically consumes the answer and sends a follow-up. Record every required stage and evidence at A. Proposed test bounds: context admission within 35 seconds of readiness, simple response within 120 seconds.

5d. **Negative control:** disable B’s injector while leaving storage and receipts operational. A must see STORED followed by overdue/unconfirmed status, never context admission or success. Restore it and prove recovery without duplicate logical processing. Separately test the supported urgent route while B remains busy.

## 6. Risks and failure modes

6a. **Deployment drift will likely break first:** healthy sidecars with missing hooks, wrong session bindings or unscheduled injectors. Detect with an installed-system conversation canary, not process heartbeats.

6b. **Concurrency and restart races:** duplicate supervisors, stale readiness, cleared flags and late receipts. Detect through session generations, attempt identifiers, queue reconciliation and crash-boundary tests.

6c. **False success:** transcript-format changes or incorrect receipt attribution. Fail evidence validation closed; retain bounded diagnostic evidence.

6d. **Invisible escalation:** the reporting path fails alongside delivery. Test an operator-visible destination independently of the message rail.

## 7. What is missing from the framing

7a. **Conversation admission and attention budgets.** Many-to-many agents can create endless acknowledgements, urgent storms and context flooding. Require recipient consent policy, bounded outstanding requests, rate limits, cancellation, message expiry and explicit broadcast membership. Otherwise successful delivery can itself make the agents unusable.

7b. **Operational ownership.** Name one maintainer accountable for the recurring two-agent acceptance test and deployment coverage. Another completed task register will not prevent this failure from recurring.
