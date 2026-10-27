<!-- T-3612 round 1, seat claude, source .context/dispatch/t3612-r1-claude/result.md, exit 0 -->
Finished the read; the answer is below and saved to `.context/working/t3612/seat-anthropic.md`. I checked the sidecar inbox first. It showed no consult addressed to me; the dm rail shared with other projects had 3 unread, which I left alone.

# Seat: Anthropic (Claude Sonnet 5.5) — TermLink independent read, commit 50dece865

## 1. What TermLink is today
TermLink is a Rust workspace of about 218K lines: cli 78K, mcp 52K, hub 29K, session 27K, sidecar 14K, bus 12K (`crates/`). It implements a hub-mediated star of sessions with these parts:
- an append-log channel bus (offsets, retention, acks, claims/leases, dedupe);
- PTY session control (spawn/exec/inject);
- presence and doorbell wake;
- TLS+HMAC auth with TOFU pinning.

It is exposed through a large CLI and about 273 MCP tools (I counted `#[tool(` in `crates/termlink-mcp`; the README says "270+"). In practice it is the nervous system of one operator's agent fleet (ring20 hosts, Claude Code workers), developed under the vendored Agentic Engineering Framework (AEF).

A second artefact of similar weight sits beside the product: about 99 `scripts/check-*` canaries and static checks among 266 scripts, 35 slash-command skills, and a guard layer (`docs/operations/guard-layer-and-canaries.md`). Use shows two things. The bus is used mostly for agent DMs and work claims. And the project's own operations (canaries, runme, review queues) are a large share of what the repo is.

## 2. What it is FOR
- **Users:** (a) the operator, a single human running many Claude Code agents across machines; (b) those agents themselves (CLAUDE.md tells them to `/be-reachable`, `/claim`, `/reply`); (c) the AEF, which dispatches workers over it. I saw no evidence of third-party production users. CLAUDE.md's CI section calls the GitHub mirror "read-only".
- **Jobs:** find a live peer; hand it a message that survives restarts; claim a work unit exclusively; drive a remote terminal (the founding job, `docs/CHARTER.md` Origin).
- **What would be lost:** cross-agent, cross-host coordination for the fleet, and the operator's ability to reach and wake sessions. The generic parts (an append-log with claims) could be replaced by NATS or Redis streams plus SSH. The irreplaceable part is the PTY endpoint combined with agent wake (doorbell, sidecar), tuned to Claude Code harness behaviour.

## 3. What it is BECOMING (2026-08-27 .. 10-27)
Commits peak at 09-20..09-30 (147 on 09-29) and again at 10-11..10-18. About 80% of file churn is outside `crates/` (`.tasks`, `.context`, framework docs).
1. **Reliable comms, "accepted means stored".** arc-003 and arc-004 are closed; arc-007 (loud-contract) is in progress. The substrate is now maintained by canaries.
2. **Audit and value-review remediation (09-09 arc-008, 09-19 arc-009).** The five-run value review verdicted DELETE as zero across runs (arc-009 description). That is despite the charter's removal-candidate language for social surfaces.
3. **Message delivery, "mailbox to prompt" (arc-010, created 09-22).** The operator interview ran 10-04..06 (T-3344). The stage chain is SENT, RECEIVED, STORED, HANDED_OVER, REPLIED. The new `termlink-sidecar` crate has 259 commits in the window, the most of any crate. This is where code effort went.
4. **Hub as the authoritative record (arc-011, 10-01, T-3304).** It adds retention-gap notices, topic ownership and idle flags, a forever-topic review, and a planned rewording of charter non-goal #2 (`arc-012.yaml` line 7). The project is moving toward being a record, against its own non-goal.
5. **Dispatched-worker contract and isolation (from 10-19: T-3469, T-3477, T-3484, T-3501..T-3504).** It adds one authoritative record per unit of work, a supervisor token, per-claim clone or copy-on-write workspaces, and conformance vectors shared with AEF and ring20 (objective O-3). This is orchestration creeping up from claims toward work dispatch.
6. **Identity and authorisation (O-7: T-3486, T-3493, T-3467, T-2422).** This is security hardening inside a trust model the charter calls a "cooperating fleet".
7. **Operator-experience governance.** This covers the runme.sh contract (T-3272/3273/3326), one decision at a time, labelled lists (T-3329) and decision briefs (T-3305). The top commit-subject words since 08-27 are "Session", "close", "finalize", "record", "operator", "runme", "interview", "reviewer". That is not crate work.

