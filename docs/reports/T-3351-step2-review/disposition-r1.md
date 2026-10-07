# T-3351 step 2, review round 1: disposition of every finding

**Task:** T-3351 · **Role:** threat modeler (revision round) · **Date:** 2026-10-07
**Document revised:** `docs/design/interactive-agent-communication-02-threat-model.md`, version 0.1 to 0.2
**Reviews answered:** `codex.md` (Codex, return for revision) and `glm.md` (GLM-5.3, fit with fixes), stored unedited, with `comparison.md`.
**Input unchanged:** `docs/design/interactive-agent-communication-01-requirements.md` v0.4.1 was not edited. Where a mitigation needs a ruled requirement changed, it is a change request (CR) that names the requirement and says what would change (section 21 of the threat model).

## 1 How the findings were handled

1.1 Every finding in `codex.md` and `glm.md` was checked against the threat model text and against the requirements text it cites (R-1, R-26, R-35, R-44, R-45, R-51, R-52, R-56, R-59, R-60, R-62, R-63, R-65, R-67, R-68). The four fidelity findings were done first, then GLM 3b.
1.2 Verdicts: **valid** (fixed), **partly** (the valid part fixed, the rest answered), **not valid** (rebutted with a section).
1.3 Result, counted per finding row in sections 2 and 3: **36 findings: 31 valid, 5 partly, 0 not valid as a whole.** One sub-claim inside Codex 1e is not valid (the render result "cannot be verified" is not a defect of the document). I looked for rebuttals and found Codex right on every substantive point; the 5 "partly" verdicts are cases where the reviewer's conclusion went further than the three or four examples they gave. Section 4 lists the reviewer *assertions* (not requests) that this revision does not uphold, mostly GLM's "met" and "faithful" statements.
1.4 Section numbers below are those of threat model v0.2.

## 2 Codex findings

