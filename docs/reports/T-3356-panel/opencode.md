I'll do the full read-only review now (hash checks, reading every document), then present the complete answer content as a plan for approval before writing the file (plan mode forbids the write until then).
All ten hashes match. Now reading the material under review in full — PC first (highest priority per the brief), then ARC, REQ, PCa, PCb, then the context docs.
PC (06) read in full. Now reading ARC (04 architecture).
ARC read in full. Now REQ (v0.5.3).
Now PCa (06a, AL-17..AL-40) — the PH-2 hardening.
Now PCb (06b, later phases AL-41..AL-53).
Now the context docs. First a quick map of EV and THR.
Review complete: all ten hashes match; REQ, ARC, PC, PCa, PCb read in full, plus THR §18 (SI register), EV §6.1–6.4/§14, FP floor table, and targeted REQ/THR sections. Since plan mode is active, I have not written the answer file yet. Here is the plan.

**Plan: write exactly one file, `docs/reports/T-3356-panel/opencode.md` (new file, no other changes), with the content below.**

Verdict reached: fit to plan the build from after the high finding is fixed. 12 findings: 1 high (forward-once can wedge role mail with no offerable copy at the current main — edge-triggered forwarding + identity-dedupe dead end, and E-35's assert would pass it), 4 medium (no per-tick time budget vs the "300 s + 1 tick" watchdog bound; O-2's watchdog guarantee is host-conditional per FCW-41/QR-5; cross-host `wall()` lease comparisons vs R-57; E-35's "visible non-final state" escape hatch + forged-Touch lease hold), 7 low (46-vs-44 FCW count in ZR-1(a); AL-6 step 8 dangling intent; quota share arithmetic; AL-17 mint ambiguity; unbounded attempt count; R-23.a's `pty inject` wording; plus soundness list and not-reached sections).

The full content to write:

```markdown
# T-3356 review panel — seat `opencode`

**Seat:** opencode · **Model:** GLM (glm-5.3, zai-coding-plan/glm-5.3), running as opencode · **Date:** 2026-11-11
**Hashes:** all ten of the brief's section 2 match (`sha256sum`, run in the repository root). No differences.

## Verdict

Fit to plan the build from **after the high finding is fixed**, with the mediums decided before their build items ship. No critical finding (no message loss, forgery or fail-open found on the delivery path). What matters most: (1) forward-once is triggered only at STORED and at restart, and a final FORWARDED copy still occupies the identity triple, so role mail can wedge with no offerable copy at the current main (OPENCODE-1); (2) the tick has no time budget, so the watchdog's "300 s + 1 tick" bound is not actually guaranteed (OPENCODE-2); (3) two ruled guarantees are quieter than ruled: the watchdog never fires on hosts where the veto cannot measure (O-2, OPENCODE-3) and the lease compares wall clocks across hosts (R-57, OPENCODE-4).

## Findings

### OPENCODE-1
- Where: PC 5.14 AL-14 (after_stored, start-up scan), 5.15 AL-15 step 7; ARC 7.2/7.3; REQ R-67.a1 (4), R-51.a
- Severity: high
- Class: algorithm
- What is wrong: forward-once is edge-triggered only (at STORED and at sidecar start), and a receiver's final FORWARDED copy still occupies the identity triple, so role mail can end up with no offerable copy at the current main.
- Why: two scenarios. (1) X accepts a FORWARD at generation g (the inner message is stored at X as role mail, AL-15 steps 6..7). The lease then moves to Z — routine, since fast takeover moves main after 5 min of holder human silence. During normal running nothing re-runs `after_stored` for X's copy (the start-up scan is the only re-trigger, AL-14 crash table row 1), X's own sessions are withheld (AL-12), and the sender does not re-send after STORED (R-31.a governs re-sends "before STORED" only). The mail waits at a non-main copy until X restarts or becomes main; R-67.a1 (4) ("a non-main copy that receives role-addressed mail forwards it once to the main the hub currently names") is implemented only for the receipt moment. (2) Y has marked its copy FORWARDED ("final at Y, never handed over at Y", AL-14 step 8, invariant 3). The sender, told FORWARD-STALE or chasing a STUCK "not handed over", re-resolves; if resolution lands on Y's agent again, the re-posted message is deduped by the identity triple (AL-15 step 7; R-51.a "STORED but not handed over: not stored twice") while hand-over at Y is barred by AL-14's invariant — no copy anywhere is offerable. The design itself asserts the dedupe ("the sender's ladder re-resolves and re-sends the same id, which the new main dedupes", AL-15 edge (1)) without covering the case that the "new main" holds only final or stale copies. Doubt marked: the built dedupe's treatment of final-state copies was not read (`deliver.rs` is cited by path, not hashed); if it re-admits messages whose prior copy is final, scenario (2) collapses into scenario (1), which stands either way.
- Fix: PC 5.14/5.15 (and ARC 7.2/7.3 with it): make forwarding level-triggered on lease-generation change for stored non-final role mail (re-evaluate when the lease reader observes a generation transition, not only at start-up), and state that identity-triple dedupe does not suppress admission when the receiver's own prior copy is final (FORWARDED): re-admit and deliver, or refuse with a notice that makes the sender use a new id. Extend E-33/E-35 (see OPENCODE-9).

### OPENCODE-2
- Where: PC 5.3 AL-3 meter_tick, 3.1.c TICK, 5.8 AL-8 invariant 2, FCW-15
- Severity: medium
- Class: algorithm
- What is wrong: the tick has no time budget: meter_tick performs one `session.input_activity` RPC per session with a 2 s timeout, before AL-5/AL-8, and the session table allows 64 live sessions (FCW-7).
- Why: 64 sessions × 2 s of timeouts is up to 128 s of per-tick work against a 30 s tick ("the tick thread evaluates every per-tick routine once per tick", 3.1.c). A set of dead or hung sessions (RPC timeouts rather than fast refusals) delays ring_tick and watchdog_tick by several ticks, so AL-8 invariant 2 ("a fire happens within 300 s + 1 tick") and FCW-15's freshness story can be violated with no fault visible. Nothing bounds the tick's duration or parallelises the reads; AL-6's re-check path adds further serial RPCs per ring.
- Fix: PC 5.3: bound the per-tick metering work (parallel or sampled RPCs; a per-tick deadline after which the remaining sessions read UNKNOWN for that tick — already the safe state), and restate the watchdog bound as "300 s + 1 tick + the tick budget".

### OPENCODE-3
- Where: PC 5.6 AL-6 step 4 / FCW-41, 5.8 AL-8, QR-5, ZR-4; ARC 6.9.0/6.9.7 (O-2)
- Severity: medium
- Class: ruling
- What is wrong: on a host where the activity veto cannot measure, the watchdog never fires, so O-2's ruled guarantee — the external 300 s watchdog "makes sure a stuck or false READY flag can never hold a message forever" — holds only on /proc-like hosts.
- Why: FCW-41 holds the watchdog when the veto is Unmeasurable; QR-5 names the consequence (a stuck READY there waits for the R-44.e notice and the operator). O-2's text has no host carve-out. R-81.3 rules "no veto where it cannot be measured" for the doorbell, but the watchdog was ruled as the backstop for exactly the stuck-flag case, and on such hosts there is no backstop. QR-5 is a step-6 residual the operator has not accepted yet (this panel precedes the disposition).
- Fix: either an operator-accepted residual in ARC 19 / PC section 8 naming the host class and its compensating control, or a reduced-check fallback for the watchdog on such hosts (foreground and harness liveness only), recorded in ARC 6.9.7.b.

### OPENCODE-4
- Where: PC 5.11 AL-11 (FCW-56, 5.11.f.3), 5.13 AL-13 step 3; REQ R-57.a
- Severity: medium
- Class: ruling
- What is wrong: the fast-takeover check and the "reading main" check compare the holder's `last_human_at` (the holder host's `wall()`, AL-11's records) against the taker's/reader's `wall()` across hosts.
- Why: R-57.a rules every deadline is measured "on one clock, the observer's own, never across hosts". FCW-56's +30 s margin covers skew under 30 s only; 5.11.f.3 concedes a larger skew can cause an early takeover — which takes main from a session the operator is actively prompting, the exact harm O-3's tightening exists to prevent. AL-13's `wall() - L.last_human_at <= 10 min` has the same shape with no margin at all. The design kept the built wall-clock lease ([B], ZR-3) but never records the cross-host read as a ruled exception to R-57.
- Fix: stamp `last_human_at` with the hub's receive time when the Touch record is posted (the hub is one place, still satisfying "every claimer … reads the holder's human activity from one place", ARC 6.13.3), or record an explicit R-57 exception with a measured skew bound under V-5.

### OPENCODE-5
- Where: PC 6.1 vs 9.1 ZR-1 (a), 10.2
- Severity: low
- Class: other
- What is wrong: 6.1 and 10.2 say 46 FCWs have no spec yet; ZR-1 (a) asks step 5 to "add specs for the 44 FCWs listed in 6.1".
- Why: the 6.1 list counts 46 ids (counted by hand; 148 + 46 = 194 is consistent), so the "44" in ZR-1 (a) is wrong and could make step 5 treat two of the listed conditions as covered.
- Fix: PC 9.1: correct "44" to "46".

### OPENCODE-6
- Where: PC 5.6 AL-6 steps 7..8, FCW-35; 5.7 AL-7 on_start
- Severity: low
- Class: algorithm
- What is wrong: the stale-checks test (step 8) runs after the durable intent write (step 7), so a ring held for staleness leaves an intent-without-outcome record in normal operation, which only a restart reconciles.
- Why: on_start treats "ring intent with no outcome" as UNSUBMITTED (FCW-37), but it runs only at start; during normal running the dangling intent never gets an outcome, so the ring log accumulates records that OBS-9 readers and the H-11 (e) accounting cannot distinguish from a crash. Safety is unaffected (step 8 precedes the keystrokes).
- Fix: PC 5.6: move the staleness check before the intent write, or append `held:stale-checks` as the outcome so every intent has one.

### OPENCODE-7
- Where: PCa 2.7 AL-23 step 8 (with ARC 6.3.1)
- Severity: low
- Class: coverage
- What is wrong: the quota arithmetic is not computable as written: "budget.count >= its share of 256" with reserved shares of 10% and 20% gives non-integer limits, and the general budget's limit (the full 256, or the remainder after the reserved shares) is unstated.
- Why: ARC says the reserved shares are "inside the aggregate"; an implementer can read AL-23 as general ≤ 256 plus reserved budgets on top (aggregate up to ~307, breaking SI-19's ceiling) or as a partition (general ≤ 179). E-50/E-51 do not pin the arithmetic.
- Fix: PCa 2.7: state integer per-class limits (e.g. receipts 26, own project 51, general 179 of 256; bytes likewise) and that the classes partition the aggregate.

### OPENCODE-8
- Where: PCa 2.1 AL-17 step 2 and the legacy migration
- Severity: low
- Class: other
- What is wrong: the mint condition "if a certificate will exist" is not decidable at first start, and the legacy migration ("writes hub.id = first16(current TLS fingerprint) once") is undefined for a hub with prior state and no certificate.
- Why: a hub started socket-only that later gains `--tcp` minted a random id; the wording invites a rebuild toward the fingerprint form. A socket-only legacy hub has no fingerprint for the migration to write. Both points can be built two ways.
- Fix: PCa 2.1: mint from the certificate only if one is present at first start, else random; the legacy migration falls back to random when no certificate exists.

### OPENCODE-9
- Where: EV 6.4 E-35.4, E-33; 05a 2.1 row R-67.a1
- Severity: medium
- Class: evidence
- What is wrong: E-35's assert passes a message that is never handed over ("exactly one HANDED_OVER **or a visible non-final state by its deadline**"), and no spec exercises a re-resolution that lands on the forwarder holding the final copy.
- Why: the soak explicitly injects lease moves every 50 messages and kills (E-35.2), i.e. it generates the wedge conditions of OPENCODE-1, and a wedged message satisfies the assert through the "visible non-final state" disjunct; 05a row R-67.a1 then reads "covered". A test that passes when the property (role mail reaches a main) is broken is a negative-control gap.
- Fix: EV 6.4: split the assert — per message id: HANDED_OVER, or an enumerated operator-visible outcome (UNCERTAIN, REFUSED); anything else FAIL; add to E-33 the variant "re-resolve lands on Y, which holds the FORWARDED final copy".

### OPENCODE-10
- Where: PC 5.9 AL-9 step 5, 5.7 AL-7, 4.2
- Severity: low
- Class: algorithm
- What is wrong: delivery attempts are unbounded in count: each proof of absence re-offers the message "as attempt + 1 with a new nonce", and rings continue after STUCK "because the message stays owed" (4.2), with no per-message attempt cap or per-message escalation.
- Why: a session whose adapter keeps reporting `complete` while the hook never surfaces the frame is re-rung and re-pulled indefinitely at a bounded rate (≥ 10 min); the sender sees STUCK, but the receiver's escalation is only the coalesced notices, and "never by a timeout alone" (AL-9 invariant 3) means only the operator stops it. Bounded and visible, hence low.
- Fix: PC 5.9: after N attempts (a value for FP-4) the message leaves the ring pipeline into a per-message needs-attention state until the operator acts, as UNCERTAIN already does for the no-complete-point case.

### OPENCODE-11
- Where: PC 5.11 AL-11 (Touch records, last_human), 8.4 QR-3; THR SI-20
- Severity: medium
- Class: threat
- What is wrong: `last_human_at` is only as trustworthy as the holder's own signed Touch records, so a compromised or misbehaving holder session (same OS user) can keep it fresh indefinitely and defeat both lapse and fast takeover — keeping main forever.
- Why: QR-3 names the forging of a human claim (`termlink main claim --origin human`) but frames the impact as renewal; the same primitive (a same-user process, RR-2/AR-6, inside SI-20's accepted pre-operator-key limit) also blocks every takeover path, which is precisely the "forgotten session renewing forever through automated prompts" risk O-3 was ruled to close (ARC 6.13.0). E-31 measures the basis share of renewals but cannot see a forged Touch, which looks legitimate by construction.
- Fix: PC 8.4 / ARC 19 (AR-6): state explicitly that until an operator key exists the tightening does not bind a same-user adversary, and consider a hub-side cross-check (Touch rate against adapter-reported prompt origins) as a detector.

### OPENCODE-12
- Where: REQ R-23.a vs PC 5.6 AL-6 session side, ARC 4.4/6.9.3
- Severity: low
- Class: coverage
- What is wrong: REQ R-23.a still says the line is typed "with `termlink pty inject`"; the design types through the session's `command.notify` (notify scope, below interact) since T-3533, and XR-9 (v0.5.3) updated R-19/R-20/R-47/R-67 but not R-23.
- Why: a builder reading REQ literally would wire the doorbell through a `control`-scoped verb instead of the notify scope that SI-2's checks and E-2/E-3 assume; a wording mismatch with no owner.
- Fix: a step-1 change request from this step's disposition, rewording R-23.a to "typed into the session by the session's notify-scope doorbell action".

## What I checked and found sound

1. Hashes: all ten of section 2 match.
2. SI-1 line: compose() output, the 150-byte test vector (5.6.c.1), E-3's regular expression and the shell-inertness probes are mutually consistent; id validation is enforced on both sides of the RPC (FCW-33/34) and at admission (FCW-100).
3. Delivery strictness: the only writer of HANDED_OVER is the finalizer's nonce-in-harness-record check (FCW-44), the claim precedes content (FCW-43), a timeout never redelivers (FCW-45/46), and the crash table covers every KillPoint; D-1.4 matches the algorithm.
4. The doorbell gate and re-checks implement O-2 D+'s conditions in the ruled order, each failing closed (AL-5, AL-6; E-1, E-2); the check-to-keystrokes interval is logged, and the residual race is kept as a limit probe (E-14), not claimed closed.
5. The ring schedule (AL-7, 4.2) is consistent with ARC 6.9.4/6.9.5, including the unsubmitted-line rule and the 0, 4, 12, 32, 52… table; D-1.2 matches; E-5's expected table is being aligned by ZR-1 (c).
6. The watchdog: audit row before keystrokes (FCW-42), silence over all three sources with arrivals excluded (FCW-14, E-9), at most one fire per 300 s, restarts never shorten the silence (rebuilt from durable logs) — sound except OPENCODE-2/-3.
7. Lease origin rules match O-3 and ARC 6.13: ignored `automated`/`unknown` claims (FCW-52/53), PostToolUse removed, the provisional 5-min claim consistent across ARC 6.13.2, AL-11 and E-23 (answer 4.5), the fast-takeover bound as E-24 tests it, and the NID rule for absent-origin projects (FCW-53, E-22) keeps the 8 live projects working.
8. The kill switch is read on every typed and hook path with unreadable = off (FCW-49); the "within one tick" bound follows from the check placement (AL-10 edge 2).
9. A single forward's mechanics: dual verification with domain separation (AL-20, V-FWD-1), original bytes unchanged, hop accounting, roster check, and the 15 s freshness rule with its stricter-than-built rationale (4.6).
10. Revocation: the failed-pull classification (4.7), rollback and fork refusal, `unverified_since` semantics and the PN-13 narrowing match SI-11 and DR-8; the signed head (ZR-9) closes the unsigned-"unchanged" hole.
11. The seam gate (AL-16) with production refusal-to-start (FCW-77/78) and the scan (ZR-1 (b)) closes QR-1's live `TERMLINK_MAIN_TEST_*` exposure consistently with NID.
12. Later-phase decision logic (AL-41..AL-44): the one-transaction consume and the journal-based recovery with OUTCOME_UNKNOWN escapes are coherent, the lock rules name every writer, and the r1 remediation (FCW-191..194) addresses the real races.
13. The FCW register: a ~40-row sample of the 194 rows matches the inline FCW definitions; every AFC-1..AFC-34 is refined at least once.

## Not reached

- TRC (04a) and COV (05a) were not read in full; coverage was checked only where EV/PC cite it (E-33/E-35, row R-67.a1).
- THR was read for section 18 (SI-1..SI-31) and cited TH rows; sections 15..16 in detail were not re-walked.
- FP was read only at the floor table (3.1, 3.4) and gates; H-n hypothesis details were not re-checked.
- EV sections 6.5..6.13 and 8..10 were skimmed; E-36..E-103 were not read one by one.
- The built code (LEASE, `deliver.rs`, `main_lease.rs`, `lease.rs`) is cited by PC but not hashed and was not read; all [B] claims are taken as stated (the one place it matters is marked in OPENCODE-1).
- The register's remaining ~150 rows and a re-render of the 23 Mermaid blocks were not re-verified (PC's own render-check outputs were not re-run).
```

Shall I write this to `docs/reports/T-3356-panel/opencode.md`? (No other file will be created, edited or deleted; no git state changes.)