## 4. Proposed purpose statement
**TermLink lets a fleet of AI agents and their operator reach each other and act on each other's terminals, with delivery they can trust: a message accepted is stored, a unit of work claimed is done once, and a failure is seen where and when it happens.**

It is a small, hub-mediated substrate (durable channels, claims, PTY endpoints) for a cooperating fleet, not a platform. Its edge over generic message queues is that the endpoints are live terminals and agent sessions that can be woken and observed. Policy, planning and governance belong to the layers above it.

## 5. Proposed objectives
1. **Reach.** A live, armed peer is discoverable and wakeable end to end. Measure: `scripts/comms-selftest.sh --peer X` passes daily on every fleet hub; stale-ghost count in `agent find-idle` is 0.
2. **No silent loss.** Every stuck message names its stage. Measure: the arc-003 prover, dead-letter and unconfirmed-delivery logs are empty; the sender-visible stage chain exists for 100% of posts; p99 `channel.post` is under 5 s under load.
3. **Exactly-once work.** A claimed unit completes once and is recovered on worker death. Measure: substrate-smoke and stuck-claims canaries are clean; conformance vectors pass on the hub backend; duplicate completions are 0 over 30 days.
4. **Terminal control works.** Spawn, exec and read round-trip, with no leaks. Measure: `session-selftest` runs daily; zombie sessions stay at or below threshold.
5. **Shipped equals live.** Releases install on Linux and macOS, and the fleet runs the expected version. Measure: release-publication, fleet-binary and fleet-capability canaries are clean; time from tag to all hubs on that version.
6. **Attributable and authorised.** Every write is traceable and checked. Measure: percentage of writes signature-verified; signed-inject allowlist enforced; zero identity-spoof findings in review.
7. **Operator load.** The operator is interrupted only for decisions. Measure: operator-relayed messages per week (not yet measured; O-6 admits this), `canary-status` all HEALTHY, failures surfaced in one place within a minute.
8. **Surface proportionality (new).** Tool and script count grows only with a measured user. Measure: MCP tools with zero calls in 60 days; ratio of guard and canary code to product code, which should fall.

## 6. Value drivers
Existing drivers (`policy/value-drivers.yaml`, v3) are protected D1 Antifragility 9, D2 Reliability 7, D3 Usability 5, D4 Portability 3, and free F-RECALL 6, F-ORCH 5. F-AUTONOMY is commented out as a candidate. These are AEF-wide drivers, not TermLink's.

Proposed weights (0-10) for judging TermLink work:

| Driver | Weight | Why |
|---|---|---|
| Delivery integrity (no silent loss, stage-visible) | 10 | It is the product; the charter says "Delivery is the product". Absorbs D2. |
| Liveness truth (reachable means reachable) | 8 | Ghosts and unwakeable agents were repeated incidents (arc-002, T-2875). |
| Attributability and authorisation | 7 | Rising in importance; cheap now, expensive after wider use. |
| Operator attention saved | 7 | Current effort already follows it, but it must be measured. |
| Verified live, not merely shipped (G-069) | 6 | Shipped-but-dark capabilities recurred. |
| Simplicity / surface budget | 6 | New; no existing driver penalises growth. |
| Antifragility (failures become tests and canaries) | 5 | Keep D1 but lower. It justifies the 99 checks and needs a counterweight. |
| Portability | 3 | Keep. macOS and Linux are covered, and MCP is already a standard. |

