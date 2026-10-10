## 1. Rulings not consistently propagated

**1a. Hub confidentiality still has the rejected baseline.** Section 22.18’s B′ ruling requires encrypted content, an encrypted hub copy and custody-based successor access. Yet §12.3 and CR-9 still leave the content copy undecided; §15.5.3 and §19.2/PR-33 defer encryption; RR-6 still describes readable content as the unresolved exposure; and §17.1/BP-3 and §17.3.2 say reading stays open. These are consequential target-state contradictions, not merely descriptions of today’s implementation.

**1b. Approval-delay remnants include the acceptance probe.** Section 6.3.e explicitly replaces the old delay, and PN-14 is marked retired. However, §18.1/SI-31’s probe still tests cancellation during a notice period rather than refusal without active confirmation. Section 15.9.9/TH-64, §19.2/PR-27 and RR-16 also retain the delay as a current mitigation. CR-14’s amendment marker is correct, but its subsequent “Requirements” sentence still says approvals take effect after a notice period. This conflicts with §22.18/OQ-14.

**1c. Dropped retention remains a live option.** CR-16 is correctly marked dropped, but §8.2.a/PR-34 and §19.1/PN-16 are not. Section 8.2’s final case, §18.1/SI-4, §19.2/PR-34, RR-10 and §21.2.3 continue offering or conditioning behavior on that option. Section 22.18/OQ-15 dropped all three IDs; these references must become explicitly historical.

**1d. Duplicate-main denial remains current elsewhere.** RR-8 correctly labels its old text historical, but §14.13’s denial row, §15.6.16–18/TH-33 and §16.2.3/AC-3 still say a second copy blocks resolution until an operator pin. CR-20 and §22.18/RR-8 instead require immediate home-hub resolution.

**1e. The sender-counter correction missed two summaries.** Section 15.3.6/TH-11 and §19.2/PR-3 still require rebuilding the sender counter from the hub. They contradict the explicit correction in CR-4 and §22.18/D4: the sender’s durable counter is authoritative; the hub’s highest-seen value can only raise it.

**1f. Vendor channels remain “open by design.”** Section 17.1/BP-14 still assigns an “outside the system” target; §§17.2 and 17.3.1 call it open by design. CR-21 and §22.18/RR-13 instead prohibit its use for governed work and require adapter disabling or logging. Section 17.4/PR-29 also leaves authorization as a future proposal without incorporating the ruled own-project default and cross-project Tier 0 route.

## 2. Incomplete change requests and invariant alignment

**2a. CR-20 omits an independently ruled requirement it changes.** Requirements **R-34.o [R]** explicitly says two live copies yield “authority unknown.” CR-20 names R-67.a, R-62.a and R-32.o, but not R-34.o. Reopening only the named requirements would preserve this contradiction. Its acceptance alignment should also include requirements R-67.e.

**2b. The history-sharing amendments promised by the register are absent.** Section 22.18/OQ-13 says CR-15 gains invitation intent and a share event, and CR-19 gains “may share history.” Neither change appears in those CRs. Section 8.6.d contains the ruling, but CR-19’s expressly “small and fixed” action vocabulary omits sharing. The step-1 handback therefore loses part of the accepted policy model.

**2c. Operator-only authorization conflicts with approved routes.** Section 9.3.c and §18.1/SI-8 say admissions, grants and allow-list changes occur “only through the operator channel.” CR-18 expressly permits policy routes before an operator-key route exists, and CR-19 permits approved routes to change policy. The invariant must distinguish an untrusted peer message granting authority from an authenticated request receiving a policy-route decision. Section 18.2 also omits these new dependencies.

**2d. OQ-16 is missing from the operative readiness checks.** Contrary to §22.18’s stated effect, §9.2.d/PR-32 and §18.1/SI-2 contain no activity veto or corresponding probe. CR-5 contains it, but the acceptance table would allow its omission. Separately, CR-5’s busy bit on presence needs reconciliation with CR-11 and §15.5.6/PR-10, which say peers see **only** reachability and version class: specify the busy bit’s audience or amend that restriction.

