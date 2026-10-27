<!-- T-3612 round 1, seat codex, source /tmp/tl-dispatch/t3612-r1-codex-cfa605e6f4b5/result.md, exit 1 -->
## 1. What TermLink is today

TermLink is an operator-run coordination runtime for terminal-based agents: real PTYs and remote execution, a persistent hub message bus, leased work claims, and increasingly a transactional worker-status service. Its Rust implementation stores message payloads in append logs, indexes them in SQLite, fsyncs payloads before committing their index, and exposes terminal and coordination operations through CLI/MCP interfaces. Around that core sits a substantial deployment, messaging, and monitoring apparatus tailored to its own agent fleet. It is more than terminal sharing, but less than a fully proven distributed execution platform: worker-contract integration remains unfinished and operational write-latency problems remain recorded. [Sources: `crates/termlink-session/src/pty.rs:183`; `crates/termlink-bus/src/log.rs:64`; `crates/termlink-bus/src/claim.rs`; `crates/termlink-bus/src/worker_contract.rs`; `scripts/tl-dispatch.sh`; T-3504; T-3488.]

## 2. What it is for

The demonstrated customer is the operator of a cooperating, multi-machine AI development fleet, especially Claude Code agents working with AEF and ring20. Jobs include finding reachable peers, exchanging requests without human relaying, dispatching isolated workers, recovering interrupted coordination, and observing or controlling remote terminals. This is visible in executable dispatch scripts, inbox commands, and the three-project worker contract—not merely README positioning. [Sources: `scripts/tl-dispatch.sh:1`; `.claude/commands/check-arc.md`; T-3484; `.context/project/objectives.yaml`.]

If it disappeared, those integrations would lose their shared transport, discovery, claim ownership, terminal access, and emerging authoritative completion record. Humans or replacement infrastructure would have to reconstruct those functions. The practical loss is reliable handoff and recoverability, not a uniquely irreplaceable terminal emulator. Evidence of the human cost is the operator manually relaying mail when identity/routing failed. I found no basis to claim broad external adoption. [Sources: `crates/termlink-bus/src/claim.rs`; `docs/reports/T-3484-termlink-build-design.md`; commits `ef1cf0a44`, `39c081723` (2026-09-30).]

## 3. What it is becoming

- **Late August–September: from asserted reliability to repeatedly checked behavior.** Bounded RPCs, unreadable-directory handling, message-path reproducers, dormant-test wiring, and audit remediation receive sustained attention. The September 9 audit arc and September 19 value-review arc institutionalize this repair work. [Commits `861bef308` (08-30), `e163fde73` (08-29), `175c8bbfb` (09-01), `540fc51d4` (09-08); arcs `arc-008`, `arc-009`.]
- **October 1: from transport to bounded coordination authority.** T-3304 explicitly chooses the hub as the trusted record within a retention window, with ownership metadata, gap signals, and bounded defaults. This reflects existing use: its decision records 75 of 111 topics retaining forever. The gap signal immediately lands. [T-3304 Decisions, IW-1–4; `.context/arcs/arc-012.yaml`; commit `44f9ee033`.]
- **October 4–6 onward: from “posted” to demonstrably handed to an agent.** The communication design separates receipt, storage, handover, and response. Current sidecar code claims delivery attempts before exposing content and checks harness transcript evidence before recording handover; uncertainty blocks automatic redelivery. This is an agent-delivery protocol, not just a mailbox. [`.context/arcs/arc-011.yaml`; `crates/termlink-sidecar/src/deliver.rs:1`, `:672`; `crates/termlink-sidecar/src/adapter/claude.rs`.]
- **October 18–27: from loose workers to accountable execution attempts.** “Accepted means stored” gains payload fsync; worker records gain transactional state transitions, supervisor credentials, fencing, and completion evidence. October 21 workspace decisions favor separate clones per claim. Backend code exists, but T-3504 CLI/MCP integration is still captured. [T-3461; T-3469 Decisions; T-3501–3504; `crates/termlink-bus/src/worker_contract.rs`; commits `22429048b`, `803cce11e`.]

## 4. Proposed purpose

**TermLink makes a cooperating fleet of terminal-based agents dependable by providing recoverable communication, accountable work handoffs, and observable terminal control.** The hub should authoritatively record coordination within explicit retention limits. Agent frameworks should decide what work matters and who should perform it; TermLink should make those decisions executable and their outcomes inspectable. [Basis: T-3304; T-3484; `docs/CHARTER.md`, non-goal 4.]

## 5. Proposed objectives

These are proposed acceptance targets, not claims of present performance.

