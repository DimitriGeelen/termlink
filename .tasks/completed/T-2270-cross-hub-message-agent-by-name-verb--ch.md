---
id: T-2270
name: "Surface identity_fingerprint in fleet presence rows (foundation for cross-hub contact-by-name)"
description: >
  T-2267 review item 4, slice 1 (foundation). Fleet presence rows carry agent_id->hub but NOT identity_fingerprint, so name->fp resolution falls back to parsing dm:* out of listen_topics (fails for any peer with no prior DM). The heartbeat envelope already carries a top-level sender_id = the T-1427 verified fingerprint; agent-listeners.sh just never projects it. Surface it as identity_fingerprint (+ a hub-independent test seam) so the resolver gets name->(hub,fp) reliably. Consumer slices (shell --to, MCP hub param, CLI fallback) are follow-ups T-2273/2274/2275.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-24T07:51:01Z
last_update: 2026-06-24T10:22:07Z
date_finished: 2026-06-24T10:22:07Z
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

# T-2270: Surface identity_fingerprint in fleet presence rows (foundation for cross-hub contact-by-name)

## Context

T-2267 review Layer 4 (the missing "message agent by name" abstraction). The Explore
map (this session) found the root cause is singular: name→fingerprint resolution is bound
to the LOCAL filesystem session registry (`manager::find_session`) in both CLI
(agent.rs:817) and MCP (tools.rs:17621); the only cross-hub presence source
(`agent-listeners-fleet`) exposes `agent_id → hub` but NOT `identity_fingerprint`. Today
the fp is recovered only by parsing a `dm:*` entry out of `listen_topics`
(agent-send.sh:113-124) — which **fails for any LIVE peer that has no prior DM topic**.

**Design decision (Option A, no producer change):** the agent-presence heartbeat envelope
already carries a top-level `sender_id` which T-1427 enforces to equal the poster's
verified identity fingerprint (`channel.post` rejects a mismatch). The bus envelope
struct confirms it (`termlink-bus/src/envelope.rs:22 pub sender_id`), and it is a sibling
of `msg_type`/`metadata`/`ts` in the `channel subscribe --json` stream. `agent-listeners.sh`
simply never projects it. Surfacing it requires **no change to the heartbeat producer and
no live-presence/protocol mutation** (avoids the PL-200 binary-swap-presence risk), and
works for ANY live agent regardless of DM history.

This task ships **slice 1 — the foundation**. Consumer slices are filed as follow-ups:
T-2273 (shell `agent-send.sh --to` → fleet + use identity_fingerprint), T-2274 (MCP
`termlink_agent_contact` gains `hub` param + fleet fallback), T-2275 (CLI `agent contact`
fleet fallback when local find_session misses).

## Acceptance Criteria

### Agent
- [x] `agent-listeners.sh` jq projection emits `identity_fingerprint` from the heartbeat envelope's top-level `sender_id` (T-1427 verified fingerprint), empty string when absent — always-included field (backward-compat, mirror of T-2091 capabilities).
- [x] `agent-listeners.sh` honors a `TERMLINK_LISTENERS_TEST_JSON=<file>` seam that feeds canned subscribe JSON and skips the live hub probe (mirror of T-2058 `TERMLINK_GROWTH_TEST_JSON`, PL-213) — enables hub-independent verification.
- [x] Fleet passthrough preserved: `agent-listeners-fleet.sh` merge carries `identity_fingerprint` per row (whole-object `group_by(.agent_id)|sort_by|last` merge + `map(.+{hub})` decoration, no field-allowlist drop — verified by reading agent-listeners-fleet.sh:225-247).
- [x] Self-contained test `tests/agent-listeners-identity-fp.sh`: canned heartbeat with `sender_id=<fp>` yields a row whose `identity_fingerprint == <fp>`; exits 0 on match, 1 otherwise. PASS.

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
bash tests/agent-listeners-identity-fp.sh

## RCA

**Symptom:** Cannot reach a LIVE peer "by name" cross-hub unless that peer already
has a `dm:*` listen_topic. A fresh peer (registered, heartbeating, no prior DM) is
unreachable by name — `agent-send.sh --to` dies "no dm:* listen_topic", and the
reporting agent concludes "I have no TermLink identity, I can't sign a cross-hub DM."

**Root cause:** fleet presence rows expose `agent_id → hub` but drop the one field
that makes name→fp resolution reliable — the verified `identity_fingerprint`. It is
present on every heartbeat envelope as `sender_id` (T-1427) but `agent-listeners.sh`'s
jq projection never selects it, so every consumer falls back to the fragile
`listen_topics` dm:* parse.

**Why structurally allowed:** the projection was authored incrementally (T-2091 added
capabilities, T-2107 added cv_key) field-by-field; `sender_id` lives at the envelope
top level, not under `.metadata`, so it was never in the "things we surface" set even
though it is the most load-bearing identity field.

**Prevention:** `identity_fingerprint` is now an always-included row field with a
hub-independent test (`tests/agent-listeners-identity-fp.sh`) asserting it equals the
envelope `sender_id`. Consumer slices can resolve name→(hub,fp) directly instead of
parsing topics.

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

### 2026-06-24T07:51:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2270-cross-hub-message-agent-by-name-verb--ch.md
- **Context:** Initial task creation

### 2026-06-24T10:17:02Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-24T10:22:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
