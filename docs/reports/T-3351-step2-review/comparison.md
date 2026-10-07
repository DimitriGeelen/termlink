# T-3351 step 2 review, round 1: comparison

Reviewed: `docs/design/interactive-agent-communication-02-threat-model.md` (worker commit b4beb426b), brief
`brief.md` (sha256 d139b4013ce15b6b…), 2026-10-07. Answers stored unedited: `codex.md` (Codex, `codex exec -s
read-only`, rc 0), `glm.md` (GLM-5.3 via opencode, rc 0; raw output in `*.raw`). This is evidence for the operator,
not a verdict (roles README 3.5). The first attempt failed with rc 126 (brief too long for one argument); the
second passed the brief on stdin (Codex) and as an attached file (GLM).

## 1 Where they disagree

| Question | Codex | GLM |
|---|---|---|
| Card conditions 6.1-6.5 | 6.1-6.4 not met in substance, 6.5 partly | All met; cosmetic defects only |
| Fidelity to ruled requirements | Four silent changes: interrupt allow-list used as a delivery gate (R-52.a, R-63.a: downgrade, never drop); revocation marks mail DEAD (R-44, R-60 reserve DEAD for an ended instance); takeover narrowed (R-67, R-68); RR-10 misstates OD-16 retention (R-35.o) | Overall faithful; two minor: "allowed peer" tier (PR-9) not ruled, 15.2.15 states an undecided rule as settled |
| Verdict | Return for revision | Fit; fix three presentational items and add three substantive ones first |

## 2 Where they agree or add to each other

1. Operator approval device / operator key compromise is not modelled (Codex 2e, 4f; GLM 2d).
2. Missing residual risks: Codex, log truncation or forking without independent checkpoints, compromised approval
   devices, ambiguous delivery recovery; GLM, the interim openness of TOFU repair paths (BP-19).
3. Circuit credentials: Codex, lifetime not bounded from issuance (delayed first use gets a fresh lifetime);
   GLM, asymmetric case when the minting (receiver's) hub is unreachable but the sender's is not.
4. J4 / TB-6: GLM, the TB-6 matrix is conditional on OQ-4 A and should say so.

## 3 Codex-only technical findings

1. Over-claimed invariants: SI-17 (nonce/record type cannot reject a well-formed forged harness record), SI-22
   (hash chain alone cannot detect every deletion), SI-21 (unwritable policy storage unsupported with a shared OS user).
2. Revocation scopes conflated (role, identity, conversation authority); partitioned versus reachable bounds.
3. Fleet admission: project-id squatting and competing claims, hub-key rotation and re-home.
4. PR-4 must acknowledge the highest CONTIGUOUS durably stored number; PR-2/PR-3 continuity across key rotation.
5. Many-to-many conversations, R-62 hand-over, approval replay within validity, crash between hand-over and record.
6. Decision framing: OQ-1 "network attacker covered by TLS" (TLS does not stop delay or denial); OQ-6 framing;
   OQ-3/PN-5/PN-7 numbers unsupported by measurements; OQ-5 and OQ-7 each mix two choices; "RR-2 already accepts"
   (7.3.a) presumes an operator acceptance.
7. Drawings: D-1 omits the store-to-hook flow and mislabels TB-6; D-2 routes the reply past the receiver's sidecar.

## 4 GLM-only presentational findings

The PR index numbered "19.2" sits between sections 21 and 22; PR-7, PR-12, PR-19 are unused holes; TH-56/TH-57 out
of order; the sender-side symmetry of TB-1 is assumed, not stated.

## 5 Disposition

One revision round (round 2 of at most 3, lesson 5.16): the threat-modeler worker checks every finding against
the text and fixes it or rebuts it with a citation; the four fidelity findings first. Then the operator decides.
