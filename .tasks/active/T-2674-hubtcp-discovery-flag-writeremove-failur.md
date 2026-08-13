---
id: T-2674
name: "hub.tcp discovery-flag write/remove failures are silently swallowed in server.rs (Directive #2)"
description: >
  hub.tcp discovery-flag write/remove failures are silently swallowed in server.rs (Directive #2)

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
created: 2026-08-13T07:30:40Z
last_update: 2026-08-13T07:30:40Z
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

# T-2674: hub.tcp discovery-flag write/remove failures are silently swallowed in server.rs (Directive #2)

## Context

The hub records its live TCP address into `<runtime_dir>/hub.tcp` (T-1026) so that
restart re-bind and address-file consumers can discover the endpoint. In BOTH hub
start paths (`run_with_tcp` ~server.rs:261 and the second launcher ~server.rs:413)
the write is `let _ = std::fs::write(&tcp_flag, local_addr.to_string());` — the
error is silently discarded. The line immediately above logs
`tracing::info!(%local_addr, "Hub listening on TCP (TLS)")`, so on a failed write
(read-only runtime_dir, disk full, perms) the hub keeps listening but advertises NO
address, and the operator sees only the reassuring "Hub listening" line — a textbook
Directive #2 silent failure ("operator believes it was recorded when it wasn't").
The `else`-branch `let _ = std::fs::remove_file(&tcp_flag)` (2 more sites) is the
same class: a failed removal of a stale flag silently leaves a DEAD endpoint
advertised to discovery consumers.

Fix must be observable-but-non-fatal: the hub must NOT die over a discovery-flag
write (bubbling `?` would be worse than the silent swallow), so the remediation is a
`tracing::warn!` naming the path + error on failure. The stale-flag `remove_file`
must suppress the expected `NotFound` (starting without a prior flag is normal) and
warn only on a real error. Same root-cause omission across 4 sibling sites in one
file → one task (CLAUDE.md one-root-cause-across-siblings rule).

## Acceptance Criteria

### Agent
- [x] Both `hub.tcp` WRITE sites in server.rs log a `tracing::warn!` (naming path +
      error) on failure instead of `let _ =`; the hub still proceeds (non-fatal).
- [x] All 3 `hub.tcp` `remove_file` sites (2 start-else-branches + 1 shutdown
      cleanup) warn on a real error but SUPPRESS `ErrorKind::NotFound` (no spurious
      warning on the normal no-prior-flag / already-gone case).
- [x] `cargo build -p termlink-hub` succeeds; no remaining bare `let _ = std::fs::write`
      / `let _ = std::fs::remove_file` on the `tcp_flag` in server.rs.

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

cargo build -p termlink-hub 2>&1 | tail -1
out=$(grep -c 'let _ = std::fs::write(&tcp_flag' crates/termlink-hub/src/server.rs 2>&1); [ "${out:-1}" -eq 0 ]
out=$(grep -c 'let _ = std::fs::remove_file(&tcp_flag' crates/termlink-hub/src/server.rs 2>&1); [ "${out:-1}" -eq 0 ]
out=$(grep -c 'hub.tcp' crates/termlink-hub/src/server.rs 2>&1); [ "${out:-0}" -ge 2 ]

## RCA

**Symptom:** On a hub whose `runtime_dir` is read-only / full / wrong-perms, the
hub logs "Hub listening on TCP (TLS)" and serves requests, but `<runtime_dir>/hub.tcp`
is never written (or a stale one is never removed). Discovery consumers and restart
re-bind then see a stale/absent address with NO log line explaining why — the
operator believes the endpoint was recorded when it silently was not.

**Root cause:** both hub start paths write the `hub.tcp` discovery flag with a bare
`let _ = std::fs::write(...)` (and clear it with a bare `let _ = std::fs::remove_file(...)`),
discarding the `io::Result`. The reassuring `tracing::info!` on the line above masks
the swallowed failure.

**Why structurally allowed:** the "record the address" write is genuinely best-effort
for the hub's OWN operation (it keeps serving without it), so `let _ =` looked
harmless — but "non-fatal" was conflated with "silent". Directive #2 requires the
FAILURE to be observable even when it is non-fatal. Nothing distinguished the two.

**Prevention:** the fix itself makes it observable (`tracing::warn!` naming the path +
error on failure). The broader convention — "a swallowed durable-write must at least
warn" — is the class the T-2531 drain-sink / future silent-swallow checks target; this
instance is captured as a learning so the next `let _ = std::fs::write` on a state
path gets the same treatment.

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

### 2026-08-13T07:30:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2674-hubtcp-discovery-flag-writeremove-failur.md
- **Context:** Initial task creation
