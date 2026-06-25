---
id: T-2281
name: "Fix stale termlink command refs in operator docs + extend hint-lint to CLAUDE.md/skills"
description: >
  Fix stale termlink command refs in operator docs + extend hint-lint to CLAUDE.md/skills

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
created: 2026-06-25T08:47:37Z
last_update: 2026-06-25T09:29:35Z
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

# T-2281: Fix stale termlink command refs in operator docs + extend hint-lint to CLAUDE.md/skills

## Context

Follow-on to T-2280: the command-hint lint shipped scanning `crates/` only. A
probe of the most-read operator surfaces (auto-loaded `CLAUDE.md` + `.claude/commands/`
skills) found stale `termlink <group> <verb>` refs that mislead every session:
`termlink fleet add` (no `add` subcommand — should be `remote profile add`),
`termlink remote call` (no CLI `remote call` — `unrecognized subcommand`, per T-2210
evidence), `termlink inbox push`. Fix them and extend the lint to guard these surfaces
so the rot can't recur. The extension also requires a correctness refinement: only
groups that actually HAVE subcommands should be validated — leaf commands take a
positional arg (`termlink ping <session>`, `spawn <name>`), so their 2nd token is an
argument, not a verb, and must not be flagged.

## Acceptance Criteria

### Agent
- [x] Stale refs fixed in current operator surfaces: `termlink fleet add` →
      `termlink remote profile add` (CLAUDE.md + `.claude/commands/preflight.md`);
      the bogus `remote call` alternative in CLAUDE.md corrected/removed; `inbox push`
      corrected. (Archival `docs/reports/T-*` are out of scope — point-in-time.)
      DONE — `.claude/commands/preflight.md` (`fleet add` → `remote profile add`,
      committed fc6f5a79) + `.claude/commands/agent-handoff.md` (`termlink inbox push`
      → `the retired inbox.push primitive`, fc6f5a79) + CLAUDE.md:489 (`remote call
      <peer> channel.post` → `the \`termlink_remote_call\` MCP tool`) + CLAUDE.md:1304
      (`fleet add` → `remote profile add`). The wrap-up gate that blocked the CLAUDE.md
      edits last session cleared on compaction (fresh budget).
- [x] `scripts/lint-command-hints.sh` refined: only validates groups that actually
      have subcommands (a `HAS_SUBCOMMANDS` set), eliminating the positional-arg
      false-positive class (ping/spawn/mirror/signal `<arg>`). Leaf top-level commands
      (in `IS_GROUP` but not `HAS_SUBCOMMANDS`) now skip 2nd-token validation — that
      token is a positional argument, not a verb.
- [x] Lint scan extended to `CLAUDE.md` + `.claude/commands/` (HINT_DIRS accepts
      files and dirs; `-d` guard → `-e`). `crates/` scan still green; all surfaces
      green after fixes (65 hints scanned in CLAUDE.md, plus every skill file).
- [x] `bash scripts/lint-command-hints.sh` exits 0 and `--self-test` still passes
      (also `--strict` exit 0 — no unknown-group hints).

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

test -x scripts/lint-command-hints.sh
bash scripts/lint-command-hints.sh
bash scripts/lint-command-hints.sh --self-test

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

### 2026-06-25T08:47:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2281-fix-stale-termlink-command-refs-in-opera.md
- **Context:** Initial task creation

### 2026-06-25T09:40:00Z — completion [agent]
- **Action:** Completed remaining T-2281 items after compaction cleared the 300k wrap-up gate
- **CLAUDE.md fixes:** line 489 `remote call <peer> channel.post` → `the \`termlink_remote_call\` MCP tool`; line 1304 (/preflight catalog Step-5 hint) `fleet add` → `remote profile add`
- **Lint refinement:** added `HAS_SUBCOMMANDS` set — only groups owning a Commands: block get 2nd-token validation; leaf commands (ping/spawn/mirror/signal) skip (positional arg, not verb). Extended `HINT_DIRS` to `CLAUDE.md` + `.claude/commands/` (`-d` guard → `-e` to accept files)
- **Verification:** `lint-command-hints.sh` exit 0 (default + `--strict` + `--self-test`); 65 hints scanned in CLAUDE.md + all skill files, all name real commands
- **Context:** No leaf-command 2-token hints exist in current sources, so the refinement is defensive prevention (eliminates the false-positive class for future hints)
