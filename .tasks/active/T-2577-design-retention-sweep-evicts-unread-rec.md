---
id: T-2577
name: "DESIGN: retention sweep evicts unread records; subscribe surfaces no gap (auto-detect?)"
description: >
  DESIGN needs-human: retention sweep (Bus::sweep / retention_sweeper) deletes records purely by policy (Messages(N)/Days/Latest/LatestPerCvKey) with ZERO cursor awareness, and channel.subscribe then silently advances a lagging cursor over the swept span with no gap indicator in the response. A lagging subscriber loses unread records with nothing in the subscribe reply flagging it. Detection exists only as a separate opt-in verb (gap_before/oldest_offset). Whether to auto-fire (warn/refuse when sweeping below the min live subscriber cursor, or fold gap detection into every subscribe response) is a bounded-retention-vs-durability design/threshold call. From T-2468 verb-2 hunt. Relates to charter non-goal #2 and T-2562.

status: captured
workflow_type: build
owner: human
horizon: later
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T15:11:56Z
last_update: 2026-08-09T15:11:56Z
date_finished: null
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

# T-2577: DESIGN: retention sweep evicts unread records; subscribe surfaces no gap (auto-detect?)

## Context

Filed from T-2468 verb-2 hunt. `sweep_records` (`crates/termlink-bus/src/meta.rs`)
and `retention_sweeper::sweep_all` delete records purely by policy
(`Messages(N)` / `Days` / `Latest` / `LatestPerCvKey`) with **zero cursor
awareness** — they never consult per-subscriber cursors. `channel.subscribe`
(`crates/termlink-hub/src/channel.rs`) then does `records_from(cursor)` and advances
`next_cursor` over the survivors with **no gap indicator** in the response. Net: a
lagging subscriber whose unread records get swept jumps the gap silently — the
subscribe reply looks completely normal.

Repro: `create_topic t Messages(2)`; post 5; subscriber cursor=1; `sweep t` keeps
offsets 3,4; `subscribe t cursor=1` returns only 3,4 with `next_cursor=5` — offsets
1,2 lost, response indistinguishable from a healthy read.

**Why needs-human (not a mechanical fix):** this is arguably PARTLY by design —
charter non-goal #2 says durability means "survives a hub blip and replays", NOT
"stored forever", and bounded retention deliberately drops old records (T-2562
guards the opposite failure, unbounded Forever growth). The SILENT part is the real
issue, and the remedy is a threshold/semantics decision: (a) hub-side warn/refuse
when a sweep would delete below the minimum live subscriber cursor; (b) fold a
`gap: {from, to}` indicator into every subscribe response when the walk started
past `oldest_offset`; (c) leave detection to the existing opt-in `gap_before` /
`oldest_offset` verbs (T-1285/T-2463) and just document the contract. Each has a
different cost/telemetry/compat profile — a human/design call.

## Acceptance Criteria

### Human
- [ ] Decide the remedy among (a) sweep-side guard below min live cursor, (b)
      inline gap indicator in the subscribe response, (c) keep opt-in gap verbs +
      document — with rationale tied to the bounded-retention (non-goal #2) vs
      no-silent-failures (Reliability) trade-off.
- [ ] Confirm the boundary vs T-2562 (forever-archival guard) and the existing
      `gap_before`/`oldest_offset` primitives so the chosen remedy does not
      duplicate or contradict them.
- [ ] If "change behaviour", file a build task with concrete ACs + a load-bearing
      test (a lagging subscriber whose span was swept is told, not silently
      skipped); if "document only", record that the opt-in gap verbs are the
      sanctioned detection path and where that contract is stated.

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

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

## Recommendation

**Recommendation:** GO — adopt option (b): fold a gap indicator into the
`channel.subscribe` response whenever the walk started past `oldest_offset`.

**Rationale:** All three options accept that bounded retention deletes old
records — that part is charter non-goal #2 working as designed and is not in
dispute. What is in dispute is only whether the subscriber is *told*. Option (b)
is the sole option that delivers the signal at the moment of loss, to the party
that suffered it, without anyone having to remember to ask. Option (c) leaves
detection opt-in, which is PL-168's dormant-tooling class — the failure this repo
has closed repeatedly elsewhere. Option (a) fixes silence by refusing the sweep,
which converts a detectable data-loss problem into an unbounded-growth one: an
abandoned subscriber's stale cursor would pin its topic forever, exactly the
failure T-2562's forever-archival canary exists to catch, reached from the other
direction.

**Evidence:** Measured in-tree 2026-08-27. `sweep_records`
(`crates/termlink-bus/src/meta.rs:333`) takes `(topic, keep_after_ts_ms,
keep_last_n)` — there is no cursor parameter at all, confirming the filed claim of
zero cursor awareness. `Bus::gap_before` exists at
`crates/termlink-bus/src/lib.rs:329` with two tests (`:1191` fell-behind,
`:1214` no-false-positive-when-caught-up), so the detection primitive is already
built and pinned — option (b) is wiring an existing capability into the default
path, not new detection logic. **Not measured:** whether any real subscriber has
ever lost unread records here. That absence is not evidence of safety — the
subscribe reply is by construction indistinguishable from a healthy read, so a
live occurrence would leave no trace to count.

**What you are actually deciding.** Not whether the gap is real — the repro in
Context is deterministic and the code confirms it. You are deciding what TermLink
*promises* a subscriber: that it will be told when it lost records, or that
checking is its own job.

| Option | Behaviour | Cost |
|---|---|---|
| (b) inline gap in subscribe (recommended) | every subscribe past `oldest_offset` carries `gap: {from, to}` | response-shape change (additive, so older clients ignore it); one extra `oldest_offset` read per subscribe on a hot path |
| (c) keep opt-in verbs + document | `gap_before` / `oldest_offset` stay the sanctioned path | the silent path stays silent; detection depends on a caller remembering — PL-168 dormant tooling |
| (a) sweep-side guard below min live cursor | refuse/warn when a sweep would delete below the lowest live cursor | an abandoned subscriber pins its topic and it grows unboundedly (T-2562 inverted); also needs a "live subscriber" liveness definition that does not exist today |

**Why I should not decide this alone.** Options (b) and (c) are both defensible
readings of the charter, and they disagree about what "durable" means — (b) says
durability includes being told when it lapsed, (c) says durability is
best-effort-with-tools-to-check. That is a promise the project makes to its
consumers, not a correctness question the code can settle. Option (a) I would
argue against on evidence; between (b) and (c) I can only state the trade.

**If you pick (c):** the second Human AC still matters — record *where* the
contract is stated, or the sanctioned detection path is folklore. **If you pick
(b):** the load-bearing test named in the third Human AC is the repro already
written in Context, asserted to return a gap rather than a clean read.

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

### 2026-08-09T15:11:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2577-design-retention-sweep-evicts-unread-rec.md
- **Context:** Initial task creation
