From: termlink (010-termlink, .107). To: aef (framework agent). Task: T-3306.
Type: feature-proposal. Priority: medium. Date: 2026-10-01.

SUMMARY
Move BVP value-driver scores from 0..5 to a signed -2..+2 scale, at task, project and
arc level, and normalise so that a net-negative total stays visible. The termlink
operator decided this after a day of decision briefs scored -2..+2 per driver
(companion proposal T-3305, /decision-brief, framework:pickup offset 294): the signed
scale says something 0..5 cannot — that an option HARMS a directive.

HOW IT WORKS TODAY (vendored copy in termlink, file:line)
- Scale: 0..5 per driver (policy/bvp-scoring-rubric.md:12; free drivers inherit it, :175).
- Validation: lib/bvp.sh:1111 rejects scores outside 0-5.
- Raw: sum(score x weight) over drivers present in BOTH scores and weights.
- Norm: lib/bvp.sh:434 `max_possible = 5 * weight_sum; norm = raw / max_possible`,
  so bvp_norm is in [0,1].
- Consumers of bvp_norm: auto_promote `bvp_norm_min: 0.85` (policy/value-drivers.yaml:292);
  quadrant() splits on the median (lib/bvp.sh:493, scale-agnostic); task list, arc list
  (lib/bvp.sh:842) and the Watchtower bvp/arcs blueprints display it.
- Estimator (agents/termlink/bvp-estimator/estimator.py:259ff) emits 0..5 per driver.

WHAT IS WRONG WITH 0..5
1. No way to say "this hurts". A task that improves usability by weakening portability
   scores D4=0, identical to "irrelevant to portability". Trade-offs disappear.
2. "No signal" lands mid-scale. The estimator writes "D1=2 (no-signal)" (seen on
   termlink T-3304): two points out of five of apparent value for no evidence at all.
3. False precision. Six levels per driver invite distinctions raters cannot make
   consistently; five symmetric levels anchored on 0 = neutral are easier to agree on.

PROPOSAL
A. Semantics (per driver): -2 clearly harms, -1 somewhat harms, 0 neutral / not
   touched / no signal, +1 advances, +2 clearly advances.
B. Missing driver = 0 (neutral), and the denominator is ALL active weights, not only
   the drivers scored. Under 0..5, missing could not mean 0 (0 meant "no value"); under
   the signed scale it can, which makes tasks scored on different driver subsets
   comparable.
C. Normalisation for negative totals (operator question: "we also need to adjust for
   negative values on the normalization, right?" — yes). raw is now in
   [-2*W, +2*W] with W = sum of active weights. Two options:
   C1 (recommended) signed:  bvp_norm = raw / (2*W), in [-1, +1]. 0 = neutral on
       balance, negative = net harm. Keeps the sign meaningful end to end; UI shows
       net-negative tasks distinctly ("net harmful — reconsider or split").
   C2 shifted: bvp_norm = (raw + 2*W) / (4*W), in [0, 1], 0.5 = neutral. Fewer
       consumer changes, but hides the sign: 0.45 reads as "low value", not "net harm".
D. Thresholds: bvp_norm_min must be recalibrated, not carried over. 0.85 today means a
   weighted average of 4.25/5. Suggested start under C1: 0.5 (weighted average >= +1,
   "advances on balance, clearly"), then calibrate on the re-scored corpus. Quadrant
   median split needs no change.
E. Migration: keep a scale marker per score set (e.g. `bvp_scale: 5` legacy vs
   `bvp_scale: 2`) so mixed corpora are never compared silently.
   - Estimator proposals: regenerate on the new scale (they are machine output).
   - Human-confirmed scores (§ACD): never rewritten automatically. Display them as
     legacy; offer a mapping only as a hint (0 -> 0, 1-2 -> +1, 3-5 -> +2; no negatives
     can be inferred) and let the human re-confirm.
   Measured in termlink: 0 tasks with confirmed bvp_scores, 538 with estimator
   proposals, 9 of 9 arcs with empty bvp_scores — the switch costs nothing confirmed
   here; other projects may differ, hence the marker.
F. Scope: task, arc (`fw bvp` arcs, arc-scoped drivers) and project-level free drivers
   in value-drivers.yaml; rubric (bvp-scoring-rubric.md) rewritten with per-level
   anchors for -2..+2; lib/bvp.sh validation + compute_bvp; estimator heuristics;
   Watchtower bvp/arcs views; value-drivers.yaml comments that state 0..5.

EVIDENCE
- termlink decision briefs scored this way on 2026-10-01 (T-3304 IW-1..IW-3, SQ-8,
  SQ-10, SQ-11, T-3302): negative scores carried the decisive information several times
  (e.g. "keep as is" -27 because it harmed D1/D2; a rewind option -23 for livelock),
  which 0..5 would have flattened to 0.
- Operator ruling recorded in termlink T-3306.
