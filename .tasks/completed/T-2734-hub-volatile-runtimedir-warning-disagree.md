---
id: T-2734
name: "Hub volatile-runtime_dir warning disagrees with preflight — root-only gate and /tmp-prefix test (herdr item 7)"
description: >
  Hub volatile-runtime_dir warning disagrees with preflight — root-only gate and /tmp-prefix test (herdr item 7)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/server.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T11:22:16Z
last_update: 2026-08-15T12:13:06Z
date_finished: 2026-08-15T12:13:06Z
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

# T-2734: Hub volatile-runtime_dir warning disagrees with preflight — root-only gate and /tmp-prefix test (herdr item 7)

## Context

Herdr adoption backlog item 7 (rank 7), plus a second defect in the same
function found while reading it.

`warn_if_volatile_default_runtime_dir_impl` (`termlink-hub/src/server.rs:52`)
is the hub-side guard against PL-021 — the repo's worst production failure
class, where `hub.secret` and the TLS cert regenerate and every TOFU-pinned
client must re-auth. It stays silent in two cases it should not:

**(a) the root-only gate.** `if uid != 0 { return false; }`, justified in the
doc comment as "non-root `/tmp/termlink-UID` is the documented default for
interactive sessions and not a footgun". `substrate-preflight.sh` FAILs on the
same state regardless of uid. **Two guards disagree about whether identical
state is dangerous**, and PL-021's consequence is uid-independent: a wiped
`/tmp` regenerates the secret whoever owns it.

**(b) the `/tmp/` prefix test.** `!resolved_str.starts_with("/tmp/")` decides
volatility by path spelling. That is exactly the assumption T-2729 removed from
preflight last session: `discovery.rs` resolves `$XDG_RUNTIME_DIR/termlink` =
`/run/user/<uid>/termlink` **before** it ever reaches `/tmp`, and that is a
tmpfs systemd destroys at the user's last logout — sooner than a reboot. It
matches no `/tmp` prefix, so the hub is silent on the volatile path a normal
systemd session actually gets. Fixing (a) alone would make the two guards agree
about uid while still disagreeing about paths, and would let this task claim
they agree.

**Scope boundary, stated:** preflight is the authoritative detector and does
real mount-type resolution (T-2729). This is a start-time warning, and reading
`/proc/mounts` here would add a Linux-only dependency that
`check-platform-lock.sh` exists to prevent (D4). So this widens the prefix set
to the volatile roots `discovery.rs` can actually produce rather than
reimplementing mount detection. Strictly better than today, and honest about
being a heuristic — the precise answer stays in preflight.

## Acceptance Criteria

### Agent
> **BOTH HALVES NOW PROVEN LOAD-BEARING (2026-08-15, second session).**
>
> **(b) prefix widening — proven by pre-existing-test failure.** With the fix
> in place the pre-existing test `silent_when_root_but_path_not_tmp` FAILED,
> because it asserted the old behaviour on `/run/user/0`. That test had encoded
> the defect as correct — see `## RCA` below, which is the finding that
> outlives the fix.
>
> **(a) uid-gate removal — proven by demonstrated temp-revert.** Re-inserting
> `if uid != 0 { return false; }` and running
> `cargo test -p termlink-hub --lib runtime_dir_warn` produced
> `5 passed; 2 failed`, failing exactly:
>   - `warns_when_non_root_on_volatile_path` — "a wiped /tmp costs the secret
>     at any uid, not just root"
>   - `warns_on_xdg_runtime_dir_path` — "/run/user is a tmpfs destroyed at last
>     logout"
>
> Two failures, not one, is the correct result: the second test uses a non-root
> uid on `/run/user`, so it is sensitive to both halves at once. Restoring gave
> a byte-identical tree (`git diff` empty) and 7/7 green.

- [x] The root-only early return is removed; the warning fires for any uid on a
      volatile default
- [x] Non-root wording does not imply the operator did something wrong — the
      default is the framework's choice, not theirs
- [x] Volatility test covers every volatile root `discovery.rs` can resolve to:
      `/run/`, `/tmp/`, `/var/tmp/`, `/dev/shm/` — not `/tmp/` alone
- [x] A persistent path (`/var/lib/termlink`) still does NOT warn — the guard
      must stay quiet where it should, or it becomes PL-219 alert fatigue
      (`silent_on_persistent_path`)
- [x] `TERMLINK_RUNTIME_DIR` set still suppresses the warning (operator has
      declared intent) — existing behaviour preserved (`silent_when_env_set`)
- [x] Truth-table tests extended: non-root volatile warns, `/run/user/N` warns,
      persistent path stays silent — 7 cases, all green
- [x] New tests demonstrated load-bearing by temp-revert (both halves, above)
- [x] `cargo test -p termlink-hub` passes
- [x] `bash scripts/run-guard-layer.sh` stays clean

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

cargo test -p termlink-hub runtime_dir_warn
cargo test -p termlink-hub
bash scripts/run-guard-layer.sh

## RCA

**Symptom.** The hub start-time guard against PL-021 — the repo's worst
production failure class — stayed silent on the volatile path a normal systemd
user session actually gets. `discovery.rs` consults `$XDG_RUNTIME_DIR` *before*
`/tmp`, so the resolved dir is `/run/user/<uid>/termlink`: a tmpfs systemd
destroys at that user's last logout, sooner than a reboot. The guard tested
`starts_with("/tmp/")`, which `/run/user/...` does not match, and additionally
returned early for any non-root uid. Two independent reasons to say nothing,
on the exact state it exists to report.

**Root cause.** The guard decided volatility by *path spelling* and *uid*,
neither of which is what makes a filesystem volatile. It also disagreed with
`substrate-preflight.sh`, which FAILs on the same state at any uid — so the
project shipped two guards giving opposite verdicts about identical state, with
no mechanism that could notice the contradiction.

**Why structurally allowed.** The interesting half. A pre-existing unit test,
`silent_when_root_but_path_not_tmp`, *asserted the defect was correct
behaviour* — its message read "non-/tmp resolution is not the PL-021 footgun",
which is false: `/run/user` is precisely that footgun. The test had been written
from the implementation rather than from what survives a reboot, so it did not
merely fail to catch the bug — it actively certified it, and made the guard look
covered. Nothing else could see the problem, because the contradiction lived
between a Rust unit test and a shell script that never run in the same context.

**Prevention.** The truth table is now written against volatility (what happens
to the bytes) rather than against spelling and privilege: 7 cases covering both
volatile roots and a persistent path, each demonstrated load-bearing by
temp-revert. The `VOLATILE_DEFAULT_ROOTS` doc comment states plainly that it is
a heuristic and that `/preflight` is authoritative, so the next reader is not
misled into thinking this is a mount-table test.

**Class note (7th instance).** This is the seventh occurrence across three
sessions of one shape: *a guard, test, or report whose verdict rests on an
assumption about its input that no longer holds* — T-2680, T-2709, T-2726,
T-2729, T-2731, T-2732, and this one. Here the assumption was "the default
runtime dir is under /tmp and only root's matters". Seven instances is a
pattern, not a run of luck; whether it becomes a registered concern is a
sovereignty call for the human, and is flagged rather than taken.

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

### 2026-08-15T11:22:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2734-hub-volatile-runtimedir-warning-disagree.md
- **Context:** Initial task creation

### 2026-08-15T12:13:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
