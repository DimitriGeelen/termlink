# T-2425: Ultra-Critical Review Round 2 — Purpose vs Field Reality

**Status:** research complete → synthesis → GO recommendation
**Type:** inception (one question → one decision)
**Question:** After round 1 (T-2419) shipped its gaps, does termlink's delivered
behavior now match its stated purpose — and which residual or newly-exposed gaps
warrant scoped correction work now?

**Method (C-001):** artifact created before research completed; updated incrementally.
Three evidence tracks: (A) closure-reality audit of round-1 gaps GAP-1..GAP-7;
(B) defaults-vs-purpose audit (retention/lifecycle/creation defaults in code);
(C) fresh field evidence since round 1.

---

## 0. Baseline: what round 1 concluded and shipped

Round 1 (docs/reports/T-2419-ultra-critical-purpose-review.md) dispositioned seven
gaps: GAP-1 README truth (T-2420 SHIPPED), GAP-2 channel.delete (T-2421 SHIPPED;
field-applied via T-2424: .107 hub → 0.11.603, 851 debris topics deleted,
1,584→733), GAP-3 MCP bloat (→ arc-005), GAP-4 authz (T-2422 INCEPT-DEFER), GAP-5
managed deploy (T-2423 INCEPT-DEFER), GAP-6 5-slice sprawl (policy learning),
GAP-7 silent-failure default (→ loud-contract work).

## 1. Fresh field evidence since round 1 (track C)

1. **T-1991 recurrence on .121** — agent-presence hit 64,808 records at
   retention=forever because .121's bespoke producer (checkout frozen ~2026-04-15)
   predates every retention rail; nothing on .121 ever ran `set-retention`. The
   retention primitive existed for months; the DEFAULT (forever) plus a stale
   producer reproduced the exact incident the primitive was built to end.
2. **Debris self-DoS on .107** — 851 of 1,584 topics were test debris; the walking
   verb (`claims-summary --all`) tripped the hub's own rate limiter. channel.delete
   provided the mop; nothing prevents re-accumulation.
3. **G-070 class** — .121's dashboard app AND its hub both ran as detached orphans
   outside systemd; the framework saw only symptoms via canaries, never the cause.
4. **G-084** — version floors structurally blind to capability absence; an 11th
   out-of-band canary had to be added.

## 2. Track A: round-1 closure reality

**5 of 7 gaps genuinely closed.**

- **GAP-1 CLOSED, truthful.** README.md:3-4 leads with the substrate identity;
  Guarantees (README.md:24-45), Trust Model incl. the G-064 caveat (:47-60), tool
  count corrected (:10). A front-door reader and an ADR reader now see the same
  system.
- **GAP-2 CLOSED.** T-2421 + T-2424 completed; `scripts/sweep-test-debris.sh`
  shipped; .107 hub live on 0.11.603 with `channel.delete`.
- **GAP-3 LIVE REMAINDER.** arc-005 in-progress (anchor T-2406); S1/S2 shipped;
  `.tasks/active/T-2408` (S3 long-tail trim) open; tools.rs still 45,619 lines.
- **GAP-4 / GAP-5 correctly parked** — T-2422 / T-2423 active inceptions awaiting
  operator decision (agent may not act; re-surfaced below).
- **GAP-6 RECORDED** — learnings.yaml:2567; T-2421 shipped with no MCP parity as
  the precedent.
- **GAP-7 LIVE REMAINDER + governance hole.** The round-trip loudness work landed
  under arc-003 (reliable-comms) which is CLOSED, yet four tasks remain open with
  NO arc tracking them: T-2385 (contact reachability preflight, partial), T-2389
  (relaunch .107 agents via T-2388 launcher), T-2402 (woken-but-silent rung),
  T-2224 (backpressure demo). Work that outlives its arc becomes invisible to arc
  governance — the exact class the arc system exists to prevent.

## 3. Track B: defaults vs purpose in code

1. **Implicit creation defaults to Forever.** `ensure_topic`
   (crates/termlink-cli/src/commands/channel.rs:2530) pattern-picks retention:
   `state:*` → Latest (T-2145); high-rate patterns → Messages(1000) (T-2126);
   **everything else → Forever**. Hub-side `channel.create` with no retention also
   defaults Forever (crates/termlink-hub/src/channel.rs:359-362).
2. **A TTL retention kind already EXISTS and is never auto-selected.**
   `Retention::Days(u32)` (crates/termlink-bus/src/retention.rs:6-29) trims by
   record age — reachable only via explicit `channel.create`/`set-retention`.
   The debris problem is not a missing primitive; it is a missing default.
3. **Retention is policy-without-enforcement.** No background sweep exists
   (retention.rs:3-4; T-1155 "enforcement is explicit, never implicit"). A topic
   with a bounded policy still grows unboundedly until an operator cron calls
   `channel.sweep`. The framework's own canary #5 (T-2252 topic-growth) exists
   solely to detect "the cron never fired" — an out-of-band watcher for an
   in-band abdication.
4. **Debris namespaces are known and enumerable.** T-2424's sweep allowlist
   (t-*, T-*, xhub-*, stress-*, scratch:*, smoke:*) is the exact namespace set
   that should never have been Forever in the first place.
5. **channel.delete is exact-name only** (by design, wildcards refused) — the mop
   works but is O(debris); prevention belongs at creation time.

## 4. Ultra-critical synthesis

Round 1 fixed **documents** and added **verbs**. The field incidents since then
show the remaining fight is with **defaults and enforcement**:

