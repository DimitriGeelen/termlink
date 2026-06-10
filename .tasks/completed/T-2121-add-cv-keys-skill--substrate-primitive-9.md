---
id: T-2121
name: "Add /cv-keys skill — substrate primitive 9 (broadcast-with-replay) discovery verb, completes /governor cv_overflow investigation arc"
description: >
  Add /cv-keys skill — substrate primitive 9 (broadcast-with-replay) discovery verb, completes /governor cv_overflow investigation arc

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
created: 2026-06-10T10:39:36Z
last_update: 2026-06-10T10:39:36Z
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

# T-2121: Add /cv-keys skill — substrate primitive 9 (broadcast-with-replay) discovery verb, completes /governor cv_overflow investigation arc

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `.claude/commands/cv-keys.md` exists and follows the structural pattern of the other substrate-read daily verbs (`/find-idle` T-2092, `/claims` T-2093, `/queue-status` T-2094, `/governor` T-2095): Invocation table, Step 1 pre-flight, Step 2 parse arguments, Step 3 run the verb, Step 4 render, Step 5 empty-result hint, Rules, Common patterns, Related.
- [x] Skill wraps `termlink channel cv-keys <TOPIC> [--hub <addr>] [--json]` (signature from `crates/termlink-cli/src/cli.rs:2575` — PL-206 author-from-source rule).
- [x] Skill explicitly positions itself as the **follow-up verb after `/governor` flags `cv_overflow > 0`** — the operator's "which topic / producer is saturating cv_index?" question.
- [x] Empty-result path (`no cv_keys recorded on topic`) is loud per pattern, NOT silent — includes diagnostic hint pointing at producer-wiring guidance (`docs/operations/substrate-broadcast-with-replay.md` + `metadata.cv_key=...` annotation requirement).
- [x] Skill is read-only by contract (states explicitly: no auth side-effects, no state mutation, local-hub-only by default).
- [x] CLAUDE.md Quick Reference table gets a `/cv-keys` row in the existing skill block (alongside `/governor`, `/find-idle`, etc.) so operators see the verb when scanning the table.
- [x] Skill cross-references T-2103 / T-2104 / T-2105 / T-2106 (substrate #9 implementation chain) AND T-2110 / T-2118 / T-2119 / T-2120 (the cv_overflow observability arc this verb diagnoses) in the Related section.

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

test -f .claude/commands/cv-keys.md
out=$(grep -l "channel cv-keys" .claude/commands/cv-keys.md 2>&1); echo "$out" | grep -q "cv-keys.md"
out=$(grep -l "governor\|cv_overflow" .claude/commands/cv-keys.md 2>&1); echo "$out" | grep -q "cv-keys.md"
out=$(grep -l "T-2106" .claude/commands/cv-keys.md 2>&1); echo "$out" | grep -q "cv-keys.md"
grep -q "cv-keys" CLAUDE.md

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

### 2026-06-10 — skill authored from source (PL-206) instead of `--help`

- **What changed:** Per PL-206 the canonical authoring source is `<verb> --help`, but the locally-installed `termlink` binary (0.11.949 at `/root/.cargo/bin/termlink`) and even the freshest local build (`target/release/termlink` at 0.11.1008) both pre-date T-2106 — `channel cv-keys` is not yet shipped. The skill was authored from the source signature at `crates/termlink-cli/src/cli.rs:2575` (positional `topic`, `--hub <addr>`, `--json`) + the renderer at `crates/termlink-cli/src/commands/channel.rs:9170` (human format: `topic=X count=N` + `<cv_key> -> @<offset>`; empty: `no cv_keys recorded on topic`; JSON: `{count, entries: [{cv_key, offset}, ...]}`).
- **Plan impact:** None — skill ships now; will work as soon as operators have a binary with T-2106 included. The skill's pre-flight check (`termlink channel cv-keys --help`) gracefully refuses on pre-T-2106 binaries with an upgrade hint.
- **Triggered:** Closes the substrate #9 inspection arc at the skill tier alongside the §6 #10 cv_overflow observability arc (T-2110/T-2118/T-2119/T-2120). No follow-up tasks filed.

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

### 2026-06-10T10:39:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2121-add-cv-keys-skill--substrate-primitive-9.md
- **Context:** Initial task creation
