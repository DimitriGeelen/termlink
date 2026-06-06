---
id: T-2016
name: "fw upgrade bare-from-consumer replay drops flags — --force-downgrade silently lost during handoff"
description: >
  fw upgrade's bare-from-consumer auto-clone path at lib/upgrade.sh:~305-310 builds a hard-coded _replay_args list (`upgrade`, target_dir, `--force` if set, `--dedupe-user-hooks` if set) before execing the cloned upstream bin/fw. Any other flags the operator passed are silently dropped. Observed today on /opt/termlink: `--force-downgrade` was discarded during handoff so the cloned upstream re-fired the split-brain REFUSED guard. Workaround: invoke upstream bin/fw directly with env-supplied FRAMEWORK_ROOT/PROJECT_ROOT to bypass the bare-from-consumer code path. Sibling to T-2014/T-2099 (same handoff site).

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
last_update: 2026-06-06T11:17:04Z
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

# T-2016: fw upgrade bare-from-consumer replay drops flags — --force-downgrade silently lost during handoff

## Context

Observed live on 2026-06-06 during the T-2014 / T-2099 follow-up work. After bootstrap-replacing the vendored framework to upstream-fresh 1.6.7 containing the T-2099 fork-bomb fix, running `.agentic-framework/bin/fw upgrade --force-downgrade` produced the SAME REFUSED message as without the flag — the framework's split-brain guard fired regardless. Tracing the handoff revealed `lib/upgrade.sh` at the bare-from-consumer auto-clone path builds `_replay_args=("upgrade" "$target_dir")` then conditionally appends `--force` and `--dedupe-user-hooks` only. Every other operator flag (including `--force-downgrade`, `--from-upstream`, `--dry-run`, anything future) is dropped before the exec to the cloned upstream's bin/fw. Workaround used: `env FRAMEWORK_ROOT=/tmp/aef-fresh PROJECT_ROOT=/opt/termlink /tmp/aef-fresh/bin/fw upgrade /opt/termlink --force-downgrade` (T-2099's env-bypass path) — bypasses the bare-from-consumer detection entirely so the flag survives.

This task is the TermLink-side tracker. The fix lands upstream in `agentic-engineering-framework`. Sibling to T-2014/T-2099 (same handoff code area, same root pathology — bare-from-consumer handoff doesn't preserve parent intent). Sibling-2 to T-2015 (CLAUDE.md clobber — same upgrade.sh, different step).

## Acceptance Criteria

### Agent
- [x] RCA captured in `## RCA` block below
- [x] Framework-agent prompt artifact written to `docs/reports/T-2016-fw-upgrade-replay-arg-drop-framework-prompt.md` for operator copy-paste
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

### 2026-06-06T11:17:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2016-fw-upgrade-bare-from-consumer-replay-dro.md
- **Context:** Initial task creation