| Id | Verdict | What changed, or the rebuttal |
|---|---|---|
| Codex 1a (6.1 not met in substance: approval replay by expiry alone, transcript forgery by writable record types, credential replay by unspecified channel binding) | partly | The matrix does answer all ten questions for 13 boundaries (14.2 to 14.14), so "not met" overstates; the three examples are valid. Approval replay: SI-13 now makes approvals single-use with a consumed id, TH-51 and 15.10.6 say expiry is a bound not the control. Transcript forgery: PR-18 and SI-17 now say the record type filters echo, not forgery, with the limit probe and RR-7 (13.2). Channel binding: 7.2.a (3) and (4) and 7.2.c now name the mechanism (possession proof covering a value exported from the connection) and SI-9 probes it |
| Codex 1b (6.2: countermeasures overclaim, RR-7 understates, TH-12, 20, 21 lack L/I reasons) | partly | Reasons were missing on 27 threats (not only the three named); all now carry a reason for likelihood and for impact (section 15, sweep of 15.2 to 15.11). RR-7 rewritten (section 20). The "countermeasures overclaim" part is the SI findings (Codex 1d). The claim that traceability is not adequate is answered by 18.2, which separates existing obligations from proposals |
| Codex 1c (6.3: "monitored" is misleading where the adversary can alter the monitor; completeness unproven) | partly | 17.1.a defines "monitored" as detection of an adversary with only that route's access, and BP-4, BP-5, BP-10 and BP-18 cells are qualified. Completeness: stated as complete for the routes the profile names (P3.2) and for those found in the record, and not provable beyond that. No route was added; the reviewer named none |
| Codex 1d (6.4 not met: SI-17, SI-22, SI-21 cannot deliver what they promise) | valid | SI-17 now states what it does not do and carries a limit probe (expected result: accepted, recorded as RR-7). SI-22 restated: detects mid-chain change; tail truncation and fork need an independent head, which the signed receipts now carry (8.4.c); one attacker holding every copy is RR-15. SI-21 restated as signature-based and made conditional on CR-14, because with one operating-system user (uid 0 here) a file the agent "cannot write" does not exist. 18.2 gives each SI its standing and its limit |
| Codex 1e (6.5: render unverifiable; D-1 omits store-to-hook flow and mislabels TB-6; D-2 reply bypasses the receiver's sidecar) | partly | D-1: the hook-reads-store link added; TB-6 drawn dotted and labelled as same-host only, conditional on OQ-4 (5.1, 5.1.1). D-2: step 12 now goes through the receiver's own sidecar, step 13 is the signed reply back (16.1, 16.1.1). Not valid as a defect: "the render result cannot be verified by a text-only review" describes the reviewer's limit; section 24 records the method, and all four drawings were re-rendered with `mmdc` after the changes and D-1 and D-2 were read as images (24.2) |
| Codex 2a (circuit lifetime not securely bounded: delayed first use gets a fresh lifetime) | valid | 7.2.a (3) and (4): establishment nonce issued by the receiver's own sidecar and a set-up window (PN-12). 7.2.d rewritten into four parts (from establishment, renewal is a new establishment, reconnect keeps the expiry, restart expires all). SI-9 probes delayed presentation, renewal, reconnect, restart. TH-50 and RR-5 updated |
| Codex 2b (revocation scopes conflated; stale allow-list vs "one tick"; rollback; failed retrieval) | valid | Section 11 rewritten: seven scopes RK-1 to RK-7, each with a reachable and a partitioned bound; 11.2.b rollback protection by rising signed sequence number; 11.2.c a failed pull is not an empty list; 11.2.d the stale rule; 11.2.e testable bounds. SI-11 and SI-23 restated. New TH-58 |
| Codex 2c (fleet admission: project-id entitlement, competing claims, stale roster, key rotation, re-home) | valid | 10.5 added (PR-26): project root key, first-seen binding, no-guess at first sight, claims are not a denial lever, signed rising roster, old-key-signed rotation, signed "moved to". New TH-59 and TH-60; CR-8 and OQ-8 extended |
| Codex 2d (PR-4 must be contiguous; PR-2 and PR-3 continuity across key rotation) | valid | 8.4.a: `up_to` is the highest contiguous durably stored number, with selective acknowledgement never counted. 8.2 and 8.3: the namespace is keyed by sender identity, not key fingerprint, and survives rotation. SI-4, SI-5, SI-6, CR-3, CR-4 updated |
| Codex 2e (many-to-many, R-62 hand-over, readiness races, approval replay, compromised approval device, crash between hand-over and record) | valid | 8.6 (many-to-many, membership, hand-over, per-recipient receipts, confidentiality inside the conversation; PR-30, SI-29, TH-61, CR-15, OQ-12, OQ-13). 9.2.d readiness race (PR-32, TH-62, RR-19). Approval replay: SI-13 single-use. Approval device: ADV-14, PR-27, TH-64, SI-31, RR-16, CR-14, OQ-14. Crash window: 13.4 (PR-31, SI-30, TH-63, RR-17) |
| Codex 3a (interrupt allow-list used as a delivery gate; R-52.a, R-63.a say downgrade, never drop) | valid | **Fidelity, fixed by withdrawal, no CR needed.** 7.2.a (2): the hub does not look at the allow-list, urgency is decided per message at the receiver (SI-18). 7.6.a, SI-10 (with a negative control that a non-listed sender gets a circuit and its urgent turn is delivered as normal), TH-46, 14.13 and CR-7 corrected. Also: the hub cannot see a turn's priority on a circuit, so the gate could not have worked |
| Codex 3b (revocation marks mail DEAD; R-44.a and R-60.a reserve DEAD for the home hub's finding that the instance ended) | valid | **Fidelity, fixed without a CR.** 11.1 (a) revocation never makes mail DEAD; RK-1: mail to a revoked agent shows WAITING FOR RECIPIENT (R-44.a "not able to receive") with a reason and a needs-attention entry; DEAD only if the home hub then finds the instance ended (11.3). If the operator wants revoke-then-DEAD, that is a CR; none is raised |
| Codex 3c (takeover narrowed by PR-15; R-67, R-68 require takeover from a stuck holder with a live sidecar) | valid | **Fidelity, fixed by withdrawal.** PR-15 and CR-12 now apply to **resume** only (R-65); 15.11.6 states that R-67 and R-68 are unchanged and why a "no process holds it" check would forbid R-68 test (2). TH-54 scenario no longer says a false DEAD re-resolves a role. The withdrawal is named in CR-12 |
| Codex 3d (RR-10 misstates OD-16 and R-35.o retention) | valid | **Fidelity.** RR-10 restated: events kept to 14 days after a final state, never cut while open (ceiling exceptions last, loudly), digests one year. 8.2 last row and TH-19 corrected; a conflicting duplicate is still caught for a year |
| Codex 3e (other changes: participants fixed at creation vs R-62; PR-5 "both disbelieved" vs R-60; CR-1 must cover R-26 and R-44) | valid | 8.3 last row and 8.6 (membership and hand-over). 10.3.c and SI-12 restated to R-60.a (2e) as ruled; the "both disbelieved" rule is withdrawn (and it handed an enrolled hub a denial lever, TH-59). CR-1 extended to R-26.o (replies) and R-44.a (UNKNOWN versus "unreconciled" state) |
| Codex 4a (RR-1, RR-3: enforced boundaries vs instructions; caps do not bound one message) | valid | RR-1 and RR-3 rewritten (section 20) |
| Codex 4b (RR-2, RR-7: root changes monitors and evidence; forged reply satisfies deadline; separate users do not contain root) | valid | RR-2 and RR-7 rewritten; 13.2 REPLIED row, 13.3.a to e; 6.4; OQ-2 states B closes only the user-level part |
| Codex 4c (RR-4, RR-5, RR-14: home hub may substitute the verification key; "cannot forge an agent's signature" understates) | valid | PR-1a: each conversation pins the other's key and it changes only by an old-key-signed replacement; RR-4 restated (a substituted key for a new conversation is effective impersonation). RR-5 and RR-14 keep their form and now name the dependencies (7.2.d, 7.3.c, TH-59) |
| Codex 4d (RR-6 / TH-22 "accepted by default" unauthorized; separate hub operator from token holders) | valid | TH-22 split into exposures (a) and (b); RR-6 split; "accepted by default" removed; OQ-7 and OQ-11 separate |
| Codex 4e (RR-8 groups induced lapse with duplicate authority; RR-9 trusted display; RR-10; RR-11 host vs hub process; RR-12 aggregate fairness; RR-13 authorization weakness) | valid | RR-8 covers the second copy only; an induced lapse is an automatic takeover as ruled (TH-33 (b)). RR-9 adds the trusted-display condition. RR-10 as 3d. RR-11 and 7.6.c: a host being up does not mean the hub process is up (G-070; hypothesis H-2). RR-12: aggregate ceiling and reserved shares (PR-28, SI-19). RR-13 and 17.4: PR-29 tightens authorization without removing the verbs (R-4 unchanged) |
| Codex 4f (missing residuals: truncation or fork, compromised approval device, ambiguous recovery) | valid | RR-15, RR-16, RR-17 added; with GLM 4b, RR-18; and RR-19 for the doorbell race |
| Codex 5a (OQ-1 "covered by TLS" misleading; OQ-6 reduces interim authentication to two options) | valid | OQ-1 option B restated (TLS gives confidentiality and integrity, not delivery or timeliness) and ADV-13 row corrected. OQ-6 gets option C: assess the channels the system already authenticates. The recommendation stays A for the floor, with the reason |
| Codex 5b (unmeasured numbers; OQ-5 and OQ-7 mix two choices) | valid | Section 19: every number marked a hypothesis with how it would be measured (H-1 to H-9); 7.3.a labels H-1. OQ-5 split (OQ-5 and OQ-10); OQ-7 split (OQ-7 and OQ-11); OQ-12, OQ-13, OQ-14 are one choice each |
| Codex 5c ("RR-2 already accepts"; proposed vs existing invariants; "firm invariant" language) | valid | 7.3.a corrected (the operator has not accepted RR-2); 9.2.a says firm only if CR-5 is accepted; 18.1 and 18.2 distinguish existing obligations from proposals |
| Codex 6a (handoff: correct impossible guarantees, hidden changes; provide invariant-to-CR table with adversary limits and achievable probes) | valid | 18.2 added: for each SI, standing, dependency (CR, OQ), limit (RR) and whether the probe can run |

## 3 GLM findings that asked for a change

| Id | Verdict | What changed, or the rebuttal |
|---|---|---|
| GLM 1f (i) PR index numbered "19.2" sits between sections 21 and 22 | valid | Index moved to 19.2 inside section 19 |
| GLM 1f (ii) TH-56 and TH-57 out of numeric order | partly | Ids are cited by later steps and were not renumbered (15.1.a says so). A numeric-order index of all 65 threats with section and category was added (15.1.a), which gives the order. Not renumbered on purpose |
| GLM 1f (iii) PR-7, PR-12, PR-19 are holes | valid | 19.2 states that they are unused, that nothing refers to them (checked by search), and that a gap is not a missing proposal. Not renumbered, for the same reason as above |
| GLM 2b TB-6 matrix is conditional on OQ-4 A | valid | 14.7 says so; 14.7.1 lists the rows to redo under B or C; OQ-4 points to it |
| GLM 2c Asymmetric case: minting hub unreachable, sender's hub reachable | valid | 7.3.c added |
| GLM 2d (i) Operator approval device or key not modelled | valid | As Codex 2e: ADV-14, PR-27, TH-64, SI-31, RR-16, CR-14, OQ-14 |
| GLM 2d (ii) Sender-side symmetry of TB-1 assumed, not stated | valid | 5.3 added; D-2 step 13 |
| GLM 2d (iii) Wholesale chain rewrite deserves explicit mention at 13.3 | valid | 13.3.d, TH-65, RR-15 |
| GLM 3b (i) PR-9 "allowed peer" tier not ruled | valid | **Fidelity.** PR-9 and 9.4: two classes only (peer, operator-class); the allow-list is a fact line, not a class; CR-6 says so |
| GLM 3b (ii) 15.2.15 states an undecided rule as settled | valid | Reworded as a proposal that depends on OQ-6 A and CR-10 (15.2.15) |
| GLM 4b BP-19 interim openness has no residual | valid | RR-18 added; the BP-19 cell and 17.2 say it is open now; an interim record of re-pins is proposed, not claimed |

GLM 6b restates three of the above (PR index, PR holes, symmetry) plus BP-19, the operator device and TB-6; covered by the rows above.

## 4 Reviewer assertions that this revision does not uphold, or only partly

| Reviewer, id | Assertion | Why not upheld |
|---|---|---|
| GLM 1a | 6.1 met | The matrix is complete in form, but three answers relied on controls that do not deliver the protection (Codex 1a). Upheld for structure, not for those three answers; now corrected |
| GLM 1c | 6.3 met | Upheld for the routes named; "monitored" needed a definition (17.1.a) |
| GLM 1d | 6.4 met | Not upheld for SI-17, SI-21 and SI-22 (Codex 1d is right; each restated). 1.3.e says the text bears Codex out |
| GLM 3a | Overall faithful | Not upheld: four requirement changes were made silently (Codex 3a to 3d), plus the PR-5 rule (Codex 3e); all withdrawn or made explicit |
| GLM 4a | Each residual honest and correctly scoped | Not upheld for RR-2, RR-4, RR-6, RR-7, RR-8, RR-10, RR-11, RR-12 (Codex 4a to 4e); rewritten |
| GLM 5a | Options fairly presented | Not upheld for OQ-1 B, OQ-5, OQ-6, OQ-7 (Codex 5a, 5b); reframed or split |
| GLM 5b | Numbers properly tagged | Partly: tagged [P] but with no evidence status; now labelled hypotheses (19.1) |
| GLM 6a | Fit overall | Not upheld before this revision; the next role would have inherited impossible guarantees (Codex 6a). 18.2 is the repair |

Reviewer statements that were upheld unchanged: GLM 2a (fleet admission adequately analysed, now extended by 10.5), GLM 2c on the circuit lifetime analysis, GLM 4c (conditionals handled), GLM 5c (nothing operator-only was decided, apart from the wording fixed in 15.2.15).

## 5 What this revision does not claim

5.1 No probe was run: no component exists yet. 18.2 says which probes can run once it does.
5.2 No number was measured (19.1).
5.3 Nothing is accepted or decided. The new open questions are OQ-10 to OQ-14; the residual risks to accept now number 19; the change requests number 15.
5.4 The drawings were re-rendered with `mmdc` (24.2), not by an independent reviewer.
