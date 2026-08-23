---
id: T-2830
name: "Pre-resolve t2687 to main integration on a throwaway branch"
description: >
  Pre-resolve t2687 to main integration on a throwaway branch

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/claim.rs, crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/dispatch.rs, crates/termlink-cli/src/commands/events.rs, crates/termlink-cli/src/commands/execution.rs, crates/termlink-cli/src/commands/metadata.rs, crates/termlink-cli/src/commands/mirror_grid.rs, crates/termlink-cli/src/commands/pty.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/commands/substrate.rs, crates/termlink-cli/src/main.rs, crates/termlink-cli/src/util.rs, crates/termlink-hub/src/artifact.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/server.rs, crates/termlink-hub/tests/no_federation_tripwire.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-mcp/tests/parity.rs, crates/termlink-protocol/src/control.rs, crates/termlink-session/build.rs, crates/termlink-session/src/ansi.rs, crates/termlink-session/src/claim_client.rs, crates/termlink-session/src/executor.rs, crates/termlink-session/src/handler.rs, crates/termlink-session/src/lib.rs, crates/termlink-session/src/pty.rs, crates/termlink-session/src/registration.rs, crates/termlink-session/src/scrollback.rs, crates/termlink-session/tests/no_spoke_mesh_tripwire.rs, scripts/agent-chat-arc-recent.sh, scripts/agent-conversation-selftest.sh, scripts/canary-status.sh, scripts/check-alloc-sink-clamps.sh, scripts/check-busy-spin.sh, scripts/check-canary-log-hygiene.sh, scripts/check-charter-drift-freshness.sh, scripts/check-charter-sentence-drift.sh, scripts/check-cron-install-drift.sh, scripts/check-drain-sink-caps.sh, scripts/check-env-var-docs.sh, scripts/check-error-code-docs.sh, scripts/check-error-code-emission.sh, scripts/check-mcp-parity-census.sh, scripts/check-platform-lock.sh, scripts/check-preflight-doc-set-drift.sh, scripts/check-release-artifact-drift.sh, scripts/check-silent-exit.sh, scripts/check-stuck-claims-freshness.sh, scripts/check-version-derivation.sh, scripts/fleet-adoption-snapshot.sh, scripts/lib/reap-topic.sh, scripts/run-guard-layer.sh, scripts/session-selftest.sh, scripts/substrate-preflight.sh, scripts/substrate-smoke.sh, scripts/substrate-worker-pickup.sh, scripts/sweep-test-debris.sh, scripts/test-agent-conversation-list.sh, scripts/test-agent-conversation-status.sh, scripts/test-agent-respond.sh, scripts/test-agent-send-auto-discover.sh, scripts/test-agent-send-orchestration.sh, scripts/test-agent-send.sh, scripts/test-agent-send-transport.sh, scripts/test-journal-mirror.sh, scripts/test-sidecar-auto-confirm.sh, tests/agent-send-grace-window.sh, tests/agent-send-idle-gate.sh, tests/canary-log-hygiene-fixtures.sh, tests/charter-drift-check-fixtures.sh, tests/chat-arc-recent-fixtures.sh, tests/cron-install-drift-fixtures.sh, tests/error-code-emission-fixtures.sh, tests/guard-layer-runner-fixtures.sh, tests/mcp-parity-census-fixtures.sh, tests/platform-lock-check-fixtures.sh, tests/reap-topic-fixtures.sh, tests/relay-b1-doorbell-rail.sh, tests/relay-b2-send-hops.sh, tests/release-artifact-drift-fixtures.sh, tests/silent-exit-check-fixtures.sh, tests/stuck-claims-check-fixtures.sh, tests/substrate-preflight-hubs-toml-fixtures.sh, tests/substrate-preflight-runtime-dir-fixtures.sh, tests/sweep-debris-census-fixtures.sh, tests/version-derivation-check-fixtures.sh, tests/wake-confirm-reply-match.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-23T10:27:23Z
last_update: 2026-08-23T11:26:07Z
date_finished: 2026-08-23T11:26:07Z
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

# T-2830: Pre-resolve t2687 to main integration on a throwaway branch

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Integration branch `integration/t2687-trial` exists, created from the t2687 head, and `origin/main` is merged into it
- [x] All 39 conflicted paths are resolved — `git diff --check` clean and no conflict markers remain anywhere in the tree
- [x] Append-only registers (`.context/project/{decisions,learnings,metrics-history}.yaml`, `.context/episodic/*`) retain every entry from BOTH sides — no id present before the merge is absent after it
- [x] `crates/termlink-mcp/src/tools.rs` resolves to main's version (T-2687 and our T-2824 are the same fix; ours is the duplicate)
- [x] `cargo build --release` succeeds on the merged tree
- [x] The merged tree's test suite passes, with any pre-existing failure identified as pre-existing by checking it against origin/main
- [x] Integration branch pushed to origin; each `--no-verify` push logged as a bypass
- [x] Merge-readiness report updated with what the PR contains and what remains a human decision

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

# The integration branch exists and carries main's history
git rev-parse --verify integration/t2687-trial
git merge-base --is-ancestor origin/main integration/t2687-trial
# No conflict markers survive anywhere in the tree
test -z "$(git grep -l '^<<<<<<< ' -- . || true)"
# tools.rs resolved to main's version (the T-2687 side, not our duplicate T-2824)
git diff --quiet origin/main -- crates/termlink-mcp/src/tools.rs
# The merged tree builds
cargo build --release --quiet
# Register union held: every decision id present on either parent is present now
bash scripts/verify-register-union.sh

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

### 2026-08-23T10:27:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2830-pre-resolve-t2687-to-main-integration-on.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-59de1bb3
- **Timestamp:** 2026-08-23T11:26:08Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#4 (Agent)** — `crates/termlink-mcp/src/tools.rs` resolves to main's version (T-2687 and our T-2824 are the same fix; ours is the duplicate)
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-mcp/src/tools.rs in: `crates/termlink-mcp/src/tools.rs` resolves to main's version (T-2687 and our T-2824 are the same fix; ours is the duplicate)`

### 2026-08-23T11:26:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
