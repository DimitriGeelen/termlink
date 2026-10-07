## 1. Disposition audit

**1a. Substantial corrections are present.** The interrupt-admission restriction is withdrawn; revocation no longer means DEAD; resume is separated from role succession; contiguous receipts, approval-device threats, asymmetric outages, conditional TB-6 analysis and the invariant dependency table are added. Findings not qualified below are adequately reflected in the supplied text.

**1b. Codex 4d: claimed correction absent.** TH-22, §15.5.3 still says “RR-6 (accepted by default today)”. It also lacks the claimed explicit split into exposures (a) and (b), although RR-6 supplies that split. This directly contradicts §20.1 and the disposition.

**1c. Codex 1b: incomplete.** TH-25 gives no impact reason; TH-40, TH-41 and TH-42 still give likelihood without reasons (§§15.5.11, 15.7.8, 15.7.11, 15.7.14). “All now carry” reasons is false.

**1d. Codex 1d, 2b, 2c and 6a: partial.** SI-11 and PR-26 still overclaim rollback protection; SI-22 overclaims checkpoint coverage; SI-21 and its dependency table understate continuing same-user/root exposure. See §3 below. Traceability is improved, but these mechanisms are not yet adequate.

**1e. Codex 2e and 3e: partial.** Membership and hand-over analysis exists, but removal, membership epochs and transfer fencing remain unspecified. TH-9 (§15.2.27) still says participants are “fixed at creation”. Crash ambiguity is acknowledged, but its requirement change is not explicitly requested.

**1f. Codex 3d: partial and substantively mistaken.** The fourteen-day correction is present. However, §8.2 and RR-10 treat the one-year retention of daily telemetry **digests** as retention of per-message cryptographic content digests. R-35.o and R-36 do not establish that guarantee.

**1g. Codex 2a and 4b: central repairs present, contradictory remnants remain.** TH-37 (§15.6.30) still counts lifetime from “first acceptance”. TH-18 (§15.3.27) still promises storage the agent cannot write, including “root-owned” storage on this root-sharing deployment. AC-2 (§16.2.2) still describes detection as stopped by controls and leaves only prevention residual, despite RR-7/RR-15.

**1h. Codex 5b and GLM 2a: overstated disposition.** OQ-12 still combines admission and hand-over authority. Fleet bootstrap has been extended, but first-claim entitlement remains unproven (§10.5.b); calling the earlier analysis adequate obscures that remaining exposure.

## 2. Fidelity to approved requirements

**2a. Blocking: R-51 and R-46.e are weakened without an explicit CR.** Sections 13.4.b, SI-30 and RR-17 replace one hand-over with possible repeated delivery. CR-1 appends write-ahead intent but neither names R-51 nor says its guarantee changes; R-46.e also promises no duplicate delivery across fallback. Explicitly request the exception and its scope. Residual acceptance alone does not amend a ruling.

**2b. Readiness safety is weakened.** SI-2 checks before typing; RR-19 accepts a subsequent race. This does not establish R-19.r/R-23.o’s “nothing ever typed into a busy prompt”. A harness exit between check and injection can also put the line into a shell, beyond RR-19’s nuisance description. Require coordinated injection or explicitly request changes to R-19, R-20 and R-23. Screen-derived gating under PR-32 additionally requires reconciliation with R-22/R-47.

**2c. Retention needs its own explicit proposal.** One-year per-message digest retention must be proposed with storage, identity mapping and ceiling semantics; it cannot be inherited from the daily-digest ruling.

**2d. CR coverage remains incomplete.** CR-13 changes allow-list and roster authority but names only R-67/R-64, omitting R-63. CR-1 should map its outage exception to R-2, R-14.o, R-15.o and R-58 as applicable. CR-5/CR-6 are listed as dependencies for SI-17 without actually requesting its evidence rule.

## 3. Remaining and newly exposed technical defects

**3a. Rollback is not defeated by restorable counters.** Restoring a sidecar or hub can restore both the signed list and its highest-seen sequence (§§11.2.b, 10.5.d). Reconciliation needs an independent current authority, with refusal while unavailable. Test restoration of *both* values. Similarly, signed grants and allow-lists need rollback/revocation protection; an old signed policy can widen current permissions. RK-6’s stale rule does not stop starts under stale grants.

**3b. Checkpoints protect only the anchored prefix.** A receipt predating a later log suffix cannot detect deletion of that suffix. RR-15 incorrectly makes control of every head the necessary condition for undetected truncation. State checkpoint coverage, publication timing and comparison requirements.

**3c. An operator key does not contain host root.** Root can replace verification code or the trusted operator public key. SI-20/SI-21’s limits cannot disappear “once an operator key exists”. Extend RR-2 to their entire lifetime under this deployment.

**3d. Approval replay needs execution semantics.** SI-13 specifies sequential reuse rejection, but not atomic consumption across concurrent requests, executors, crashes or restored backups. Bind approvals to their executor/target and require durable atomic consumption or explicitly bounded recovery.

**3e. Membership rules conflict.** RK-3 permits participant removal, while SI-29 requires every membership change to be countersigned by “the new one”. Define removal authority, effective sequence/epoch and fencing of the old instance after hand-over.

## 4. Operator decisions and completion

**4a. Not yet knowingly decidable:** RR-10, RR-15, RR-17 and RR-19 require the corrections above. RR-14 must distinguish mistaken approval from first-claim squatting without an operator mistake. RR-12’s “essential flows alive” needs bounded receipt traffic and actual reserved capacity.

**4b. OQ-10 is misleading:** a hub content copy cannot protect turns acknowledged STORED during a hub outage until replication occurs. State that exposure. OQ-11 must explain that hub-supplied substituted recipient keys defeat encryption on first contact. OQ-12 needs separate choices.

**4c. Verdict: not fit yet.** Conditions 6.1/6.2 are structurally met; 6.4 remains substantively deficient. The drawings and textual equivalents exist; rendering is reported, not independently verified here. Profile completeness cannot be independently established from the supplied material.

**4d. Minimum repair:** correct the false dispositions and contradictory remnants, explicitly request the delivery/readiness/retention changes, repair rollback/checkpoint/approval/membership guarantees, and update the affected residuals and dependency table. No new implementation is needed for this handoff.
