# T-3344 step-1 review round 2 — Codex

## 1. Round-1 completion findings

**1a. Partly fixed.** v0.2 is substantially better as interview material, but remaining authority and guarantee inconsistencies prevent treating it as authoritative (§§5, 8; R-19.e).

**1b. Partly fixed.** R-3.e, R-14.e, R-17.e and R-37.e now explicitly acknowledge undecided bounds; they still lack determinate timing/threshold tests. R-1.e still leaves the operator leg uncovered. Recording gaps is progress, not verification completion.

**1c. Partly fixed.** R-16.f, R-20.f and R-23.f supply adversary scope. R-34.e adds hostile impersonation, but does not establish how routing knows Q’s true project despite its assertion of P. R-34.f’s qualification does not resolve that dependency.

**1d. Partly fixed.** §§5, 7 and 10 remain structurally adequate; authority inconsistencies remain below.

**1e. Partly fixed.** §11.6 supplies concrete reported rendering results, though no artifacts independently inspected here. D-3 marks provisional semantics more clearly; drawing inconsistencies remain.

**1f. Fixed.** Header and §11.1 correctly separate recorded open gaps from interview completion and sign-off.

## 2. Round-1 fidelity findings

**2a. Fixed.** R-7.a restores unqualified independence; R-29.a restores “halt.” R-7.e marks restricted testing provisional; R-29.g records scope uncertainty.

**2b. Fixed.** FIFO, fail-closed readiness and doorbell restrictions are explicitly proposals in R-18, R-21 and R-23, including their criteria.

**2c. Partly fixed.** R-9.e, R-11.e and R-12.e tag the added obligations. However, R-12.d still requires an unchanged hub counter without that qualification.

**2d. Fixed.** R-34.a restores mandatory `to_circuit`; R-38.a restores the database and learning intent.

**2e. Fixed.** §6.1.c labels priorities proposals, R-9 becomes P2, and §10 abandons blanket “none changed.”

## 3. Round-1 neutrality findings

**3a. Fixed.** OD-17.d requires explicit accept/reject/defer dispositions.

**3b. Fixed.** OD-6.d acknowledges honest pull-only alternatives; OD-9.d acknowledges alternative packaging.

**3c. Fixed.** OD-10.d separates project and session isolation; OD-14.d leaves canary compatibility to the operator; OD-15.d preserves R-9’s scope.

**3d. Partly fixed.** OD-3.d explicitly asks both rung corrections and urgent compression; OD-14.d explicitly requests the escalation destination. OD-7 now includes ownership, but adds unsupported allocations without clearly identifying collector synthesis.

**3e. Not fixed—evidence limitation persists.** §1 lists comparison files, but their contents are not supplied. Quotations and completeness remain independently unverifiable.

## 4. Round-1 acceptance findings

**4a. Partly fixed.** R-1.e exercises conversation; R-4.e exercises injection and read-back. R-36.e adds a reflection record but leaves meaningful reflection undefined.

**4b. Partly fixed.** R-10.e requires successful re-resolution. R-28.e removes assumed terminal naming, but “when the agent replies” tests forwarding rather than requiring the woken agent to produce either permitted outcome.

**4c. Partly fixed.** R-30.e clarifies successive intervals; R-38.e defines a test consumer/contract. R-19.e still permits escalation instead of demonstrated delivery; “only (3) counts as delivery” incorrectly includes its escalation branch.

## 5. Round-1 threat-handoff findings

**5a. Partly fixed.** §6.1.h and glossary tags help, but proposals reappear unqualified in R-14.e and R-38.e. §10 does not enumerate all assumptions, including R-30.e.

**5b. Partly fixed.** D-3 labels provisional lifecycle choices; D-2 distinguishes rejection. However, D-2 continues into RECEIVED/STORED after rejection.

**5c. Not fixed.** GP-5/CAND-2 still equate deduplication with “exactly-once.” Durability failure scope, trusted evidence producers and precise adversary privileges remain insufficiently explicit. §§2.3/11.4 correctly retain proposed threat-list status.

## 6. New problems introduced or exposed

**6a. Authority drift.** §3.3.1.g calls SENT/REPLIED confirmed; §5 calls them proposals. R-36.e newly mandates a missing-digest escalation destination without marking it proposed.

**6b. Interview bias.** OD-7.B/C adds ownership commitments unsupported by the supplied quotations. OD-2.d now conflicts with revised R-23 by treating proposed content exclusion as already established.

**6c. Undecided policy.** D-2’s “no fallback” after rejection needs explicit proposal status, independently of the proposed state name.

## 7. Verdict

**7a. Yes with named small fixes:** reconcile authority labels, correct rejection flow, qualify ownership/no-fallback proposals, repair R-19/R-28 criteria, and explicitly record guarantee/trust boundaries. Then conduct the interview and pass its dispositions plus unresolved gaps to the threat modeler. No implementation decisions need inventing to make these corrections.
