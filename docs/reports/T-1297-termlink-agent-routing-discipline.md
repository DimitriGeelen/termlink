# T-1297: TermLink agent-routing discipline — research artifact

**Status:** Inception (pre-spike)
**Created:** 2026-04-26
**Owner:** human
**Related:** T-1291 (declarative heal), T-243 (multi-turn agent dialog)

## The incident that triggered this inception

2026-04-26: email-archive agent on .107 was coordinating cross-host work for the email-archive cutover to Penelope LXC on Ring20. It needed to delegate two pieces of infrastructure work:

- T-1191 — provision Penelope LXC (CT 250 on proxmox2)
- T-1194 — create Garage S3 bucket + RW key for email-archive blobs

It emitted both delegation events to **`framework-agent`** on the .107 hub (192.168.10.107:9100). framework-agent is a governance-role session — it handles `channel:learnings`, framework pickups, and cross-project framework coordination. It is NOT the email-archive product session.

Operator caught it ("why do put framework-agent as source agent to contact SILLY?"). Agent re-emitted both events on the right bus (`email-archive`, session `tl-pljpkait`). The wrong-bus traffic stayed on the hub at framework-agent's seq 906/907 — no harm, just nobody listens on that channel for `infra.*` topics.

Cost so far: one human-cycle of catch-and-correct. Cost projected as more product agents come online: linear with session count.

## Dialogue Log

**Q (operator):** how can .121 contact you?

**A (agent):** [walked through the TermLink hub address card — gave `framework-agent` as the target on .107]

**Q (operator):** [tested the path; 5-min poll, no reply]

**A (agent):** [diagnosed PTY inject vs bus events, recommended event.broadcast on bus topics — framework-agent topic]

**Q (operator):** why do put framework-agent as source agent to contact SILLY?

**A (agent):** [acknowledged routing mistake; identified `email-archive` (id `tl-pljpkait`, role `email-archive`) as the correct session; re-emitted T-1191 + T-1194 delegations on email-archive bus seq 213/214; left framework-agent emits at seq 906/907 in place as orphans]

**Q (operator):** do we need a structural fix?

**A (agent):** Yes — recurring class. Three options proposed:
1. `termlink whoami` — agent self-lookup, removes the guess.
2. Topic↔role lint at emit time — catches misroute as a warning.
3. framework-agent rejects non-framework topics — destination-side hard reject.

Initial pick: (1) + (3).

**Q (operator):** 3 not sure can that any agent is asked to relay topics

**A (agent):** Right — destination-rejection breaks legit relay. Strike (3). Revised pick: (1) + soft hub warning on self-emit role mismatch.

**Q (operator):** why not 2 also??

**A (agent):** Conflated (2) with (3) and threw both out. (2) actually survives the relay test: when framework-agent forwards `infra.lxc.reply` to .107, the destination IS email-archive (role match), not framework-agent. (2) catches *originator confusion*, not *forwarder traffic*. Revised pick: (1) + (2).

**Q (operator):** [authorize inception capture] yes

**A (agent):** [this artifact + T-1297 task]

## Three-option matrix (post-dialogue)

| Option | What it does | Relay-safe? | Maintenance cost | Verdict |
|---|---|---|---|---|
| 1. `termlink whoami` | Read-only RPC: returns caller's session identity on hub. | Yes (read-only). | Low — derived from existing session registry. | **In** — root-cause fix. |
| 2. Topic↔role soft-lint at emit | Warns when self-emit topic doesn't match self-role tags. `relay_for` declarations suppress false positives. | Yes (relay = different destination role, not destination=self). | Medium — small mapping table grows slowly with topic catalog. | **In** — defense in depth. |
| 3. framework-agent rejects non-framework topics | Hard reject at destination. | **No** — breaks legit relay where framework-agent forwards cross-project traffic. | Low. | **Out** — destination-rejection is the wrong layer. |

## Why both (1) and (2), not just (1)

Single-mechanism fixes have a known failure mode: the mechanism becomes optional, agents skip it, regression returns silently.

- (1) alone: agents call `whoami`, get the right answer, emit correctly. But a stale memory entry, a buggy agent, or a copy-pasted command from another project skips `whoami` — and we're back to the original failure with no detection.
- (2) alone: catches misroutes, but every misroute is a warning event the operator has to read and triage. Without (1), agents have no cheap way to do the right thing pre-emit; they get warned but can't easily self-correct.
- (1) + (2): (1) provides the easy path to right; (2) catches when agents skip (1) or when (1)'s answer is wrong. Failures are caught at both layers.

## Pre-spike inclination

GO on combined fix. Decomposable into:

- **Build A** — `termlink whoami` RPC + CLI subcommand. ~½ day.
- **Build B** — Topic↔role mapping format + soft-lint hub-side. ~1 day.
- **Build C** — `relay_for` per-session declaration + integration with (B). ~½ day.

Total: ~2 dev-days. Reversible (lint can be disabled per-emit, RPC is additive).

Locked AFTER spikes 1-3 in T-1297.

## Spike scope refresher

- **Spike 1 — Quantify.** Walk recent emit history. Count misroutes. Goal: confirm >1 incident or de-prioritize.
- **Spike 2 — `whoami` prototype.** Verify lookup is unambiguous in current 7-session bus.
- **Spike 3 — Mapping table format.** Centralized vs distributed; opt-in `relay_for` shape.

## Out-of-scope (deferred to follow-up inceptions)

- **Auto-rewrite of misrouted emits.** If we can detect, we could in principle rewrite. But auto-rewriting traffic without human-in-the-loop is a sovereignty concern. Detection first.
- **Cross-hub topic propagation rules.** Single-hub first.
- **Hub-driven session capability advertisement.** A larger change that subsumes the topic↔role mapping into a general capability protocol. Worth its own inception if (B) feels too rigid.

## Notes for next session

If picking up this inception cold: the dialogue log above captures the conversation that produced the option matrix. Spike 1 is the first hard evidence step — without misroute volume data, the inclination above is just opinion.
