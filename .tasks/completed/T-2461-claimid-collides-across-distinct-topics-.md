---
id: T-2461
name: "claim_id collides across distinct topics sharing a 16-char sanitized prefix at the same offset+nanosecond, misreported as ClaimConflict and spuriously denying a free slot — make claim_id collision-proof via a monotonic seq (round-15 F1)"
description: >
  claim_id collides across distinct topics sharing a 16-char sanitized prefix at the same offset+nanosecond, misreported as ClaimConflict and spuriously denying a free slot — make claim_id collision-proof via a monotonic seq (round-15 F1)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-22T19:15:05Z
last_update: 2026-07-22T19:18:41Z
date_finished: 2026-07-22T19:18:41Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
---

# T-2461: claim_id collides across distinct topics sharing a 16-char sanitized prefix at the same offset+nanosecond, misreported as ClaimConflict and spuriously denying a free slot — make claim_id collision-proof via a monotonic seq (round-15 F1)

## Context

Round-15 adversarial claim-CAS hunt (F1). The hunt VERIFIED the claim
compare-and-swap is race-free (atomic `INSERT` guarded by `UNIQUE INDEX
idx_claims_topic_offset_active(topic,offset)` at `meta.rs:809`, all claim ops
serialized under a single-connection `Mutex<Connection>`). But it found one LOW
correctness bug in the id generator: `generate_claim_id` (`meta.rs:750`) composes
`clm-{now_ns}-{topic_tag}-{offset}` where `topic_tag` = the first **16** sanitized
chars of the topic. The `claim_id` is the table PRIMARY KEY. Two DISTINCT topics
sharing their first-16 sanitized chars (e.g. `arc-parallel-substrate-a` and
`...-b` both sanitize to `arc_parallel_sub`), claimed at the SAME offset within the
same nanosecond (serialized calls can read identical `SystemTime` ns on coarse
clocks), produce an IDENTICAL `claim_id` → the `INSERT` fails on the `claim_id` PK,
NOT on the `UNIQUE(topic,offset)` index. The error handler (`meta.rs:393`) maps ANY
`ConstraintViolation` → `ClaimConflict{topic,offset}`, so a legitimately-FREE slot
on a different topic is spuriously denied (an orchestrator sees a phantom conflict
and may back off a slot that is actually free — counter to throughput). The in-code
comment even encodes the buggy reasoning (it assumes the UNIQUE(topic,offset) index
covers the collision, but that index guards the FULL topic while the id uses the
16-char PREFIX). Self-heals on retry (nanos advance), hence LOW. `claim_id` is only
ever passed as an opaque SQL param (never parsed/split — grep-verified), so making
it collision-proof is safe. Fix: append a process-monotonic `AtomicU64` seq so no
two claim_ids in a process lifetime can ever be equal; `now_ns` still disambiguates
across restart.

## Acceptance Criteria

### Agent
- [x] `generate_claim_id` appends a process-global monotonic `AtomicU64` sequence so distinct calls can never produce an equal `claim_id`, even for two distinct topics sharing a 16-char prefix at the same offset+nanosecond; the misleading collision comment is corrected. — `CLAIM_ID_SEQ.fetch_add` + rewritten doc-comment (meta.rs:750-786).
- [x] The id composition is extracted to a pure `compose_claim_id(now_ns, seq, topic, offset)` and unit-tested: with a FIXED `now_ns`, two distinct topics that sanitize to the same 16-char tag at the same offset get DISTINCT claim_ids (the seq disambiguates) — deterministically proving the collision is closed. — `compose_claim_id_distinct_for_shared_prefix_topics_at_same_instant` + 2 siblings (meta.rs claim_id_tests).
- [x] A bus-level test confirms two distinct topics can each claim the SAME offset without a spurious `ClaimConflict` (distinct-(topic,offset) pairs are independent). Full `cargo test -p termlink-bus --lib` stays green. — `claim_distinct_topics_same_offset_both_succeed` (lib.rs); 91 bus tests green (+4).

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

cargo test -p termlink-bus --lib claim_id 2>&1 | tail -5 | grep -qE 'test result: ok'
cargo test -p termlink-bus --lib 2>&1 | tail -5 | grep -qE 'test result: ok'

## RCA

**Symptom:** A `channel.claim` on a legitimately-free `(topic, offset)` is
occasionally rejected with `ClaimConflict` even though no live lease holds that
slot — specifically when another topic sharing the same 16-char sanitized prefix
was claimed at the same offset within the same nanosecond.

**Root cause:** `claim_id` (the claims-table PRIMARY KEY) is derived from a LOSSY
projection of the topic — `topic.chars().take(16)` sanitized (`meta.rs:759-763`).
The uniqueness the PK needs is over the FULL topic, but the id only carries a
16-char prefix, so two distinct full-topics can map to the same `claim_id`. When
that PK collision fires, the constraint-violation handler (`meta.rs:391-402`)
cannot tell it apart from the legitimate `UNIQUE(topic,offset)` violation and
blanket-maps both to `ClaimConflict`.

**Why structurally allowed:** The id generator's own comment asserted the
collision "needs two claims... on the same (topic-prefix, offset), which the
UNIQUE(topic, offset) index already blocks anyway" — a false equivalence between
the 16-char PREFIX (in the id) and the FULL topic (in the index). The reasoning
error was baked into the comment, and no test exercised two distinct topics with a
shared prefix, so the gap was invisible. The blanket ConstraintViolation→
ClaimConflict mapping then hid WHICH constraint fired.

**Prevention:** (1) a process-monotonic seq makes `claim_id` collision-proof by
construction — the PK can no longer collide for distinct calls, so any remaining
ConstraintViolation is genuinely a `UNIQUE(topic,offset)` conflict and the mapping
is correct again; (2) a deterministic unit test (fixed `now_ns`, shared-prefix
topics) pins the collision closed; (3) learning candidate: "an identifier that is
a PRIMARY KEY must be derived from the FULL uniqueness domain, never a lossy
prefix — and a blanket ConstraintViolation→one-error mapping hides which
constraint actually fired."

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-22T19:15:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2461-claimid-collides-across-distinct-topics-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-2faa613b
- **Timestamp:** 2026-07-22T19:18:48Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 32
     - evidence: `cargo test -p termlink-bus --lib claim_id 2>&1 | tail -5 | grep -qE 'test result: ok'`
  2. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 33
     - evidence: `cargo test -p termlink-bus --lib 2>&1 | tail -5 | grep -qE 'test result: ok'`

### 2026-07-22T19:18:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
