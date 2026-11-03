# T-3353 AC1: independent review of step 4 (architect) output, ARC4 v0.2

Reviewer: judge-t-3353-r5-claude-code-fb0793567bd1 (rung-5-panel:claude-code). Revision d48fa1537cecb7036b492e851c2f6b2a75a8d1bb.
Verdict: **green**

## Scope
- Card: `docs/design/roles/cards/architect.md` section 6, conditions 6.1 to 6.5 (plus card 3.2 to 3.9, which those conditions rest on).
- Reviewed: `docs/design/interactive-agent-communication-04-architecture.md` (ARC4 v0.2), the sub-document `-04a-traceability.md`, and the hand-back `interactive-agent-communication/role-handback-step-04.json`.
- I treated the round-1 evidence (AC1-judge-t-3353-r1-c975edf4772f.md, amber) as input and re-checked it myself. I did not take it on trust.

## Integrity
- The hand-back validates against `schemas/role-handback-1.schema.json` (python jsonschema).
- The output sha256 `96659c76…08cc` matches ARC4 at this revision. The 04a sha `597988d7…af9a` matches.
- All 7 hand-back input hashes match. All hashes in ARC4 sections 1.1, 1.2, 1.3 and 1.5 match (REQ, TM, FP, RUL, ARCH, ARCH2, GAP, LEASE, RULES aside, COMMON, CARD, ADAPTER, PROFILE, README, CHAIN, SCHEMA, REV1).
- The chain yaml step 4 names `cards/architect.md`, `task: T-3353` and this output.

## 6.1 Every R-n, TH-n and SI-n addressed; no unexplained element: MET
- My own script collected ids from the sources: R-1..R-82 from REQ (82), TH-1..TH-65 from TM (65), BP-1..19 and TB-1..13 from TM, SI and FX from FP. It found every one in ARC4 or 04a.
- 04a tables 2 and 3 use two columns per row. I confirmed that both halves list every R and TH (41+41, 33+32).
- The one id missing is SI-32. It is only a "consider" proposal inside FP CX-1, a change request to step 2, and is not a floor item. ARC4 carries its substance as BP-20 [P] and 12.6.
- 04a table 4 maps K-9a..K-31 to what they serve.

## 6.2 Baseline, transitions and target, with boundaries and instrumentation: MET (the round-1 gap is closed)
- 11.4 says D-1's built-coloured subset is the baseline view. It names the baseline TBs, the built [i] points, and which TBs do not exist yet.
- The 11.5 table gives, for each step (baseline, T1..T6, target), the TB-n introduced (+) or changed (~) and the [i] that shows the step took effect, and adds the rule "done only when every [i] reads as stated".
- I checked the TB numbers against TM section 5 (lines 203-215). They are used consistently: TB-2 doorbell, TB-7 circuit, TB-8 hub-to-hub, TB-10 operator, TB-11 starts and grants, TB-12 lease, TB-13 estate.
- The D-1 relabel is correct: operator zone TB-10, lease TB-12 [i].

## 6.3 Every bypass route removed or a monitored residual: MET
- Section 14 has BP-1..BP-19, all 19 of TM, plus the proposed BP-20 and BP-21. Each row has a design element and an outcome. Every "open" row cites an accepted RR or a later phase.

## 6.4 Every open decision is a DR with options and a recommendation: MET (the round-1 minor is closed)
- DR-1..DR-14 each have context, options, a recommendation and consequences. They now also have Status ("proposed", with the O-n that rules it or "with the approval of this step") and Decision ("none yet") columns.
- 18.1 states the transition rule.
- 18.2 disposes of all 18 FP 14.1.b items: built, decided [P] or open. This includes the BV composition and the claim/transcript-complete signal that round 1 raised.
- O-1..O-7 are in section 21, and they match the hand-back's decisions_needed.

## 6.5 Drawings D-1..D-5 with id, caption, text equivalent, rendering without error: MET
- Captions are at 5.2, 5.4, 7.6, 10.1 and 11.1. Text equivalents are at 5.3, 5.5, 7.7, 10.2 and 11.2.
- D-3 has a primary flow, a refusal flow and a forward flow. D-4 has three state machines.
- I re-rendered all 5 Mermaid blocks myself in a fresh temp dir (mmdc, /usr/bin/chromium --no-sandbox). All 5 exited 0. The SVG sizes were 47343, 26237, 43435, 73445 and 24588 bytes, identical to ARC4 24.3. There were 0 "syntax error" or "parse error" hits in the SVGs or logs.

## Fit for step 5
- Section 15 now has rows for K-12, K-17, K-24 and K-25. Each has a property, a check and what the check does not prove.
- 15.1 says K-1..K-8 and K-18 are built and accepted. K-26..K-31 must add their rows in their sub-designs before build.
- Every PH-2 row in section 6 has a proof and a mutant. The build table B-1..B-24 has dependencies.
- I checked 7.3's "same project's roster" for PH-2. It exists: sidecar rosters are built from the key register (ARCH2 P7-2). So it is not a PH-3 dependency.

## Spot-checks against the code at this revision
- `termlink-hub/src/tls.rs:48` `hub_id_from_fingerprint`: matches.
- `auth.rs`: the `Notify` scope ranks below `Interact`: matches.
- `handler.rs`: the test asserts `[sidecar] 2 peer messages waiting (ids …)`, so the DR-6 alphabet claim is true.
- `crates/termlink-sidecar/src/` has `lease.rs`, `liveness.rs` and `activity.rs`.

## Non-blocking notes (no change needed for green; the architect may fold them into a later revision)
1. D-1 labels the home-hub zone "(TB-5, TB-6)". TM defines TB-6 as sidecar to sidecar on one host *without* the hub. 11.4 says it does not exist on the baseline, and 11.5 T6 says it "stays unused". The label should read "(TB-5)", or TB-12 should go there.
2. 4.4 cites `handler.rs:611`. The function is at line 618 (already noted in round 1).
3. 11.5 T4 files "`sidecar:` alias ended" under TB-12. BP-9 places the alias with broadcast topics, which is closer to TB-1/TB-5.
