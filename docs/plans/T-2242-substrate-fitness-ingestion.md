# Substrate-fitness remediation — ingestion plan (T-2242)

<!-- Framework-agent ingestion of HANDOFF-termlink-remediation-2026-06-22.
     Built on DISCOVERY-termlink-comms-analysis-2026-06-22 (T-2241).
     Arc: arc-substrate-fitness (arc-002, status: draft).
     BINDING: research is not authorization. This plan SLICES and SURFACES; it
     approves nothing. Drivers are proposed, not approved. Q1-Q3 + the T-2025
     revisit are Sovereign and unresolved. No build tasks are minted herein. -->

## 0. What this is

Per the handoff §7, the framework agent holds **Initiative**: draft the arc, slice the
suggested tasks, propose (not approve) scoped drivers, and **surface** the Sovereign
decisions. This document is the ingestion artifact — "enough structure to ingest, not a
finished work package." It also records the §7-mandated verification of the §1 finding→gap
mapping against current source.

## 0.5 RESOLVED decisions (2026-06-22 — human, via T-2242 walkthrough)

All five surfaced decisions were taken with the human. This section is the durable record of
outcomes; §4 below retains the option sets that were weighed.

- **Q4 — reopen T-2025? → NO.** Verified read-only: `termlink agent find-idle` returns
  `idle: []` — the derived LIVE view correctly classifies the ~4.7-day-stale heartbeats as
  not-LIVE, so T-2025's correctness claim **holds** (discovery returns empty, not ghosts). F3 is
  a *volume* symptom (→ R2) + an *operational* one (binary upgrade, already watched by the
  T-2239 frozen-husk canary), NOT a gap-#7 architecture defect. **R3 dissolves** (re-registration
  = operational/canaried; reaping folds into R2); **R6 drops** from keystone to optional telemetry
  (absence-detection partially already exists as T-2239).
- **Q3 — arc scope → ONE arc.** Inception + build coexist; start/stop/pause/horizon are
  task-level; multiple arcs would need a master arc. R5's design-first cadence rides on its task.
- **Q1 — presence retention → two-step.** `days:2` on `agent-presence` NOW (interim; shipped
  mode; config-only; drains the 30k stale beats) + **latest-per-key compaction as R2's build
  target** — the only mode that closes the T-1991 agent-*count* scaling (NOT shipped: the enum
  has forever/days/messages/Latest[keep-1]; per-key compaction = new work pairing eviction with
  the existing cv_key). R2 scope flag: needs a change-retention-on-existing-topic path (no such
  verb found today).
- **Q2 — telemetry → local-first capture+aggregate, daily aggregated push over TermLink.**
  Capture+rollup stay local (resilient to hub/agent loss; sees crashes; no observer effect); a
  once-a-day **aggregated** batch is pushed over TermLink to a collector agent (centralized +
  actionable). The push is best-effort and **rides R4's durable queue**; raw Tier-0 stays local
  for forensics; the collection topic gets the same bounded/aggregated retention (dogfoods Q1).
  Retention shape = **tiered rollup** (raw 24–48 h; aggregates kept long). Key insight: presence
  (current-state → compact/expire) and telemetry (time-series → retain-aggregated) are **opposite
  data classes**; the F1 bug was applying forever-raw to current-state data; aggregation dissolves
  the AS_FAILURE_OBSERVABILITY ↔ AS_RESOURCE_FOOTPRINT tension.
- **Driver weights → 6 / 5 / 4** (COORDINATION_TRUTH / FAILURE_OBSERVABILITY / RESOURCE_FOOTPRINT).
  Human applies via `fw arc approve-driver arc-substrate-fitness "<name>" --weight N --i-am-human`
  (agent-gated — sovereignty). AS_COORDINATION_TRUTH pole re-anchored off the dead R3 (→ R4/R2).

**Surviving arc shape:** R4 (keystone, now) → R2 (days:2 now + per-key compaction) → R7
(hygiene) → R1 (minor — cv_key on the `register` path) → R5 (telemetry inception, design above).
**R3 + R6 dropped per Q4=NO.**

