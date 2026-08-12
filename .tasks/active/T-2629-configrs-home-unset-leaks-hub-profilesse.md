---
id: T-2629
name: "config.rs HOME-unset leaks hub profiles+secrets to world-writable /tmp (T-2607 outlier)"
description: >
  config.rs HOME-unset leaks hub profiles+secrets to world-writable /tmp (T-2607 outlier)

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
created: 2026-08-12T09:26:52Z
last_update: 2026-08-12T09:30:35Z
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

# T-2629: config.rs HOME-unset leaks hub profiles+secrets to world-writable /tmp (T-2607 outlier)

## Context

`crates/termlink-cli/src/config.rs::termlink_config_dir()` (the resolver under
`hubs_config_path()` → every fleet **discover-peers** verb and `save_hubs_config`)
still does `std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())`. This is
the exact defect T-2607 unified away in `termlink-session/src/identity_dir.rs` for
the trust plane — but `config.rs` is an outlier T-2607 did not cover. Found by the
T-2628 round-3 portability hunt.

## Acceptance Criteria

### Agent
- [x] `termlink_config_dir()` no longer resolves to the shared, world-writable `/tmp/.termlink` when `HOME` is unset/empty; it falls back to a UID-namespaced private location (mirrors T-2607's `resolve_identity_dir` last-resort convention), NEVER the shared dir
- [x] Behavior is preserved for the common case: when `HOME` is set (non-empty), the resolved dir is still `$HOME/.termlink` (no regression for correctly-configured hosts)
- [x] An exported-but-empty `HOME=` is treated as unset, not as root `/`
- [x] A pure, env-free resolution core is extracted and unit-tested; a load-bearing test proves the guard via temp-revert (reverting to the old `/tmp` fallback makes the test FAIL)
- [x] `cargo test -p termlink --bins` passes; fix committed and finalized through P-011; pushed to OneDev

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
cargo test -p termlink --bins config_dir

## RCA

**Symptom:** With `HOME` unset/empty (systemd unit lacking `Environment=HOME`,
`env -i`, minimal container, cron), fleet discover-peers verbs (`fleet doctor`/
`verify`/`status`/`list`/`governor-status`) read `/tmp/.termlink/hubs.toml` — a
nonexistent, world-writable path — and report "no hubs configured", conflating
"HOME misconfigured" with "no peers exist". Worse, `save_hubs_config` (profile add)
would PERSIST hub profiles + `bootstrap_from` trust anchors into world-writable
`/tmp/.termlink/`, readable/plantable by any local user.

**Root cause:** `termlink_config_dir()` uses `HOME.unwrap_or("/tmp")` — the exact
silent-relocation-to-shared-/tmp pattern T-2607 identified and unified away for the
trust plane, but `termlink-cli/src/config.rs` was an outlier T-2607's sweep
(scoped to `termlink-session`) never touched.

**Why structurally allowed:** T-2607 fixed the identity/trust-plane helpers in one
crate; nothing enforced the same convention on the CLI crate's config-dir resolver,
and the `/tmp` fallback only manifests when HOME is unset — an uncommon dev
condition, so it stayed dark.

**Prevention:** Extract a pure `resolve_config_dir_from()` core with a load-bearing
unit test asserting the all-unset branch never lands in shared `/tmp/.termlink`
(temp-revert to the old fallback fails the test), mirroring `identity_dir.rs`'s
regression guard. Follow-on outliers ([3] infrastructure.rs, substrate.rs log path)
filed separately.

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

### 2026-08-12T09:26:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2629-configrs-home-unset-leaks-hub-profilesse.md
- **Context:** Initial task creation
