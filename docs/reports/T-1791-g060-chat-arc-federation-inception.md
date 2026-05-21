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

### Spike 1 — re-count under quiet conditions (CONFIRMED A-1, partially confirmed A-4)

Commands used: `termlink channel list --json` on `.107` (local) and `--hub 192.168.10.122:9100` on `.122`. Both queried within ~30 seconds, no live posters intervening.

| Hub | `agent-chat-arc` count | Retention | Delta vs PL-176 |
|---|---|---|---|
| .107 (this host) | 1804 | forever | +4 since 1800 |
| .122 (ring20-management) | 490 | forever | +4 since 486 |
| **Gap** | **1314** | — | **stable** (no growth) |

**A-1 CONFIRMED.** The disparity is real, not measurement artefact at the count level. **A-4 PARTIAL.** The gap is **stable in absolute terms**, not actively widening — both hubs gained the same 4 messages. This is inconsistent with "federation is currently broken" (which would grow the gap monotonically); more consistent with "federation never existed but two hubs happen to receive the same volume of new posts independently".

### Spike 2 — chat-arc-vs-other comparison (FALSIFIED A-2)

Commands used: `comm -12` on sorted topic names + per-topic `count` join via Python script.

Topic-count summary:
- `.107`: **1143 topics**, mostly local activity + test artefacts
- `.122`: **43 topics**, much smaller volume
- **Intersection: only 12 topics**, mostly high-volume coordination topics

Per-shared-topic delta (`.107 - .122`), sorted by absolute delta:

| .107 | .122 | delta | topic |
|---|---|---|---|
| 1804 | 490 | **+1314** | agent-chat-arc |
| 533 | 11 | **+522** | broadcast:global |
| 112 | 1 | **+111** | channel:learnings |
| 21 | 8 | +13 | framework:pickup |
| 20 | 27 | **−7** | dm:9219671e…:d1993c2c… |
| 7 | 1 | +6 | routing:lint |
| 9 | 6 | +3 | multi-agent-e2e-10679 (and 4 others) |
| 6 | 6 | 0 | multi-agent-e2e-7274 |

Also: **31 topics exist ONLY on .122** — test artefacts (xhub-bidir-B-*, xhub-post-*, xhub-real-*, t1443-cross-host-smoke-*, cross-hub-probe-*) plus two DM topics `dm:33df8954…:ring20-management-agent` and `dm:9219671e…:9219671e…`. These topics were never reflected back to .107.

**A-2 FALSIFIED.** Chat-arc is **not chat-arc-specific** — multiple high-volume topics show massive deltas (broadcast:global +522, channel:learnings +111). And one DM topic goes the OTHER way. The pattern is **systemic across all topics**, not a chat-arc bug.

### Spike 3 — federation pairing audit (REVEALS THE STRUCTURAL TRUTH)

Two sub-investigations:

**3a. Connectivity & version health:** `termlink fleet doctor --include-pin-check` on .107 reports all 5 configured hubs reachable, all PASS, pins OK. Version skew detected (one hub on 0.9.0, two on 0.9.2110, two on 0.9.2127) — but this is operational hygiene, not federation specifically.

**3b. Federation code path:** `grep -rn` across `crates/termlink-hub/src/` and `crates/termlink-protocol/src/` for `federat`, `peer_subscribe`, `cross_hub`, `topic_replic`, `inter_hub`, `hub.peer`, `hub.subscribe` (channel-flavored). **Result: no inter-hub channel-topic replication primitive exists in the codebase.**

The only `forward_to_*` paths in `router.rs` (1649: `forward_to_target`, 2844: `forward_to_remote_session_via_tcp`) operate at the SESSION level — routing an RPC from one hub to a session on another hub. They don't replicate channel-topic state between hubs.

**3c. Smoking-gun evidence — offset 486 on .122 chat-arc.** Read directly via `termlink channel subscribe agent-chat-arc --cursor 484 --limit 6 --hub 192.168.10.122:9100`. The envelope is a PROBE message from cohort-agent stating:

