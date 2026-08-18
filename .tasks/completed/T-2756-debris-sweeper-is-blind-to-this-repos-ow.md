---
id: T-2756
name: "debris sweeper is blind to this repo's own prover topics"
description: >
  debris sweeper is blind to this repo's own prover topics

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [scripts/sweep-test-debris.sh, tests/sweep-debris-census-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-15T22:45:30Z
last_update: '2026-08-18T18:59:16Z'
date_finished: 2026-08-15T22:54:11Z
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:59Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=3 (body:portability-abstraction); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:16Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2756: debris sweeper is blind to this repo's own prover topics

## Context

`scripts/sweep-test-debris.sh` (T-2424) reports `topics=771 debris-candidates=1`
against a hub carrying roughly 630 test-debris topics (T-2754 / T-2755). An
operator reading that output concludes the hub is clean. It is not — the tool
structurally cannot see the debris, and says nothing about that.

This is the T-2680 disease in a cleanup tool instead of a canary: a count over a
partially-examined surface, printed bare, is read as a statement about the whole
surface. T-2747 solved the same problem for MCP parity by printing the census
(`260 tools: 24 asserted, 236 acknowledged`) on **both** output paths.

**This task does NOT change deletion policy.** The original intent was to widen
the allowlist; reading the code showed that cannot work, and the reason matters:

- `deny_topic` (`sweep-test-debris.sh:75`) denies `agent-conv-*`, and deny runs
  **before** allow (`:99-100`). Every leaked `agent-conv-selftest-*`,
  `agent-conv-list-*`, and `agent-conv-status-test-*` topic is therefore denied
  outright. Adding allow patterns for them would change nothing.
- That deny is **correct and deliberate** — real doorbell+mail conversation
  threads live under `agent-conv-*`. Sweeping them would require carving an
  exception into the guard that protects live conversation data. That is a
  human design decision, not an allowlist tweak, and is explicitly out of scope.

So the debris splits three ways, and only the tool can tell them apart:

1. **denied** — matched the deny guard (`agent-conv-*` and friends). Protected.
2. **allowed** — matched the debris allowlist. The 1 candidate reported today.
3. **unclassified** — matched neither (`dummy-*`, `arc004-dbg*`,
   `agent-presence-t2302-*`, `drain-fix-verify-*`). The conservative
   "unknown topics are NOT debris" default is right, but it is currently
   **silent**, which is what makes the bare `debris-candidates=1` misleading.

Making that split visible is a truthfulness fix. Every safety property —
deny-first, dry-run default, exact-name deletes, the conservative default — is
preserved unchanged.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The summary line reports the full three-way census — denied / allowed / unclassified — so `debris-candidates=N` can no longer be read as "everything else is clean". Live: `topics=771  candidates=1  denied=478 (protected)  unclassified=292 (not swept)`.
- [x] The census prints on **every** output path that reports a count: the dry-run path, the `--yes` path, and the `nothing to sweep` path. A clean-looking result is exactly where the disclaimer matters most (T-2680). Fixture 7 pins the clean path specifically.
- [x] A scope note names the limitation in words on those same paths: topics outside the allowlist are not swept, and denied topics are protected on purpose — so the reader is told the number is about the allowlist, not about the hub.
- [x] `--list-only` output is UNCHANGED (bare candidate names, one per line) — it is a piping contract, and census text on stdout would corrupt any consumer. Census/scope text goes to stderr there, or is omitted.
- [x] `--explain` (or equivalent) lists the unclassified topics, so an operator can see what the tool declined to classify rather than having to diff `channel list` by hand.
- [x] Deletion behaviour is byte-for-byte unchanged: the same topic set is deleted for the same inputs. The deny guard, the allowlist, the deny-before-allow order, the dry-run default and the exact-name delete are all untouched. Confirmed against the live hub: the candidate set is the same single topic before and after.
- [x] A fixture suite `tests/sweep-debris-census-fixtures.sh` drives the classifier over a canned topic list covering all three classes, and asserts (a) the census counts are correct, (b) a denied topic never reaches the candidate set even if an allow pattern would match it, and (c) `--list-only` stdout stays parseable. 20 assertions.

## Decisions

**Rejected the original plan after reading the code.** This task was created to
widen the sweeper's debris allowlist so it could see the ~630 leaked topics. That
cannot work: `deny_topic` (`:75`) denies `agent-conv-*` and deny is evaluated
before allow (`:99-100`), so every `agent-conv-selftest-*` / `agent-conv-list-*`
topic is excluded regardless of any allow pattern added. Sweeping them would mean
carving an exception into the guard protecting live doorbell+mail conversation
threads — a human design decision about deletion policy, not an allowlist tweak.
Recorded rather than silently re-scoped, because "widen the allowlist" is the
obvious-looking fix and the next person will propose it too.

**Chose truthfulness over capability.** The defect that remained is real and
independent: the tool reported `debris-candidates=1` against a hub holding ~630
debris topics with no indication it structurally could not see them. Fixing the
report preserves every safety property (deny-first, dry-run default, conservative
default, exact-name deletes) while removing the misread. Capability — actually
removing the 630 — stays with the human via T-2755.

**Asserted deny-before-allow structurally, not behaviourally.** A single topic
name matching both a deny and an allow pattern is not constructible with the
current pattern sets (all are prefix-anchored and the two sets share no prefix),
so no output-level fixture can prove the ordering. The fixture asserts it against
the source instead and says why in a comment, rather than shipping a test that
looks like it proves ordering but cannot.

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
# Fixture suite passes (hermetic — stub binary, no live hub).
bash tests/sweep-debris-census-fixtures.sh
# The dry-run path reports the census, not a bare candidate count.
bash -c 'bash scripts/sweep-test-debris.sh 2>&1 | grep -q "unclassified"'
# The scope limitation is stated in words, not just implied by numbers.
bash -c 'bash scripts/sweep-test-debris.sh 2>&1 | grep -qi "not swept\|allowlist"'
# --list-only stdout stays a bare, parseable name list (piping contract).
bash -c 'bash scripts/sweep-test-debris.sh --list-only 2>/dev/null | grep -qv " " || true'
bash -c 'test -z "$(bash scripts/sweep-test-debris.sh --list-only 2>/dev/null | grep -i "census\|unclassified\|denied" || true)"'
# The deny guard still beats the allowlist (safety property, not cosmetics).
# Asserts ORDERING directly — the deny call site must precede the allow call site —
# rather than matching one particular source formatting. A reorder fires this.
bash -c 'd=$(grep -n "deny_topic \"\$n\"" scripts/sweep-test-debris.sh | head -1 | cut -d: -f1); a=$(grep -n "allow_topic \"\$n\"" scripts/sweep-test-debris.sh | head -1 | cut -d: -f1); test -n "$d" && test -n "$a" && test "$d" -lt "$a"'
# Dry-run is still the default — no --yes means no deletion.
grep -q 'yes" -ne 1' scripts/sweep-test-debris.sh
# The guard layer stays green.
bash scripts/run-guard-layer.sh
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

### 2026-08-15T22:45:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2756-debris-sweeper-is-blind-to-this-repos-ow.md
- **Context:** Initial task creation

### 2026-08-15T22:54:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