Keep D1, D2 and D4 in meaning. Change D3 to "operator attention". Drop F-RECALL, which belongs to the framework, not the substrate. Treat F-ORCH as a tripwire against creep rather than a positive driver.

## 7. Non-goals
Keep the five in `docs/CHARTER.md` and add:
1. **Not a record store, once the contradiction is resolved.** If arc-011 makes the hub the authoritative coordination record, rewrite non-goal #2 honestly ("record of coordination for a bounded window") rather than drift.
2. **Not a work-dispatch or orchestration engine.** The worker contract (T-3484, T-3501..04), supervisor and workspace provisioning (T-3469/3477) are policy and belong in AEF. Keep only the claim, lease and record primitives.
3. **Not a governance or operations platform.** Runme, the review queue, audit remediation and the canary registry belong in AEF or an ops repo.
4. **Not a social surface.** The MCP surface still has reactions, polls, emoji stats, `top_starrers`, `top_pinners` and typing, which contradicts non-goal 3. Arc-009's "DELETE zero" shows no removal happened.
5. **Not a general agent-messaging standard.** Avoid parallel protocols (hub RPC, sidecar API, channels, DM, doorbell).

## 8. Tensions and contradictions
1. **Non-goal 3 vs code.** Social-analytics tools remain in the MCP surface, and arc-009 recorded DELETE as zero. I believe the code; the non-goal is aspirational.
2. **Non-goal 2 vs arc-011.** The arc builds an authoritative record, a forever-topic review and a backfill. I believe the arc; the charter is stale or will be rewritten.
3. **Non-goal 4 vs O-3 / T-3484.** O-3 covers dispatch, dead-worker recovery and isolated workspaces, which is orchestration by another name. I believe the effort is on orchestration.
4. **Two purpose statements.** The `CHARTER.md` canonical sentence (bus plus terminal endpoints) differs from the `objectives.yaml` headline ("trusted coordination substrate … dispatch and hand back work"). The sentence-drift canary guards the charter and README/ARCHITECTURE copies, not the headline.
5. **Value drivers vs product.** The driver set is AEF memory and recall themes (F-RECALL). Nothing in it covers delivery integrity, the product's stated core.
6. **Verbs vs effort.** The README names four verbs. Recent effort is dominated by verb 2 (the delivery rail and sidecar) and by process and governance. Verb 4, the founding verb, gets a daily canary and little new work.
7. **Trust model vs hardening.** The charter disclaims multi-tenant guarantees, yet O-7 builds per-agent authorisation. Either widen the stated scope or cap the work.
8. **Guard-layer growth.** There are about 99 `check-*` scripts, each tied to a past incident, and I found none retired. The detection layer risks outweighing the product it watches.
9. **Process fragility.** The operator-facing contracts live in CLAUDE.md, which `fw upgrade` can clobber. CLAUDE.md documents a loss of 40 lines on 2026-08-25.
10. **Blurred repo roles.** Product, AEF vendoring (2,130 `.agentic-framework/docs` files touched) and the operations ledger are mixed together. About 80% of file churn is non-crate.

## 9. Confidence and limits
- **Q1: medium-high.** LOC and tool counts are measured. Usage is inferred from CLAUDE.md, arcs and commit subjects, not runtime telemetry.
- **Q2: medium.** I saw no user or telemetry data. The single-operator reading comes from the "operator" language and the mirror setup.
- **Q3: medium-high** for arcs and dates, medium for the ordering of shifts. I sampled arc files, one report (T-3477) and commit statistics. I did not read the `.tasks` Decisions sections in bulk (about 3,300 task files).
- **Q4-Q7:** judgement; the weights are opinion.
- **Q8: high** for items 1-4 (direct text), medium for 6, 8 and 10. Those counts come from `git log --name-only` and are a crude proxy for effort. I did not measure per-line effort, and I did not read the underlying reports.
- **Not read:** most `.tasks` Decisions, `docs/architecture/parallel-execution-substrate.md`, the handovers, and any runtime logs or canary outputs. I did not run builds or tests (read-only).
