---
id: T-2743
name: "Prove a backgrounded session survives its launcher disconnecting (SIGHUP)"
description: >
  Prove a backgrounded session survives its launcher disconnecting (SIGHUP)

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
created: 2026-08-15T19:14:52Z
last_update: 2026-08-15T19:14:52Z
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

# T-2743: Prove a backgrounded session survives its launcher disconnecting (SIGHUP)

## Context

Herdr adoption backlog rank 17 (worker 4, §6.5), which necessarily settles rank
11 (worker 4, R2) with it. The two cannot be separated: rank 17 asks for a test
of the "survives SSH disconnect" property, and a guard for a property that does
not currently hold cannot be added without making it hold.

The `background` backend exists to produce a process that outlives the shell
that launched it. Today all three spawn sites — `execution.rs:541`,
`dispatch.rs:992`, `tools.rs:13799` — run `setsid sh -c <cmd>` and, on spawn
error, fall back to a bare `sh -c` with no `setsid` and no `nohup`. `setsid(1)`
is util-linux; macOS does not ship it, so on macOS the fallback always fires and
the child stays in the launcher's session. It starts, so the call site reports
success — and then dies on SIGHUP when the SSH connection drops.

`.context/checks/platform-lock-allowlist:23-32` acknowledges this as "degraded
but functional". Worker 4 disputes that, and is right: the allowlist's own rule
demands the reason state how the non-Linux path *behaves*, and "functional for
spawning" is not "functional for surviving disconnect" — which is the entire
feature. That entry describes the mechanism and omits the consequence, so it
reads as an accepted trade-off when it is a silent failure of the thing being
acknowledged.

Worker 4 flagged the hazard as UNREPRODUCED — it did not test whether SIGHUP
actually reaches the child. This task does not need that test to proceed,
because it does not rest on the hazard: the fix replaces the `setsid(1)` binary
with the POSIX `setsid(2)` syscall in `pre_exec`, which is present on both
platforms, needs no fallback, and makes the disagreement moot by deleting the
sites rather than re-arguing the judgment recorded in the allowlist.

Property under test, stated so it is checkable without an SSH session or a
macOS host: **a backgrounded child must lead its own session**
(`getsid(child) == child`, and `!= getsid(launcher)`). That is precisely what
detaches it from the launcher's SIGHUP, and it is observable in-process.

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
> **LOAD-BEARING PROOF (2026-08-15).** Deleting the `pre_exec` block from
> `spawn_detached` — which is exactly the child the macOS fallback produced —
> fails `a_backgrounded_child_leads_its_own_session` with
> `left: 900848, right: 919185`: the child's session id came back as the
> launcher's rather than its own pid. The control
> `a_plain_child_stays_in_the_launchers_session` stayed green throughout, as it
> must, since it asserts the reverted behaviour. Restoring returns the module to
> 14/14 and the tree to zero diff (`TEMP-REVERT` count 0). The proof is the
> point of the task: it shows a Linux host can now catch the macOS-only defect.

- [x] The detachment property is asserted by a test: a backgrounded child leads its own session (`getsid(pid) == pid`) and does not share the launcher's session
- [x] A control test proves the assertion can distinguish — a child spawned without the syscall shares the launcher's session, which is exactly what the macOS fallback produced (PL-219: an assertion that cannot fail is not a guard)
- [x] All three spawn sites (`execution.rs`, `dispatch.rs`, `tools.rs`) go through the syscall path — verified by grep, not by fixing the one I happened to read first (PL-344)
- [x] No `Command::new("setsid")` remains in the product crates, and no bare-`sh -c` fallback remains at those sites
- [x] A `setsid(2)` failure surfaces as a spawn error rather than a silently non-detached child — the feature fails loud or works, never quietly half-works
- [x] The three `cmd:setsid` platform-lock allowlist entries are removed because the sites are gone, and `check-platform-lock.sh` is clean with no new acknowledgement added in their place (8 sites scanned → 5, all acknowledged)
- [x] The stale README §Backends claim about macOS `setsid` daemonization is corrected to match the new behaviour
- [x] Load-bearing proof recorded: reverting the syscall to the old spawn shape makes the detachment test fail

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

cargo test -p termlink --bins
cargo test -p termlink-mcp --lib
bash scripts/check-platform-lock.sh
bash tests/platform-lock-check-fixtures.sh

## RCA

**Symptom:** on macOS, a session started with the `background` backend did not
survive the launching terminal closing or the SSH connection dropping — the one
property that backend exists to provide. Nothing reported an error; the spawn
returned success.

**Root cause:** the code obtained detachment by exec'ing the `setsid(1)` binary,
which is util-linux and absent on macOS. The spawn therefore errored and an
`.or_else` fallback ran a bare `sh -c` instead, producing a child in the
launcher's session — reachable by the SIGHUP sent on hangup. The mechanism was
platform-specific while the guarantee was advertised as universal.

**Why structurally allowed:** two separate blindnesses, and the second is the
one worth keeping. (i) The survival property had no test — it was argued from
the mechanism, so when the mechanism stopped applying nothing noticed; that is
herdr backlog item 17, and it names itself as the guard that would have caught
item 11. (ii) The platform-lock check *did* flag all three sites, and they were
acknowledged in the allowlist as "degraded but functional". That reason
described the mechanism ("process starts, no session leader") and omitted the
consequence, so the acknowledgement read as an accepted trade-off rather than
the silent feature loss it was. An allowlist entry whose stated reason is
incomplete is worse than no entry: it converts an open question into a closed
one and stops anybody looking. The file's own rule already demanded the reason
state how the non-Linux path *behaves* — the rule was right and the entry did
not meet it.

**Prevention:** the property is now tested rather than argued —
`a_backgrounded_child_leads_its_own_session` asserts `getsid(child) == child`,
with `a_plain_child_stays_in_the_launchers_session` as the control that proves
the assertion can fail. Both run on Linux, so the macOS-only defect is now
catchable on the CI that exists. Beyond the test, the fix removes the platform
dependency entirely: `setsid(2)` is POSIX, so there is no fallback left to
degrade into and no acknowledgement left to get wrong.

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

### 2026-08-15T19:14:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2743-prove-a-backgrounded-session-survives-it.md
- **Context:** Initial task creation
