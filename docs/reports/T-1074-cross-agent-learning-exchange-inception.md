# T-1074 — Periodic Cross-Agent Learning Exchange (Inception)

**Status:** started-work · inception · owner: human
**Task file:** `.tasks/active/T-1074-periodic-cross-agent-learning-exchange--.md`
**Research artifact:** this file
**Related inception:** T-1155 (channel-based communication bus, GO 2026-04-20)

## Problem Statement (mirrored from task)

Agents in the fleet (this dev box on .107, ring20 LXC sessions, parallel Claude instances, framework-agent) accumulate learnings — bugs encountered, workarounds discovered, protocol gotchas, structural insights — but those learnings stay **local** until something forces an exchange. Current exchange channels are all ad-hoc:

- Pickup envelopes (manual, "this is worth sharing")
- termlink inject (interactive, targeted)
- cross-project git mirrors (slow, human-driven)

**Hypothesis:** a periodic (e.g. 15-min) cron that asks every reachable peer "what did you learn since last exchange?" would surface insights that would otherwise decay with the session.

## Spike Results

### S-1 — Channel inventory (real evidence, 2026-04-20T14:40Z)

**Fleet reachability (`termlink fleet doctor`):**
```
--- local-test (127.0.0.1:9100) ---     [PASS] connected in 83ms
--- ring20-dashboard (.121:9100) ---    [FAIL] Token validation failed (secret mismatch, T-1051 class)
--- ring20-management (.122:9100) ---   [FAIL] Cannot connect (G-009 /var/log cascade)
Fleet summary: 3 hub(s), 1 ok, 0 warn, 2 fail
```

**Local sessions (`termlink list`):** 4 ready sessions across 2 hosts (framework-agent, termlink-agent, ntb-dev-test, email-archive).

**Existing exchange primitives:**

| Channel                   | Reliability              | Failure mode                                | Fit for learnings exchange |
|--------------------------|--------------------------|---------------------------------------------|----------------------------|
| `event.broadcast`        | Hub-dependent, push      | Dropped on hub rotation (T-1051 lineage)    | Partial — no backfill      |
| Pickup envelopes (files) | Manual, durable          | No pull, no subscribe, human-routed          | No — not automatic         |
| `termlink inject`        | Interactive only         | Requires live session                        | No — synchronous           |
| `kv set/get`             | Hub-scoped, point-read   | No deltas, no subscribe                      | No — no push model         |
| `learnings.yaml` (file)  | Git-sync, slow           | Requires commit + mirror                     | No — latency, manual       |
| **T-1155 channel bus**   | **Durable, pull+push**   | **None (offline queue, log-append)**         | **Yes — designed for this**|

**Key finding:** Fleet is unhealthy **right now** (2/3 configured peers unreachable). A naive 15-min cron would fire-and-fail on 2/3 peers indefinitely until someone heals auth (ring20-dashboard) or fixes infra (ring20-management). Wasted calls + log noise + false-alarm potential.

### S-2 — Cadence model

Candidates:
- **15-min timer** — simple, but decouples "something happened" from "pull cycle" → delays insights 0–15 min + wastes calls on idle peers.
- **On learnings.yaml write** — event-driven; zero waste; immediate propagation.
- **On session-end** — batches, but loses in-session insights until handover.
- **On commit** — git-centric; misses agents that don't commit frequently.

**Verdict:** Event-driven (post on write) strictly dominates cron-polling once a durable subscribe channel exists. The only reason the task proposed cron was because no subscribe channel existed. **T-1155 changes that.**

### S-3 — Schema

Observed schema in `/opt/termlink/.context/project/learnings.yaml` (73 entries, L-001..L-073):

```yaml
- id: L-001                  # stable, project-scoped
  learning: "..."            # free-text
  source: P-002              # practice/decision ref
  task: T-043                # origin task
  date: 2026-03-08           # ISO date
  context: "..."             # free-text
  application: TBD           # how to apply
```

Wire format decision: **post the YAML entry verbatim** as the channel.post payload. Add two envelope fields: `origin_project` (e.g. "termlink") and `origin_hub_fingerprint` (T-1052 R1 — lets receivers spot pre-rotation learnings).

**Dedup key:** `(origin_project, id)`. Same-class collision is cheap to handle — receiver sees `(termlink, L-042)` twice, stores once.

### S-4 — Dedup + merge policy

Receiver-side rule:
1. Read incoming entry.
2. Check local `.context/project/learnings.yaml` for `origin_project:id` already recorded (use a `received_from:` field to stamp origin).
3. If absent: append to a new file `.context/project/received-learnings.yaml` (separate from local learnings to preserve authorship).
4. Surface in Watchtower "fleet insights" panel. **Never auto-apply** — humans decide if a received learning graduates to local rules.

