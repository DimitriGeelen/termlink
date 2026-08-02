---
id: T-2491
name: "remote-exec and session-exec --json branches fail open reporting ok:true exit 0 when exit_code is missing"
description: >
  remote-exec and session-exec --json branches fail open reporting ok:true exit 0 when exit_code is missing

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
created: 2026-08-02T09:03:56Z
last_update: 2026-08-02T09:03:56Z
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

# T-2491: remote-exec and session-exec --json branches fail open reporting ok:true exit 0 when exit_code is missing

## Context

Two `--json` exec-result renderers default a missing/malformed `exit_code` to **0 (success)**:
`cmd_remote_exec` (`crates/termlink-cli/src/commands/remote.rs:2019`) and `cmd_exec`
(`crates/termlink-cli/src/commands/session.rs:963`). Each computes `{"ok": exit_code == 0}` from that
default. The adjacent TEXT branch of both functions (remote.rs:2033, session.rs:977) and the sibling
`exec_rpc` helper (push.rs:177) all default to **-1 (failure)** — so the `--json` path is the lone
fail-OPEN site, and the correct value is not a judgement call but the codebase's own established
fail-closed convention. Impact: when a hub returns `RpcResponse::Success` whose `result` lacks a
well-formed `exit_code` (fleet version skew — pervasive here; or any alternate/future handler shape),
`termlink … exec --json` prints `{"ok":true,…}` and returns exit 0, so an orchestrator/canary treats a
remote command that failed or never reported an outcome as success. The `--json` surface is precisely
the one automation consumes. Directive-#2 (no silent failures). Found by firing-#13 adversarial audit;
verified current in code (both sites confirmed).

## Acceptance Criteria

### Agent
- [x] A single shared fail-closed helper (`exec_json_envelope`, commands/mod.rs) computes the exec
      `--json` envelope: a missing/malformed `exit_code` defaults to **-1** (so `ok` is `false`),
      matching the text branch + push.rs convention.
- [x] Both `cmd_remote_exec` (remote.rs) and `cmd_exec` (session.rs) `--json` branches use the shared
      helper — no remaining `unwrap_or(0)` on `exit_code` in a CLI `--json` exec path.
- [x] New unit test: the helper on a result WITHOUT `exit_code` yields `ok:false` and a non-zero
      (negative) effective exit code — NOT `ok:true`/0. (`missing_exit_code_fails_closed`; plus
      `malformed_exit_code_fails_closed`, `nonzero_exit_code_not_ok`.)
- [x] New unit test: the helper on a result with `exit_code:0` yields `ok:true` and preserves the
      other result fields (stdout/stderr) in the envelope. (`zero_exit_code_ok_and_fields_preserved`.)
- [x] `cargo test -p termlink --bin termlink exec_json_envelope` passes (4/4); `cargo check -p termlink` clean.

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
cargo test -p termlink --bin termlink exec_json_envelope
cargo check -p termlink

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

**Symptom:** `termlink remote exec … --json` (and `termlink exec … --json`) prints `{"ok":true,…}`
and exits 0 for a remote command whose result carried no well-formed `exit_code` — so an automated
consumer records success for a command that failed or never reported an outcome. The same response
through the non-`--json` path correctly fails (exit 255).

**Root cause:** The `--json` branches defaulted `result["exit_code"].as_i64()` to `0` (`unwrap_or(0)`)
and derived `ok` from that, i.e. they failed OPEN — treating "no exit code" as "succeeded". The text
branches and `push.rs::exec_rpc` default the same read to `-1` (fail CLOSED). The two branches of one
function disagreed on the meaning of a missing field.

**Why structurally allowed:** The exit-code defaulting was duplicated inline across four sites with no
single source of truth, so a wrong default in one copy could not be caught by the others; the
`--json` copies happened to pick the fail-open default. No test exercised a Success response missing
`exit_code`, so the divergence between the JSON and text branches of the same command was never
observed — and the fail-open path is the machine-consumed one, the least likely to be eyeballed.

**Prevention:** A single `exec_json_envelope` helper now owns the fail-closed default (missing →
`-1` → `ok:false`) and both `--json` sites call it, so the convention has one source of truth and a
future copy cannot silently re-introduce a fail-open default. Two unit tests lock it (missing
`exit_code` → `ok:false`/negative; `exit_code:0` → `ok:true` + fields preserved). Learning: when a
fallible read is defaulted inline at multiple sites, a wrong default hides among the right ones —
extract one helper so "what does a missing field mean?" is answered once, fail-closed.

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

### 2026-08-02T09:03:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2491-remote-exec-and-session-exec---json-bran.md
- **Context:** Initial task creation