1. **No acknowledged message disappears without explanation.** Across crash, restart, disk-error, retention, and replay tests, require zero unexplained losses; every send ends in evidenced progress or a named unresolved/failure state. Measure payload recovery and sender-visible stages separately. [Basis: T-3461; `crates/termlink-sidecar/src/deliver.rs`; objectives O-2.]
2. **Work ownership and completion remain unambiguous after failure.** Require all shared contract vectors to pass, zero accepted stale-owner writes, and one terminal record per attempt in restart/death/conflicting-completion scenarios. Do not equate this with exactly-once external side effects. [Basis: `crates/termlink-bus/src/worker_contract.rs`; T-3484; T-3501.]
3. **Discovery predicts actual reachability.** Every advertised ready peer must complete a probe; expired or unwakeable peers must leave the ready view within a declared timeout. Measure false-ready rate and withdrawal latency daily. [Basis: `crates/termlink-bus/src/claim.rs`, `IdleAgent`; arc-002; objectives O-1.]
4. **Coordination remains responsive under declared load.** Meet T-3488’s existing 24-hour criterion: no create/post above five seconds below 90% disk utilization, and fivefold fewer slow requests in saturated hours than its baseline. Publish load conditions and latency distributions. [Basis: T-3488 acceptance criteria and October 22 update.]
5. **Routine operation needs no human courier.** Over a 30-day fleet trial, require zero manual message relays and at least 95% of recoverable incidents resolved or presented with one actionable command. Count interventions rather than only green canaries. [Basis: objectives O-6; `runme.sh:10`; commit `ef1cf0a44`.]
6. **Supported installations deliver the same core behavior.** Gate Linux/macOS releases on spawn/exec/output/cleanup, message recovery, and contract tests; require all declared fleet hosts to report matching expected capabilities and zero accumulating orphan sessions. [Basis: objectives O-4/O-5; commit `d918ea5c0`; T-3291–3299.]

## 6. Proposed value drivers

Weights total 100; they are recommendations, not edits to protected policy.

| Driver | Weight | Existing-policy disposition and reason |
|---|---:|---|
| Coordination correctness and recoverability | 30 | Keep D2 Reliability, redefine around messages, ownership, and completion rather than memory continuity. |
| Observable failure and justified trust | 25 | Change D1 Antifragility into demonstrated fault detection/recovery; reward fewer silent failures, not more repair machinery. |
| Operator and agent usability | 20 | Keep D3; measure interventions, diagnosis time, and tool-call burden. |
| Bounded operating cost | 15 | Add explicit disk, latency, process, and context-budget costs. |
| Interoperability and deployability | 10 | Keep D4 but change “source-controlled memory” into portable protocols, supported hosts, and harness adapters. |

The current weights—D1 9, D2 7, D3 5, D4 3, Recall 6, Orchestration 5—mostly describe framework development. Drop **F-RECALL** from TermLink product prioritization: instruction synchronization and recall are AEF concerns. Replace **F-ORCH**’s reward for expanded executor surface with verified handoff outcomes under correctness/usability. Keep inactive **F-AUTONOMY** inactive as a separate driver; fewer human gates are valuable only when guarantees remain intact. [Sources: `policy/value-drivers.yaml`; arc-002 scoped drivers; arc-005 context-cost evidence; T-3488.]

## 7. Non-goals

- No archival database, general query language, or automatic inter-hub federation. A bounded authoritative coordination record is compatible with these exclusions. [T-3304 IW-1/IW-3; `docs/CHARTER.md`.]
- No task-priority, model-selection, or business-workflow policy engine. Host admission/deadline mechanisms may belong here, but T-3596 should not become a second scheduler competing with AEF. [Charter non-goal 4; T-3596; `scripts/tl-dispatch.sh`, model-resolution boundary.]
- No adversarial multi-tenant security promise; attribution and scoped authority inside a cooperating fleet remain necessary. [Charter non-goal 5; T-3493; objectives O-7.]
- No social-feature growth or governance machinery justified solely by its own maintenance. Require a coordination outcome for both. [Charter non-goal 3; arcs 008/009; `policy/value-drivers.yaml`.]

## 8. Tensions and contradictions

**Record versus non-record:** the charter forbids a system of record, while T-3304 explicitly authorizes one for coordination and worker tables implement authoritative state. I believe the dated decision and implementation; revise the charter to distinguish bounded authority from archival storage. [T-3304 IW-1; `crates/termlink-bus/src/worker_contract.rs`.]

**Star versus direct delivery:** the charter says strict star; closed arc-003 advertises direct host-to-host delivery, and the newer communication arc describes hub-arranged circuits. The no-spoke-mesh test itself admits it is source-scoped and cannot prove runtime destinations. I believe coordination authority is hub-centered, but the absolute topology claim needs reconciliation. [`.context/arcs/reliable-comms.yaml`; `.context/arcs/arc-011.yaml`; `crates/termlink-session/tests/no_spoke_mesh_tripwire.rs`.]

**Reliability versus its overhead:** extensive guard work coexists with basic defects: September’s pickup canary stopped at offset 99; October’s read-activity bookkeeping held locks while writes stalled. More monitoring is not automatically more dependable service. [Commit `99bc055df`; T-3488 October 22 update.]

**Intent versus freshness:** objectives still describe retention gaps and supervisor credentials as future work although code implements them. Meanwhile the README acknowledges restart-limited post deduplication; O-3’s “done once” must not be read as arbitrary exactly-once execution. Prefer code for implementation status and operational evidence for achieved guarantees. [Objectives O-2/O-3/O-7; commit `44f9ee033`; `crates/termlink-bus/src/worker_contract.rs`; `README.md`, Guarantees.]

## 9. Confidence and limits

Confidence: **1 high; 2 medium; 3 high on direction, medium on deployment; 4 medium; 5 medium; 6 medium; 7 medium; 8 high**. Recommendations involve judgment; adoption and operational success are less verifiable than source structure.

I verified `/opt/termlink` HEAD as `50dece8656221dec01ee4bdb24758b98e37310f5`, sampled the 2,087 commits in the requested window, and read selected implementation, tests, arcs, decisions, designs, and handover material. I did not run builds/tests, contact services, inspect live fleet state, or exhaustively audit authorization/concurrency. Reported measurements are repository evidence, not independently reproduced results. The vendored framework was treated as a dependency, not counted as TermLink’s product.