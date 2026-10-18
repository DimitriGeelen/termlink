# T-3440 5c — draft scoring specs for the six arc-scoped BVP drivers

Operator ruling 5c = A′ (2026-10-18, T-3440 Decisions): fix the persistence bug first
(T-3472, done), then draft specs, keep only the ones that discriminate, and the
operator attaches them. These files are drafts; nothing here is attached to policy.

## arc-substrate-fitness (human-approved drivers, 2026-06-22)

Scored with the estimator's own `score_declarative` (read-only) over the arc's 14
member tasks and a control set of 15 completed tasks from outside the arc (every
25th of T-29xx..T-34xx). Score 0-5; 0 is a measured no-signal.

| Driver | arc members (14) | non-zero | controls (15) | non-zero |
|---|---|---|---|---|
| AS_COORDINATION_TRUTH | 0 5 5 3 3 3 3 3 0 3 0 1 0 0 | 9/14 | 0 1 0 0 0 0 0 0 1 0 0 1 0 0 0 | 3/15 |
| AS_FAILURE_OBSERVABILITY | 5 1 1 1 0 1 0 5 0 3 3 3 5 5 | 11/14 | 1 0 3 0 0 0 1 0 0 0 0 3 0 0 0 | 4/15 |
| AS_RESOURCE_FOOTPRINT | 0 3 5 3 3 1 1 3 5 3 0 0 3 5 | 11/14 | 3 1 0 0 0 1 0 0 0 0 0 0 0 0 0 | 3/15 |

Arc member order: T-2243 T-2244 T-2245 T-2246 T-2247 T-2248 T-2249 T-2250 T-2251
T-2252 T-2253 T-2254 T-3465 T-3466. Controls: T-2964 T-3004 T-3041 T-3070 T-3098
T-3144 T-3174 T-3210 T-3239 T-3273 T-3298 T-3334 T-3394 T-3420 T-3450.

Reading: all three separate arc work from unrelated work, and they separate from each
other (T-2243 dead-letter: observability 5, footprint 0; T-2251 audit-log bound:
footprint 5, coordination 0). The new R5 tasks T-3465/T-3466 score 5 on
observability, which is what they are for. Keyword scoring is coarse (one matching
keyword can award a level; reviewers' caveat), so these rank by topic, not by proven
effect.

Caveat on the tool: `fw bvp driver --explain` only resolves a driver for tasks whose
arc carries it, so it cannot score a control set; the table uses `score_declarative`
directly.

## arc-011 (reviewer-approved 2026-10-06; specs lost by the T-3472 bug)

Redrafted from the original rationales (proposal commit f16232341). Following the
claude -p review, AS_DELIVERY_TRUTH and AS_OPERATOR_SIMPLICITY favour proof surfaces
(receipt stages, the e2e prover, the sidecar store/pull path, provisioning and the
kill-switch drill) over topic words.

Round 1 was too noisy: keyword matching is case-insensitive, so `STORED` matched every
"stored", and generic words (operator, install, stage, deadline, silent) fired across
the controls (non-zero: arc 33/33/44 of 56 vs controls 6/8/8 of 17). Round 2 narrowed
them. Members: all 56 tasks tagged arc-011. Controls: 17 completed tasks outside
arc-011 and arc-substrate-fitness (every 20th of T-29xx..T-34xx).

| Driver | arc-011 (56): L1-2 / L3-4 / L5 | non-zero | controls (17): L1-2 / L3-4 / L5 | non-zero |
|---|---|---|---|---|
| AS_DELIVERY_TRUTH | 1 / 16 / 2 | 19/56 (34%) | 1 / 1 / 1 | 3/17 (18%) |
| AS_FAILURE_SURFACED | 0 / 4 / 8 | 12/56 (21%) | 1 / 0 / 1 | 2/17 (12%) |
| AS_OPERATOR_SIMPLICITY | 0 / 7 / 7 | 14/56 (25%) | 0 / 0 / 2 | 2/17 (12%) |

Reading: all three separate arc work from unrelated work by roughly 2x, weaker than the
arc-substrate-fitness specs, and they cover fewer members (many arc-011 tasks are
design and planning tasks the signals do not touch; those score a measured 0). The
same caveat applies: topic signals, not proven effect.
