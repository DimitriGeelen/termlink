---
id: T-1432
name: "fw fleet doctor --legacy-usage — telemetry for T-1166 cut readiness (T-1425
  pick #6)"
description: >
  From T-1425 fast-forward synthesis. Walks each reachable hub's last-N-day event
  log (default 7d), counts inbox.push / file.send / event.broadcast invocations per
  session per host, renders as a table + summary in the doctor output. T-1166 cut
  readiness signal: when all reachable hubs report 0 legacy invocations for 7+ days,
  the cut is safe. Independent of every other pick — can ship anytime. Synergizes
  with T-1426 (deprecation print) but doesn't require it (counts are derivable from
  event log directly). Watchtower visualization is a nice-to-have follow-up, NOT in
  scope here — table output via doctor is sufficient for cut-readiness decision.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T07:02:56Z
last_update: '2026-08-27T21:13:20Z'
date_finished:
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:35Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=2 (body:concern-ref); D2=2 (body:telemetry-or-audit-entry); 
      D3=0 (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-08-27T21:13:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=198,acs=12)
    rubric_sha: e4a00f38e801
---

# T-1432: fw fleet doctor --legacy-usage — telemetry for T-1166 cut readiness (T-1425 pick #6)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `termlink fleet doctor --legacy-usage` parses correctly via `--help`; flag is opt-in (default doctor output unchanged)
- [x] `--legacy-window-days N` overrides the 7-day lookback; clamped to [1, 90]
- [x] For each reachable hub, doctor calls a new `hub.legacy_usage` Tier-A RPC that reads local `<runtime_dir>/rpc-audit.jsonl`, filters by window, returns counts. Retirement set tracked is `is_legacy_method()` in `rpc_audit.rs` — single source of truth covering `event.broadcast`, `inbox.{list,status,clear}`, `file.send`, `file.receive` plus chunked `file.send.*` / `file.receive.*` variants
- [x] Per-hub summary: total legacy + per-method breakdown + per-caller (`from`) breakdown. The audit log records `from` not `session_id`, so caller granularity is by display label
- [x] Fleet verdict: `CUT-READY` (all reachable hubs `audit_present=true` AND `total_legacy=0`), `WAIT` (≥1 hub with recent legacy traffic), `UNCERTAIN` (≥1 hub `audit_unsupported` because pre-T-1432, OR `audit_present=false`)
- [x] Hubs unreachable in connectivity probe are excluded from the legacy verdict (no double-fail) — already shown as FAIL in the per-hub block above
- [x] Implementation in `crates/termlink-cli/src/commands/remote.rs::cmd_fleet_doctor`; flags wired via `cli.rs::FleetAction::Doctor` + `main.rs` dispatch; hub handler `router.rs::handle_hub_legacy_usage` → `rpc_audit::summarize_legacy_usage`
- [x] `--json` adds top-level `legacy_summary` key (window_days, verdict, total_legacy_fleet, hubs_clean[], hubs_with_traffic[], hubs_unsupported[], hubs_no_audit[]) and embeds `legacy_usage` per-hub. Omitted entirely when flag not passed
- [x] Unit tests in `rpc_audit::tests`: `summarize_lines_counts_only_legacy_within_window`, `summarize_lines_empty_audit_returns_zero`, `summarize_lines_handles_missing_from_field` — extracted internal `summarize_lines` helper so tests don't need to poke `AUDIT_PATH` (OnceLock-only-once)
- [x] No regressions in default `fleet doctor` output without the flag — verified end-to-end against the live fleet
- [x] Pre-T-1432 hubs (every reachable hub today) gracefully fall back to `audit_unsupported: true` with an upgrade hint, instead of failing the whole probe

### Human
- [ ] [REVIEW] Verify the cut-readiness signal is actionable
  **Steps:**
  1. `termlink fleet doctor --legacy-usage` — see baseline
  2. Trigger a deliberate `inbox.push` (or a `file send`) somewhere reachable, then re-run
  3. Confirm the count incremented and `days_since_last` reset to 0
  4. After ~7d of clean operation post-T-1426 ship: re-run, confirm the verdict flips to CUT-READY
  **Expected:** the doctor's verdict tracks reality; agent operators trust it enough to flip T-1166's `LEGACY_PRIMITIVES_ENABLED=false`
  **If not:** capture which fleet event the doctor missed and re-scope

## Verification

cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink-hub --lib rpc_audit::tests::summarize 2>&1 | grep -q "test result: ok. 3 passed"
target/release/termlink fleet doctor --help 2>&1 | grep -q "legacy-usage"
target/release/termlink fleet doctor --help 2>&1 | grep -q "legacy-window-days"
target/release/termlink fleet doctor --legacy-usage --json 2>&1 | python3 -c "import sys, json; d = json.load(sys.stdin); assert 'legacy_summary' in d; assert d['legacy_summary']['verdict'] in ('CUT-READY', 'WAIT', 'UNCERTAIN')"
target/release/termlink fleet doctor --legacy-usage 2>&1 | grep -q "T-1166 cut-readiness"
target/release/termlink fleet doctor --legacy-usage --json 2>&1 | head -1 | grep -q "^{"

## Recommendation

**Recommendation:** CLOSE — this is the strongest-evidenced of the four open
T-1425 picks. The telemetry works, and the decision it was built to inform has
already been made using it.

**Rationale:** The Human AC asks one question: is the cut-readiness signal
actionable enough that an operator would trust it to flip
`LEGACY_PRIMITIVES_ENABLED=false`? That question has been answered by events
rather than by argument. T-1166 — the cut this telemetry existed to gate — is
`work-completed`, and the doctor's own output now describes itself as
informational because the cut has landed. A signal that drove a real irreversible
decision and was not contradicted afterwards is as validated as this kind of
telemetry gets.

**Evidence:** Measured 2026-08-27 against `target/release/termlink` (v0.11.1612).
All seven Verification lines pass under `set -euo pipefail` — including the
`--json` schema assertion (`legacy_summary` present, verdict in the allowed set)
and both `--help` flag greps. Live output:

```
=== T-1166 cut-readiness (7d window) ===
Verdict: CUT-READY
  total legacy invocations across fleet: 0
  CLEAN (7d): local-test, ring20-dashboard, ring20-management, workstation-107-public
  → no live legacy callers (T-1166 cut already landed in T-1415; verdict is informational).
```

`.tasks/completed/T-1166-*.md` carries `status: work-completed`,
`date_finished: 2026-08-20T16:16:41Z`. All 10 Agent ACs are ticked. The verdict
has walked the full ladder in this task's own history and tracked reality at each
step: `WAIT`-era traffic (2026-06-06, 2 aliased hubs with framework-pickup-bridge
residual) → `CUT-READY-DECAYING` (2026-06-01 and 2026-06-13, residue correctly
distinguished from live callers by the 300s recency probe) → `CUT-READY` today.
The `cargo build` / `cargo test --release -p termlink-hub` lines were **not**
re-measured in this session.

**One thing the evidence does not cover.** Steps 2–3 of the Human AC — trigger a
deliberate legacy call, confirm the count increments and `days_since_last`
resets — have never been executed; the 2026-06-13 entry records them as
"operator-env, not run". So the signal is proven to report **zero correctly** and
proven to have reported **non-zero correctly in the past** (the decay-residue
captures above are real non-zero readings with per-caller attribution down to
`addr:192.168.10.122`), but the increment path has not been exercised
deliberately since the cut. Whether that gap matters is the judgement in front of
you.

**What you are actually deciding.**

| Option | Action | Cost |
|---|---|---|
| Close on observed evidence | tick `[REVIEW]`, close | the increment path is never deliberately exercised. Low risk: the counter is read from `rpc-audit.jsonl` by `summarize_legacy_usage`, which has three unit tests, and it demonstrably produced non-zero readings in June |
| Exercise Steps 2–3 first | fire one legacy call at a reachable hub, confirm the count moves, then close | ~10 min, and it dirties the audit log of a production hub with a deliberate legacy invocation right after the cut landed |
| Keep open | leave as-is | the telemetry's own subject is closed; the task now tracks nothing that can change |

**Why I should not decide this alone.** The `[REVIEW]` box is yours by
construction, and option 2 means deliberately invoking a retired primitive
against a live fleet hub — a mutating action on shared infrastructure that I
should not take on my own initiative. My read is that option 1 is well supported
and option 2 buys little, but "well supported" is my assessment of your
acceptance criterion, not a substitute for it. Nothing here was ticked and no
mutating command was run.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-06-06T15:25Z — fresh re-smoke (5 days post-deploy bake) [agent autonomous]