**2e. The evidence-rule dependency points to the wrong CRs.** Section 18.2/SI-17 and §19.2/PR-18 list CR-5 and CR-6, although CR-17 expressly introduces the harness-written evidence rule. Section 21.2.4 identifies CR-17 correctly. This can mislead the security-floor dependency mapping.

## 3. Final-record status remains contradictory

**3a. Accepted decisions still appear unmade.** Section 19.1 says no numbers come from rulings, contrary to §22.18/OQ-3 and OQ-9’s adopted starting values and first-build measurement obligations. Section 7.3.a says RR-2 has not been accepted; §22.3 leaves separate accounts merely optional, despite §22.18/OQ-2’s committed target. Section 21.2.2 still awaits OQ-16, and RR-18/BP-19 still call the adopted interim re-pin audit merely proposed (§22.18/OQ-8). These need status corrections; they do not require new decisions.

## 4. Verdict and fixes, smallest first

**Verdict: consistent after listed fixes.** The register supplies the decisions, but the body and acceptance tables do not yet reliably express them.

**4a.** Correct dependency links for SI-17/PR-18 and add R-34.o and R-67.e to CR-20’s affected requirements (§2a, §2e above).

**4b.** Mark PR-34/PN-16 and their operative references dropped; correct the counter summaries (§1c, §1e).

**4c.** Update accepted-status language, adopted numbers, account-separation target and interim re-pin audit (§3a).

**4d.** Replace stale duplicate-main and vendor-channel target descriptions (§1d, §1f).

**4e.** Complete CR-15/CR-19’s history-sharing amendments and align SI-8 with approved routes (§2b–2c).

**4f.** Update SI-31’s confirmation probe and SI-2/PR-32’s activity-veto checks; resolve the busy-bit visibility conflict (§1b, §2d).

**4g.** Propagate B′ into CR-9, confidentiality mitigations, residual risks and bypass targets (§1a).
95,465
## 1. Rulings not consistently propagated

**1a. Hub confidentiality still has the rejected baseline.** Section 22.18’s B′ ruling requires encrypted content, an encrypted hub copy and custody-based successor access. Yet §12.3 and CR-9 still leave the content copy undecided; §15.5.3 and §19.2/PR-33 defer encryption; RR-6 still describes readable content as the unresolved exposure; and §17.1/BP-3 and §17.3.2 say reading stays open. These are consequential target-state contradictions, not merely descriptions of today’s implementation.

**1b. Approval-delay remnants include the acceptance probe.** Section 6.3.e explicitly replaces the old delay, and PN-14 is marked retired. However, §18.1/SI-31’s probe still tests cancellation during a notice period rather than refusal without active confirmation. Section 15.9.9/TH-64, §19.2/PR-27 and RR-16 also retain the delay as a current mitigation. CR-14’s amendment marker is correct, but its subsequent “Requirements” sentence still says approvals take effect after a notice period. This conflicts with §22.18/OQ-14.

**1c. Dropped retention remains a live option.** CR-16 is correctly marked dropped, but §8.2.a/PR-34 and §19.1/PN-16 are not. Section 8.2’s final case, §18.1/SI-4, §19.2/PR-34, RR-10 and §21.2.3 continue offering or conditioning behavior on that option. Section 22.18/OQ-15 dropped all three IDs; these references must become explicitly historical.

**1d. Duplicate-main denial remains current elsewhere.** RR-8 correctly labels its old text historical, but §14.13’s denial row, §15.6.16–18/TH-33 and §16.2.3/AC-3 still say a second copy blocks resolution until an operator pin. CR-20 and §22.18/RR-8 instead require immediate home-hub resolution.

**1e. The sender-counter correction missed two summaries.** Section 15.3.6/TH-11 and §19.2/PR-3 still require rebuilding the sender counter from the hub. They contradict the explicit correction in CR-4 and §22.18/D4: the sender’s durable counter is authoritative; the hub’s highest-seen value can only raise it.