> "Both hubs running independent agent-chat-arc topics (.107=1800 msgs, .122=486 msgs) — **they don't auto-federate**. Will follow with brand bundle in next post if this lands. Every cohort→ring20 message can be `remote_call channel.post`'d into .122 from now on, no operator-relay needed."

The cohort-agent had **already determined this** during T-209 / T-1438 work on 2026-05-21 — channel topics with the same name on different hubs are **independent**. Cross-hub coordination happens at the CLIENT level: a client uses `--hub <addr>` or `remote_call channel.post` to push the same message into multiple hubs.

**A-3 FALSIFIED (the wrong way).** Not "subscriptions are healthy" but "subscriptions don't exist as a concept for channel topics". There is no peer-subscription state to be healthy or unhealthy.

## Diagnosis: H-d (refined) — the FRAMING was wrong, not the federation

**The "federation gap" is not a code bug or operational drift.** PL-176's framing rested on a false premise: that channel topics auto-federate between hubs. They do not. TermLink hubs maintain independent topic storage. Topics with the same name on different hubs are unrelated state — the way two unrelated databases happen to have a table named `users`.

The "DM federation" PL-176 observed working **also wasn't federation** — it was the cohort-agent (and other actors) manually cross-posting via `remote_call channel.post`. PL-176's diagnostic comparison (chat-arc disparity vs DM apparent-sync) compared **inconsistent cross-posting** (chat-arc, lower fraction explicitly cross-posted) against **consistent cross-posting** (DMs, cohort-agent always cross-posts).

The 1314-message gap on chat-arc reflects:
- Messages posted directly to .107's hub (by clients connected to .107) without being cross-posted to .122
- Messages posted directly to .122's hub (by clients connected to .122) without being cross-posted to .107
- Plus a smaller fraction of messages that WERE cross-posted (which appear on both)

There is no "fix" in the federation path because there is no federation path.

## Implications for T-1166

T-1166's retirement of `event.broadcast` / `inbox.push` / `file.send/receive` is **not blocked by G-060** under this corrected diagnosis. The legacy primitives being retired were ALSO single-hub (event.broadcast was hub-local). Channel topics replace them at parity — same single-hub semantics, much richer primitive set (now with full MCP parity per T-1789 + T-1790 + PL-177).

The retirement story is sound. What G-060 actually reveals is a **mental-model gap**: when the cohort agent and others assumed channel topics federate, they built (correct) workarounds via `remote_call channel.post` cross-posting, but the assumption persisted in observations like PL-176. The structural risk is that future agents (and humans) might make the same wrong assumption.

## Final recommendation

**Recommendation: DEFER with G-060 reframe** (not the originally-anticipated "DEFER pending more evidence"). The original GO/NO-GO/DEFER axes don't apply because the framing was wrong.

**Concretely propose three follow-up tasks (none of them T-1166 blockers):**

1. **Downgrade G-060** from `severity: high, type: gap` to `severity: low, type: documentation-gap` (or close entirely with a learning capturing the refined framing).
2. **Documentation task** (small, scoped): add a section to `docs/operations/` or `CLAUDE.md` stating "channel topics are per-hub; cross-hub message visibility requires explicit `--hub <addr>` posting or `remote_call channel.post`. Topics with the same name on different hubs are independent state."
3. **Optional inception** (separate, much larger): does the fleet WANT automatic inter-hub channel-topic federation? Concrete benefits (cleaner agent UX) vs costs (significantly more state-sync complexity, consistency models, conflict resolution). NOT decided here — a future architectural question.

T-1166 retirement cuts can proceed without waiting on any of the three. The MCP-parity arc closure (T-1789 / T-1790 / PL-177) remains sufficient justification for retiring the legacy primitives.

**Update PL-176** to correct the framing: DM topics also don't auto-federate; the apparent sync was cohort-agent cross-posting. Add the diagnostic recipe: `comm -12` on `termlink channel list` topic names from both hubs, compare counts per shared topic, look for the cross-posting pattern in envelope sender_ids.

