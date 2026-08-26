---
id: T-2016
name: "fw upgrade bare-from-consumer replay drops flags — --force-downgrade silently
  lost during handoff"
description: >
  fw upgrade's bare-from-consumer auto-clone path at lib/upgrade.sh:~305-310 builds
  a hard-coded _replay_args list (`upgrade`, target_dir, `--force` if set, `--dedupe-user-hooks`
  if set) before execing the cloned upstream bin/fw. Any other flags the operator
  passed are silently dropped. Observed today on /opt/termlink: `--force-downgrade`
  was discarded during handoff so the cloned upstream re-fired the split-brain REFUSED
  guard. Workaround: invoke upstream bin/fw directly with env-supplied FRAMEWORK_ROOT/PROJECT_ROOT
  to bypass the bare-from-consumer code path. Sibling to T-2014/T-2099 (same handoff
  site).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-06T11:17:04Z
last_update: 2026-08-20T18:17:18Z
date_finished:
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
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=2 (body:lightly-promoted); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2016: fw upgrade bare-from-consumer replay drops flags — --force-downgrade silently lost during handoff

## Context

Observed live on 2026-06-06 during the T-2014 / T-2099 follow-up work. After bootstrap-replacing the vendored framework to upstream-fresh 1.6.7 containing the T-2099 fork-bomb fix, running `.agentic-framework/bin/fw upgrade --force-downgrade` produced the SAME REFUSED message as without the flag — the framework's split-brain guard fired regardless. Tracing the handoff revealed `lib/upgrade.sh` at the bare-from-consumer auto-clone path builds `_replay_args=("upgrade" "$target_dir")` then conditionally appends `--force` and `--dedupe-user-hooks` only. Every other operator flag (including `--force-downgrade`, `--from-upstream`, `--dry-run`, anything future) is dropped before the exec to the cloned upstream's bin/fw. Workaround used: `env FRAMEWORK_ROOT=/tmp/aef-fresh PROJECT_ROOT=/opt/termlink /tmp/aef-fresh/bin/fw upgrade /opt/termlink --force-downgrade` (T-2099's env-bypass path) — bypasses the bare-from-consumer detection entirely so the flag survives.

This task is the TermLink-side tracker. The fix lands upstream in `agentic-engineering-framework`. Sibling to T-2014/T-2099 (same handoff code area, same root pathology — bare-from-consumer handoff doesn't preserve parent intent). Sibling-2 to T-2015 (CLAUDE.md clobber — same upgrade.sh, different step).

## Acceptance Criteria

### Agent
- [x] RCA captured in `## RCA` block below
- [x] Framework-agent prompt artifact written to `docs/reports/T-2016-fw-upgrade-replay-arg-drop-framework-prompt.md` for operator copy-paste
> **2026-08-20:** still blocked, correctly. No upstream fix has landed — the code is
> byte-identical to the 2026-06-06 diagnosis. What HAD been missing was the filing itself:
> AC 2 wrote a report to `docs/reports/` "for operator copy-paste" and nobody pasted it.
> Now filed at `framework:pickup` **offset 28**, with the scope corrected — **three** live
> flags are dropped (`--force-downgrade`, `--strict`, `--no-self-vendor`), not one.
> This AC stays unticked: filing is not fixing, and ticking it would be the
> installation-vs-outcome error logged four times this session.
>
- [ ] After upstream fix lands in vendored `.agentic-framework/lib/upgrade.sh`, re-run `fw upgrade --force-downgrade` and confirm the flag survives through the bare-from-consumer handoff (split-brain refusal no longer fires when --force-downgrade is on)

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
> **2026-08-20 — this criterion has been overtaken, and is left for you to dispose of.**
> The artifact exists so an operator could copy-paste it to the framework agent. That relay
> did not happen in 76 days, so the report was filed directly over `framework:pickup`
> (**offset 28**) instead — with a wider and more accurate scope than the artifact carries:
> three dropped flags rather than one, plus a corrected non-finding (`--dry-run` is NOT
> dropped; a guard at `:460` returns before the handoff).
> So "is the artifact operator-ready?" is no longer the question that matters, and the
> artifact is now the less accurate of the two records. Tick it, or retire it in favour of
> the filing — either is reasonable and both are yours.
>
- [ ] [REVIEW] Framework-agent prompt at `docs/reports/T-2016-fw-upgrade-replay-arg-drop-framework-prompt.md` is operator-ready
  **Steps:**
  1. Open the file
  2. Read top-to-bottom as if you knew nothing about the bare-from-consumer handoff
  3. Verify it contains: symptom, repro, file:line root cause, recommended fix shape (whitelist vs pass-through-all)
  **Expected:** Self-contained prompt — no follow-up clarifying questions needed from framework-agent
  **If not:** Note what's missing and revise the artifact, not the upstream fix

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

test -f docs/reports/T-2016-fw-upgrade-replay-arg-drop-framework-prompt.md
grep -q '_replay_args' docs/reports/T-2016-fw-upgrade-replay-arg-drop-framework-prompt.md
grep -q 'Root cause' .tasks/active/T-2016-fw-upgrade-bare-from-consumer-replay-dro.md

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

**Symptom:** Operator runs `.agentic-framework/bin/fw upgrade --force-downgrade` (or any other extension flag the framework supports). The bare-from-consumer auto-clone detection fires, upstream is cloned, and the handoff exec runs — but the flag is not on the exec'd command line. The cloned upstream's bin/fw therefore re-evaluates without the flag and hits the same `--force-downgrade`-gated guard the operator was trying to bypass. The visible symptom is "I passed --force-downgrade and the REFUSED message fires anyway."

**Root cause:** `lib/upgrade.sh` at the bare-from-consumer auto-clone path builds the handoff argument list explicitly:

```sh
local _replay_args=("upgrade" "$target_dir")
[ "$force" = true ] && _replay_args+=("--force")
[ "$dedupe_user_hooks" = true ] && _replay_args+=("--dedupe-user-hooks")
```

This whitelists exactly two flags. Anything else the operator passed is silently dropped before the handoff. The list pre-dates `--force-downgrade`'s addition; it was never updated when that flag was added to the upgrade verb. It will silently drop every future flag too.

**Why structurally allowed:**
- The handoff design assumes the parent already parsed flags into named booleans (`force`, `dedupe_user_hooks`) and the child only needs the named ones. Any flag NOT promoted to a parent-side variable can't be replayed.
- No test exercises "handoff preserves operator flags".
- The framework's `do_upgrade --help` documents flags including `--force-downgrade` but the bare-from-consumer replay does not consume that help text dynamically — the replay list is a static hand-written subset.
- Sibling pathology to T-2014 / T-2099 (same handoff site, same "parent intent not preserved across handoff" failure family). The fork-bomb fix (T-2099) preserved FRAMEWORK_ROOT and PROJECT_ROOT via env. Flag preservation is the next-level concern.

**Prevention:**
1. **Primary** — pass-through-all. Capture the original argv when `do_upgrade` is invoked from the consumer path, replay all of it minus `upgrade` and `target_dir` (the positional). Something like:
   ```sh
   # at do_upgrade entry, capture before flag-stripping:
   local _all_args=("$@")
   # ... later, when building _replay_args:
   local _replay_args=("upgrade" "$target_dir" "${_all_args[@]:1}")
   # (skip the verb itself which is at index 0)
   ```
   Caveat: requires care around `--from-upstream URL` (already consumed by parent to know the clone URL) — likely needs explicit exclusion.
2. **Secondary** — explicit whitelist that includes `--force-downgrade`, `--from-upstream`, `--dry-run`, plus any future flag. Lower drift than option 1 but requires maintenance every time a flag is added.
3. **Test** — `tests/e2e/upgrade-test.sh` adds a regression: seed a consumer that triggers bare-from-consumer, invoke with `--dry-run` (or any flag with observable behavior), assert the cloned upstream's bin/fw saw the flag. Without `--from-upstream` plumbing in fixtures this needs care, but the principle is the same.
4. **Doc** — comment at the `_replay_args` site stating the invariant: "Any flag the operator passes to `fw upgrade` MUST survive the bare-from-consumer handoff. Adding a new upgrade flag without updating this list will cause it to be silently lost. Prefer pass-through-all over hand-curated whitelist."

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

**Recommendation:** DEFER — park this tracker against the next re-vendor of
`.agentic-framework/`, rather than closing it or leaving it hot as actionable work.

**Rationale:** Every part of this task that TermLink controls is done: the defect is
diagnosed, its scope corrected from one dropped flag to three, filed upstream, and
guarded locally by a wrapper. The single remaining Agent AC — "after upstream fix lands
in vendored `lib/upgrade.sh`, re-run and confirm the flag survives" — is gated on an
actor this project does not control and cannot schedule. Leaving it `horizon: now` has
already caused it to be picked up twice as top actionable work, each time producing the
same finding that nothing has changed; that is attention the task cannot repay. Closing
it outright would discard the re-verification obligation, which is precisely this task's
own documented failure mode.

**Evidence:** Re-checked in this tree today. `.agentic-framework/lib/upgrade.sh:1068-1070`
still reads `_replay_args=("upgrade" "$target_dir")` with conditional appends for
`--force` and `--dedupe-user-hooks` only — byte-identical in shape to the 2026-06-06
diagnosis, so no upstream fix has landed. `scripts/fw-upgrade-safe.sh` exists (7564 bytes,
executable) and warns on the three dropped flags per the 2026-08-26 note. Filing is at
`framework:pickup` offset 28; offset 46 was withdrawn and retracted at 47.

**What you are actually deciding.** Two things, and they are separable.

*First, dispose of the Human AC.* "[REVIEW] Framework-agent prompt is operator-ready" was
overtaken on 2026-08-20: the artifact was superseded by the offset-28 filing, which
carries the wider and more accurate scope. The AC's own note says tick it or retire it,
either is reasonable. Nothing downstream depends on the answer.

*Second, the task's disposition.* Three realistic options:

| Option | Behaviour | Cost |
|---|---|---|
| DEFER (recommended) | stays open, stops presenting as now-actionable, re-checked at the next re-vendor | needs `revisit_at` / `revisit_evidence_needed` set, or the deferral has no reminder |
| CLOSE | tracker retired; defect lives on at offset 28 + the local wrapper | the "confirm the flag survives" check has no home — it is **not** currently registered in `.vendor-divergence.yaml`, so closing drops it silently |
| KEEP-OPEN as-is | unchanged | it re-surfaces as top HV/LC work and the next pickup re-derives the same "still blocked, correctly" |

**One thing that is not measured.** Whether upstream intends to act on offset 28 at all.
The filing was made 2026-08-18; no receipt or reply has been checked as part of this
recommendation. If upstream is not consuming that rail, DEFER is deferring against an
event that may never arrive, and CLOSE becomes the more honest answer.

**Why I should not decide this alone.** The choice between DEFER and CLOSE is a judgement
about how long this project is willing to hold an obligation on another project's
roadmap — and, if CLOSE, whether the re-check is worth re-homing. That is a
prioritisation call about your attention, not a correctness question the code can settle.

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

### 2026-06-06T15:12Z — pickup envelope delivered to framework:pickup [agent autonomous]

Operator nudge: "why not send a pickup note?" — taken. Posted structured `pickup-bug-report` envelope to `framework:pickup` topic.

**Delivery:**
- Topic: `framework:pickup`
- msg_type: `pickup-bug-report`
- pickup_id: `termlink-T-2016-2026-06-06`
- Offset: **35**
- ts: `1780752504015` (2026-06-06T15:12Z)
- Payload: RCA + proposed pass-through fix + ref to `docs/reports/T-2016-fw-upgrade-replay-arg-drop-framework-prompt.md`
- Sibling pickup T-2015 at offset 34 in same dispatch.

**Queue health flag:** see T-2015 update — `framework:pickup` has 36 envelopes and 0 receipts; reader-side has not been acking. Pickup landed correctly on the durable channel; consumption is the open question.

### 2026-06-06T11:17:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2016-fw-upgrade-bare-from-consumer-replay-dro.md
- **Context:** Initial task creation

### 2026-08-20 — filed upstream, 76 days after the report was written to a file nobody read [agent]

Picked up as the top actionable HV/LC task (BVP 78). Its remaining Agent AC waits on "after
upstream fix lands in vendored `lib/upgrade.sh`, re-run and confirm the flag survives". No fix
has landed. The code is byte-identical to the 2026-06-06 diagnosis.

**Why it never landed.** AC 2, ticked, reads:

> Framework-agent prompt artifact written to
> `docs/reports/T-2016-fw-upgrade-replay-arg-drop-framework-prompt.md` for operator copy-paste

A carefully-prepared report was written to a file, the AC was marked done, and the relay never
happened. Filed today over `framework:pickup` — **offset 28** — the rail that was available the
whole time. This is the third instance logged this session of an obligation recorded in a place
with no reader (the others: T-1166's bake checkpoint in prose, 71 days unrun; the "Framework bug
to file" note at the end of commit `444a7e9b3`, never filed).

**Re-measured before filing, and the scope is wider than the title.** The parser accepts
**seven** flags; `_replay_args` replays **two**:

| flag | replayed? | |
|---|---|---|
| `--force` | yes | |
| `--dedupe-user-hooks` | yes | |
| `--from-upstream` | no | **correct** — deliberate, with a comment; upstream IS the source by then |
| `--force-downgrade` | **no** | the reported symptom |
| `--strict` | **no** | behaviour-changing (`:847`, `:1542`) |
| `--no-self-vendor` | **no** | behaviour-changing (`:519`) |
| `--dry-run` | **no** | but **unreachable** — see below |

So **three** live flags are silently dropped, not one.

**A wrong hypothesis, checked before it became a claim.** I believed `--dry-run` was dropped
too, which would have been much worse — a preview invocation performing a real upgrade. It is
not: a `dry_run` guard at `:460` returns before the clone, so the handoff is never reached. I
was one step from filing that as the headline and it would have been false.

The guard does print `[dry-run] would re-invoke: ... upgrade $target_dir --dry-run`, describing
an argv the live path does not construct. Harmless while the paths are disjoint; a lie rather
than an inaccuracy if they ever converge. Reported as cosmetic, four lines above the real bug.

**The allowlist is the defect, not the missing entries.** `_replay_args` enumerates what to
KEEP, so every flag added later is dropped by default and nothing says so. Carrying the
operator's argv forward and subtracting the single flag that must not be replayed fails safe in
the direction that matters, and makes `--from-upstream`'s exclusion explicit rather than
implicit in what the list omits. Suggested as a shape, not a patch — `lib/upgrade.sh` is
vendored (G-062) and a local edit is erased by the next re-vendor, which is precisely what
T-2812 registered this week.

**Remaining AC unchanged and still unticked.** "Confirm the flag survives" needs an upstream fix
to exist first. Filing does not satisfy it, and ticking it would be the installation-vs-outcome
error this session has now found four times.

## 2026-08-26 — consumer-side guard landed; and a correction I owe this task

**Recommendation:** leave every AC as it stands. Nothing here is a fix; the defect
is in vendored code and remains upstream's (G-062).

### What landed

`scripts/fw-upgrade-safe.sh` now warns **before** the run when an invocation
carries a flag the bare-from-consumer handoff would discard
(`--force-downgrade`, `--strict`, `--no-self-vendor`). Verified with three
negative controls — `--force`, `--dry-run` and `--dedupe-user-hooks` do not warn
— and two positives. The warning is deliberately placed ahead of the
dirty-tree refusal: a warning about the operator's own input costs nothing and
is useful even when the script then declines to run. The first version had it
after, and printed nothing in exactly the case that surfaced it.

This protects this consumer only. The wrapper's hand-maintained list will drift
from the framework's parser the same way the framework's replay list drifted
from its own — which is an argument for the inversion proposed at
`framework:pickup` offset 28, not for copying the wrapper.

### The correction

I re-derived this task's scope from scratch and filed it upstream as new
(`framework:pickup` offset 46), stating that "our own task did not name
`--strict`". **That is false.** The Agent AC block above carries a dated
2026-08-20 note reading *"the scope corrected — three live flags are dropped
(`--force-downgrade`, `--strict`, `--no-self-vendor`), not one"*, and offset 28
filed all three on 2026-08-18 — with a better remedy than the one I proposed.

Offset 46 is withdrawn; retraction at offset 47.

What went wrong is worth keeping, because it is this task's own failure mode
inverted. I checked the CODE carefully — enumerated the parser's case arms,
verified the `--dry-run` short-circuit before believing it, ran negative
controls. I did not check the RECORD. I read the `description:` frontmatter,
which names only `--force-downgrade`, and stopped. "Nothing in the description
names `--strict`" and "nothing names `--strict`" are different claims, and I
published the second having established only the first.

Offset 28 closes on obligations written where no reader looks. This is the
inverse: a report written without looking where the reader already wrote. Same
root, opposite direction — and the cheap guard against it is to read a task's
AC block, not just its description, before claiming novelty.