Why separate file: T-1074 task's scope fence explicitly excludes auto-apply; physical separation enforces that fence structurally.

### S-5 — Security model

T-1155 S-4 already landed this: **ed25519 self-sovereign agent keys** (T-1159 build task). Each agent signs its posts with its private key; receivers verify signature against published public key. This separates identity-trust from transport-trust and structurally solves the rotation problem (ring20-dashboard's current auth failure wouldn't affect signature verification).

Riding T-1159 means T-1074's security model is **zero additional work** — just use the bus identity.

## Recommendation

**Decision:** GO — **but pivot implementation to T-1155 bus** (do not build the standalone cron).

### Rationale (5-point evidence)

1. **Fleet unreachability (S-1):** 2/3 configured peers currently fail `fleet doctor`. A cron-polling design wastes cycles against sick peers; an event-driven bus design doesn't fire against unreachable peers at all (bus offline-queue holds until recipient reconnects, per T-1161).

2. **Cadence dominance (S-2):** Event-driven "post on write" strictly beats 15-min polling once a subscribe channel exists. Latency drops from 0–15 min to seconds; wasted calls go to zero.

3. **Schema + dedup fits naturally (S-3, S-4):** `(origin_project, L-NNN)` is a natural idempotency key. Bus already has idempotency via T-1161's SQLite queue `(sender_id, client_seq)`. Same primitive, no new plumbing.

4. **Security is solved (S-5):** Ed25519 agent identity (T-1159) separates identity from transport trust. No new auth design needed. The T-1051 rotation pain that would otherwise plague a cron is structurally avoided.

5. **Scope economy:** The "do a 15-min cron + its own wire format + its own auth + its own dedup" plan is 4 moving parts duplicating what T-1155 already has. One follow-up task on top of the bus (post-on-write + a receive handler) subsumes the whole cron design.

### Follow-up build tasks

One new task, **deferred until T-1158 (bus crate) lands**:

- **T-1168** (new): Add `channel:learnings` topic publisher to `fw context add-learning` + subscriber daemon that writes into `received-learnings.yaml`.
  - Depends on: T-1158 (bus crate), T-1159 (ed25519 identity), T-1160 (channel API).
  - Scope: ~1 Rust module + 1 shell hook + 1 Watchtower panel. Fits in one session.

### Go/no-go criteria evaluation

- **GO if root cause identified with bounded fix path:** ✅ Root cause is "no subscribe channel" — bounded by T-1155 scope.
- **GO if fix is scoped, testable, reversible:** ✅ T-1168 is one topic on an existing bus; removing it is one config flip.
- **NO-GO if fundamental redesign:** ✅ Not applicable — this rides T-1155, no new infra.
- **NO-GO if cost > benefit:** ✅ One small task on top of already-approved bus work.

## Assumption status (from task)

- **A1** (peers expose deltas via learnings.yaml): ✅ Confirmed — 73 entries with stable L-IDs, timestamped, machine-parseable.
- **A2** (15-min cadence): ❌ **Invalidated by S-2** — event-driven beats timer.
- **A3** (pull > push): ❌ **Invalidated by S-2 + S-4** — push-on-write with durable queue is strictly better given T-1155.
- **A4** (KB-scale): ✅ Confirmed — 73 entries fit in <50KB.
- **A5** (simple Q/A shape): ✅ Confirmed at schema level; delivery mechanism pivots from Q/A to pub/sub.
- **A6** (PL-IDs dedupe): ✅ Confirmed — `(origin_project, L-NNN)` is stable.

## Dialogue Log

### 2026-04-20T14:45Z — autonomous continuation of prior inception

**Context:** Task flagged in handover "Awaiting Your Action" queue with unchecked Agent ACs. Human user issued standing "proceed until 300k, apply governance" directive. Ran S-1..S-5 per original exploration plan.

**Key course correction:** Original plan designed a standalone 15-min cron. But T-1155 (GO decision 2026-04-20 ~11:00Z, same session lineage) approved a channel bus that structurally subsumes the polling design. Rather than build the cron and then deprecate it at T-1166, pivot now.

**Outcome:** GO recorded earlier remains correct; the recommendation here replaces the "preliminary direction" placeholder with the bus-pivot rationale. One new follow-up (T-1168), deferred to bus delivery timeline.

---

_Research artifact filled 2026-04-20T14:50Z. Original skeleton 2026-04-19 (T-1139 audit remediation, C-001 gate)._
