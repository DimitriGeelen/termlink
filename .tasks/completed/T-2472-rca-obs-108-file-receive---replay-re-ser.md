---
id: T-2472
name: "RCA OBS-108 file receive --replay re-serves earliest transfer + tautological SHA verify false-green"
description: >
  OBS-108: termlink file receive --replay re-serves the same earliest historical transfer on every invocation (later transfers unreachable), AND prints 'SHA-256 verified' exit 0 on the wrong file (tautological digest check = false green). RCA the replay cursor bug + the integrity-check bug; remediate; incept structural fix if warranted. Peer 832 relayed via operator.

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
created: 2026-07-31T11:34:16Z
last_update: 2026-07-31T11:50:02Z
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

# T-2472: RCA OBS-108 file receive --replay re-serves earliest transfer + tautological SHA verify false-green

## Context

RCA of OBS-108 (peer 832 relayed via operator). Full RCA in
`docs/reports/OBS-108-file-receive-replay-rca.md`. Two INDEPENDENT bugs found:
**#1** `file receive --replay` is hardwired to the oldest transfer (subscribe at
cursor 0 → `artifacts.first()`); **#2** the `SHA-256 verified` line is a
false-green (inline path reports `sha256(received_bytes)` — tautological; chunked
path reports the sender's own manifest digest — never a caller-supplied expected
digest). **This task carries the RCA + the bug #2 fix** (the operator-critical
false-green — the tool the operator was told to trust for independent digest
confirmation gave a clean pass on the wrong file). Bug #1 → separate task per
"one bug = one task". No inception: both are bounded point-fixes on a
retirement-track primitive (T-1166/T-1415, verbs retained as thin wrappers).

## Acceptance Criteria

### Agent
- [x] `termlink file receive` accepts `--expected-sha256 <hex>`; when the served
      file's actual sha256 != expected, the command prints a loud `SHA-256 MISMATCH`
      line and exits NON-zero (no "verified", no exit 0) — the operator's
      "on mismatch, don't re-pin, tell me" contract.
- [x] When `--expected-sha256` matches, output reads `SHA-256 verified against expected: <hex>` and exits 0.
- [x] When `--expected-sha256` is ABSENT, the success line no longer claims a bare
      `SHA-256 verified`: inline path labels it computed-from-received-bytes,
      chunked path labels it sender-manifest — the tautology is removed.
- [x] RCA section below is filled; OBS-108 registered in `.context` (gap + learning).
- [x] A unit test covers the mismatch → non-zero-exit path; `cargo check -p termlink` is clean.

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
cargo check -p termlink 2>&1 | tail -2
cargo test -p termlink --bin termlink reconcile 2>&1 | grep -q '3 passed'
grep -q 'expected_sha256' crates/termlink-cli/src/commands/file.rs
grep -q 'OBS-108' .context/project/concerns.yaml

## RCA

**Symptom:** Peer 832 sent two repaired fixtures; operator ran `termlink file
receive --replay` to independently confirm digests with an explicit "on mismatch,
don't re-pin, tell me". The verb re-served the same months-old transfer
(`escalation-patterns.yaml`) on all four invocations and printed `SHA-256 verified`
+ exit 0 every time — right digest, wrong file, clean success.

**Root cause (bug #2 — this task):** The `SHA-256 verified` line
(`crates/termlink-cli/src/commands/file.rs:597`) reports `s.sha256`, which is
never compared against a caller-supplied expected digest. Inline path
(`file.rs:516-518`) computes `sha256(received_payload)` and reports it — a
tautology that always passes. Chunked path reports the sender's own manifest
`artifact_ref` (bytes ARE checked against it in
`crates/termlink-session/src/artifact.rs:534`, so transit integrity holds) — but
that is the SENDER's declaration, not the RECEIVER's expectation, so it cannot
detect "served the wrong (old) transfer". Either way the word "verified" claims
an independent check that never happened.

**Root cause (bug #1 — separate task):** `--replay` subscribes at cursor 0
(`file.rs:432`) and `process_artifact_batch` returns `artifacts.first()`
(`file.rs:484`) = lowest offset = oldest artifact, with no persisted cursor and no
transfer-id/offset/`--since` selector — hardwired to the earliest envelope forever.

**Why structurally allowed:** the integrity check was self-referential by
construction — there was no seam for the receiver to assert an *expected* digest,
so the one command whose entire job is integrity confirmation could return a
false-green with exit 0 and no test caught it (the check can only ever pass).

**Prevention:** (a) `--expected-sha256` gives the receiver an independent assertion
that FAILS LOUD (non-zero exit) on mismatch; (b) honest labelling removes the word
"verified" when no independent digest was supplied; (c) a unit test locks the
mismatch → non-zero-exit contract; (d) OBS-108 registered as a gap so the class is
tracked through T-1166 retirement.

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

### 2026-07-31T11:34:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2472-rca-obs-108-file-receive---replay-re-ser.md
- **Context:** Initial task creation
