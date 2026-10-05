# GLM review — CAND-3 verification tiers (2026-10-05, rerun with files copied locally)

# External review: CAND-3 tiered verification (T-3344, OD-17 CAND-3)

**Q1 — the selection rule**

1. [AGREE] Lowest-tier-first with a negative control at that tier is the right cost answer to the operator's "too heavy" objection, and keeps P1.2.e's negative control per tier, not just at the top.
2. [CHANGE] "The tier that can see its failure" is knowable only after a failure-mode analysis, and the plan doesn't require one: all three production failures were classes the local checks *claimed* to cover and passed anyway. Each requirement's tier must be justified by naming the failure mode and the component whose observation catches it, re-audited when new incidents land.

**Q2 — claims placed too low**

3. [CHANGE] Per-stage idempotence at tier 1 is too low: CAND-2 as ruled (OD-17.3) puts the stage memory "at the hub per OD-5", and today's hub dedupe holds (sender, id) for 5 minutes only (`dedupe.rs:41`, T-2049) — duplicate suppression past 5 min and across a hub restart is a tier-2 property of the settlement record, not fixture logic.
4. [CHANGE] Consent allow-list at tier 1 is too low: failure #3 was a key-*binding* failure (fresh heartbeat, signing key stored under another name, 32,205 refusals); the allow-list rule is green while the deployed signing identity is wrong. Needs tier 3 with real keys; the canary is only a late backstop.
5. [CHANGE] Tier 3 must run through the deployed scheduler and hooks, not a test-invoked tick: failure #1 was healthy logic with nothing scheduling the injector (R-17 status: `*/5` driver, checks no prompt). If the scripted peer drives the tick, tier 3 passes while production is dark.
6. [MISSING] Nothing checks agent↔inbox↔hub *binding*: failure #2 (second hub, other runtime dir) had two locally-healthy hubs. Tier 2's authenticated hub-id RPC is necessary, not sufficient; add a binding assertion — the hub the session writes to is the hub owning the inbox — with a wrong-runtime-dir negative control. CAND-8 exists for exactly this and is absent from the plan.

**Q3 — missing tiers**

7. [MISSING] Two hubs / cross-host: OD-1 ruled circuits between hubs; OD-11 adds card exchange, "moved to hub X", the same-pid-at-X-and-Y flag, only-home-hub-says-DEAD. No listed tier can observe a cross-hub failure; tiers 1–4 as drafted are single-hub. Needs a two-real-hub tier (or an explicit tier-4 extension) before arc-011 closes on a circuit design.
8. [MISSING] Adapter parity: OD-7 ruled C with "a parity test per release" for the harness adapter contract; the tier list tests only Claude Code. Tier 3 with one harness proves nothing about the contract.
9. [MISSING] Restart/reboot: R-8 (SIGKILL respawn, systemd and non-systemd), R-15 (kill after STORED), GP-5/CAND-2 (restart mid-delivery, no second hand-over) name crash behavior at no tier. Fold into tiers 2–3 as negative controls, but name them.
10. [MISSING] Deployed install vs source tree: R-39 (clean host from the release artifact) and OD-14's own rationale ("healthy-looking sidecars with no scheduler, wrong hub, missing hooks … detected only by an end-to-end canary") — every tier as described runs from the checkout. The daily canary must run *from the deployed artifact*, and the closing rule should include an installed-artifact run, or the plan under-tests precisely the observed failure class.

**Q4 — negative controls at tiers 2–4**

11. [CHANGE] Concrete and mandatory, plus a kill-check (the test must be shown green before the break and red after; an always-red test is as worthless as always-green):
    - Tier 2: start a second hub in a different runtime dir and point the receiver at it — the binding/round-trip check must go red (reproduces failure #2).
    - Tier 3: store the receiver's signing key under the wrong name (reproduces failure #3) — the delivery-confirmation flow must fail visibly, never pass on a fresh heartbeat.
    - Tier 4: kill the receiver's sidecar after STORED — the sender must show STUCK within the OD-3 deadline, the canary must escalate by the next session (OD-14 3d), and REPLIED must never appear.

**Q5 — closing rule**

12. [CHANGE] Not sufficient against "shipped but dark": a one-time live pass plus a daily canary leaves up to 24 h dark, and the canary covers one agent pair, not per-host drift. Add continuous per-agent reachability (CAND-6: up, right hub, adapter present, last surface time), rotate the canary across hosts/pairs, and re-run the fleet negative-control drill after known-breaking events such as cert rotation (OD-12: rotation renames every inbox).

**Q6 — weight to remove**

13. [CHANGE] A mutant for every requirement on every push over-tests P3 items (R-36 reflection, R-38 future reader): keep mutant-per-push for the invariants (idempotence, ordering, consent, R-34 never-fallback) and use lighter property tests elsewhere; the fixture retention test duplicates the tier-2 sweep; "conversation continues" and "dead agent resumed mid-conversation" are one scenario, not two.

**Verdict:** Direction right, but as drafted it would have passed all three cited failures — add the four missing tiers (cross-hub, adapter parity, restart, deployed install), raise idempotence/consent/scheduling to the tier where those failures actually live, and make the canary continuous, per-host, and deployed-artifact-rooted.
