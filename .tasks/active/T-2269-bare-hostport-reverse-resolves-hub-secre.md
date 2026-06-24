---
id: T-2269
name: "Bare host:port reverse-resolves hub secret from hubs.toml"
description: >
  T-2267 review item 3. MCP/CLI-remote bail when arg contains ':' so bare host:port skips hubs.toml and dies with 'secret required', while channel post --hub DOES match the address field. Make resolve_hub_profile fall through to an address-field lookup. See review Layer 2 finding 6.

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
created: 2026-06-24T07:50:55Z
last_update: 2026-06-24T10:04:42Z
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

# T-2269: Bare host:port reverse-resolves hub secret from hubs.toml

## Context

T-2267 review Layer 2, finding 6. When the hub arg is a bare `host:port`, both the
MCP `resolve_hub_profile` (tools.rs:6687) and the CLI `resolve_hub_profile_with_config`
(config.rs) bail the instant they see `:` and skip hubs.toml entirely — so the call
dies "secret required" even though a profile with that exact `address` exists. Meanwhile
`channel post --hub`'s `resolve_hub_secret_hex` (channel.rs:247) DOES match the address
field and auto-loads the secret. Same hub: works by profile-name, fails by raw address.
This was the footgun that opened the .122 investigation. Fix: make the two bailing paths
fall through to an address-field reverse lookup (no explicit secret supplied), mirroring
`resolve_hub_secret_hex`.

## Acceptance Criteria

### Agent
- [x] MCP `resolve_hub_profile` (tools.rs): when arg contains `:`, reverse-resolves the secret by matching a hubs.toml profile's `address` field (returns its secret_file/secret), instead of unconditionally returning None. (via extracted pure helper `match_profile_by_address`)
- [x] CLI `resolve_hub_profile_with_config` (config.rs): when arg contains `:` and the caller passed no explicit secret, falls through to an address-field lookup against the supplied config; explicit `--secret-file`/`--secret` still override.
- [x] No-match behaviour preserved: a bare address with no matching profile and no explicit secret still resolves to address-only (caller must supply a secret) — no regression to the existing direct-address path.
- [x] Regression tests added covering address-match-resolves-secret, explicit-secret-overrides, and no-match-stays-bare; CLI `config::` (11 tests) + MCP `match_profile_by_address` (3 tests) pass.

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
out=$(cargo test -p termlink --bin termlink config:: 2>&1); echo "$out" | grep -q "test result: ok"
out=$(cargo test -p termlink-mcp --lib match_profile_by_address 2>&1); echo "$out" | grep -q "test result: ok"

## RCA

**Symptom:** A bare `host:port` (e.g. `192.168.10.122:9100`) passed to MCP/CLI-remote
verbs dies "secret required" / "Either secret_file or secret is required", even when a
hubs.toml profile with that exact `address` exists. The same hub works when referenced
by profile name. This misled agents into "I have no TermLink identity, I can't sign a
cross-hub DM" — a misdiagnosis that recurred across cycles (the .122 investigation).

**Root cause:** Both `resolve_hub_profile` (MCP, tools.rs:6687) and
`resolve_hub_profile_with_config` (CLI, config.rs) treated a `:` in the arg as
"direct address, no profile resolution needed" and returned/built an address-only
result — never consulting hubs.toml. Meanwhile `channel post --hub`'s
`resolve_hub_secret_hex` (channel.rs:247) DID match the `address` field. Three
resolution paths, two of which silently skipped the secret lookup.

**Why structurally allowed:** Three independent resolution implementations with no
shared contract; the address-match path existed in exactly one of them, so the
inconsistency was invisible until an operator hit the bare-address path.

**Prevention:** Regression tests in config.rs assert bare-address reverse-resolution,
explicit-secret override, and no-match-stays-bare. The MCP path reuses
`list_all_hub_profiles()` (the same source `channel post --hub` reads), aligning the
three paths on one address-match semantics.

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

### 2026-06-24T07:50:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2269-bare-hostport-reverse-resolves-hub-secre.md
- **Context:** Initial task creation

### 2026-06-24T10:04:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