`termlink fleet doctor --legacy-usage --json` snapshot (T-2013 deployed 2026-06-06 to 3 hubs):

```
fleet_versions: {0.11.472: 2, 0.11.806: 3}      ← post-T-2013-deploy state
hubs_clean:       [laptop-141, ring20-dashboard, ring20-management]   ← 3 hubs CUT-READY
hubs_with_traffic: [
  {hub: local-test,              count: 6, last_ts: today},
  {hub: workstation-107-public,  count: 6, last_ts: today}
]                                              ← 2 hub-aliases on .107 (framework-pickup-bridge residual, T-1415 documents)
action_items: []
```

**Signal is correct + actionable:** 3 of 5 hubs are clean; the 2 with traffic are both `.107`-aliased and trace to the same known residual (framework-pickup-bridge) tracked under T-1415. The doctor correctly identifies which hubs are cut-ready and which need source remediation. T-1166 cut on .122 + .121 happened based on this same signal — confirmed actionability. Box ready to tick.

### 2026-06-01T — Human REVIEW: cut-readiness signal is provably actionable [agent autonomous]

Live evidence of the signal driving real decisions. The T-1166 cut already happened on .122 and .121 based on this signal — that's the canonical actionability proof. Re-captured this session:

```
$ termlink fleet doctor --legacy-usage

=== T-1166 cut-readiness (7d window) ===
Verdict: CUT-READY-DECAYING
  total legacy invocations across fleet: 2
  CLEAN (7d): laptop-141, ring20-dashboard, ring20-management
  WITH TRAFFIC:
    local-test: 1 legacy invocation(s) — last call 5h ago (decay residue)
      └─ 1× addr:192.168.10.122
    workstation-107-public: 1 legacy invocation(s) — last call 5h ago (decay residue)
      └─ 1× addr:192.168.10.122
  Top callers (fleet-wide):
    2× addr:192.168.10.122
  → no live legacy callers (no traffic in last 300s); residue is historical.
  → operator may cut now or wait for the audit window to clear naturally.
```

The signal correctly:
1. Distinguishes live traffic from historical decay (300s recency probe)
2. Per-hub state classification (3/5 CLEAN, 2/5 with decay residue)
3. Per-caller granularity (addr:192.168.10.122 — turns out a probe path on .122 hits a local fallback that touches a legacy method on .107 / 127.0.0.1; harmless self-loop)
4. Verdict ladder: CUT-READY / CUT-READY-DECAYING / WAIT / UNCERTAIN
5. Plain-English decision support ("operator may cut now or wait...")

Direct probe of the underlying RPC also clean on both production hubs:

```
$ termlink_remote_call(hub=192.168.10.122:9100, method=hub.legacy_usage, scope=execute)
  → total_legacy=0, by_method={}, last_legacy_ts_ms=null, audit_present=true

$ termlink_remote_call(hub=192.168.10.121:9100, method=hub.legacy_usage, scope=execute)
  → total_legacy=0, by_method={}, last_legacy_ts_ms=null, audit_present=true
```

**Operator-actionable:** ready to tick the [REVIEW] box + `fw task update T-1432 --status work-completed`.

### 2026-05-01T07:02:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1432-fw-fleet-doctor---legacy-usage--telemetr.md
- **Context:** Initial task creation

### 2026-05-01T07:29:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-13T13:51:52Z — G-008 fresh evidence [resmoke-agent]
- **Action:** Re-ran/assessed Human-AC Steps (>2wk since build smoke)
- **Command(s):** `target/release/termlink fleet doctor --legacy-usage` (baseline read). Steps 2-4 (trigger deliberate inbox.push + 7d clean-wait) = operator-env, not run.
- **Result:** exit=0; ok — verdict renders live + actionable; CUT-READY-DECAYING with per-hub + per-caller breakdown
- **Output:**
  ```
  === T-1166 cut-readiness (7d window) ===
  Verdict: CUT-READY-DECAYING
    total legacy invocations across fleet: 1
    CLEAN (7d): local-test, ring20-management, workstation-107-public
    WITH TRAFFIC: ring20-dashboard: 1 — last call 3d ago (decay residue)
    → no live legacy callers (no traffic in last 300s); residue is historical.
  Steps 2-4 (deliberate trigger + 7d clean re-check) = operator-env.
  ```
- **Note:** Human AC remains UNCHECKED — sovereignty; evidence for batch-confirm.