1. **The substrate still defaults to hoarding.** Every generically-named topic —
   including the five known test-debris namespaces — is born immortal (Forever).
   The T-2424 sweep removed 851 corpses; the maternity ward is unchanged. A
   substrate that requires a quarterly manual purge to keep its own observability
   verbs alive has a creation-time defect, not a cleanup defect.
2. **Retention that isn't enforced is documentation, not policy.** T-1155's
   "explicit, never implicit" pushed enforcement out to per-host crons — and the
   per-host cron is empirically the least reliable component in the entire estate
   (T-1991 twice, .121's never-installed cron, canary #5's existence). The cost of
   the purity has exceeded its value: an opt-in, env-gated hub-side sweep interval
   preserves explicitness (operator sets one env var in the unit file — the same
   place TERMLINK_RUNTIME_DIR already lives) while removing the N-crons failure
   surface.
3. **Arc governance leaks at closure boundaries.** Four live loudness tasks
   outlived arc-003. Nothing in the audit checks "open task references a closed
   arc" — work becomes orphaned precisely when its parent arc declares success.
4. **The two big inceptions (authz, deploy) remain the deepest unpaid debts** —
   both correctly parked awaiting operator; both re-surfaced in this report's
   recommendation.

## 5. Identified gaps and per-gap disposition

| # | Gap | Disposition |
|---|-----|-------------|
| R2-GAP-A | Debris namespaces born Forever — re-accumulation guaranteed post-sweep | **BUILD NOW** (T-2426): `ensure_topic` + hub-create auto-pick `Days(7)` for the known debris namespaces (t-*/T-*/xhub-*/stress-*/scratch:*/smoke:*); loud stderr note on auto-pick; tests. |
| R2-GAP-B | Retention enforcement depends on per-host crons that empirically don't fire | **BUILD NOW** (T-2427): env-gated hub-side periodic sweep `TERMLINK_SWEEP_INTERVAL_SECS` (default OFF = exact current behavior; T-1155 explicitness preserved — one opt-in env var in the unit file replaces N crons); telemetry counter in governor_status; tests. |
| R2-GAP-C | Loudness tasks orphaned by arc-003 closure; no arc tracks them | **GOVERNANCE NOW** (under T-2425): file arc-006 `comms-loudness` anchored on T-2385 bundling T-2385/T-2389/T-2402/T-2224. |
| R2-GAP-D | arc-005 S3 long-tail trim (GAP-3 remainder) | **EXISTING TASK** — advance T-2408 as budget allows; arc-005 cannot close without it. |
| R2-GAP-E | Authz (G-064) + managed deploy — round-1 DEFER inceptions still undecided | **RE-SURFACE** to operator: T-2422 / T-2423 decisions remain the highest-leverage unpaid debts (deploy = #1 incident source, 68 learnings). |

**Recommendation: GO** — round 2 found a coherent, buildable gap class (defaults &
enforcement) that round 1's verb-and-document work exposed but did not touch, plus
one governance leak. R2-GAP-A/B are scoped, testable, and low-risk (both preserve
current behavior unless opted in / only affect known-debris names). R2-GAP-C is
bookkeeping. R2-GAP-D/E are existing work re-prioritized.

## 6. Execution record (same session)

- **R2-GAP-A → T-2426 SHIPPED:** `is_debris_pattern` predicate (mirrored CLI+hub
  per T-2069, locked to the sweep script's allowlist) — CLI `ensure_topic` and
  hub `channel.create` now default debris-namespace topics to `Days(7)` instead
  of `Forever` (explicit retention always wins; loud stderr note / tracing line
  on auto-pick). 8 new tests; full suites green (hub 968 + CLI 412, 0 failed).
  Closed through the P-011 gate.
- **R2-GAP-B → T-2427 SHIPPED:** new `crates/termlink-hub/src/retention_sweeper.rs`
  — `TERMLINK_SWEEP_INTERVAL_SECS` (absent/0/unparseable ⇒ disabled with loud
  warn; clamped [30, 86400]) arms a hub-internal sweep loop enforcing every
  bounded topic per interval; per-topic errors warn-and-continue; three new
  `retention_sweep_*` telemetry fields on `hub.governor_status` so "armed but
  never firing" is visible. Default OFF = exact T-1155 behavior. 4 new tests +
  extended governor field test; hub suite 416 passed. Closed through the gate.
- **R2-GAP-C → arc-007 `comms-loudness` FILED:** anchored on T-2385; T-2385 /
  T-2389 / T-2402 / T-2224 re-homed via `arc_id` (arc-006 number skipped —
  reserved by T-1918 comments).
- **R2-GAP-B field application → T-2428:** .107 hub unit gains
  `TERMLINK_SWEEP_INTERVAL_SECS=3600` (git-tracked source + installed copy);
  binary rebuilt/installed; restart through the unit; verification in-task.
- **R2-GAP-D:** T-2408 (arc-005 S3) left as the next build item — existing task,
  arc-005 remains in-progress.
- **R2-GAP-E:** T-2422 / T-2423 re-surfaced to operator (unchanged).

## Dialogue Log

- 2026-07-21: Operator re-issued the standing directive verbatim ("please ultra
  critically review termlink's purpose and goals and identify gaps or needed
  adjustment, incept these and build these and test these, drive to completion") —
  fifth issuance, first after round 1 closed. Interpreted as: round-2 pass — audit
  round-1 closures, find missed gaps, build/test actionable ones, drive to
  completion.
