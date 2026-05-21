# T-1791: G-060 agent-chat-arc federation gap — RCA inception

**Status:** started-work (inception, exploration not yet begun)
**Filed:** 2026-05-21
**Inception owner:** human (per framework rules)
**Recommendation at filing:** DEFER (insufficient evidence)
**Triggering observation:** PL-176 / G-060 — 1800 vs 486 msg disparity between .107 and ring20-management (.122) on agent-chat-arc topic, 2026-05-21

## Why this inception exists

T-1166 retirement of legacy `event.broadcast` / `inbox.push` / `file.send/receive` primitives has now reached MCP-parity closure (T-1789 channel_poll_results + T-1790 channel_info + PL-177 arc-closure learning). The structural argument behind T-1166 has been: *channel-topic federation is the canonical replacement; legacy primitives can be cut once parity is in place.*

PL-176 / G-060 challenges that premise on the highest-volume topic in the system. If chat-arc fails to federate at scale, then "channel-topic federation as canonical replacement" is not yet a complete story — and cutting legacy fanout primitives while this gap is open would degrade rather than improve fleet coordination.

This inception answers **one question**: what is the root cause of the agent-chat-arc federation disparity, and which of (fix / accept-and-retire / defer) should T-1166 wait on?

## Hypothesis space

Four candidate root causes, listed by current prior:

### H-a: federation logic bug
The federation code path has a defect specific to agent-chat-arc (e.g. a code branch that handles high-fanout topics differently, a corner case in the cursor advance, a serialization bug on a specific msg_type used heavily in chat-arc).

**What this looks like:** chat-arc shows a disparity, similar-volume project topics do not. Both peers have the subscription. The disparity grows over time.

**Implication if confirmed:** GO — bounded code fix in `termlink-hub` federation path. Write a regression test, ship, retire legacy.

### H-b: load-driven loss
The federation protocol drops or backpressures messages under high volume. Chat-arc is the only topic large enough to trigger it. The current design cannot scale to a single-topic fanout above some throughput threshold.

**What this looks like:** disparity correlates with traffic bursts. Lower-volume topics federate cleanly. Restarting either peer doesn't help (the loss is in-flight, not state).

**Implication if confirmed:** NO-GO — accept the gap. Retire chat-arc as a single shared topic in favor of per-hub topics + read-cross-hub aggregation. Or invest in a federation redesign (separate task).

### H-c: operational drift
One or both hubs lost their peer subscription due to a restart-without-persistence event (PL-021 echo), a config drift, or a manual `tofu clear` that wasn't restored. Federation isn't broken — it's not currently running between this pair.

**What this looks like:** counts diverged from a specific date forward (the date of the drift event). Other topics may show the same drift because the subscription drift affects all topics, not just chat-arc. DM topics may have re-established because a fresh `agent contact` re-subscribed them implicitly.

**Implication if confirmed:** DEFER — operational fix (re-peer). The inception's recommendation is "DO re-peer, then re-measure". If after re-peer the federation works cleanly, no code change needed and T-1166 cut can proceed.

### H-d: measurement artefact
The 1800 vs 486 figures are not directly comparable. Different retention windows, different counting semantics (per-topic vs per-fingerprint), or different protocol-version interpretations could produce numerically different but operationally equivalent counts.

**What this looks like:** when both hubs are queried with identical commands at identical wall-clock times, the disparity narrows or disappears. Or: the "1800" figure includes retained-but-tombstoned messages while "486" doesn't.

**Implication if confirmed:** DEFER (with G-060 downgrade to LOW or closed) — re-instrument the measurement, re-observe in a follow-up.

## Exploration plan (read-only, time-boxed)

**Spike 1 — re-count under quiet conditions (10 min).**
- `termlink topics --json` on .107 and on .122 at the same wall-clock minute (use `termlink remote exec` for the .122 side)
- Cross-check with `termlink agent topic-stats --topic agent-chat-arc` per hub
- Record: count, retention kind/value, latest offset, latest ts
- Outcome: confirms or falsifies A-1 (the disparity is real)

**Spike 2 — chat-arc-vs-other comparison (20 min).**
- Enumerate all topics present on BOTH hubs (`termlink channel list | jq` per hub)
- For each topic in the intersection, compute count delta (left - right)
- Bucket by category: chat-arc, dm:*, project topics (anything else)
- Outcome: confirms or falsifies A-2 (chat-arc is the outlier)

**Spike 3 — federation pairing audit (20 min).**
- Per hub, identify peer subscription state — code reading first (find where peer subscriptions are persisted), then read state
- Identify when each side last received a message from the other (look for envelopes whose `sender_id` matches the peer's identity fingerprint, sort by latest)
- Outcome: confirms or falsifies A-3 (subscriptions are healthy)

After spikes, classify the gap as (a)/(b)/(c)/(d) and write the recommendation in the task file's Recommendation section.

## Decision matrix

| Diagnosis | Recommendation | Next task |
|---|---|---|
| (a) federation logic bug | GO (fix) | Build task: pinpoint defect, write regression test, ship |
| (b) load-driven loss | NO-GO (accept; retire chat-arc as shared topic) | Build task: design per-hub topics + cross-hub read aggregation; OR inception: federation redesign |
| (c) operational drift | DEFER (re-peer, then re-measure) | Operational task (owner: human): re-peer both hubs; then follow-up inception to confirm post-fix federation works |
| (d) measurement artefact | DEFER (downgrade G-060) | Follow-up: improve the diagnostic command; close G-060 if disparity is illusory |

## Dialogue Log

### 2026-05-21 — Inception scope confirmed

- **User asked:** "lets incept: G-060 (high): agent-chat-arc federation gap — would benefit from an inception task before further T-1166 cuts"
- **Agent response:** Filed T-1791 with DEFER recommendation. Task scoped to ONE question (root cause) with three time-boxed read-only spikes. No code changes, no operational changes in this inception. Build task or operational task follows the GO/NO-GO/DEFER outcome.
- **Outcome:** Inception task created. Presenting template for review before spikes run.

## Findings

<!-- Filled in as exploration progresses. Each spike produces a finding subsection. -->

### Spike 1 — re-count under quiet conditions

<!-- to be filled when spike runs -->

### Spike 2 — chat-arc-vs-other comparison

<!-- to be filled when spike runs -->

### Spike 3 — federation pairing audit

<!-- to be filled when spike runs -->

## Final recommendation

<!-- Replaces the filing-time DEFER with a fully-evidenced GO/NO-GO/DEFER citing the spikes. -->