**1f. Vendor channels remain “open by design.”** Section 17.1/BP-14 still assigns an “outside the system” target; §§17.2 and 17.3.1 call it open by design. CR-21 and §22.18/RR-13 instead prohibit its use for governed work and require adapter disabling or logging. Section 17.4/PR-29 also leaves authorization as a future proposal without incorporating the ruled own-project default and cross-project Tier 0 route.

## 2. Incomplete change requests and invariant alignment

**2a. CR-20 omits an independently ruled requirement it changes.** Requirements **R-34.o [R]** explicitly says two live copies yield “authority unknown.” CR-20 names R-67.a, R-62.a and R-32.o, but not R-34.o. Reopening only the named requirements would preserve this contradiction. Its acceptance alignment should also include requirements R-67.e.

**2b. The history-sharing amendments promised by the register are absent.** Section 22.18/OQ-13 says CR-15 gains invitation intent and a share event, and CR-19 gains “may share history.” Neither change appears in those CRs. Section 8.6.d contains the ruling, but CR-19’s expressly “small and fixed” action vocabulary omits sharing. The step-1 handback therefore loses part of the accepted policy model.

**2c. Operator-only authorization conflicts with approved routes.** Section 9.3.c and §18.1/SI-8 say admissions, grants and allow-list changes occur “only through the operator channel.” CR-18 expressly permits policy routes before an operator-key route exists, and CR-19 permits approved routes to change policy. The invariant must distinguish an untrusted peer message granting authority from an authenticated request receiving a policy-route decision. Section 18.2 also omits these new dependencies.

**2d. OQ-16 is missing from the operative readiness checks.** Contrary to §22.18’s stated effect, §9.2.d/PR-32 and §18.1/SI-2 contain no activity veto or corresponding probe. CR-5 contains it, but the acceptance table would allow its omission. Separately, CR-5’s busy bit on presence needs reconciliation with CR-11 and §15.5.6/PR-10, which say peers see **only** reachability and version class: specify the busy bit’s audience or amend that restriction.

**2e. The evidence-rule dependency points to the wrong CRs.** Section 18.2/SI-17 and §19.2/PR-18 list CR-5 and CR-6, although CR-17 expressly introduces the harness-written evidence rule. Section 21.2.4 identifies CR-17 correctly. This can mislead the security-floor dependency mapping.

## 3. Final-record status remains contradictory

**3a. Accepted decisions still appear unmade.** Section 19.1 says no numbers come from rulings, contrary to §22.18/OQ-3 and OQ-9’s adopted starting values and first-build measurement obligations. Section 7.3.a says RR-2 has not been accepted; §22.3 leaves separate accounts merely optional, despite §22.18/OQ-2’s committed target. Section 21.2.2 still awaits OQ-16, and RR-18/BP-19 still call the adopted interim re-pin audit merely proposed (§22.18/OQ-8). These need status corrections; they do not require new decisions.

## 4. Verdict and fixes, smallest first

**Verdict: consistent after listed fixes.** The register supplies the decisions, but the body and acceptance tables do not yet reliably express them.

**4a.** Correct dependency links for SI-17/PR-18 and add R-34.o and R-67.e to CR-20’s affected requirements (§2a, §2e above).

**4b.** Mark PR-34/PN-16 and their operative references dropped; correct the counter summaries (§1c, §1e).

**4c.** Update accepted-status language, adopted numbers, account-separation target and interim re-pin audit (§3a).

**4d.** Replace stale duplicate-main and vendor-channel target descriptions (§1d, §1f).

**4e.** Complete CR-15/CR-19’s history-sharing amendments and align SI-8 with approved routes (§2b–2c).

**4f.** Update SI-31’s confirmation probe and SI-2/PR-32’s activity-veto checks; resolve the busy-bit visibility conflict (§1b, §2d).

**4g.** Propagate B′ into CR-9, confidentiality mitigations, residual risks and bypass targets (§1a).
