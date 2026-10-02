## 1. Choice and strongest objection

Choose **C, amended: receivers own observations; TermLink owns a shared telemetry protocol and a retention-bounded query projection.** Do not make TermLink the authority on stages it cannot observe.

The incident was an observability and execution failure: a poller’s watermark was presented as delivery, although no functioning route into the agent’s session existed. Changing receiver ownership alone would not fix that.

**The strongest argument against C is that asynchronous mirroring creates two histories.** A message can reach an agent while its telemetry remains buffered, lost, or rejected. The hub must distinguish “no progress” from “no recent observation.” Otherwise C reproduces the original false confidence with more precise-looking timestamps.

## 2. Is the hub the right home?

Yes, for **operational telemetry within one hub’s scope**, provided “all events kept” means “all events retained within a declared retention window.” Indefinite history would contradict the charter. Anyone requiring longer history needs an explicit export/archive owner.

Hub telemetry about direct HTTP delivery is not inherently a second message bus. It becomes one if telemetry events trigger delivery, retries, or work execution as an alternative transport. Keep observation separate from those commands.

However, C does not solve fleet-wide telemetry across independent hubs. Each agent needs a declared home hub; the operator can query those hubs separately or use an external, read-only aggregation service. Do not quietly introduce federation through telemetry.

Per-project ledgers are useful local outboxes and diagnostic evidence, but poor shared operational interfaces: availability depends on access to each project’s filesystem.

## 3. What breaks first, and detection

3a. **A — fragmented visibility and adoption.** Agents outside AEF lack a common reporting contract; inaccessible ledgers prevent shared diagnosis. Detect through a roster of expected agents, advertised capabilities, and reporting freshness. AEF could implement a standard, but ownership alone does not make that standard universal.

3b. **B — mistaken authority and migration gaps.** The binary cannot truthfully assert HANDED_OVER or REPLIED without evidence from agent integrations. Existing receivers may coexist during migration and duplicate delivery. Detect using an end-to-end canary through the actual prompt hook, stage provenance, and duplicate-message counters. Binary packaging improves deployment consistency; it does not establish semantic completeness.

3c. **C — telemetry divergence.** Local buffering fills, reports arrive out of order, and a missing event looks like stalled delivery. Detect through reporter heartbeats, sequence gaps, outbox age/depth, and reconciliation against receiver state. Report ingestion lag separately from message-handling delay.

3d. **D — the existing silent stall persists.** Mail continues receiving reassuring watermarks while no injection mechanism runs. Immediately expose “injectable session unavailable,” pending-message age, and last successful injection. Deferring ownership need not mean deferring this repair.

## 4. Minimal actionable record and digest

4a. **Identity and context.** Use a stable message identity qualified by sender identity, plus conversation, sender, recipient, intended circuit/session, transport path, and available hub coordinates. Define whether retries preserve identity and how delivery attempts are distinguished.

4b. **Append-only observations.** Each event needs a unique event ID, message/attempt identity, stage, observing component, occurrence time, hub ingestion time, and outcome or reason code. Preserve evidence and corrections rather than overwriting history. Include reply-message linkage for REPLIED.

4c. **Precise semantics.** RECEIVED means durably accepted by the recipient’s receiving component—not merely posted to the hub. INJECTED means successfully surfaced through the declared session mechanism; it does not mean understood. REPLIED means a correlated response was emitted; it does not mean the request was resolved. Model rejection, delivery failure, and escalation separately from the normal progression.

4d. **Operational context.** Advertise supported stages, receiver/session availability, telemetry freshness, retention coverage, and an expected response deadline or service class. Without expectations, elapsed time cannot reliably identify a problem. Cross-host delay calculations must acknowledge clock uncertainty.

4e. **Pull interface.** Support queries by agent, message, conversation, stage, age, and time range, with pagination and explicit completeness/freshness indicators.

4f. **Daily offer.** Include unresolved messages grouped by required action, oldest age and overdue count, stage-delay distributions with sample counts, failures, missing observations, and comparison with the previous period. Every exception should identify an owner and a concrete next action. Include a pull cursor or reference for details, not message bodies by default.

Daily reporting supports reflection; urgent stalls need earlier alerts.

## 5. Missing requirements and alternatives

5a. **R1 conflates notification with transport.** Require timestamped, sender-visible stage events. Synchronous callbacks to sender endpoints introduce availability coupling; durable asynchronous publication can satisfy the underlying need.

5b. **There is no stated recovery owner.** Specify who notices overdue work, retries safely, repairs injection, or escalates to the human. R4 needs an accountable action loop, not merely generated commentary.

5c. **Trust and privacy are unspecified.** Authenticate reporters, constrain whose stages they may assert, and authorize telemetry queries. Metadata can reveal sensitive project activity.

5d. **A smaller migration option exists.** Standardize events and capability reporting, ship a supported receiver/injection integration with every deployment, then adapt AEF. R5 requires deployable functionality—not necessarily absorbing every receiver into one binary.