**Human's next actions:** (1) run the 3 `approve-driver` commands; (2) `fw arc start
arc-substrate-fitness`; (3) build R4 first (minted + set as the focused task for post-compact continuity).

## 1. Verification of §1 mapping against source (§7 requirement)

§7: *"Verify the §1 finding→gap mapping against current source before building on it; if the
repo contradicts a finding, the repo wins — flag it."* Done. Result: gap **numbers** are
correct; two **finding framings** are contradicted by source.

### ✅ Affirmed against `docs/architecture/parallel-execution-substrate.md` §6

| Handoff claim | Source | Verdict |
|---|---|---|
| F-INSTR/R4 → gap #5 (outbound queue) | §6 line 230 "Client-side reconnect + outbound queue for spokes" | correct |
| F2/R1 → gap #9 (current-value key) | §6 line 267 "Broadcast-with-replay / current-value key" | correct |
| F1/F4/R2/R7 → gap #10 (retention/compaction) | §6 line 269 "Published throughput…budget with retention/compaction" | correct |
| R4 premise: poison-drop is a silent DELETE, no dead-letter | `offline_queue.rs:223-225` — `pop()` runs `DELETE FROM pending_posts WHERE id=?1` on BOTH delivery and poison-drop | **confirmed** |

### 🚩 FLAG 1 — repo contradicts F2's framing ("producers emit no cv_key")

- `scripts/listener-heartbeat.sh:173` **emits `--metadata "cv_key=$agent_id"` by default**
  (T-2107 wiring present and correct; `--no-cv-key` opt-out only). So the cv_index emptiness
  is **NOT** a wiring regression.
- The cv_key-emitting producer (`/be-reachable` → `listener-heartbeat.sh`) is **not running**:
  `~/.termlink/be-reachable.log` = "Terminated"; no `listener-heartbeat` process in `ps`.
- The presence producers that ARE running are `termlink register --shell` PTY sessions
  (4 visible in `ps`, started ~Jun-17). The `register` self-heartbeat path **does not** add
  `cv_key` metadata (grep finds no `cv_key` in `crates/termlink-session/` heartbeat code or
  the `register` command — only in the shell script, the hub-side `cv_index.rs`, and the
  `channel cv-keys` read verb).
- cv_index is in-memory, cleared at the Jun-17 hub restart, and only repopulated by a
  cv_key-emitting heartbeat — of which none is running. **Hence count=0.**

**Consequence for slicing:** R1 ("engage the cv_index path") is largely a **no-op against
current code** — the path is engaged; nothing feeds it. R1 collapses into R3 (get a
cv_key-emitting producer running and re-registering). The genuine independent R1 residue is
**narrower than the handoff states**: *extend `cv_key` emission to the `register`/PTY presence
producer path* so cv_index is populated regardless of which producer is live. Re-scoped below.

### 🚩 FLAG 2 — F3/R3/R6 reopen the T-2025 NO-GO

Design-doc §6 #7 (hub-persistent presence + circuit-breaker) was decided **NO-GO by the T-2025
inception (2026-06-08)** and re-scoped to documentation-only. Its load-bearing rationale
(§6 lines 235-244): presence DATA is durable (SQLite-backed); the LIVE/STALE/OFFLINE view is
derived client-side and *"refreshes within one heartbeat interval (~30s) — a brief
view-staleness window, not data loss."*

The discovery's **F3 is new empirical evidence that this assumption fails in practice**:
`agent-presence` `last_ts` predates the Jun-17 restart, so the view has been stale for
**~4.7 days**, not ~30s — because the refresh depends on producers continuing to heartbeat,
and the running `register --shell` producers evidently do not post periodic heartbeats to
`agent-presence` (consistent with a pre-T-2230/T-2235 binary, the frozen-husk class).

**Consequence for slicing:** R3 and R6 are **not net-new builds** — they are a request to
**revisit the T-2025 NO-GO** with F3 as new evidence. Reopening a NO-GO is **Sovereign**.
Modelled below as a single revisit-inception, not as build tasks. (Antifragile: a failure is a
learning event that can re-open a decision — but only the human re-opens it.)

> Cross-reference: arc-001 (`arc-parallel-substrate`) lists T-2025 as a deferred blocker but
> mislabels its primitive number as "#4"; the design doc numbers it #7. Numbering is cosmetic;
> the NO-GO and its rationale are what matter here. Flagged for tidy-up, not blocking.

## 2. Corrected lock structure & task slicing (plan form — NOT minted)

Locks per handoff §3 (one closed before the next opens). Slicing/sizing/AC remain the
framework agent's lane; these are ingestion-ready drafts with real ACs, deliberately **not**
minted as `T-XXXX` build tasks (G-020 pickup-governance: a detailed spec is not authorization;
mint after the human ratifies drivers + Q1-Q3 + the T-2025 revisit).

### Lock 1 — Coordination truth

**R1 (RE-SCOPED per FLAG 1) — Emit `cv_key` from the `register`/PTY presence producer path.**
*gap #9.* Not "turn on T-2107" (already on for the shell producer). Make the binary's
`register` self-heartbeat add `metadata.cv_key=$agent_id` so cv_index is populated by whichever
producer is live.
- AC: `termlink register` heartbeat envelopes to `agent-presence` carry `metadata.cv_key`.
- AC: with a registered session live, `hub status --governor` shows `cv_index_entries_active > 0`
  for `agent-presence`; `channel cv-keys agent-presence` returns ≥1 entry.
- AC: `--no-cv-key` parity opt-out exists (symmetry with the shell path).
- Note: depends on R3 to have a live producer to observe; small once R3 lands.

**R2 — Bound the presence log (retention/compaction).** *gap #10 / F1.* Move `agent-presence`
off `retention: forever` to the policy chosen in **Q1**.
- AC: `agent-presence` retention is the Q1-chosen policy (not `forever`); topic stops growing
  monotonically (verify count stabilises across a heartbeat window).
- AC: existing readers/late-joiners still resolve current presence after the change.
- Blocked on Q1.

**R3 (RECAST per FLAG 2) — Revisit T-2025: refreshing/non-stale presence across restart.**
*gap #7 / F3.* **Inception/decision, not build.** Present F3 as new evidence; let the human
decide GO/NO-GO on some form of: producer re-registration on restart + stale-entry reaping so
discovery reads live agents only.
- AC (of the inception): F3 evidence written up against the T-2025 rationale; one
  recommendation (GO/NO-GO/DEFER) surfaced; **human decides** (agent does not `decide`).
- Absorbs the discovery's most correctness-urgent defect (discovery currently returns dead
  agents) — but via a re-opened Sovereign decision.

### Lock 2 — Observable governance delivery

**R4 — Dead-letter the discarded outbound.** *gap #5 / F-INSTRUMENTATION (premise confirmed).*
Replace the silent poison-drop `DELETE` (`offline_queue.rs:223-225`, threshold logic in
`bus_client.rs`) with a durable dead-letter record (post + reason + ts + attempts) that
survives, surfaced via `queue-status`.
- AC: a post that crosses `POISON_THRESHOLD` lands in a dead-letter store, not a bare DELETE.
- AC: dead-letter rows are readable (e.g. via `queue-status`/a read verb) with reason + attempts.
- AC: regression test — N poison cycles ⇒ N recoverable dead-letter rows, 0 silent losses.
- **Highest-certainty, fully-verified, uncontested. Pulled to front of Lock 2 so Lock 1 runs
  observably. The single best HV/LC build in the arc.**

**R5 — Telemetry plane (design-first inception).** *F-INSTRUMENTATION.* Durable, **local-first,
NOT-over-TermLink** per-agent telemetry (discards, flaps, breaker trips, reconnects, RTT,
clean-exit marker), collected out-of-band per **Q2**.
- AC (of the inception): design/inception doc; transport-is-local-first constraint justified;
  recommendation surfaced; **human decides**. No build under this id.

**R6 — External observer for the can't-self-report class (design-first keystone).**
*F-INSTRUMENTATION / producer-not-judge.* Hub-side record of delivery success/absence +
absence-detection ("heartbeat stopped, no clean-exit"). **Overlaps R3's T-2025 revisit** —
recommend folding R6's "hub-side presence/absence record" into the same revisit-inception as R3
rather than a separate keystone, since both are gap #7 hub-side state that T-2025 ruled on.
- AC (of the inception): scoped against R3's revisit; human decides as one Sovereign call.

### Lock 3 — Bounded growth / hygiene

**R7 — Bound audit/inbox/test-topic growth.** *gap #10 partial / F4 (confirmed).*
- AC: `rpc-audit.jsonl` (1.36 GB) is rotated/bounded.
- AC: the 19 stale inbox transfers to 7 dead smoke targets are drained/expired.
- AC: test/smoke topics (981/1420) reaped or namespaced so they stop polluting discovery.
- Cheap; last; also de-noises the next discovery's measurement surface.

## 3. Drivers (PROPOSED — weights Sovereign)

Written to `proposed_scoped_drivers:` in `arc-substrate-fitness.yaml` (NOT to `scoped_drivers:`).
All three survive verification (they are value dimensions, independent of the contested
framings). Approval + weights are the human's:

```
fw arc approve-driver arc-substrate-fitness "AS_COORDINATION_TRUTH"    --weight <N> --i-am-human
fw arc approve-driver arc-substrate-fitness "AS_FAILURE_OBSERVABILITY" --weight <N> --i-am-human
fw arc approve-driver arc-substrate-fitness "AS_RESOURCE_FOOTPRINT"    --weight <N> --i-am-human
```

Handoff's *suggested* weights (yours to set/override): COORDINATION_TRUTH 6, FAILURE_OBSERVABILITY 5,
RESOURCE_FOOTPRINT 4 — encoding "correctness & observability rank above efficiency; a heavier
substrate is the cheap error, a blind/stale one the expensive error." Cap is 3 drivers (met);
weight ≤6 (met). Rejected candidates (AS_VERIFIABLE_NOW, AS_GOVERNANCE_PLANE_PROTECTION)
preserved in the handoff §5 for artefact discipline.

## 4. Sovereign questions — RESOLVED 2026-06-22 (outcomes in §0.5; option sets retained below)

- **Q1 — Presence retention policy (feeds R2).** compact-to-latest-per-key | short TTL |
  message-cap. Handoff lean: compact-to-latest-per-key. **Unresolved.**
- **Q2 — Telemetry collection method (feeds R5).** out-of-band local dumps | in-band. Handoff
  lean: out-of-band for failure/historical, in-band only for a live snapshot. (Same axis as the
  discovery's Q1.) **Unresolved.**
- **Q3 — Arc budget / scope split.** one arc (Locks 1–3) | split Lock 2's telemetry+observer
  into its own arc after R4 ships. Handoff lean: one arc through R4, reassess. **Unresolved.**
- **Q4 (new, from FLAG 2) — Reopen the T-2025 NO-GO?** R3/R6 depend on this. The discovery's F3
  is new evidence the NO-GO's "refresh within ~30s" assumption fails. **Reopening is Sovereign —
  unresolved.**

## 5. Sovereign boundary — what I did NOT do

- Did **not** approve any driver or set any weight (`scoped_drivers:` left empty).
- Did **not** `fw arc start` (arc stays `draft`) and did **not** `fw inception decide` anything.
- Did **not** mint R1–R7 as build tasks (G-020 — detailed spec ≠ authorization; mint after
  ratification).
- Did **not** modify any TermLink/hub state, source, or config — this is planning only.
- Surfaced Q1–Q4 + driver weights + the two flags for human disposition.

*Research is not authorization. The repo won both contradictions; the findings were corrected
accordingly.*
